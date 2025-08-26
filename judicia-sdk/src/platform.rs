//! Platform integration utilities and API wrappers

use crate::{
    error::{PluginError, PluginResult},
    types::*,
};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;
use wasm_bindgen::prelude::*;

/// High-level platform API wrapper for plugins
pub struct Platform {
    plugin_id: Uuid,
    capabilities: Vec<Capability>,
}

impl Platform {
    pub fn new(plugin_id: Uuid, capabilities: Vec<Capability>) -> Self {
        Self {
            plugin_id,
            capabilities,
        }
    }
    
    /// Check if the plugin has a specific capability
    pub fn has_capability(&self, capability: Capability) -> bool {
        self.capabilities.contains(&capability)
    }
    
    /// Ensure the plugin has a required capability or return error
    pub fn require_capability(&self, capability: Capability) -> PluginResult<()> {
        if self.has_capability(capability) {
            Ok(())
        } else {
            Err(PluginError::MissingCapability(capability.to_string()))
        }
    }
    
    /// Trigger a judging request (requires TriggerJudging capability)
    pub async fn trigger_judging(&self, request: JudgingRequest) -> PluginResult<Uuid> {
        self.require_capability(Capability::TriggerJudging)?;
        
        match trigger_judging(&request).await {
            Ok(job_id) => Ok(job_id),
            Err(e) => Err(PluginError::PlatformApiError(format!("Failed to trigger judging: {:?}", e))),
        }
    }
    
    /// Emit a platform event (requires EmitEvent capability)
    pub async fn emit_event(&self, event_type: &str, payload: Value) -> PluginResult<()> {
        self.require_capability(Capability::EmitEvent)?;
        
        let event = PlatformEvent {
            id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            source_plugin_id: Some(self.plugin_id),
            timestamp: chrono::Utc::now(),
            payload,
            metadata: HashMap::new(),
        };
        
        match emit_platform_event(&event).await {
            Ok(_) => Ok(()),
            Err(e) => Err(PluginError::PlatformApiError(format!("Failed to emit event: {:?}", e))),
        }
    }
    
    /// Execute a database query (requires ReadDatabase or WriteDatabase capability)
    pub async fn query_database(&self, query: &str, params: Vec<Value>) -> PluginResult<Value> {
        // Check for either read or write capability based on query type
        let is_write_query = query.trim().to_lowercase().starts_with("insert") ||
                            query.trim().to_lowercase().starts_with("update") ||
                            query.trim().to_lowercase().starts_with("delete");
        
        if is_write_query {
            self.require_capability(Capability::WriteDatabase)?;
        } else {
            if !self.has_capability(Capability::ReadDatabase) && 
               !self.has_capability(Capability::WriteDatabase) {
                return Err(PluginError::MissingCapability("read_database or write_database".to_string()));
            }
        }
        
        let db_query = DatabaseQuery {
            query: query.to_string(),
            parameters: params,
            timeout_ms: Some(30000), // 30 second timeout
        };
        
        match database_query(&db_query).await {
            Ok(result) => Ok(result),
            Err(e) => Err(PluginError::DatabaseError(format!("Database query failed: {:?}", e))),
        }
    }
    
    /// Send a notification to a user (requires SendNotifications capability)
    pub async fn send_notification(&self, notification: Notification) -> PluginResult<()> {
        self.require_capability(Capability::SendNotifications)?;
        
        match send_notification(&notification).await {
            Ok(_) => Ok(()),
            Err(e) => Err(PluginError::PlatformApiError(format!("Failed to send notification: {:?}", e))),
        }
    }
    
    /// Register an HTTP route handler (requires RegisterRoutes capability)
    pub async fn register_route(&self, method: &str, path: &str, handler: &str) -> PluginResult<()> {
        self.require_capability(Capability::RegisterRoutes)?;
        
        match register_http_route(method, path, handler).await {
            Ok(_) => Ok(()),
            Err(e) => Err(PluginError::PlatformApiError(format!("Failed to register route: {:?}", e))),
        }
    }
    
    /// Get user information (requires AccessUserData capability)
    pub async fn get_user(&self, user_id: Uuid) -> PluginResult<Value> {
        self.require_capability(Capability::AccessUserData)?;
        
        match get_user_info(&user_id).await {
            Ok(user_data) => Ok(user_data),
            Err(e) => Err(PluginError::PlatformApiError(format!("Failed to get user info: {:?}", e))),
        }
    }
    
