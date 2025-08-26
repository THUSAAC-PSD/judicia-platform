//! Error types and handling for Judicia plugins

use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasm_bindgen::prelude::*;

/// Main error type for plugin operations
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum PluginError {
    /// Plugin initialization failed
    #[error("Plugin initialization failed: {0}")]
    InitializationError(String),
    
    /// Plugin configuration is invalid
    #[error("Invalid plugin configuration: {0}")]
    ConfigurationError(String),
    
    /// Required capability not granted
    #[error("Missing required capability: {0}")]
    MissingCapability(String),
    
    /// Platform API call failed
    #[error("Platform API error: {0}")]
    PlatformApiError(String),
    
    /// Database operation failed
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    /// Network/HTTP operation failed
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// File system operation failed
    #[error("File system error: {0}")]
    FileSystemError(String),
    
    /// Plugin method not implemented
    #[error("Method not implemented: {0}")]
    NotImplemented(String),
    
    /// Invalid input provided
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    /// Resource limit exceeded
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    /// Plugin timeout
    #[error("Operation timed out: {0}")]
    Timeout(String),
    
    /// Generic plugin error
    #[error("Plugin error: {0}")]
    Generic(String),
    
    /// JavaScript/WebAssembly interface error
    #[error("JavaScript error: {0}")]
    JavaScriptError(String),
    
    /// Plugin security violation
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    
    /// Plugin version compatibility error
    #[error("Version compatibility error: {0}")]
    VersionError(String),
    
    /// Plugin dependency error
    #[error("Dependency error: {0}")]
    DependencyError(String),
}

impl From<JsValue> for PluginError {
    fn from(js_value: JsValue) -> Self {
        if let Some(string_value) = js_value.as_string() {
            PluginError::JavaScriptError(string_value)
        } else {
            PluginError::JavaScriptError("Unknown JavaScript error".to_string())
        }
    }
}

impl Into<JsValue> for PluginError {
    fn into(self) -> JsValue {
        JsValue::from_str(&self.to_string())
    }
}

impl From<serde_json::Error> for PluginError {
    fn from(error: serde_json::Error) -> Self {
        PluginError::SerializationError(error.to_string())
    }
}

impl From<anyhow::Error> for PluginError {
    fn from(error: anyhow::Error) -> Self {
        PluginError::Generic(error.to_string())
    }
}


/// Error handling utilities for plugins
pub struct ErrorHandler;

impl ErrorHandler {
    /// Log an error with proper context
    pub fn log_error(error: &PluginError, context: Option<&str>, plugin_name: Option<&str>) {
        let context_str = context.unwrap_or("unknown");
        let plugin_str = plugin_name.unwrap_or("unknown plugin");
        
        let error_data = serde_json::json!({
            "plugin": plugin_str,
            "context": context_str,
            "error_type": format!("{:?}", error),
            "error_message": error.to_string(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        // Log to platform logging system
        use crate::types::log_message;
        log_message("error", &error.to_string(), &serde_wasm_bindgen::to_value(&error_data).unwrap_or(JsValue::NULL));
    }
    
    /// Create a user-friendly error message
    pub fn user_message(error: &PluginError) -> String {
        match error {
            PluginError::MissingCapability(cap) => {
                format!("This plugin requires the '{}' permission to function properly.", cap)
            }
            PluginError::ConfigurationError(_) => {
                "Plugin configuration is invalid. Please check your settings.".to_string()
            }
            PluginError::NetworkError(_) => {
                "Network connection error. Please check your internet connection.".to_string()
            }
            PluginError::DatabaseError(_) => {
                "Database operation failed. Please try again later.".to_string()
            }
            PluginError::Timeout(_) => {
                "Operation timed out. Please try again.".to_string()
            }
            PluginError::SecurityViolation(_) => {
                "Access denied. You don't have permission to perform this action.".to_string()
            }
            _ => "An unexpected error occurred. Please contact support.".to_string(),
        }
    }
    
    /// Convert error to HTTP status code
    pub fn to_http_status(error: &PluginError) -> u16 {
        match error {
            PluginError::MissingCapability(_) | PluginError::SecurityViolation(_) => 403,
            PluginError::InvalidInput(_) | PluginError::ConfigurationError(_) => 400,
            PluginError::NotImplemented(_) => 501,
            PluginError::Timeout(_) => 408,
            PluginError::NetworkError(_) => 503,
            PluginError::DatabaseError(_) => 503,
            _ => 500,
        }
    }
}

/// Create a plugin error with context
pub fn plugin_error(error_type: &str, msg: &str) -> PluginError {
    match error_type {
        "InitializationError" => PluginError::InitializationError(msg.to_string()),
        "ConfigurationError" => PluginError::ConfigurationError(msg.to_string()),
        "MissingCapability" => PluginError::MissingCapability(msg.to_string()),
        "DatabaseError" => PluginError::DatabaseError(msg.to_string()),
        "NetworkError" => PluginError::NetworkError(msg.to_string()),
        "InvalidInput" => PluginError::InvalidInput(msg.to_string()),
        "SecurityViolation" => PluginError::SecurityViolation(msg.to_string()),
        _ => PluginError::Generic(msg.to_string()),
    }
}

// Re-import logging function from platform
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["judicia", "platform"])]
    fn log_message(level: &str, message: &str, metadata: &JsValue);
}

/// Logging utilities for plugins
pub struct Logger;

impl Logger {
    pub fn debug(message: &str, metadata: Option<serde_json::Value>) {
        Self::log("debug", message, metadata);
    }
    
    pub fn info(message: &str, metadata: Option<serde_json::Value>) {
        Self::log("info", message, metadata);
    }
    
    pub fn warn(message: &str, metadata: Option<serde_json::Value>) {
        Self::log("warn", message, metadata);
    }
    
    pub fn error(message: &str, metadata: Option<serde_json::Value>) {
        Self::log("error", message, metadata);
    }
    
    fn log(level: &str, message: &str, metadata: Option<serde_json::Value>) {
        let metadata_js = metadata
            .and_then(|m| serde_wasm_bindgen::to_value(&m).ok())
            .unwrap_or(JsValue::NULL);
            
        use crate::types::log_message;
        log_message(level, message, &metadata_js);
    }
}