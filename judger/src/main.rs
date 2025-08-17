mod config;
mod coordinator;
mod database;
mod executor;
mod sandbox;

use anyhow::Result;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::Config,
    coordinator::Coordinator,
    database::Database,
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "judicia_judger=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Arc::new(Config::from_env()?);
    let db = Database::new(&config.database_url).await?;
    
    let coordinator = Coordinator::new(config, db).await?;
    
    tracing::info!("Starting Judicia Judger...");
    coordinator.start().await?;
    
    Ok(())
}