use anyhow::Result;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use evaluation_engine::{Config, EvaluationCoordinator};

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
    
    let coordinator = EvaluationCoordinator::new(config).await?;
    
    tracing::info!("Starting Judicia Evaluation Engine...");
    coordinator.start().await?;
    
    Ok(())
}