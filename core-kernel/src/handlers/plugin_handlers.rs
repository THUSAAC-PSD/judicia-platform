use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;

use crate::KernelState;

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
    pub dependencies: Vec<String>,
    pub capabilities: Vec<String>,
    pub components: Vec<ComponentRegistration>,
    pub routes: Vec<RouteRegistration>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentRegistration {
    pub name: String,
    pub props: Option<Value>,
    pub slots: Vec<String>,
    pub events: Vec<ComponentEvent>,
    pub styles: Option<ComponentStyles>,
    pub lazy: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentEvent {
    pub name: String,
    pub description: Option<String>,
    pub payload: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentStyles {
    pub css: Option<String>,
    pub class_names: HashMap<String, String>,
    pub css_variables: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteRegistration {
    pub path: String,
    pub title: Option<String>,
    pub icon: Option<String>,
    pub requires_auth: bool,
    pub required_capabilities: Vec<String>,
    pub exact: bool,
    pub children: Vec<RouteRegistration>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MicrofrontendConfig {
    pub name: String,
    pub url: String,
    pub scope: String,
    pub module: String,
    pub r#type: String, // "module" or "systemjs"
    pub props: Option<Value>,
    pub css: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginDiscoveryResponse {
    pub plugins: Vec<PluginMetadata>,
    pub microfrontends: HashMap<String, MicrofrontendConfig>,
}

/// Get all registered plugins for frontend discovery
pub async fn get_plugin_discovery(
    State(state): State<KernelState>,
) -> Result<Json<PluginDiscoveryResponse>, StatusCode> {
    // Mock plugin registry - in a real implementation, this would come from database
    let plugins = vec![
        PluginMetadata {
            id: "standard-judge".to_string(),
            name: "Standard Judge".to_string(),
            version: "1.0.0".to_string(),
            author: "Judicia Platform Team".to_string(),
            description: "Standard competitive programming judge plugin".to_string(),
            homepage: Some("https://github.com/judicia/judicia-platform".to_string()),
            repository: Some("https://github.com/judicia/judicia-platform".to_string()),
            license: Some("MIT".to_string()),
            dependencies: vec![],
            capabilities: vec![
                "read_problems".to_string(),
                "read_submissions".to_string(),
                "write_submissions".to_string(),
                "register_components".to_string(),
                "emit_events".to_string(),
            ],
            components: vec![
                ComponentRegistration {
                    name: "JudgingStatus".to_string(),
                    props: Some(json!({"submissionId": "string"})),
                    slots: vec![],
                    events: vec![
                        ComponentEvent {
                            name: "judging-complete".to_string(),
                            description: Some("Fired when judging completes".to_string()),
                            payload: Some(json!({"verdict": "string", "score": "number"})),
                        }
                    ],
                    styles: Some(ComponentStyles {
                        css: Some(".judging-status { padding: 1rem; }".to_string()),
                        class_names: HashMap::from([
                            ("container".to_string(), "judging-status".to_string()),
                            ("progress".to_string(), "judging-progress".to_string()),
                        ]),
                        css_variables: HashMap::from([
                            ("--primary-color".to_string(), "#3b82f6".to_string()),
                        ]),
                    }),
                    lazy: false,
                },
                ComponentRegistration {
                    name: "VerdictDisplay".to_string(),
                    props: Some(json!({"verdict": "string", "score": "number"})),
                    slots: vec![],
                    events: vec![],
                    styles: None,
                    lazy: false,
                }
            ],
            routes: vec![
                RouteRegistration {
                    path: "/judge/status".to_string(),
                    title: Some("Judge Status".to_string()),
                    icon: Some("⚖️".to_string()),
                    requires_auth: true,
                    required_capabilities: vec!["read_submissions".to_string()],
                    exact: true,
                    children: vec![],
                }
            ],
        },
        PluginMetadata {
            id: "icpc-contest".to_string(),
            name: "ICPC Contest".to_string(),
            version: "1.0.0".to_string(),
            author: "Judicia Platform Team".to_string(),
            description: "ICPC-style contest management plugin".to_string(),
            homepage: Some("https://github.com/judicia/judicia-platform".to_string()),
            repository: Some("https://github.com/judicia/judicia-platform".to_string()),
            license: Some("MIT".to_string()),
            dependencies: vec![],
            capabilities: vec![
                "read_contests".to_string(),
                "write_contests".to_string(),
                "read_problems".to_string(),
                "read_submissions".to_string(),
                "register_components".to_string(),
                "register_routes".to_string(),
                "emit_events".to_string(),
            ],
            components: vec![
                ComponentRegistration {
                    name: "ScoreBoard".to_string(),
                    props: Some(json!({"contestId": "string"})),
                    slots: vec![],
                    events: vec![
                        ComponentEvent {
                            name: "team-click".to_string(),
                            description: Some("Fired when a team is clicked".to_string()),
                            payload: Some(json!({"teamId": "string"})),
                        }
                    ],
                    styles: Some(ComponentStyles {
                        css: Some(".scoreboard { width: 100%; }".to_string()),
                        class_names: HashMap::new(),
                        css_variables: HashMap::new(),
                    }),
                    lazy: true,
                },
                ComponentRegistration {
                    name: "ContestDashboard".to_string(),
                    props: Some(json!({"contestId": "string"})),
                    slots: vec!["sidebar".to_string(), "main".to_string()],
                    events: vec![],
                    styles: None,
                    lazy: false,
                }
            ],
            routes: vec![
                RouteRegistration {
                    path: "/contest".to_string(),
                    title: Some("Contests".to_string()),
                    icon: Some("🏆".to_string()),
                    requires_auth: true,
                    required_capabilities: vec!["read_contests".to_string()],
                    exact: false,
                    children: vec![
                        RouteRegistration {
                            path: "/contest/:id".to_string(),
                            title: Some("Contest Details".to_string()),
                            icon: None,
                            requires_auth: true,
                            required_capabilities: vec!["read_contests".to_string()],
                            exact: true,
                            children: vec![],
                        },
                        RouteRegistration {
                            path: "/contest/:id/scoreboard".to_string(),
                            title: Some("Scoreboard".to_string()),
                            icon: Some("📊".to_string()),
                            requires_auth: false,
                            required_capabilities: vec![],
                            exact: true,
                            children: vec![],
                        }
                    ],
                }
            ],
        },
        PluginMetadata {
            id: "notification-system".to_string(),
            name: "Notification System".to_string(),
            version: "1.0.0".to_string(),
            author: "Judicia Platform Team".to_string(),
            description: "Real-time notification system plugin".to_string(),
            homepage: Some("https://github.com/judicia/judicia-platform".to_string()),
            repository: Some("https://github.com/judicia/judicia-platform".to_string()),
            license: Some("MIT".to_string()),
            dependencies: vec![],
            capabilities: vec![
                "notifications".to_string(),
                "register_components".to_string(),
                "subscribe_events".to_string(),
            ],
            components: vec![
                ComponentRegistration {
                    name: "NotificationCenter".to_string(),
                    props: None,
                    slots: vec![],
                    events: vec![
                        ComponentEvent {
                            name: "notification-click".to_string(),
                            description: Some("Fired when notification is clicked".to_string()),
                            payload: Some(json!({"notificationId": "string"})),
                        }
                    ],
                    styles: None,
                    lazy: false,
                }
            ],
            routes: vec![
                RouteRegistration {
                    path: "/notifications".to_string(),
                    title: Some("Notifications".to_string()),
                    icon: Some("🔔".to_string()),
                    requires_auth: true,
                    required_capabilities: vec!["notifications".to_string()],
                    exact: true,
                    children: vec![],
                }
            ],
        },
        PluginMetadata {
            id: "announcement-system".to_string(),
            name: "Announcement System".to_string(),
            version: "1.0.0".to_string(),
            author: "Judicia Platform Team".to_string(),
            description: "Contest announcement and communication system plugin".to_string(),
            homepage: Some("https://github.com/judicia/judicia-platform".to_string()),
            repository: Some("https://github.com/judicia/judicia-platform".to_string()),
            license: Some("MIT".to_string()),
            dependencies: vec![],
            capabilities: vec![
                "register_components".to_string(),
                "subscribe_events".to_string(),
                "emit_events".to_string(),
            ],
            components: vec![
                ComponentRegistration {
                    name: "AnnouncementBanner".to_string(),
                    props: Some(json!({"contestId": "string?"})),
                    slots: vec![],
                    events: vec![],
                    styles: Some(ComponentStyles {
                        css: Some(".announcement-banner { position: sticky; top: 0; z-index: 50; }".to_string()),
                        class_names: HashMap::new(),
                        css_variables: HashMap::new(),
                    }),
                    lazy: false,
                },
                ComponentRegistration {
                    name: "AnnouncementList".to_string(),
                    props: Some(json!({"limit": "number?"})),
                    slots: vec![],
                    events: vec![],
                    styles: None,
                    lazy: true,
                }
            ],
            routes: vec![
                RouteRegistration {
                    path: "/announcements".to_string(),
                    title: Some("Announcements".to_string()),
                    icon: Some("📢".to_string()),
                    requires_auth: true,
                    required_capabilities: vec![],
                    exact: true,
                    children: vec![],
                }
            ],
        }
    ];

    // Mock microfrontend configurations - in production, this would be configurable
    let microfrontends = HashMap::from([
        (
            "standard-judge".to_string(),
            MicrofrontendConfig {
                name: "standard-judge".to_string(),
                url: "http://localhost:5001/assets/remoteEntry.js".to_string(),
                scope: "standardJudge".to_string(),
                module: "./Plugin".to_string(),
                r#type: "module".to_string(),
                props: None,
                css: vec![],
            }
        ),
        (
            "icpc-contest".to_string(),
            MicrofrontendConfig {
                name: "icpc-contest".to_string(),
                url: "http://localhost:5002/assets/remoteEntry.js".to_string(),
                scope: "icpcContest".to_string(),
                module: "./Plugin".to_string(),
                r#type: "module".to_string(),
                props: None,
                css: vec!["http://localhost:5002/assets/contest.css".to_string()],
            }
        ),
        (
            "notification-system".to_string(),
            MicrofrontendConfig {
                name: "notification-system".to_string(),
                url: "http://localhost:5003/assets/remoteEntry.js".to_string(),
                scope: "notificationSystem".to_string(),
                module: "./Plugin".to_string(),
                r#type: "module".to_string(),
                props: None,
                css: vec![],
            }
        ),
        (
            "announcement-system".to_string(),
            MicrofrontendConfig {
                name: "announcement-system".to_string(),
                url: "http://localhost:5004/assets/remoteEntry.js".to_string(),
                scope: "announcementSystem".to_string(),
                module: "./Plugin".to_string(),
                r#type: "module".to_string(),
                props: None,
                css: vec![],
            }
        )
    ]);

    let response = PluginDiscoveryResponse {
        plugins,
        microfrontends,
    };

    Ok(Json(response))
}

/// Get specific plugin metadata
pub async fn get_plugin_metadata(
    State(_state): State<KernelState>,
    axum::extract::Path(plugin_id): axum::extract::Path<String>,
) -> Result<Json<Value>, StatusCode> {
    // In a real implementation, this would query the database
    match plugin_id.as_str() {
        "standard-judge" => Ok(Json(json!({
            "id": "standard-judge",
            "name": "Standard Judge",
            "version": "1.0.0",
            "status": "active",
            "endpoints": [
                "/api/judge/submit",
                "/api/judge/status/:id",
                "/api/judge/verdict/:id",
                "/api/judge/config"
            ],
            "health": {
                "status": "healthy",
                "last_check": "2024-01-01T00:00:00Z",
                "response_time": "150ms"
            }
        }))),
        "icpc-contest" => Ok(Json(json!({
            "id": "icpc-contest",
            "name": "ICPC Contest",
            "version": "1.0.0",
            "status": "active",
            "endpoints": [
                "/api/contest/create",
                "/api/contest/:id/scoreboard",
                "/api/contest/:id/teams",
                "/api/contest/:id/clarifications"
            ],
            "health": {
                "status": "healthy",
                "last_check": "2024-01-01T00:00:00Z",
                "response_time": "200ms"
            }
        }))),
        _ => Err(StatusCode::NOT_FOUND),
    }
}

/// Get plugin health status
pub async fn get_plugin_health(
    State(_state): State<KernelState>,
) -> Result<Json<Value>, StatusCode> {
    // Mock health data - in production, this would check actual plugin health
    Ok(Json(json!({
        "plugins": {
            "standard-judge": {
                "status": "healthy",
                "last_check": "2024-01-01T00:00:00Z",
                "response_time": "150ms",
                "memory_usage": "45MB",
                "cpu_usage": "2.1%"
            },
            "icpc-contest": {
                "status": "healthy",
                "last_check": "2024-01-01T00:00:00Z",
                "response_time": "200ms",
                "memory_usage": "38MB",
                "cpu_usage": "1.8%"
            },
            "notification-system": {
                "status": "healthy",
                "last_check": "2024-01-01T00:00:00Z",
                "response_time": "85ms",
                "memory_usage": "22MB",
                "cpu_usage": "0.9%"
            },
            "announcement-system": {
                "status": "healthy",
                "last_check": "2024-01-01T00:00:00Z",
                "response_time": "95ms",
                "memory_usage": "19MB",
                "cpu_usage": "0.7%"
            }
        },
        "overall": {
            "total_plugins": 4,
            "healthy_plugins": 4,
            "warning_plugins": 0,
            "error_plugins": 0,
            "total_memory": "124MB",
            "total_cpu": "5.5%"
        }
    })))
}