    /// Store a file (requires FileStorage capability)
    pub async fn store_file(&self, path: &str, content: &[u8], content_type: &str) -> PluginResult<String> {
        self.require_capability(Capability::FileStorage)?;
        
        match store_file(path, content, content_type).await {
            Ok(file_url) => Ok(file_url),
            Err(e) => Err(PluginError::FileSystemError(format!("Failed to store file: {:?}", e))),
        }
    }
    
    /// Load a file (requires FileStorage capability)  
    pub async fn load_file(&self, path: &str) -> PluginResult<Vec<u8>> {
        self.require_capability(Capability::FileStorage)?;
        
        match load_file(path).await {
            Ok(content) => Ok(content),
            Err(e) => Err(PluginError::FileSystemError(format!("Failed to load file: {:?}", e))),
        }
    }
}

/// Builder for creating judging requests
pub struct JudgingRequestBuilder {
    request: JudgingRequest,
}

impl JudgingRequestBuilder {
    pub fn new(submission_id: Uuid, problem_id: Uuid, language_id: Uuid) -> Self {
        Self {
            request: JudgingRequest {
                submission_id,
                problem_id,
                language_id,
                source_code: String::new(),
                test_cases: Vec::new(),
                time_limit_ms: 1000,
                memory_limit_kb: 256 * 1024,
                priority: 1,
                metadata: HashMap::new(),
            },
        }
    }
    
    pub fn source_code(mut self, code: &str) -> Self {
        self.request.source_code = code.to_string();
        self
    }
    
    pub fn time_limit(mut self, ms: u32) -> Self {
        self.request.time_limit_ms = ms;
        self
    }
    
    pub fn memory_limit(mut self, kb: u32) -> Self {
        self.request.memory_limit_kb = kb;
        self
    }
    
    pub fn priority(mut self, priority: u8) -> Self {
        self.request.priority = priority;
        self
    }
    
    pub fn test_case(mut self, test_case: TestCase) -> Self {
        self.request.test_cases.push(test_case);
        self
    }
    
    pub fn metadata(mut self, key: &str, value: Value) -> Self {
        self.request.metadata.insert(key.to_string(), value);
        self
    }
    
    pub fn build(self) -> JudgingRequest {
        self.request
    }
}

/// Builder for creating notifications
pub struct NotificationBuilder {
    notification: Notification,
}

impl NotificationBuilder {
    pub fn new(recipient_id: Uuid) -> Self {
        Self {
            notification: Notification {
                recipient_id,
                title: String::new(),
                message: String::new(),
                notification_type: NotificationType::Info,
                urgency: NotificationUrgency::Normal,
                metadata: HashMap::new(),
            },
        }
    }
    
    pub fn title(mut self, title: &str) -> Self {
        self.notification.title = title.to_string();
        self
    }
    
    pub fn message(mut self, message: &str) -> Self {
        self.notification.message = message.to_string();
        self
    }
    
    pub fn notification_type(mut self, notification_type: NotificationType) -> Self {
        self.notification.notification_type = notification_type;
        self
    }
    
    pub fn urgency(mut self, urgency: NotificationUrgency) -> Self {
        self.notification.urgency = urgency;
        self
    }
    
    pub fn metadata(mut self, key: &str, value: Value) -> Self {
        self.notification.metadata.insert(key.to_string(), value);
        self
    }
    
    pub fn build(self) -> Notification {
        self.notification
    }
}

/// Event subscription helper
pub struct EventSubscription {
    plugin_id: Uuid,
    event_patterns: Vec<String>,
}

impl EventSubscription {
    pub fn new(plugin_id: Uuid) -> Self {
        Self {
            plugin_id,
            event_patterns: Vec::new(),
        }
    }
    
    pub fn subscribe_to(mut self, pattern: &str) -> Self {
        self.event_patterns.push(pattern.to_string());
        self
    }
    
    pub fn subscribe_to_all_submissions(self) -> Self {
        self.subscribe_to("submission.*")
    }
    
