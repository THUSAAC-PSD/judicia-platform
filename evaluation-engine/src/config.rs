use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database_url: String,
    pub rabbitmq_url: String,
    pub work_dir: String,
    pub max_concurrent_jobs: Option<i32>,
    pub worker_count: Option<i32>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: "postgresql://localhost/judicia".to_string(),
            rabbitmq_url: "amqp://localhost:5672".to_string(),
            work_dir: "/tmp/judicia".to_string(),
            max_concurrent_jobs: Some(4),
            worker_count: Some(2),
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/judicia".to_string());
        
        let rabbitmq_url = std::env::var("RABBITMQ_URL")
            .unwrap_or_else(|_| "amqp://localhost:5672".to_string());
        
        let work_dir = std::env::var("WORK_DIR")
            .unwrap_or_else(|_| "/tmp/judicia".to_string());
        
        let max_concurrent_jobs = std::env::var("MAX_CONCURRENT_JOBS")
            .ok()
            .and_then(|s| s.parse().ok());
            
        let worker_count = std::env::var("WORKER_COUNT")
            .ok()
            .and_then(|s| s.parse().ok());

        Ok(Config {
            database_url,
            rabbitmq_url,
            work_dir,
            max_concurrent_jobs,
            worker_count,
        })
    }
}