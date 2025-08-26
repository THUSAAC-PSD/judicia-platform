use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{Datelike, Timelike};

use abac_engine::{ABACEngine, AccessRequest, Subject, Action, Resource, Environment, AttributeValue, Decision};
use capability_provider::{CapabilityProvider, JudiciaCapabilityProvider};
use event_bus::{EventBus, RabbitMQEventBus, EventManager};
use plugin_runtime::{plugin_manager::PluginManager, PluginMetadata};
use shared::PluginHttpRoute;
use evaluation_engine::job_queue::{JobQueue, RabbitMQJobQueue, EvaluationJob};

use crate::{config::Config, database::Database};

/// Internal struct for resolved plugin routes
#[derive(Debug, Clone)]
struct ResolvedRoute {
    pub plugin_id: Uuid,
    pub plugin_name: String,
    pub handler_function: String,
    pub required_permission: Option<String>,
    pub rate_limit_override: Option<i32>,
}

/// The Judicia Core Kernel
/// 
/// This is the heart of the plugin-based architecture.
/// It provides:
/// - Plugin lifecycle management
/// - Capability API for plugins
/// - Request routing to plugins
/// - Event-driven communication
pub struct JudiciaKernel {
    config: Arc<Config>,
    db: Database,
    plugin_manager: PluginManager,
    capability_provider: Arc<dyn CapabilityProvider + Send + Sync>,
    event_bus: Arc<dyn EventBus + Send + Sync>,
    event_manager: Arc<EventManager>,
    abac_engine: Arc<ABACEngine>,
    job_queue: Arc<dyn JobQueue + Send + Sync>,
}

impl JudiciaKernel {
    pub async fn new(config: Arc<Config>, db: Database) -> Result<Self> {
        // Initialize event bus
        let event_bus = Arc::new(RabbitMQEventBus::new(&config.rabbitmq_url).await?);
        
        // Initialize job queue
        let job_queue = Arc::new(RabbitMQJobQueue::new(&config.rabbitmq_url).await?);
        
        // Initialize capability provider
        let capability_provider = Arc::new(JudiciaCapabilityProvider::new(
            Arc::new(db.pool().clone()),
            event_bus.clone(),
            job_queue.clone(),
        ));
        
        // Initialize plugin manager with capability provider
        let plugin_manager = PluginManager::new(capability_provider.clone())?;
        
        // Initialize ABAC engine with default policies
        let abac_engine = Arc::new(ABACEngine::new());
        Self::setup_default_policies(&abac_engine);
        
        // Initialize event manager
        let event_manager = Arc::new(EventManager::new(event_bus.clone()));
        
        Ok(Self {
            config,
            db,
            plugin_manager,
            capability_provider,
            event_bus,
            event_manager,
            abac_engine,
            job_queue,
        })
    }
    
    pub async fn new_test_mode(config: Arc<Config>) -> Result<Self> {
        use crate::database::Database;
        use event_bus::MockEventBus;
        use capability_provider::MockCapabilityProvider;
        use evaluation_engine::job_queue::MockJobQueue;
        
        // Use mock implementations for testing
        let event_bus = Arc::new(MockEventBus::new());
        let job_queue = Arc::new(MockJobQueue::new());
        let capability_provider = Arc::new(MockCapabilityProvider::new(job_queue.clone()));
        
        // Initialize plugin manager with mock capability provider
        let plugin_manager = PluginManager::new(capability_provider.clone())?;
        
        // Initialize ABAC engine with default policies
        let abac_engine = Arc::new(ABACEngine::new());
        Self::setup_default_policies(&abac_engine);
        
        // Initialize event manager with mock event bus
        let event_manager = Arc::new(EventManager::new(event_bus.clone()));
        
        // Create a mock database (won't be used in test mode)
        let db = Database::new_mock()?;
        
        Ok(Self {
            config,
            db,
            plugin_manager,
            capability_provider,
            event_bus,
            event_manager,
            abac_engine,
            job_queue,
        })
    }
    
