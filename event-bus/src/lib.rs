use anyhow::Result;
use async_trait::async_trait;
use dashmap::DashMap;
use futures_util::StreamExt;
use lapin::{
    options::*, publisher_confirm::Confirmation, types::FieldTable, BasicProperties,
    Connection, ConnectionProperties,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

pub mod events;
pub mod subscription;
pub mod manager;

pub use manager::{EventManager, PluginEventEmitter, EventListener};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub event_type: String,
    pub source_plugin_id: Option<Uuid>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct EventSubscription {
    pub subscriber_id: Uuid,
    pub event_pattern: String, // Support wildcards like "submission.*" or "contest.problem.*"
    // Note: callback removed from struct to avoid Debug trait issues
    // Callbacks are managed separately by the event bus implementation
}

#[async_trait]
pub trait EventBus {
    async fn publish(&self, event: Event) -> Result<()>;
    async fn subscribe(&self, pattern: &str, subscriber_id: Uuid) -> Result<mpsc::Receiver<Event>>;
    async fn unsubscribe(&self, subscriber_id: Uuid) -> Result<()>;
}

#[derive(Debug)]
struct ConsumerInfo {
    pattern: String,
    shutdown_sender: oneshot::Sender<()>,
}

pub struct RabbitMQEventBus {
    connection: Connection,
    subscriptions: Arc<DashMap<Uuid, Vec<ConsumerInfo>>>, // subscriber_id -> ConsumerInfo
}

impl RabbitMQEventBus {
    pub async fn new(rabbitmq_url: &str) -> Result<Self> {
        let connection = Connection::connect(rabbitmq_url, ConnectionProperties::default()).await?;
        
        // Create exchange for events
        let channel = connection.create_channel().await?;
        channel
            .exchange_declare(
                "judicia_events",
                lapin::ExchangeKind::Topic,
                ExchangeDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;
        
        Ok(Self {
            connection,
            subscriptions: Arc::new(DashMap::new()),
        })
    }
}

#[async_trait]
impl EventBus for RabbitMQEventBus {
    async fn publish(&self, event: Event) -> Result<()> {
        let channel = self.connection.create_channel().await?;
        let routing_key = &event.event_type;
        let payload = serde_json::to_vec(&event)?;
        
        let confirm = channel
            .basic_publish(
                "judicia_events",
                routing_key,
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default(),
            )
            .await?
            .await?;
        
        match confirm {
            Confirmation::Ack(_) => {
                tracing::debug!("Event published: {} ({})", event.event_type, event.id);
                Ok(())
            }
            Confirmation::Nack(_) => Err(anyhow::anyhow!("Failed to publish event")),
            Confirmation::NotRequested => Ok(()),
        }
    }
    
    async fn subscribe(&self, pattern: &str, subscriber_id: Uuid) -> Result<mpsc::Receiver<Event>> {
        let channel = self.connection.create_channel().await?;
        
        // Create a queue for this subscriber
        let queue_name = format!("subscriber_{}_{}", subscriber_id, pattern);
        let queue_options = QueueDeclareOptions {
            durable: false,
            exclusive: true,
            auto_delete: true,
            ..Default::default()
        };
        channel
            .queue_declare(
                &queue_name,
                queue_options,
                FieldTable::default(),
            )
            .await?;
        
        // Bind queue to exchange with pattern
        channel
            .queue_bind(
                &queue_name,
                "judicia_events",
                pattern,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;
        
        let consumer_tag = format!("consumer_{}", subscriber_id);
        // Create consumer
        let mut consumer = channel
            .basic_consume(
                &queue_name,
                &consumer_tag,
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;
        
        let (tx, rx) = mpsc::channel(100);
        
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();
        self.subscriptions.entry(subscriber_id).or_default().push(ConsumerInfo {
            pattern: pattern.to_string(),
            shutdown_sender: shutdown_tx,
        });
        let subscription_clone = self.subscriptions.clone();

        // Spawn task to handle incoming events
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(delivery) = consumer.next() => {
                        if let Ok(delivery) = delivery {
                            if let Ok(event) = serde_json::from_slice::<Event>(&delivery.data) {
                                if tx.send(event).await.is_err() {
                                    tracing::warn!("Receiver for subscriber {} dropped. Shutting down consumer.", subscriber_id);
                                    break;
                                }
                            }
                            let _ = delivery.ack(BasicAckOptions::default()).await;
                        }
                    },
                    _ = &mut shutdown_rx => {
                        tracing::info!("Shutdown signal received for subscriber {}. Closing consumer.", subscriber_id);
                        break;
                    },
                    else => {
                        break;
                    }
                }
            }

            tracing::debug!("Cleaning up resources for subscriber {}", subscriber_id);
            if let Err(e) = channel.basic_cancel(&consumer_tag, BasicCancelOptions::default()).await {
                 tracing::error!("Failed to cancel consumer for {}: {}", subscriber_id, e);
            }
            let _ = channel.queue_delete(&queue_name, QueueDeleteOptions::default()).await;

            subscription_clone.remove(&subscriber_id);
            tracing::info!("Consumer and resources for subscriber {} cleaned up.", subscriber_id);
        });
        
        Ok(rx)
    }
    
    async fn unsubscribe(&self, subscriber_id: Uuid) -> Result<()> {
        if let Some((_, consumers)) = self.subscriptions.remove(&subscriber_id) {
            let _ = consumers.into_iter().map(|c| c.shutdown_sender.send(())).collect::<Vec<_>>();
            tracing::info!("Sent shutdown signal to subscriber {}", subscriber_id);
        } else {
            tracing::warn!("Attempted to unsubscribe non-existent subscriber {}", subscriber_id);
        }
        
        // Clean up RabbitMQ queue and consumer
        // This requires keeping track of channels and consumers
        
        Ok(())
    }
}

// Mock implementation for testing
pub struct MockEventBus {
    subscriptions: Arc<DashMap<Uuid, String>>,
}

impl MockEventBus {
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(DashMap::new()),
        }
    }
}

#[async_trait]
impl EventBus for MockEventBus {
    async fn publish(&self, event: Event) -> Result<()> {
        tracing::debug!("Mock: Published event {} ({})", event.event_type, event.id);
        Ok(())
    }
    
    async fn subscribe(&self, pattern: &str, subscriber_id: Uuid) -> Result<mpsc::Receiver<Event>> {
        self.subscriptions.insert(subscriber_id, pattern.to_string());
        let (tx, rx) = mpsc::channel(100);
        // Mock subscription - events won't actually be delivered in test mode
        tracing::debug!("Mock: Subscribed {} to pattern {}", subscriber_id, pattern);
        drop(tx); // Close sender immediately since this is just a mock
        Ok(rx)
    }
    
    async fn unsubscribe(&self, subscriber_id: Uuid) -> Result<()> {
        self.subscriptions.remove(&subscriber_id);
        tracing::debug!("Mock: Unsubscribed {}", subscriber_id);
        Ok(())
    }
}