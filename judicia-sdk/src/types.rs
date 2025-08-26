//! Core types and traits for Judicia plugin development

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use wasm_bindgen::prelude::*;

/// Result type for plugin operations
pub type PluginResult<T> = Result<T, PluginError>;

/// Main trait that all Judicia plugins must implement
#[async_trait::async_trait(?Send)]
pub trait PluginMethods {
    /// Called when the plugin is first loaded and initialized
    async fn on_initialize(&mut self, context: &PluginContext) -> PluginResult<()>;
    
    /// Called when the plugin receives a platform event
    async fn on_event(&mut self, event: &PlatformEvent) -> PluginResult<()>;
    
    /// Called when the plugin is being unloaded/cleaned up
    async fn on_cleanup(&mut self) -> PluginResult<()> {
        Ok(())
    }
    
    /// Called to handle HTTP requests (for plugins that register routes)
    async fn on_http_request(&mut self, request: &HttpRequest) -> PluginResult<HttpResponse> {
        Err(PluginError::NotImplemented("HTTP request handling not implemented".into()))
    }
    
    /// Called for frontend rendering (for plugins with UI components)
    async fn on_render(&self, component: &str, props: &serde_json::Value) -> PluginResult<String> {
        Err(PluginError::NotImplemented("Frontend rendering not implemented".into()))
    }
}

/// Core plugin trait for metadata and lifecycle
pub trait Plugin {
    fn new() -> Self;
    fn metadata(&self) -> PluginInfo;
}

/// Plugin metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub dependencies: Vec<String>,
    pub frontend_components: Vec<String>,
    pub api_routes: Vec<String>,
}

/// Context provided to plugins during initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct PluginContext {
    pub plugin_id: Uuid,
    pub platform_version: String,
    pub config: HashMap<String, serde_json::Value>,
    pub granted_capabilities: Vec<Capability>,
    pub data_directory: String,
    pub log_level: String,
}

/// Platform events that plugins can subscribe to and react to
#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct PlatformEvent {
    pub id: Uuid,
    pub event_type: String,
    pub source_plugin_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub payload: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

/// HTTP request structure for plugins that handle web requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub body: Option<String>,
    pub user_id: Option<Uuid>,
    pub permissions: Vec<String>,
}

/// HTTP response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub content_type: String,
}

impl Default for HttpResponse {
    fn default() -> Self {
        Self {
            status_code: 200,
            headers: HashMap::new(),
            body: String::new(),
            content_type: "application/json".to_string(),
        }
    }
}

/// Platform capabilities that plugins can request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[wasm_bindgen]
pub enum Capability {
    /// Trigger code evaluation/judging
    TriggerJudging,
    /// Emit platform events
    EmitEvent,
    /// Subscribe to platform events  
    SubscribeEvents,
    /// Access contest database (read)
    ReadDatabase,
    /// Access contest database (write)
    WriteDatabase,
    /// Register HTTP API routes
    RegisterRoutes,
    /// Access user information
    AccessUserData,
    /// Send notifications to users
    SendNotifications,
    /// Access file storage
    FileStorage,
    /// Execute system commands (dangerous)
    SystemExecution,
    /// Network access (external APIs)
    NetworkAccess,
    /// Frontend component registration
    RegisterComponents,
    /// Administrative operations
    AdminOperations,
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Capability::TriggerJudging => write!(f, "trigger_judging"),
            Capability::EmitEvent => write!(f, "emit_event"),
            Capability::SubscribeEvents => write!(f, "subscribe_events"),
            Capability::ReadDatabase => write!(f, "read_database"),
            Capability::WriteDatabase => write!(f, "write_database"),
            Capability::RegisterRoutes => write!(f, "register_routes"),
            Capability::AccessUserData => write!(f, "access_user_data"),
            Capability::SendNotifications => write!(f, "send_notifications"),
            Capability::FileStorage => write!(f, "file_storage"),
            Capability::SystemExecution => write!(f, "system_execution"),
            Capability::NetworkAccess => write!(f, "network_access"),
            Capability::RegisterComponents => write!(f, "register_components"),
            Capability::AdminOperations => write!(f, "admin_operations"),
        }
    }
}

