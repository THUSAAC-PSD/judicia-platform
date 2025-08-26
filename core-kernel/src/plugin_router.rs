use axum::{
    extract::{Path as AxumPath, State},
    http::{Method, StatusCode, Uri},
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{KernelState, handlers::{plugin_handlers, contest_extension_handlers, scoring_handlers, rejudge_handlers}};
use shared::{PluginHttpRoute, PluginPermission};

/// Plugin-aware router that forwards requests to appropriate plugins
pub fn create_plugin_router() -> Router<KernelState> {
    Router::new()
        // Plugin management routes
        .route("/api/plugins", get(list_plugins))
        .route("/api/plugins/routes", get(list_plugin_routes))
        .route("/api/plugins/discovery", get(plugin_handlers::get_plugin_discovery))
        .route("/api/plugins/:id/metadata", get(plugin_handlers::get_plugin_metadata))
        .route("/api/plugins/health", get(plugin_handlers::get_plugin_health))
        .route("/api/health", get(health_check))
        
        // Contest extension routes
        .route("/api/contests/:contest_id/extensions", post(contest_extension_handlers::create_contest_extension))
        .route("/api/contests/:contest_id/extensions", get(contest_extension_handlers::get_contest_extensions))
        .route("/api/contests/:contest_id/extensions/:extension_id", put(contest_extension_handlers::update_contest_extension))
        .route("/api/contests/:contest_id/extensions/:extension_id", delete(contest_extension_handlers::delete_contest_extension))
        .route("/api/contests/:contest_id/users/:user_id/time", get(contest_extension_handlers::get_user_contest_time))
        
        // Scoring system routes
        .route("/api/contests/:contest_id/scoring/method", put(scoring_handlers::update_contest_scoring_method))
        .route("/api/contests/:contest_id/scoring/scores", get(scoring_handlers::get_contest_scores))
        .route("/api/contests/:contest_id/users/:user_id/problems/:problem_id/score", get(scoring_handlers::get_user_problem_score_detail))
        .route("/api/contests/:contest_id/scoring/recalculate", post(scoring_handlers::recalculate_contest_scores_endpoint))
        
        // Rejudge system routes
        .route("/api/rejudge/requests", post(rejudge_handlers::create_rejudge_request))
        .route("/api/rejudge/jobs", get(rejudge_handlers::list_rejudge_jobs))
        .route("/api/rejudge/jobs/:job_id", get(rejudge_handlers::get_rejudge_job))
        .route("/api/rejudge/jobs/:job_id", delete(rejudge_handlers::cancel_rejudge_job))
        .route("/api/rejudge/batch", post(rejudge_handlers::batch_rejudge_submissions))
        
        // Catch-all route for plugin requests
        .fallback(plugin_handler)
}

/// List all loaded plugins
async fn list_plugins(State(state): State<KernelState>) -> Json<Value> {
    let plugins = state.kernel.list_plugins();
    Json(json!({
        "plugins": plugins,
        "count": plugins.len()
    }))
}

/// List all available plugin routes
async fn list_plugin_routes(State(state): State<KernelState>) -> Json<Value> {
    let plugins = state.kernel.list_plugins();
    let mut all_routes = Vec::new();
    let plugins_count = plugins.len();
    
    for plugin in plugins {
        // Generate the available routes for each plugin
        let plugin_routes = vec![
            json!({
                "plugin_id": plugin.id,
                "plugin_name": plugin.name,
                "path": format!("/api/{}/handle", plugin.name),
                "method": "POST",
                "handler_function": "handle_request",
                "description": "Main handler for plugin requests",
                "required_permission": null,
                "rate_limit": null
            }),
            json!({
                "plugin_id": plugin.id,
                "plugin_name": plugin.name,
                "path": format!("/api/{}/info", plugin.name),
                "method": "GET", 
                "handler_function": "get_info",
                "description": "Get plugin information",
                "required_permission": null,
                "rate_limit": null
            }),
        ];
        all_routes.extend(plugin_routes);
    }
    
    Json(json!({
        "routes": all_routes,
        "count": all_routes.len(),
        "plugins_with_routes": plugins_count
    }))
}

/// Health check endpoint
async fn health_check(State(state): State<KernelState>) -> Json<Value> {
    let healthy = state.kernel.health();
    Json(json!({
        "status": if healthy { "healthy" } else { "unhealthy" },
        "kernel": "judicia-core",
        "version": "0.1.0"
    }))
}

/// Main plugin request handler
/// This is where all plugin requests are routed
async fn plugin_handler(
    method: Method,
    uri: Uri,
    State(state): State<KernelState>,
    body: axum::body::Bytes,
) -> Result<Json<Value>, StatusCode> {
    let path_str = uri.path();
    let method_str = method.as_str();
    
    tracing::info!("🎯 Plugin request: {} {}", method_str, path_str);
    
    match state.kernel.route_request(path_str, method_str, &body).await {
        Ok(response) => {
            // Try to parse as JSON, fallback to raw response
            if let Ok(json_response) = serde_json::from_slice::<Value>(&response) {
                Ok(Json(json_response))
            } else {
                Ok(Json(json!({
                    "data": String::from_utf8_lossy(&response)
                })))
            }
        }
        Err(e) => {
            tracing::error!("Plugin request failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}