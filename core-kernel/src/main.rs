mod config;
mod database;
mod event_handlers;
mod handlers;
mod kernel;
mod plugin_router;

use anyhow::Result;
use axum::{
    http::{HeaderName, HeaderValue, Method},
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::Config,
    database::Database,
    kernel::JudiciaKernel,
};

#[derive(Clone)]
pub struct KernelState {
    pub kernel: Arc<JudiciaKernel>,
    pub db: sqlx::PgPool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "judicia_kernel=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    println!("🚀 Starting Judicia Core Kernel...");

    let config = Arc::new(Config::from_env()?);
    
    // Check if we're in test mode (skip database)
    let test_mode = std::env::var("TEST_MODE").unwrap_or_else(|_| "false".to_string()) == "true";
    
    let (kernel, db_pool) = if test_mode {
        println!("⚠️  Running in TEST MODE - database disabled");
        // Create kernel without database for testing plugin loading only
        let kernel = Arc::new(JudiciaKernel::new_test_mode(config.clone()).await?);
        // Create a mock pool for test mode
        let mock_pool = sqlx::PgPool::connect("postgresql://localhost/test").await.unwrap_or_else(|_| {
            // If that fails, create a minimal mock pool
            panic!("Cannot create test database pool")
        });
        (kernel, mock_pool)
    } else {
        let db = Database::new(&config.database_url).await?;
        
        // Run migrations
        db.migrate().await?;
        println!("✅ Database connected and migrations applied");
        
        let db_pool = db.pool().clone();
        
        // Initialize the Judicia Kernel
        let kernel = Arc::new(JudiciaKernel::new(config.clone(), db).await?);
        (kernel, db_pool)
    };
    
    // Load plugins
    kernel.load_plugins().await?;
    
    // Start event handlers
    kernel.start_event_handlers().await?;
    
    let kernel_state = KernelState { kernel, db: db_pool };

    println!("🔧 Setting up CORS...");
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH])
        .allow_headers([
            "content-type".parse::<HeaderName>()?,
            "authorization".parse::<HeaderName>()?,
            "x-requested-with".parse::<HeaderName>()?,
        ])
        .allow_credentials(true);

    println!("🌐 Setting up plugin-aware routes...");
    let app = plugin_router::create_plugin_router()
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors),
        )
        .with_state(kernel_state);

    println!("🎯 Binding to server address: {}", config.server_address);
    let listener = tokio::net::TcpListener::bind(&config.server_address).await?;
    
    tracing::info!("🎉 Judicia Core Kernel running on {}", config.server_address);
    println!("🎉 Judicia Core Kernel started successfully!");
    println!("   📋 Available endpoints:");
    println!("   - GET  /api/health       - Kernel health check");
    println!("   - GET  /api/plugins      - List loaded plugins");
    println!("   - *    /*                - Plugin request routing");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}