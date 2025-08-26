//! Utility functions and helpers for Judicia plugin development

use crate::{error::PluginError, types::*};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;
use wasm_bindgen::prelude::*;

/// Time and date utilities
pub mod time {
    use super::*;
    
    /// Get current UTC timestamp
    pub fn now() -> DateTime<Utc> {
        Utc::now()
    }
    
    /// Format timestamp for display
    pub fn format_timestamp(timestamp: DateTime<Utc>, format: &str) -> String {
        timestamp.format(format).to_string()
    }
    
    /// Get human-readable time ago string
    pub fn time_ago(timestamp: DateTime<Utc>) -> String {
        let duration = Utc::now().signed_duration_since(timestamp);
        
        if duration.num_seconds() < 60 {
            "just now".to_string()
        } else if duration.num_minutes() < 60 {
            format!("{} minutes ago", duration.num_minutes())
        } else if duration.num_hours() < 24 {
            format!("{} hours ago", duration.num_hours())
        } else if duration.num_days() < 30 {
            format!("{} days ago", duration.num_days())
        } else if duration.num_days() < 365 {
            format!("{} months ago", duration.num_days() / 30)
        } else {
            format!("{} years ago", duration.num_days() / 365)
        }
    }
    
    /// Parse ISO 8601 timestamp
    pub fn parse_iso8601(timestamp: &str) -> PluginResult<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(timestamp)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| PluginError::InvalidInput(format!("Invalid timestamp: {}", e)))
    }
    
    /// Get start of day in UTC
    pub fn start_of_day(date: DateTime<Utc>) -> DateTime<Utc> {
        date.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc()
    }
    
    /// Get end of day in UTC
    pub fn end_of_day(date: DateTime<Utc>) -> DateTime<Utc> {
        date.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc()
    }
}

/// String manipulation utilities
pub mod strings {
    use super::*;
    
    /// Truncate string to specified length with ellipsis
    pub fn truncate(text: &str, max_length: usize) -> String {
        if text.len() <= max_length {
            text.to_string()
        } else {
            format!("{}...", &text[..max_length.saturating_sub(3)])
        }
    }
    
    /// Convert string to slug format (lowercase, hyphens)
    pub fn slugify(text: &str) -> String {
        text.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }
    
    /// Capitalize first letter of each word
    pub fn title_case(text: &str) -> String {
        text.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
    }
    
    /// Extract file extension from filename
    pub fn file_extension(filename: &str) -> Option<String> {
        filename.rfind('.')
            .map(|pos| filename[pos + 1..].to_lowercase())
    }
    
    /// Generate random string of specified length
    pub fn random_string(length: usize) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        Utc::now().timestamp_nanos_opt().unwrap_or(0).hash(&mut hasher);
        
        let hash = hasher.finish();
        let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        
        (0..length)
            .map(|i| {
                let index = ((hash >> (i % 8)) % chars.len() as u64) as usize;
                chars.chars().nth(index).unwrap_or('a')
            })
            .collect()
    }
    
    /// Escape HTML entities
    pub fn escape_html(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }
    
    /// Clean and normalize whitespace
    pub fn normalize_whitespace(text: &str) -> String {
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

/// JSON manipulation utilities
pub mod json {
    use super::*;
    
    /// Safely get nested JSON value with path notation (e.g., "user.profile.name")
    pub fn get_nested<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
        path.split('.')
            .fold(Some(value), |current, key| {
                current.and_then(|v| v.get(key))
            })
    }
    
    /// Set nested JSON value with path notation
    pub fn set_nested(value: &mut Value, path: &str, new_value: Value) -> PluginResult<()> {
        let keys: Vec<&str> = path.split('.').collect();
        let last_key = keys.last().ok_or_else(|| {
            PluginError::InvalidInput("Empty path".to_string())
        })?;
        
        let mut current = value;
        for key in &keys[..keys.len() - 1] {
            if !current.is_object() {
                *current = serde_json::json!({});
            }
            let obj = current.as_object_mut()
                .ok_or_else(|| PluginError::InvalidInput("Invalid path".to_string()))?;
            current = obj.entry(*key).or_insert_with(|| serde_json::json!({}));
        }
        let parent = current;
        
        if let Some(obj) = parent.as_object_mut() {
            obj.insert(last_key.to_string(), new_value);
        }
        
        Ok(())
    }
    
    /// Merge two JSON objects recursively
    pub fn merge_objects(target: &mut Value, source: &Value) {
        if let (Some(target_obj), Some(source_obj)) = (target.as_object_mut(), source.as_object()) {
            for (key, value) in source_obj {
                if target_obj.contains_key(key) {
                    merge_objects(&mut target_obj[key], value);
                } else {
                    target_obj.insert(key.clone(), value.clone());
                }
            }
        } else {
            *target = source.clone();
        }
    }
    
    /// Convert JSON value to query string
    pub fn to_query_string(value: &Value) -> String {
        match value.as_object() {
            Some(obj) => {
                obj.iter()
                    .filter_map(|(key, val)| {
                        val.as_str().map(|s| format!("{}={}", 
                            urlencoding::encode(key), 
                            urlencoding::encode(s)
                        ))
                    })
                    .collect::<Vec<_>>()
                    .join("&")
            }
            _ => String::new(),
        }
    }
    
    /// Parse query string to JSON object
    pub fn from_query_string(query: &str) -> Value {
        let mut result = serde_json::Map::new();
        
        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                if let (Ok(decoded_key), Ok(decoded_value)) = (
                    urlencoding::decode(key),
                    urlencoding::decode(value)
                ) {
                    result.insert(decoded_key.to_string(), Value::String(decoded_value.to_string()));
                }
            }
        }
        
        Value::Object(result)
    }
}

