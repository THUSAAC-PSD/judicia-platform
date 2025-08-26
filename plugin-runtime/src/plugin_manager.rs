use crate::{LoadedPlugin, PluginMetadata, PluginRuntime, WasmPluginRuntime};
use anyhow::Result;
use capability_provider::CapabilityProvider;
use dashmap::DashMap;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct PluginManager {
    runtime: Arc<dyn PluginRuntime + Send + Sync>,
    plugins: Arc<DashMap<Uuid, PluginMetadata>>,
    plugin_registry: Arc<DashMap<String, Uuid>>, // name -> plugin_id mapping
}

impl PluginManager {
    pub fn new(capability_provider: Arc<dyn CapabilityProvider + Send + Sync>) -> Result<Self> {
        let runtime = Arc::new(WasmPluginRuntime::new(capability_provider)?);
        
        Ok(Self {
            runtime,
            plugins: Arc::new(DashMap::new()),
            plugin_registry: Arc::new(DashMap::new()),
        })
    }
    
    pub async fn load_plugin_from_path(&self, wasm_path: &Path) -> Result<Uuid> {
        let plugin = self.runtime.load_plugin(wasm_path).await?;
        let plugin_id = plugin.metadata.id;
        
        // Register the plugin
        self.plugins.insert(plugin_id, plugin.metadata.clone());
        self.plugin_registry.insert(plugin.metadata.name.clone(), plugin_id);
        
        tracing::info!("Loaded plugin: {} ({})", plugin.metadata.name, plugin_id);
        Ok(plugin_id)
    }
    
    pub async fn call_plugin_function(
        &self,
        plugin_id: Uuid,
        function: &str,
        args: &[u8],
    ) -> Result<Vec<u8>> {
        if !self.plugins.contains_key(&plugin_id) {
            return Err(anyhow::anyhow!("Plugin not found: {}", plugin_id));
        }
        
        self.runtime.call_function(plugin_id, function, args).await
    }
    
    pub fn get_plugin_by_name(&self, name: &str) -> Option<Uuid> {
        self.plugin_registry.get(name).map(|entry| *entry.value())
    }
    
    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        self.plugins
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
    
    pub async fn unload_plugin(&self, plugin_id: Uuid) -> Result<()> {
        if let Some((_, metadata)) = self.plugins.remove(&plugin_id) {
            self.plugin_registry.remove(&metadata.name);
            self.runtime.unload_plugin(plugin_id).await?;
            tracing::info!("Unloaded plugin: {} ({})", metadata.name, plugin_id);
        }
        Ok(())
    }
}