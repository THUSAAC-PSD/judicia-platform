use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database_url: String,
    pub rabbitmq_url: String,
    pub work_dir: String,
    pub max_concurrent_jobs: usize,
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
            .unwrap_or_else(|_| "4".to_string())
            .parse()
            .unwrap_or(4);

        Ok(Config {
            database_url,
            rabbitmq_url,
            work_dir,
            max_concurrent_jobs,
        })
    }
}