    pub fn subscribe_to_all_contests(self) -> Self {
        self.subscribe_to("contest.*")
    }
    
    pub fn subscribe_to_user_events(self) -> Self {
        self.subscribe_to("user.*")
    }
    
    pub fn subscribe_to_judging_events(self) -> Self {
        self.subscribe_to("judging.*")
    }
    
    // In a real implementation, this would register the subscriptions with the platform
    pub async fn register(self) -> PluginResult<()> {
        // Platform API call to register event subscriptions
        Ok(())
    }
}

/// Configuration management utilities
pub struct ConfigManager {
    config_values: HashMap<String, Value>,
}

impl ConfigManager {
    pub fn new(config_values: HashMap<String, Value>) -> Self {
        Self { config_values }
    }
    
    pub fn get_string(&self, key: &str) -> PluginResult<String> {
        self.config_values
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| PluginError::ConfigurationError(format!("Missing config key: {}", key)))
    }
    
    pub fn get_bool(&self, key: &str) -> PluginResult<bool> {
        self.config_values
            .get(key)
            .and_then(|v| v.as_bool())
            .ok_or_else(|| PluginError::ConfigurationError(format!("Missing config key: {}", key)))
    }
    
    pub fn get_number(&self, key: &str) -> PluginResult<f64> {
        self.config_values
            .get(key)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| PluginError::ConfigurationError(format!("Missing config key: {}", key)))
    }
    
    pub fn get_array(&self, key: &str) -> PluginResult<Vec<Value>> {
        self.config_values
            .get(key)
            .and_then(|v| v.as_array())
            .map(|arr| arr.clone())
            .ok_or_else(|| PluginError::ConfigurationError(format!("Missing config key: {}", key)))
    }
    
    pub fn get_object(&self, key: &str) -> PluginResult<HashMap<String, Value>> {
        self.config_values
            .get(key)
            .and_then(|v| v.as_object())
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .ok_or_else(|| PluginError::ConfigurationError(format!("Missing config key: {}", key)))
    }
    
    pub fn get_optional_string(&self, key: &str) -> Option<String> {
        self.get_string(key).ok()
    }
    
    pub fn get_optional_bool(&self, key: &str) -> Option<bool> {
        self.get_bool(key).ok()
    }
    
    pub fn get_optional_number(&self, key: &str) -> Option<f64> {
        self.get_number(key).ok()
    }
}

/// Utility functions for working with plugin data
pub mod utils {
    use super::*;
    
    /// Validate that all required capabilities are granted
    pub fn validate_capabilities(
        required: &[Capability],
        granted: &[Capability],
    ) -> PluginResult<()> {
        for capability in required {
            if !granted.contains(capability) {
                return Err(PluginError::MissingCapability(capability.to_string()));
            }
        }
        Ok(())
    }
    
    /// Parse a semver version string
    pub fn parse_version(version: &str) -> PluginResult<(u32, u32, u32)> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return Err(PluginError::InvalidInput("Invalid version format".to_string()));
        }
        
        let major = parts[0].parse::<u32>()
            .map_err(|_| PluginError::InvalidInput("Invalid major version".to_string()))?;
        let minor = parts[1].parse::<u32>()
            .map_err(|_| PluginError::InvalidInput("Invalid minor version".to_string()))?;
        let patch = parts[2].parse::<u32>()
            .map_err(|_| PluginError::InvalidInput("Invalid patch version".to_string()))?;
        
        Ok((major, minor, patch))
    }
    
    /// Check if a version satisfies a requirement
    pub fn version_satisfies(version: &str, requirement: &str) -> PluginResult<bool> {
        let (v_major, v_minor, v_patch) = parse_version(version)?;
        let (r_major, r_minor, r_patch) = parse_version(requirement)?;
        
        // Simple semantic version checking - in production this would be more sophisticated
        Ok(v_major >= r_major && v_minor >= r_minor && v_patch >= r_patch)
    }
    
    /// Generate a plugin-specific cache key
    pub fn cache_key(plugin_id: Uuid, key: &str) -> String {
        format!("plugin:{}:{}", plugin_id, key)
    }
    
    /// Sanitize user input for safe use in queries
    pub fn sanitize_input(input: &str) -> String {
        input.replace('\'', "''").replace(';', "")
    }
}