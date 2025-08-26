//! # Judicia Plugin SDK
//! 
//! The Judicia SDK provides a powerful, type-safe way to create WebAssembly plugins
//! for the Judicia online judge platform. It includes procedural macros that generate
//! the necessary boilerplate code for plugin lifecycle management, capability declarations,
//! and platform integration.
//! 
//! ## Features
//! 
//! - **Declarative Plugin Definition**: Use `#[judicia_plugin]` to define your plugin
//! - **Automatic Capability Management**: Declare required platform capabilities
//! - **Type-Safe API Bindings**: Strongly-typed interfaces to platform services
//! - **Event-Driven Architecture**: React to platform events and emit custom events
//! - **Resource Management**: Automatic cleanup and lifecycle management
//! - **Frontend Integration**: Seamless integration with web components
//! 
//! ## Basic Usage
//! 
//! ```rust
//! use judicia_sdk::prelude::*;
//! 
//! #[judicia_plugin]
//! pub struct MyPlugin {
//!     name: "my-plugin",
//!     version: "1.0.0",
//!     author: "Your Name",
//!     description: "A sample Judicia plugin",
//!     capabilities: [
//!         Capability::TriggerJudging,
//!         Capability::EmitEvent,
//!         Capability::AccessDatabase
//!     ]
//! }
//! 
//! impl PluginMethods for MyPlugin {
//!     async fn on_initialize(&mut self, context: &PluginContext) -> PluginResult<()> {
//!         info!("Plugin {} initialized!", self.metadata.name);
//!         Ok(())
//!     }
//! 
//!     async fn on_event(&mut self, event: &PlatformEvent) -> PluginResult<()> {
//!         match event.event_type.as_str() {
//!             "submission.created" => self.handle_submission(event).await?,
//!             _ => {}
//!         }
//!         Ok(())
//!     }
//! }
//! ```

// Re-export procedural macros
pub use judicia_sdk_macros::{judicia_plugin, frontend_component, capability};

// Re-export runtime types and utilities
pub use judicia_sdk_runtime::*;

/// Prelude module containing commonly used imports
pub mod prelude {
    pub use judicia_sdk_macros::{judicia_plugin, frontend_component, capability};
    pub use judicia_sdk_runtime::{
        PluginMethods,
        PluginContext,
        PluginResult,
        PlatformEvent,
        Capability,
        PluginMetadata,
        DatabaseQuery,
        EventEmitter,
        JudgingRequest,
        FrontendProps,
        ComponentType,
    };
    
    // Re-export common external types
    pub use serde::{Deserialize, Serialize};
    pub use uuid::Uuid;
    pub use chrono::{DateTime, Utc};
    pub use anyhow::Result;
    pub use tracing::{debug, error, info, trace, warn};
    pub use wasm_bindgen::prelude::*;
}