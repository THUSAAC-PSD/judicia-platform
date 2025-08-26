use anyhow::Result;
use async_trait::async_trait;
use capability_provider::CapabilityProvider;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;
use wasmtime::{Caller, Config, Engine, Instance, Linker, Module, Store};

pub mod host_functions;
pub mod plugin_manager;
pub mod sandbox;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub plugin_type: PluginType,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginType {
    Contest,
    Problem,
    Utility,
}

pub struct LoadedPlugin {
    pub metadata: PluginMetadata,
    pub instance: Instance,
    pub store: Store<PluginState>,
}

pub struct PluginState {
    pub plugin_id: Uuid,
    pub capabilities: Vec<String>,
    pub capability_provider: Arc<dyn CapabilityProvider + Send + Sync>,
}

#[async_trait]
pub trait PluginRuntime {
    async fn load_plugin(&self, wasm_path: &Path) -> Result<LoadedPlugin>;
    async fn call_function(&self, plugin_id: Uuid, function: &str, args: &[u8]) -> Result<Vec<u8>>;
    async fn unload_plugin(&self, plugin_id: Uuid) -> Result<()>;
}

pub struct WasmPluginRuntime {
    engine: Engine,
    // Store only metadata in the DashMap for now
    // Full plugin management would require a more sophisticated approach
    plugin_metadata: Arc<DashMap<Uuid, PluginMetadata>>,
    capability_provider: Arc<dyn CapabilityProvider + Send + Sync>,
}

impl WasmPluginRuntime {
    pub fn new(capability_provider: Arc<dyn CapabilityProvider + Send + Sync>) -> Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);
        
        let engine = Engine::new(&config)?;
        
        Ok(Self {
            engine,
            plugin_metadata: Arc::new(DashMap::new()),
            capability_provider,
        })
    }
}

#[async_trait]
impl PluginRuntime for WasmPluginRuntime {
    async fn load_plugin(&self, wasm_path: &Path) -> Result<LoadedPlugin> {
        tracing::info!("Loading plugin from: {:?}", wasm_path);
        
        // 1. Read and validate the WASM binary
        let wasm_bytes = std::fs::read(wasm_path)?;
        
        // 2. Create a WASM module
        let module = Module::new(&self.engine, &wasm_bytes)?;
        
        // 3. Create plugin state
        let plugin_id = Uuid::new_v4();
        let plugin_state = PluginState {
            plugin_id,
            capabilities: vec![], // TODO: Extract from plugin metadata
            capability_provider: self.capability_provider.clone(),
        };
        
        // 4. Create store with plugin state
        let mut store = Store::new(&self.engine, plugin_state);
        
        // 5. Set up host function bindings
        let mut linker = Linker::new(&self.engine);
        self.setup_host_functions(&mut linker)?;
        
        // 6. Instantiate the plugin (use async instantiation)
        let instance = linker.instantiate_async(&mut store, &module).await?;
        
        // 7. Extract plugin metadata (try to call _plugin_metadata if it exists)
        let metadata = self.extract_plugin_metadata(&mut store, &instance).await
            .unwrap_or_else(|_| PluginMetadata {
                id: plugin_id,
                name: wasm_path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                version: "0.1.0".to_string(),
                author: "unknown".to_string(),
                description: "Plugin loaded from WASM binary".to_string(),
                plugin_type: PluginType::Utility,
                capabilities: vec![],
            });
        
        // Store the plugin metadata for tracking
        self.plugin_metadata.insert(plugin_id, metadata.clone());
        
        let loaded_plugin = LoadedPlugin {
            metadata,
            instance,
            store,
        };
        
        tracing::info!("Successfully loaded plugin: {} ({})", loaded_plugin.metadata.name, plugin_id);
        Ok(loaded_plugin)
    }
    
    async fn call_function(&self, plugin_id: Uuid, function: &str, args: &[u8]) -> Result<Vec<u8>> {
        tracing::debug!("Calling function {} on plugin {}", function, plugin_id);
        
        // Check if plugin exists
        if !self.plugin_metadata.contains_key(&plugin_id) {
            return Err(anyhow::anyhow!("Plugin not found: {}", plugin_id));
        }
        
        // TODO: In a full implementation, we would:
        // 1. Retrieve the actual plugin instance from storage
        // 2. Allocate memory in WASM for input args
        // 3. Call the function with proper parameters
        // 4. Read the result from WASM memory
        // 5. Deallocate WASM memory
        
        // For now, return a placeholder response
        Ok(format!(r#"{{"function": "{}", "plugin_id": "{}", "args_len": {}}}"#, 
                  function, plugin_id, args.len()).into_bytes())
    }
    
    async fn unload_plugin(&self, plugin_id: Uuid) -> Result<()> {
        if self.plugin_metadata.remove(&plugin_id).is_some() {
            tracing::info!("Unloaded plugin: {}", plugin_id);
        }
        Ok(())
    }
}

