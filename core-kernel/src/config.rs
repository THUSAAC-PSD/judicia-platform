use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server_address: String,
    pub database_url: String,
    pub redis_url: String,
    pub rabbitmq_url: String,
    pub jwt_secret: String,
    pub supabase_url: Option<String>,
    pub supabase_anon_key: Option<String>,
    pub supabase_service_role_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // Check if Supabase configuration is provided
        let supabase_url = std::env::var("SUPABASE_URL").ok();
        let supabase_anon_key = std::env::var("SUPABASE_ANON_KEY").ok();
        let supabase_service_role_key = std::env::var("SUPABASE_SERVICE_ROLE_KEY").ok();
        
        // Build database URL - prefer explicit DATABASE_URL, fall back to Supabase, then local
        let database_url = if let Ok(explicit_url) = std::env::var("DATABASE_URL") {
            explicit_url
        } else if let Some(ref supabase_url) = supabase_url {
            // Convert Supabase URL to PostgreSQL connection string
            // Supabase provides the database URL in the project settings
            supabase_url.clone()
        } else {
            "postgresql://localhost/judicia".to_string()
        };
        
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        
        let rabbitmq_url = std::env::var("RABBITMQ_URL")
            .unwrap_or_else(|_| "amqp://localhost:5672".to_string());
        
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string());
        
        let server_address = std::env::var("SERVER_ADDRESS")
            .unwrap_or_else(|_| "0.0.0.0:5000".to_string());

        Ok(Config {
            server_address,
            database_url,
            redis_url,
            rabbitmq_url,
            jwt_secret,
            supabase_url,
            supabase_anon_key,
            supabase_service_role_key,
        })
    }
    
    /// Check if Supabase configuration is available
    pub fn is_supabase_enabled(&self) -> bool {
        self.supabase_url.is_some() && self.supabase_service_role_key.is_some()
    }
    
    /// Get the Supabase REST API URL
    pub fn supabase_rest_url(&self) -> Option<String> {
        self.supabase_url.as_ref().map(|url| format!("{}/rest/v1", url))
    }
    
    /// Get the Supabase Auth URL
    pub fn supabase_auth_url(&self) -> Option<String> {
        self.supabase_url.as_ref().map(|url| format!("{}/auth/v1", url))
    }
}