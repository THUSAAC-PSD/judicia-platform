use crate::kernel::JudiciaKernel;
use anyhow::Result;
use event_bus::{Event, EventListener, EventManager};
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Core system event handlers that respond to platform events
pub struct CoreEventHandlers {
    kernel: Arc<JudiciaKernel>,
    event_manager: Arc<EventManager>,
}

impl CoreEventHandlers {
    pub fn new(kernel: Arc<JudiciaKernel>) -> Self {
        let event_manager = kernel.event_manager();
        Self {
            kernel,
            event_manager,
        }
    }
    
    /// Start listening to core system events
    pub async fn start_listening(&self) -> Result<()> {
        let subscriber_id = Uuid::new_v4();
        
        // Subscribe to plugin events
        let plugin_events = self.event_manager
            .subscribe_to_plugin_events(subscriber_id)
            .await?;
        
        // Subscribe to submission events  
        let submission_events = self.event_manager
            .subscribe_to_submission_events(subscriber_id)
            .await?;
        
        // Subscribe to system events
        let system_events = self.event_manager
            .subscribe_to_system_events(subscriber_id)
            .await?;
        
        // Start event processing tasks
        let kernel_arc = self.kernel.clone();
        tokio::spawn(async move {
            Self::handle_plugin_events(kernel_arc, plugin_events).await;
        });
        
        let kernel_arc = self.kernel.clone();
        tokio::spawn(async move {
            Self::handle_submission_events(kernel_arc, submission_events).await;
        });
        
        let kernel_arc = self.kernel.clone();
        tokio::spawn(async move {
            Self::handle_system_events(kernel_arc, system_events).await;
        });
        
        tracing::info!("✅ Core event handlers started listening");
        Ok(())
    }
    
    /// Handle plugin-related events
    async fn handle_plugin_events(
        _kernel: Arc<JudiciaKernel>,
        receiver: mpsc::Receiver<Event>,
    ) {
        let mut listener = EventListener::new(receiver);
        
        let _ = listener.listen(|event| async move {
            match event.event_type.as_str() {
                "plugin.loaded" => {
                    tracing::info!("🔌 Plugin loaded event received: {}", event.id);
                    if let Ok(payload) = serde_json::from_value::<serde_json::Value>(event.payload) {
                        if let Some(plugin_name) = payload.get("plugin_name") {
                            tracing::info!("   Plugin: {}", plugin_name);
                        }
                    }
                }
                "plugin.unloaded" => {
                    tracing::info!("🔌 Plugin unloaded event received: {}", event.id);
                }
                "plugin.error" => {
                    tracing::error!("🔌 Plugin error event received: {}", event.id);
                    if let Ok(payload) = serde_json::from_value::<serde_json::Value>(event.payload) {
                        if let Some(error) = payload.get("error") {
                            tracing::error!("   Error: {}", error);
                        }
                    }
                }
                _ => {
                    tracing::debug!("🔌 Other plugin event: {}", event.event_type);
                }
            }
            Ok(())
        }).await;
    }
    
    /// Handle submission-related events
    async fn handle_submission_events(
        _kernel: Arc<JudiciaKernel>,
        receiver: mpsc::Receiver<Event>,
    ) {
        let mut listener = EventListener::new(receiver);
        
        let _ = listener.listen(|event| async move {
            match event.event_type.as_str() {
                "submission.submitted" => {
                    tracing::info!("📝 Submission submitted event received: {}", event.id);
                    // In a full implementation, this could trigger:
                    // - Validation checks
                    // - Contest timing checks
                    // - Queue the submission for judging
                }
                "submission.judged" => {
                    tracing::info!("⚖️ Submission judged event received: {}", event.id);
                    // In a full implementation, this could trigger:
                    // - Update leaderboards
                    // - Send notifications
                    // - Check for first blood
                }
                "judging.requested" => {
                    tracing::info!("🎯 Judging requested event received: {}", event.id);
                    // This event is handled by the evaluation engine workers
                }
                _ => {
                    tracing::debug!("📝 Other submission event: {}", event.event_type);
                }
            }
            Ok(())
        }).await;
    }
    
    /// Handle system-related events
    async fn handle_system_events(
        _kernel: Arc<JudiciaKernel>,
        receiver: mpsc::Receiver<Event>,
    ) {
        let mut listener = EventListener::new(receiver);
        
        let _ = listener.listen(|event| async move {
            match event.event_type.as_str() {
                "system.maintenance.start" => {
                    tracing::warn!("🔧 System maintenance started: {}", event.id);
                }
                "system.maintenance.end" => {
                    tracing::info!("✅ System maintenance ended: {}", event.id);
                }
                "system.backup.complete" => {
                    tracing::info!("💾 System backup completed: {}", event.id);
                }
                _ => {
                    tracing::debug!("🏛️ Other system event: {}", event.event_type);
                }
            }
            Ok(())
        }).await;
    }
}

/// Utility functions for emitting common events
impl JudiciaKernel {
    /// Emit a system maintenance event
    pub async fn emit_system_maintenance(&self, starting: bool) -> Result<()> {
        let event_type = if starting {
            "system.maintenance.start"
        } else {
            "system.maintenance.end"
        };
        
        self.event_manager().emit_simple_event(
            event_type,
            serde_json::json!({
                "timestamp": chrono::Utc::now(),
                "initiated_by": "core_kernel"
            }),
            None,
        ).await
    }
    
    /// Emit a submission judged event
    pub async fn emit_submission_judged(
        &self,
        submission_id: Uuid,
        user_id: Uuid,
        problem_id: Uuid,
        verdict: &str,
        execution_time_ms: Option<i32>,
        execution_memory_kb: Option<i32>,
    ) -> Result<()> {
        self.event_manager().emit_simple_event(
            "submission.judged",
            serde_json::json!({
                "submission_id": submission_id,
                "user_id": user_id,
                "problem_id": problem_id,
                "verdict": verdict,
                "execution_time_ms": execution_time_ms,
                "execution_memory_kb": execution_memory_kb,
                "timestamp": chrono::Utc::now()
            }),
            None,
        ).await
    }
    
    /// Start the core event handlers
    pub async fn start_event_handlers(self: &Arc<Self>) -> Result<()> {
        let handlers = CoreEventHandlers::new(self.clone());
        handlers.start_listening().await
    }
}