    /// Load plugins from the plugins directory
    pub async fn load_plugins(&self) -> Result<()> {
        tracing::info!("🔌 Loading plugins...");
        
        // For now, let's try to load our test plugin
        let test_plugin_path = std::path::Path::new("plugins/hello-world/target/wasm32-wasip1/release/hello_world.wasm");
        
        if test_plugin_path.exists() {
            match self.plugin_manager.load_plugin_from_path(test_plugin_path).await {
                Ok(plugin_id) => {
                    tracing::info!("✅ Loaded test plugin: {}", plugin_id);
                    
                    // Emit plugin loaded event
                    if let Err(e) = self.event_manager.emit_simple_event(
                        "plugin.loaded",
                        serde_json::json!({
                            "plugin_id": plugin_id,
                            "plugin_name": "hello-world",
                            "version": "1.0.0",
                            "path": test_plugin_path.to_string_lossy()
                        }),
                        None,
                    ).await {
                        tracing::warn!("Failed to emit plugin loaded event: {}", e);
                    }
                }
                Err(e) => {
                    tracing::warn!("❌ Failed to load test plugin: {}", e);
                    
                    // Emit plugin error event
                    if let Err(event_err) = self.event_manager.emit_simple_event(
                        "plugin.error",
                        serde_json::json!({
                            "plugin_name": "hello-world",
                            "error": e.to_string(),
                            "phase": "loading"
                        }),
                        None,
                    ).await {
                        tracing::warn!("Failed to emit plugin error event: {}", event_err);
                    }
                }
            }
        } else {
            tracing::info!("ℹ️  Test plugin not found at {:?}", test_plugin_path);
        }
        
        tracing::info!("✅ Plugin loading complete");
        Ok(())
    }
    
