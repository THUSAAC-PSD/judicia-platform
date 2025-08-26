use anyhow::Result;
use std::time::Duration;

/// Plugin sandbox configuration and resource limits
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub max_memory_bytes: u64,
    pub max_execution_time: Duration,
    pub allowed_capabilities: Vec<String>,
    pub network_access: bool,
    pub file_system_access: FileSystemAccess,
}

#[derive(Debug, Clone)]
pub enum FileSystemAccess {
    None,
    ReadOnly(Vec<String>), // Allowed paths
    Limited(Vec<String>),  // Read-write to specific paths
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: 64 * 1024 * 1024, // 64MB
            max_execution_time: Duration::from_secs(10),
            allowed_capabilities: vec![],
            network_access: false,
            file_system_access: FileSystemAccess::None,
        }
    }
}

pub struct PluginSandbox {
    config: SandboxConfig,
}

impl PluginSandbox {
    pub fn new(config: SandboxConfig) -> Self {
        Self { config }
    }
    
    pub fn validate_capability_request(&self, capability: &str) -> Result<bool> {
        Ok(self.config.allowed_capabilities.contains(&capability.to_string()))
    }
    
    pub fn check_resource_usage(&self, memory_used: u64) -> Result<()> {
        if memory_used > self.config.max_memory_bytes {
            return Err(anyhow::anyhow!("Plugin exceeded memory limit"));
        }
        Ok(())
    }
    
    pub fn get_timeout(&self) -> Duration {
        self.config.max_execution_time
    }
}