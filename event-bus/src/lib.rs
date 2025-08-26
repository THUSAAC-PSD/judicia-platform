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
use tokio::sync::mpsc;
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

pub struct RabbitMQEventBus {
    connection: Connection,
    subscriptions: Arc<DashMap<Uuid, String>>, // subscriber_id -> pattern
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
        let queue_name = format!("subscriber_{}", subscriber_id);
        channel
            .queue_declare(
                &queue_name,
                QueueDeclareOptions::default(),
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
        
        // Create consumer
        let mut consumer = channel
            .basic_consume(
                &queue_name,
                &format!("consumer_{}", subscriber_id),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;
        
        let (tx, rx) = mpsc::channel(100);
        self.subscriptions.insert(subscriber_id, pattern.to_string());
        
        // Spawn task to handle incoming events
        tokio::spawn(async move {
            while let Some(delivery) = consumer.next().await {
                if let Ok(delivery) = delivery {
                    if let Ok(event) = serde_json::from_slice::<Event>(&delivery.data) {
                        if tx.send(event).await.is_err() {
                            break; // Receiver dropped
                        }
                    }
                    let _ = delivery.ack(BasicAckOptions::default()).await;
                }
            }
        });
        
        Ok(rx)
    }
    
    async fn unsubscribe(&self, subscriber_id: Uuid) -> Result<()> {
        self.subscriptions.remove(&subscriber_id);
        
        // TODO: Clean up RabbitMQ queue and consumer
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