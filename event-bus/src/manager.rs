use crate::{Event, EventBus};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

/// High-level event manager providing utilities for event management
/// and common event patterns
pub struct EventManager {
    event_bus: Arc<dyn EventBus + Send + Sync>,
}

impl EventManager {
    pub fn new(event_bus: Arc<dyn EventBus + Send + Sync>) -> Self {
        Self { event_bus }
    }
    
    /// Publish an event to the bus
    pub async fn publish(&self, event: Event) -> Result<()> {
        self.event_bus.publish(event).await
    }
    
    /// Create a simple event with minimal parameters
    pub async fn emit_simple_event(
        &self,
        event_type: &str,
        payload: serde_json::Value,
        source_plugin_id: Option<Uuid>,
    ) -> Result<()> {
        let event = Event {
            id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            source_plugin_id,
            timestamp: chrono::Utc::now(),
            payload,
        };
        
        self.publish(event).await
    }
    
    /// Subscribe to multiple event patterns at once
    pub async fn subscribe_to_patterns(
        &self,
        patterns: &[&str],
        subscriber_id: Uuid,
    ) -> Result<Vec<mpsc::Receiver<Event>>> {
        let mut receivers = Vec::new();
        
        for pattern in patterns {
            let rx = self.event_bus.subscribe(pattern, subscriber_id).await?;
            receivers.push(rx);
        }
        
        Ok(receivers)
    }
    
    /// Subscribe to all events matching a namespace (e.g., "submission.*")
    pub async fn subscribe_to_namespace(
        &self,
        namespace: &str,
        subscriber_id: Uuid,
    ) -> Result<mpsc::Receiver<Event>> {
        let pattern = format!("{}.*", namespace);
        self.event_bus.subscribe(&pattern, subscriber_id).await
    }
    
    /// Subscribe to all system events
    pub async fn subscribe_to_system_events(
        &self,
        subscriber_id: Uuid,
    ) -> Result<mpsc::Receiver<Event>> {
        self.subscribe_to_namespace("system", subscriber_id).await
    }
    
    /// Subscribe to all contest events
    pub async fn subscribe_to_contest_events(
        &self,
        subscriber_id: Uuid,
    ) -> Result<mpsc::Receiver<Event>> {
        self.subscribe_to_namespace("contest", subscriber_id).await
    }
    
    /// Subscribe to all submission events
    pub async fn subscribe_to_submission_events(
        &self,
        subscriber_id: Uuid,
    ) -> Result<mpsc::Receiver<Event>> {
        self.subscribe_to_namespace("submission", subscriber_id).await
    }
    
    /// Subscribe to all plugin system events
    pub async fn subscribe_to_plugin_events(
        &self,
        subscriber_id: Uuid,
    ) -> Result<mpsc::Receiver<Event>> {
        self.subscribe_to_namespace("plugin", subscriber_id).await
    }
    
    /// Unsubscribe from all events for a subscriber
    pub async fn unsubscribe(&self, subscriber_id: Uuid) -> Result<()> {
        self.event_bus.unsubscribe(subscriber_id).await
    }
    
    /// Create a plugin-specific event emitter
    pub fn plugin_emitter(&self, plugin_id: Uuid) -> PluginEventEmitter {
        PluginEventEmitter::new(self.event_bus.clone(), plugin_id)
    }
}

/// Plugin-specific event emitter that automatically sets the source plugin ID
pub struct PluginEventEmitter {
    event_bus: Arc<dyn EventBus + Send + Sync>,
    plugin_id: Uuid,
}

impl PluginEventEmitter {
    pub fn new(event_bus: Arc<dyn EventBus + Send + Sync>, plugin_id: Uuid) -> Self {
        Self { event_bus, plugin_id }
    }
    
    /// Emit an event from this plugin
    pub async fn emit(&self, event_type: &str, payload: serde_json::Value) -> Result<()> {
        let event = Event {
            id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            source_plugin_id: Some(self.plugin_id),
            timestamp: chrono::Utc::now(),
            payload,
        };
        
        self.event_bus.publish(event).await
    }
    
    /// Emit a plugin loaded event
    pub async fn emit_plugin_loaded(&self, plugin_name: &str, version: &str) -> Result<()> {
        self.emit(
            "plugin.loaded",
            serde_json::json!({
                "plugin_id": self.plugin_id,
                "plugin_name": plugin_name,
                "version": version
            })
        ).await
    }
    
    /// Emit a plugin unloaded event
    pub async fn emit_plugin_unloaded(&self, plugin_name: &str) -> Result<()> {
        self.emit(
            "plugin.unloaded",
            serde_json::json!({
                "plugin_id": self.plugin_id,
                "plugin_name": plugin_name
            })
        ).await
    }
    
    /// Emit a plugin error event
    pub async fn emit_plugin_error(&self, error: &str) -> Result<()> {
        self.emit(
            "plugin.error",
            serde_json::json!({
                "plugin_id": self.plugin_id,
                "error": error,
                "timestamp": chrono::Utc::now()
            })
        ).await
    }
}

/// Event listener utilities for common patterns
pub struct EventListener {
    receiver: mpsc::Receiver<Event>,
}

impl EventListener {
    pub fn new(receiver: mpsc::Receiver<Event>) -> Self {
        Self { receiver }
    }
    
    /// Listen for events and process them with a handler function
    pub async fn listen<F, Fut>(&mut self, mut handler: F) -> Result<()>
    where
        F: FnMut(Event) -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        while let Some(event) = self.receiver.recv().await {
            if let Err(e) = handler(event).await {
                tracing::error!("Event handler error: {}", e);
            }
        }
        Ok(())
    }
    
    /// Filter events by type before processing
    pub async fn listen_filtered<F, Fut>(&mut self, event_type: &str, mut handler: F) -> Result<()>
    where
        F: FnMut(Event) -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        let target_type = event_type.to_string();
        while let Some(event) = self.receiver.recv().await {
            if event.event_type == target_type {
                if let Err(e) = handler(event).await {
                    tracing::error!("Event handler error: {}", e);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MockEventBus;
    
    #[tokio::test]
    async fn test_event_manager_simple_emit() {
        let event_bus = Arc::new(MockEventBus::new());
        let manager = EventManager::new(event_bus);
        
        let result = manager.emit_simple_event(
            "test.event",
            serde_json::json!({"message": "Hello World"}),
            None,
        ).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_plugin_event_emitter() {
        let event_bus = Arc::new(MockEventBus::new());
        let plugin_id = Uuid::new_v4();
        let emitter = PluginEventEmitter::new(event_bus, plugin_id);
        
        let result = emitter.emit_plugin_loaded("test-plugin", "1.0.0").await;
        assert!(result.is_ok());
    }
}