/// Validation utilities
pub mod validation {
    use super::*;
    
    /// Validate email address format
    pub fn is_valid_email(email: &str) -> bool {
        let email_regex = regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
        email_regex.is_match(email)
    }
    
    /// Validate UUID format
    pub fn is_valid_uuid(uuid_str: &str) -> bool {
        Uuid::parse_str(uuid_str).is_ok()
    }
    
    /// Validate that string is not empty or just whitespace
    pub fn is_non_empty(text: &str) -> bool {
        !text.trim().is_empty()
    }
    
    /// Validate string length
    pub fn validate_length(text: &str, min: usize, max: usize) -> PluginResult<()> {
        let len = text.len();
        if len < min {
            Err(PluginError::InvalidInput(
                format!("Text too short (minimum {} characters)", min)
            ))
        } else if len > max {
            Err(PluginError::InvalidInput(
                format!("Text too long (maximum {} characters)", max)
            ))
        } else {
            Ok(())
        }
    }
    
    /// Validate that value is within numeric range
    pub fn validate_range<T: PartialOrd + std::fmt::Display>(
        value: T, 
        min: T, 
        max: T
    ) -> PluginResult<()> {
        if value < min {
            Err(PluginError::InvalidInput(
                format!("Value {} is below minimum {}", value, min)
            ))
        } else if value > max {
            Err(PluginError::InvalidInput(
                format!("Value {} is above maximum {}", value, max)
            ))
        } else {
            Ok(())
        }
    }
    
    /// Validate that required fields are present in JSON object
    pub fn validate_required_fields(
        data: &Value,
        required_fields: &[&str]
    ) -> PluginResult<()> {
        let obj = data.as_object().ok_or_else(|| {
            PluginError::InvalidInput("Expected JSON object".to_string())
        })?;
        
        for field in required_fields {
            if !obj.contains_key(*field) {
                return Err(PluginError::InvalidInput(
                    format!("Missing required field: {}", field)
                ));
            }
        }
        
        Ok(())
    }
}

/// Caching utilities for plugins
pub mod cache {
    use super::*;
    use std::collections::HashMap;
    
    /// Simple in-memory cache with TTL
    pub struct Cache {
        data: HashMap<String, CacheEntry>,
        default_ttl_seconds: u64,
    }
    
    struct CacheEntry {
        value: Value,
        expires_at: DateTime<Utc>,
    }
    
    impl Cache {
        pub fn new(default_ttl_seconds: u64) -> Self {
            Self {
                data: HashMap::new(),
                default_ttl_seconds,
            }
        }
        
        pub fn get(&mut self, key: &str) -> Option<&Value> {
            self.cleanup_expired();
            self.data.get(key).map(|entry| &entry.value)
        }
        
        pub fn set(&mut self, key: &str, value: Value, ttl_seconds: Option<u64>) {
            let ttl = ttl_seconds.unwrap_or(self.default_ttl_seconds);
            let expires_at = Utc::now() + chrono::Duration::seconds(ttl as i64);
            
            self.data.insert(key.to_string(), CacheEntry {
                value,
                expires_at,
            });
        }
        
        pub fn remove(&mut self, key: &str) -> Option<Value> {
            self.data.remove(key).map(|entry| entry.value)
        }
        
        pub fn clear(&mut self) {
            self.data.clear();
        }
        
        pub fn size(&self) -> usize {
            self.data.len()
        }
        
        fn cleanup_expired(&mut self) {
            let now = Utc::now();
            self.data.retain(|_, entry| entry.expires_at > now);
        }
    }
}

/// HTTP utilities for plugins
pub mod http {
    use super::*;
    
    /// Build HTTP response with common headers
    pub fn response(status: u16, body: &str) -> HttpResponse {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("X-Plugin-Response".to_string(), "true".to_string());
        
        HttpResponse {
            status_code: status,
            headers,
            body: body.to_string(),
            content_type: "application/json".to_string(),
        }
    }
    
    /// Build JSON response
    pub fn json_response<T: serde::Serialize>(status: u16, data: &T) -> PluginResult<HttpResponse> {
        let body = serde_json::to_string(data)
            .map_err(|e| PluginError::SerializationError(e.to_string()))?;
        
        Ok(response(status, &body))
    }
    