/// Database query structure for plugins with database access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseQuery {
    pub query: String,
    pub parameters: Vec<serde_json::Value>,
    pub timeout_ms: Option<u32>,
}

/// Judging request structure for evaluation plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgingRequest {
    pub submission_id: Uuid,
    pub problem_id: Uuid,
    pub language_id: Uuid,
    pub source_code: String,
    pub test_cases: Vec<TestCase>,
    pub time_limit_ms: u32,
    pub memory_limit_kb: u32,
    pub priority: u8,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Test case structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub id: Uuid,
    pub input: String,
    pub expected_output: String,
    pub points: i32,
    pub is_sample: bool,
}

/// Event emitter for plugins to publish events
#[derive(Debug, Clone)]
pub struct EventEmitter {
    plugin_id: Uuid,
}

impl EventEmitter {
    pub fn new(plugin_id: Uuid) -> Self {
        Self { plugin_id }
    }
    
    pub async fn emit(&self, event_type: &str, payload: serde_json::Value) -> PluginResult<()> {
        let event = PlatformEvent {
            id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            source_plugin_id: Some(self.plugin_id),
            timestamp: Utc::now(),
            payload,
            metadata: HashMap::new(),
        };
        
        // Call platform API to emit event
        emit_platform_event(&event).await
    }
}

/// Notification structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub recipient_id: Uuid,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub urgency: NotificationUrgency,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NotificationType {
    Info,
    Success, 
    Warning,
    Error,
    Announcement,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NotificationUrgency {
    Low,
    Normal,
    High,
    Critical,
}

// Platform API function declarations (implemented by the platform)
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["judicia", "platform"])]
    async fn emit_platform_event(event: &PlatformEvent) -> PluginResult<()>;
    
    #[wasm_bindgen(js_namespace = ["judicia", "platform"])]
    async fn trigger_judging(request: &JudgingRequest) -> PluginResult<Uuid>;
    
    #[wasm_bindgen(js_namespace = ["judicia", "platform"])]
    async fn database_query(query: &DatabaseQuery) -> PluginResult<serde_json::Value>;
    
    #[wasm_bindgen(js_namespace = ["judicia", "platform"])]
    async fn send_notification(notification: &Notification) -> PluginResult<()>;
    
    #[wasm_bindgen(js_namespace = ["judicia", "platform"])]
    async fn register_http_route(
        method: &str, 
        path: &str, 
        handler_name: &str
    ) -> PluginResult<()>;
    
    #[wasm_bindgen(js_namespace = ["judicia", "platform"])]
    fn log_message(level: &str, message: &str, metadata: &JsValue);
    
    #[wasm_bindgen(js_namespace = ["judicia", "platform"])]
    async fn get_user_info(user_id: &Uuid) -> PluginResult<serde_json::Value>;
    
    #[wasm_bindgen(js_namespace = ["judicia", "platform"])]
    async fn store_file(
        path: &str, 
        content: &[u8], 
        content_type: &str
    ) -> PluginResult<String>;
    
    #[wasm_bindgen(js_namespace = ["judicia", "platform"])]
    async fn load_file(path: &str) -> PluginResult<Vec<u8>>;
}

/// Trait for plugins that provide metadata
pub trait PluginMetadata {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn author(&self) -> &str;
    fn description(&self) -> &str;
}

/// Configuration helper for plugins
#[derive(Debug, Clone)]
pub struct PluginConfig {
    values: HashMap<String, serde_json::Value>,
}

impl PluginConfig {
    pub fn new(values: HashMap<String, serde_json::Value>) -> Self {
        Self { values }
    }
    
    pub fn get<T>(&self, key: &str) -> Option<T> 
    where
        T: for<'de> Deserialize<'de>,
    {
        self.values.get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
    
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get(key)
    }
    
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key)
    }
    
    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.get(key)
    }
}