impl WasmPluginRuntime {
    fn setup_host_functions(&self, linker: &mut Linker<PluginState>) -> Result<()> {
        // Define host functions that plugins can call
        
        // platform::trigger_judging(submission_id: u64) -> i32
        linker.func_wrap1_async(
            "platform",
            "trigger_judging",
            |mut caller: Caller<'_, PluginState>, submission_id: i64| Box::new(async move {
                tracing::info!("Plugin called trigger_judging with submission_id: {}", submission_id);
                
                // Get the capability provider from plugin state
                let capability_provider = {
                    let data = caller.data();
                    data.capability_provider.clone()
                };
                
                // Convert i64 to UUID (this is simplified - in practice you'd need proper UUID handling)
                let submission_uuid = Uuid::from_u128(submission_id as u128);
                
                match capability_provider.trigger_judging(submission_uuid).await {
                    Ok(_) => {
                        tracing::info!("✅ Successfully triggered judging for submission: {}", submission_uuid);
                        0 // Success
                    }
                    Err(e) => {
                        tracing::error!("❌ Failed to trigger judging: {}", e);
                        1 // Error
                    }
                }
            })
        )?;
        
        // platform::emit_event(event_type_ptr: i32, event_type_len: i32, payload_ptr: i32, payload_len: i32) -> i32
        linker.func_wrap4_async(
            "platform", 
            "emit_event",
            |mut caller: Caller<'_, PluginState>, event_type_ptr: i32, event_type_len: i32, payload_ptr: i32, payload_len: i32| Box::new(async move {
                tracing::info!("Plugin called emit_event");
                
                let capability_provider = {
                    let data = caller.data();
                    data.capability_provider.clone()
                };
                
                let plugin_id = {
                    let data = caller.data();
                    data.plugin_id
                };
                
                // TODO: Extract strings from WASM memory
                // For now, create a dummy event
                let event = event_bus::Event {
                    id: Uuid::new_v4(),
                    event_type: "plugin.custom_event".to_string(),
                    source_plugin_id: Some(plugin_id),
                    timestamp: chrono::Utc::now(),
                    payload: serde_json::json!({
                        "event_type_ptr": event_type_ptr,
                        "event_type_len": event_type_len,
                        "payload_ptr": payload_ptr,
                        "payload_len": payload_len
                    }),
                };
                
                match capability_provider.emit_event(event).await {
                    Ok(_) => {
                        tracing::info!("✅ Successfully emitted event from plugin: {}", plugin_id);
                        0 // Success
                    }
                    Err(e) => {
                        tracing::error!("❌ Failed to emit event: {}", e);
                        1 // Error
                    }
                }
            })
        )?;
        
        // database::execute_private_sql(sql_ptr: i32, sql_len: i32, params_ptr: i32, params_len: i32) -> i32
        linker.func_wrap4_async(
            "database",
            "execute_private_sql",
            |mut caller: Caller<'_, PluginState>, sql_ptr: i32, sql_len: i32, _params_ptr: i32, _params_len: i32| Box::new(async move {
                tracing::info!("Plugin called execute_private_sql");
                
                let capability_provider = {
                    let data = caller.data();
                    data.capability_provider.clone()
                };
                
                let plugin_id = {
                    let data = caller.data();
                    data.plugin_id
                };
                
                // TODO: Extract SQL string from WASM memory
                let sql = format!("SELECT 1 -- dummy query from plugin {}", plugin_id);
                let params = vec![];
                
                match capability_provider.execute_private_sql(plugin_id, &sql, &params).await {
                    Ok(_result) => {
                        tracing::info!("✅ Successfully executed private SQL for plugin: {}", plugin_id);
                        0 // Success
                    }
                    Err(e) => {
                        tracing::error!("❌ Failed to execute private SQL: {}", e);
                        1 // Error
                    }
                }
            })
        )?;
        
        // websocket::send_message(user_id: u64, message_ptr: i32, message_len: i32) -> i32  
        linker.func_wrap3_async(
            "websocket",
            "send_message",
            |mut caller: Caller<'_, PluginState>, user_id: i64, _message_ptr: i32, message_len: i32| Box::new(async move {
                tracing::info!("Plugin called send_message to user {} with {} bytes", user_id, message_len);
                
                let capability_provider = {
                    let data = caller.data();
                    data.capability_provider.clone()
                };
                
                // Convert i64 to UUID (simplified)
                let user_uuid = Uuid::from_u128(user_id as u128);
                
                // TODO: Extract message from WASM memory
                let message = b"Hello from plugin!";
                
                match capability_provider.send_message(user_uuid, message).await {
                    Ok(_) => {
                        tracing::info!("✅ Successfully sent message to user: {}", user_uuid);
                        0 // Success
                    }
                    Err(e) => {
                        tracing::error!("❌ Failed to send message: {}", e);
                        1 // Error
                    }
                }
            })
        )?;
        
        // log::info(ptr: i32, len: i32) -> i32
        linker.func_wrap(
            "log",
            "info", 
            |_caller: Caller<'_, PluginState>, _ptr: i32, _len: i32| -> i32 {
                // TODO: Extract string from WASM memory and log it
                tracing::info!("📝 Plugin log message");
                0
            },
        )?;
        
        Ok(())
    }
    
    async fn extract_plugin_metadata(&self, _store: &mut Store<PluginState>, _instance: &Instance) -> Result<PluginMetadata> {
        // TODO: Try to call _plugin_metadata and _plugin_metadata_len functions
        // and deserialize the result into PluginMetadata
        Err(anyhow::anyhow!("Metadata extraction not implemented"))
    }
}