    /// Get list of loaded plugins
    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        self.plugin_manager.list_plugins()
    }
    
    /// Route an HTTP request to the appropriate plugin based on registered routes
    pub async fn route_request(&self, path: &str, method: &str, body: &[u8]) -> Result<Vec<u8>> {
        tracing::debug!("🚀 Routing {} {} to plugins", method, path);
        
        // Try to find a specific plugin route first
        if let Some(route_info) = self.resolve_plugin_route(path, method).await? {
            return self.execute_plugin_route(route_info, body).await;
        }
        
        // Fallback to simple routing for backward compatibility
        if path.starts_with("/api/plugin/") {
            let plugins = self.plugin_manager.list_plugins();
            if let Some(plugin) = plugins.first() {
                tracing::info!("🎯 Fallback: calling handle_request on plugin: {}", plugin.name);
                match self.plugin_manager.call_plugin_function(plugin.id, "handle_request", body).await {
                    Ok(response) => return Ok(response),
                    Err(e) => {
                        tracing::error!("Plugin call failed: {}", e);
                        return Ok(format!(r#"{{"error": "Plugin call failed: {}"}}"#, e).into_bytes());
                    }
                }
            }
        }
        
        // Default response for non-plugin routes
        Ok(format!(r#"{{"message": "Route {} {} handled by core kernel", "plugins_loaded": {}}}"#, 
                  method, path, self.plugin_manager.list_plugins().len()).into_bytes())
    }
    
    /// Resolve a plugin route from the registered routes
    async fn resolve_plugin_route(&self, path: &str, method: &str) -> Result<Option<ResolvedRoute>> {
        // In a full implementation, this would query the database for registered routes
        // For now, we'll simulate some registered routes based on loaded plugins
        
        let plugins = self.plugin_manager.list_plugins();
        
        for plugin in plugins {
            // Create simulated routes for each plugin
            let plugin_routes = vec![
                PluginHttpRoute {
                    id: uuid::Uuid::new_v4(),
                    plugin_id: plugin.id,
                    path: format!("/api/{}/handle", plugin.name),
                    method: method.to_string(),
                    handler_function: "handle_request".to_string(),
                    required_permission: None,
                    rate_limit_override: None,
                },
                PluginHttpRoute {
                    id: uuid::Uuid::new_v4(),
                    plugin_id: plugin.id,
                    path: format!("/api/{}/info", plugin.name),
                    method: "GET".to_string(),
                    handler_function: "get_info".to_string(),
                    required_permission: None,
                    rate_limit_override: None,
                },
            ];
            
            for route in plugin_routes {
                if self.matches_route_pattern(&route.path, path) && route.method == method {
                    return Ok(Some(ResolvedRoute {
                        plugin_id: route.plugin_id,
                        plugin_name: plugin.name.clone(),
                        handler_function: route.handler_function,
                        required_permission: route.required_permission,
                        rate_limit_override: route.rate_limit_override,
                    }));
                }
            }
        }
        
        Ok(None)
    }
    
    /// Execute a resolved plugin route
    async fn execute_plugin_route(&self, route: ResolvedRoute, body: &[u8]) -> Result<Vec<u8>> {
        tracing::info!("🎯 Executing route: {} -> {}::{}", 
                      route.plugin_name, route.plugin_name, route.handler_function);
        
        // Check permissions using ABAC engine
        if let Some(permission) = &route.required_permission {
            tracing::debug!("🔒 Route requires permission: {}", permission);
            
            // Create access request for ABAC evaluation
            let access_request = self.create_access_request_for_route(&route, permission);
            
            match self.abac_engine.evaluate(&access_request)? {
                Decision::Permit => {
                    tracing::debug!("✅ ABAC authorization granted");
                }
                Decision::Deny => {
                    tracing::warn!("❌ ABAC authorization denied");
                    return Ok(format!(r#"{{"error": "Access denied", "plugin": "{}"}}"#, route.plugin_name).into_bytes());
                }
                Decision::NotApplicable => {
                    tracing::warn!("❓ ABAC no applicable policies, denying by default");
                    return Ok(format!(r#"{{"error": "No authorization policy applicable", "plugin": "{}"}}"#, route.plugin_name).into_bytes());
                }
            }
        }
        
        // TODO: Apply rate limiting here
        if let Some(rate_limit) = route.rate_limit_override {
            tracing::debug!("⏱️ Route has rate limit override: {} req/sec", rate_limit);
            // In full implementation:
            // - Track requests per user/IP
            // - Return 429 Too Many Requests if exceeded
        }
        
        // Execute the plugin function
        match self.plugin_manager.call_plugin_function(
            route.plugin_id, 
            &route.handler_function, 
            body
        ).await {
            Ok(response) => {
                tracing::info!("✅ Plugin route executed successfully");
                Ok(response)
            }
            Err(e) => {
                tracing::error!("❌ Plugin route execution failed: {}", e);
                Ok(format!(r#"{{"error": "Plugin execution failed: {}", "plugin": "{}"}}"#, 
                          e, route.plugin_name).into_bytes())
            }
        }
    }
    
    /// Check if a path matches a route pattern
    fn matches_route_pattern(&self, pattern: &str, path: &str) -> bool {
        // Simple exact match for now
        // In full implementation, this would support:
        // - Path parameters (/api/plugin/{plugin_id}/handle)
        // - Wildcards (/api/plugin/*/handle)
        // - Regular expressions
        pattern == path
    }
    
    /// Get the capability provider for plugin access
    pub fn capability_provider(&self) -> Arc<dyn CapabilityProvider + Send + Sync> {
        self.capability_provider.clone()
    }
    
    /// Get the event bus for plugin communication
    pub fn event_bus(&self) -> Arc<dyn EventBus + Send + Sync> {
        self.event_bus.clone()
    }
    
    /// Get the event manager for high-level event operations
    pub fn event_manager(&self) -> Arc<EventManager> {
        self.event_manager.clone()
    }
    
    /// Get the ABAC engine for authorization
    pub fn abac_engine(&self) -> Arc<ABACEngine> {
        self.abac_engine.clone()
    }
    
    /// Get the job queue for evaluation tasks
    pub fn job_queue(&self) -> Arc<dyn JobQueue + Send + Sync> {
        self.job_queue.clone()
    }
    
    /// Submit an evaluation job (convenience method)
    pub async fn submit_evaluation_job(&self, job: EvaluationJob) -> Result<()> {
        self.job_queue.submit_job(job).await
    }
    
    /// Health check for the kernel
    pub fn health(&self) -> bool {
        // TODO: Check plugin manager, database, event bus health
        true
    }
    
    /// Set up default ABAC policies for the system
    fn setup_default_policies(abac_engine: &ABACEngine) {
        // Admin access policy
        let admin_policy = ABACEngine::create_role_based_policy(
            "admin_full_access",
            "Administrators have full access to all resources",
            vec!["admin".to_string(), "superadmin".to_string()],
            "*",
            "*"
        );
        abac_engine.add_policy(admin_policy);
        
        // Contest admin policy
        let contest_admin_policy = ABACEngine::create_role_based_policy(
            "contest_admin_access",
            "Contest admins can manage contests and problems",
            vec!["contest_admin".to_string()],
            "contest",
            "*"
        );
        abac_engine.add_policy(contest_admin_policy);
        
        // Contestant basic access policy
        let contestant_policy = ABACEngine::create_role_based_policy(
            "contestant_basic_access",
            "Contestants can view problems and submit solutions",
            vec!["contestant".to_string()],
            "problem",
            "read"
        );
        abac_engine.add_policy(contestant_policy);
        
        // Plugin access policy
        let plugin_access_policy = ABACEngine::create_plugin_policy(
            "plugin_access",
            "Users must have plugin-specific permissions",
            "*",
            "plugin_access"
        );
        abac_engine.add_policy(plugin_access_policy);
        
        tracing::info!("✅ ABAC engine initialized with {} default policies", 4);
    }
    
    /// Create an access request for route authorization
    fn create_access_request_for_route(&self, route: &ResolvedRoute, permission: &str) -> AccessRequest {
        // Create a mock subject for now (in full implementation, extract from JWT/session)
        let mut subject_attributes = HashMap::new();
        subject_attributes.insert("roles".to_string(), AttributeValue::Array(vec![
            AttributeValue::String("contestant".to_string())
        ]));
        subject_attributes.insert("permissions".to_string(), AttributeValue::Array(vec![
            AttributeValue::String(permission.to_string())
        ]));
        
        let subject = Subject {
            user_id: Uuid::new_v4(), // Mock user ID
            attributes: subject_attributes,
        };
        
        let mut action_attributes = HashMap::new();
        action_attributes.insert("name".to_string(), AttributeValue::String(route.handler_function.clone()));
        
        let action = Action {
            name: route.handler_function.clone(),
            attributes: action_attributes,
        };
        
        let mut resource_attributes = HashMap::new();
        resource_attributes.insert("plugin_id".to_string(), AttributeValue::String(route.plugin_id.to_string()));
        resource_attributes.insert("plugin_name".to_string(), AttributeValue::String(route.plugin_name.clone()));
        resource_attributes.insert("type".to_string(), AttributeValue::String("plugin".to_string()));
        
        let resource = Resource {
            type_name: "plugin".to_string(),
            id: route.plugin_id.to_string(),
            attributes: resource_attributes,
        };
        
        let mut env_attributes = HashMap::new();
        let now = chrono::Utc::now();
        env_attributes.insert("hour".to_string(), AttributeValue::Number(now.hour() as f64));
        env_attributes.insert("day_of_week".to_string(), AttributeValue::Number(now.weekday().number_from_monday() as f64));
        
        let environment = Environment {
            timestamp: now,
            ip_address: Some("127.0.0.1".to_string()), // Mock IP
            user_agent: Some("judicia-kernel".to_string()),
            attributes: env_attributes,
        };
        
        AccessRequest {
            subject,
            action,
            resource,
            environment,
        }
    }
}