    /// Build error response
    pub fn error_response(status: u16, message: &str) -> HttpResponse {
        let error_body = serde_json::json!({
            "error": true,
            "message": message,
            "status": status
        });
        
        response(status, &error_body.to_string())
    }
    
    /// Extract Bearer token from Authorization header
    pub fn extract_bearer_token(headers: &HashMap<String, String>) -> Option<String> {
        headers.get("Authorization")
            .or_else(|| headers.get("authorization"))
            .and_then(|auth| {
                if auth.starts_with("Bearer ") {
                    Some(auth[7..].to_string())
                } else {
                    None
                }
            })
    }
    
    /// Parse Content-Type header
    pub fn parse_content_type(headers: &HashMap<String, String>) -> (String, HashMap<String, String>) {
        let content_type = headers.get("Content-Type")
            .or_else(|| headers.get("content-type"))
            .unwrap_or(&"text/plain".to_string())
            .clone();
        
        let mut params = HashMap::new();
        let parts: Vec<&str> = content_type.split(';').collect();
        let main_type = parts[0].trim().to_string();
        
        for part in &parts[1..] {
            if let Some((key, value)) = part.split_once('=') {
                params.insert(
                    key.trim().to_string(),
                    value.trim().trim_matches('"').to_string()
                );
            }
        }
        
        (main_type, params)
    }
}

/// Logging utilities
pub mod logging {
    use super::*;
    
    /// Log levels
    #[derive(Debug, Clone, Copy)]
    pub enum LogLevel {
        Trace,
        Debug,
        Info,
        Warn,
        Error,
    }
    
    impl LogLevel {
        pub fn as_str(&self) -> &str {
            match self {
                LogLevel::Trace => "trace",
                LogLevel::Debug => "debug", 
                LogLevel::Info => "info",
                LogLevel::Warn => "warn",
                LogLevel::Error => "error",
            }
        }
    }
    
    /// Structured log entry
    #[derive(Debug, Clone)]
    pub struct LogEntry {
        pub level: LogLevel,
        pub message: String,
        pub metadata: HashMap<String, Value>,
        pub timestamp: DateTime<Utc>,
    }
    
    impl LogEntry {
        pub fn new(level: LogLevel, message: &str) -> Self {
            Self {
                level,
                message: message.to_string(),
                metadata: HashMap::new(),
                timestamp: Utc::now(),
            }
        }
        
        pub fn with_metadata(mut self, key: &str, value: Value) -> Self {
            self.metadata.insert(key.to_string(), value);
            self
        }
        
        pub fn emit(self) {
            let metadata = serde_json::to_value(&self.metadata).unwrap_or(Value::Null);
            use crate::types::log_message;
            log_message(
                self.level.as_str(),
                &self.message,
                &serde_wasm_bindgen::to_value(&metadata).unwrap_or(JsValue::NULL)
            );
        }
    }
    
    /// Log macros (would be actual macros in real implementation)
    pub fn trace(message: &str) {
        LogEntry::new(LogLevel::Trace, message).emit();
    }
    
    pub fn debug(message: &str) {
        LogEntry::new(LogLevel::Debug, message).emit();
    }
    
    pub fn info(message: &str) {
        LogEntry::new(LogLevel::Info, message).emit();
    }
    
    pub fn warn(message: &str) {
        LogEntry::new(LogLevel::Warn, message).emit();
    }
    
    pub fn error(message: &str) {
        LogEntry::new(LogLevel::Error, message).emit();
    }
}

// URL encoding utilities (simple implementation)
mod urlencoding {
    pub fn encode(input: &str) -> String {
        input.chars()
            .map(|c| match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                _ => format!("%{:02X}", c as u8),
            })
            .collect()
    }
    
    pub fn decode(input: &str) -> Result<String, std::str::Utf8Error> {
        let mut result = Vec::new();
        let mut chars = input.chars();
        
        while let Some(c) = chars.next() {
            match c {
                '%' => {
                    let hex: String = chars.by_ref().take(2).collect();
                    if hex.len() == 2 {
                        if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                            result.push(byte);
                        }
                    }
                }
                _ => result.extend_from_slice(c.to_string().as_bytes()),
            }
        }
        
        std::str::from_utf8(&result).map(|s| s.to_string())
    }
}

// Simple regex implementation (in real code, would use actual regex crate)
mod regex {
    pub struct Regex {
        pattern: String,
    }
    
    impl Regex {
        pub fn new(pattern: &str) -> Result<Self, &'static str> {
            Ok(Self {
                pattern: pattern.to_string(),
            })
        }
        
        pub fn is_match(&self, text: &str) -> bool {
            // Simplified email validation - in real implementation would use proper regex
            if self.pattern.contains("@") {
                text.contains('@') && text.contains('.')
            } else {
                false
            }
        }
    }
}