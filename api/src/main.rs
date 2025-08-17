mod config;
mod auth;
mod handlers;
mod database;
mod queue;
mod middleware;
mod websocket;

use anyhow::Result;
use axum::{
    http::{HeaderName, HeaderValue, Method},
    routing::{get, post},
    Router,
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
    handlers::*,
    queue::Queue,
    websocket::websocket_handler,
};

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub queue: Arc<Queue>,
    pub config: Arc<Config>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "judicia_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Arc::new(Config::from_env()?);
    let db = Database::new(&config.database_url).await?;
    
    // Run migrations
    db.migrate().await?;
    
    println!("Database connected and migrations run successfully");
    
    let queue = Queue::new(&config.rabbitmq_url).await?;
    println!("Queue connected successfully");

    let app_state = AppState {
        db,
        queue: Arc::new(queue),
        config: config.clone(),
    };

    println!("Setting up CORS...");
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([
            "content-type".parse::<HeaderName>()?,
            "authorization".parse::<HeaderName>()?,
            "x-requested-with".parse::<HeaderName>()?,
        ])
        .allow_credentials(true);

    println!("Setting up routes...");
    let protected_routes = Router::new()
        // Protected routes (requiring authentication)
        .route("/api/auth/me", get(auth_handlers::me))
        .route("/api/problems", post(problem_handlers::create_problem))
        .route("/api/submissions", post(submission_handlers::submit_code))
        .route("/api/submissions/:id", get(submission_handlers::get_submission))
        .route("/api/problems/:id/submissions", get(submission_handlers::get_problem_submissions))
        .route("/api/contests", post(contest_handlers::create_contest))
        .route("/api/contest-admins", post(contest_admin_handlers::assign_contest_admin))
        .route("/api/contest-admins/:contest_id/:user_id", axum::routing::delete(contest_admin_handlers::remove_contest_admin))
        .route("/api/contests/:id/admins", get(contest_admin_handlers::list_contest_admins))
        .route("/api/contest-admins/:contest_id/:user_id/check", get(contest_admin_handlers::check_contest_admin))
        .route("/api/my/administered-contests", get(contest_admin_handlers::get_administered_contests))
        .route_layer(axum::middleware::from_fn_with_state(app_state.clone(), crate::auth::auth_middleware));

    let app = Router::new()
        // Public auth routes
        .route("/api/auth/login", post(auth_handlers::login))
        .route("/api/auth/register", post(auth_handlers::register))
        .route("/api/auth/register-admin", post(auth_handlers::register_admin))
        // Public routes
        .route("/api/problems", get(problem_handlers::list_problems))
        .route("/api/problems/:id", get(problem_handlers::get_problem))
        .route("/api/languages", get(language_handlers::list_languages))
        .route("/api/contests", get(contest_handlers::list_contests))
        .route("/api/contests/:id", get(contest_handlers::get_contest))
        .route("/api/contests/:id/problems", get(contest_handlers::get_contest_problems))
        // Merge protected routes
        .merge(protected_routes)
        // WebSocket route
        .route("/api/submissions/:id/stream", get(websocket_handler))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors),
        )
        .with_state(app_state);

    println!("Binding to server address: {}", config.server_address);
    let listener = tokio::net::TcpListener::bind(&config.server_address).await?;
    tracing::info!("Server running on {}", config.server_address);
    println!("Server started successfully on {}", config.server_address);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}