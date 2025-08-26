use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Stdio,
    time::{Duration, Instant},
};
use tokio::{fs, process::Command};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Layered security sandbox configuration
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub docker_enabled: bool,
    pub isolate_enabled: bool,
    pub isolate_box_id: Option<u32>,
    pub max_execution_time_ms: u32,
    pub max_memory_kb: u32,
    pub max_file_size_kb: u32,
    pub max_open_files: u32,
    pub allowed_syscalls: Vec<String>,
    pub network_access: bool,
    pub temp_dir: PathBuf,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            docker_enabled: true,
            isolate_enabled: true,
            isolate_box_id: None,
            max_execution_time_ms: 5000,
            max_memory_kb: 256 * 1024, // 256MB
            max_file_size_kb: 1024,    // 1MB
            max_open_files: 64,
            allowed_syscalls: vec![
                "read".to_string(),
                "write".to_string(),
                "exit_group".to_string(),
                "brk".to_string(),
                "mmap".to_string(),
                "munmap".to_string(),
                "fstat".to_string(),
            ],
            network_access: false,
            temp_dir: std::env::temp_dir().join("judicia_sandbox"),
        }
    }
}

/// Sandbox execution mode
#[derive(Debug, Clone)]
pub enum SandboxMode {
    /// Use Docker container isolation
    Docker { image: String },
    /// Use ioi/isolate for lightweight sandboxing
    Isolate { box_id: u32 },
    /// Use both Docker and isolate (layered security)
    Layered { image: String, box_id: u32 },
    /// Unsafe mode for development/testing only
    Native,
}

#[derive(Debug, Clone)]
pub struct Sandbox {
    config: SandboxConfig,
    mode: SandboxMode,
    work_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub time_ms: u32,
    pub memory_kb: u32,
    pub verdict: SandboxVerdict,
    pub system_info: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SandboxVerdict {
    OK,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    RuntimeError,
    SecurityViolation,
    InternalError,
}

impl Sandbox {
    /// Create a new sandbox with the specified configuration and mode
    pub fn new(config: SandboxConfig, mode: SandboxMode) -> Result<Self> {
        let work_dir = config.temp_dir.join(format!("sandbox_{}", Uuid::new_v4()));
        
        Ok(Self {
            config,
            mode,
            work_dir,
        })
    }
    
    /// Create a new sandbox with default configuration
    pub fn new_default() -> Result<Self> {
        Self::new(SandboxConfig::default(), SandboxMode::Native)
    }
    
    /// Initialize the sandbox environment
    pub async fn initialize(&self) -> Result<()> {
        // Create work directory
        fs::create_dir_all(&self.work_dir)
            .await
            .context("Failed to create sandbox work directory")?;
        
        match &self.mode {
            SandboxMode::Docker { .. } => {
                self.initialize_docker().await?;
            }
            SandboxMode::Isolate { box_id } => {
                self.initialize_isolate(*box_id).await?;
            }
            SandboxMode::Layered { box_id, .. } => {
                self.initialize_docker().await?;
                self.initialize_isolate(*box_id).await?;
            }
            SandboxMode::Native => {
                debug!("Using native execution mode - security features disabled");
            }
        }
        
        info!("Sandbox initialized: {:?}", self.mode);
        Ok(())
    }
    
    /// Execute a command in the sandbox
    pub async fn execute_command(
        &self,
        command: &str,
        input_data: &str,
        source_file: Option<&Path>,
    ) -> Result<SandboxResult> {
        let start_time = Instant::now();
        
        // Copy source files to sandbox if provided
        if let Some(source) = source_file {
            let filename = source.file_name().unwrap_or_default();
            let dest = self.work_dir.join(filename);
            fs::copy(source, dest).await?;
        }
        
        let result = match &self.mode {
            SandboxMode::Docker { image } => {
                self.execute_docker(command, input_data, image).await?
            }
            SandboxMode::Isolate { box_id } => {
                self.execute_isolate(command, input_data, *box_id).await?
            }
            SandboxMode::Layered { image, box_id } => {
                self.execute_layered(command, input_data, image, *box_id).await?
            }
            SandboxMode::Native => {
                self.execute_native(command, input_data).await?
            }
        };
        
        let elapsed = start_time.elapsed();
        
        Ok(SandboxResult {
            time_ms: elapsed.as_millis() as u32,
            ..result
        })
    }
    
    /// Execute using Docker container isolation
    async fn execute_docker(&self, command: &str, input_data: &str, image: &str) -> Result<SandboxResult> {
        let container_name = format!("judicia_eval_{}", Uuid::new_v4());
        
        let mut cmd = Command::new("docker");
        cmd.args(&[
            "run",
            "--name", &container_name,
            "--rm",
            "--network=none", // Disable network access
            "--memory", &format!("{}k", self.config.max_memory_kb),
            "--cpus", "1",
            "--ulimit", &format!("nofile={}:{}", self.config.max_open_files, self.config.max_open_files),
            "--ulimit", &format!("fsize={}000", self.config.max_file_size_kb), // in bytes
            "--read-only",
            "--tmpfs", "/tmp:rw,noexec,nosuid,size=100m",
            "-v", &format!("{}:/sandbox:ro", self.work_dir.display()),
            "-w", "/sandbox",
            "--user", "nobody:nogroup",
            image,
            "timeout", &format!("{}s", self.config.max_execution_time_ms / 1000),
            "sh", "-c", command,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
        
        let result = self.execute_with_timeout(cmd, input_data).await?;
        
        // Clean up container if it still exists
        let _ = Command::new("docker")
            .args(&["rm", "-f", &container_name])
            .output()
            .await;
        
        Ok(result)
    }
    
    /// Execute using ioi/isolate sandbox
    async fn execute_isolate(&self, command: &str, input_data: &str, box_id: u32) -> Result<SandboxResult> {
        // Initialize isolate box
        let mut init_cmd = Command::new("isolate");
        init_cmd.args(&[
            "--box-id", &box_id.to_string(),
            "--init",
        ]);
        init_cmd.output().await?;
        
        // Get box directory
        let mut info_cmd = Command::new("isolate");
        info_cmd.args(&[
            "--box-id", &box_id.to_string(),
            "--info",
        ]);
        let info_output = info_cmd.output().await?;
        let box_info = String::from_utf8_lossy(&info_output.stdout);
        
        // Copy files to isolate box
        if let Some(line) = box_info.lines().find(|l| l.starts_with("box-path:")) {
            let box_path = line.strip_prefix("box-path:").unwrap().trim();
            let box_dir = Path::new(box_path).join("box");
            
            // Copy work directory contents to box
            if self.work_dir.exists() {
                let mut entries = fs::read_dir(&self.work_dir).await?;
                while let Some(entry) = entries.next_entry().await? {
                    let src = entry.path();
                    let filename = entry.file_name();
                    let dest = box_dir.join(filename);
                    fs::copy(src, dest).await?;
                }
            }
        }
        
        // Execute command with isolate
        let mut cmd = Command::new("isolate");
        cmd.args(&[
            "--box-id", &box_id.to_string(),
            "--time", &format!("{:.3}", self.config.max_execution_time_ms as f64 / 1000.0),
            "--wall-time", &format!("{:.3}", (self.config.max_execution_time_ms * 2) as f64 / 1000.0),
            "--memory", &self.config.max_memory_kb.to_string(),
            "--fsize", &self.config.max_file_size_kb.to_string(),
            "--processes", "1",
            "--open-files", &self.config.max_open_files.to_string(),
            "--stderr-to-stdout",
            "--run",
            "--", "sh", "-c", command,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
        
        let result = self.execute_with_timeout(cmd, input_data).await?;
        
        // Cleanup isolate box
        let mut cleanup_cmd = Command::new("isolate");
        cleanup_cmd.args(&[
            "--box-id", &box_id.to_string(),
            "--cleanup",
        ]);
        let _ = cleanup_cmd.output().await;
        
        Ok(result)
    }
    
    /// Execute with layered Docker + isolate security
    async fn execute_layered(&self, command: &str, input_data: &str, image: &str, box_id: u32) -> Result<SandboxResult> {
        // For layered security, we run isolate inside Docker
        // This provides multiple layers of isolation
        
        let container_name = format!("judicia_layered_{}", Uuid::new_v4());
        
        let isolate_command = format!(
            "isolate --box-id={} --init && \
             isolate --box-id={} --time={:.3} --memory={} --fsize={} \
             --processes=1 --open-files={} --stderr-to-stdout --run -- sh -c '{}' && \
             isolate --box-id={} --cleanup",
            box_id,
            box_id,
            self.config.max_execution_time_ms as f64 / 1000.0,
            self.config.max_memory_kb,
            self.config.max_file_size_kb,
            self.config.max_open_files,
            command.replace("'", "'\\''"), // Escape single quotes
            box_id
        );
        
        let mut cmd = Command::new("docker");
        cmd.args(&[
            "run",
            "--name", &container_name,
            "--rm",
            "--network=none",
            "--memory", &format!("{}k", self.config.max_memory_kb + 50 * 1024), // Extra memory for isolate overhead
            "--cpus", "1",
            "--read-only",
            "--tmpfs", "/tmp:rw,noexec,nosuid,size=200m",
            "-v", &format!("{}:/sandbox:ro", self.work_dir.display()),
            "-w", "/sandbox",
            "--user", "root", // isolate needs root to set up namespaces
            "--cap-add", "SYS_ADMIN", // Required for isolate
            image,
            "timeout", &format!("{}s", (self.config.max_execution_time_ms + 5000) / 1000),
            "sh", "-c", &isolate_command,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
        
        let result = self.execute_with_timeout(cmd, input_data).await?;
        
        // Clean up container
        let _ = Command::new("docker")
            .args(&["rm", "-f", &container_name])
            .output()
            .await;
        
        Ok(result)
    }
    
    /// Execute with native process (unsafe - for development only)
    async fn execute_native(&self, command: &str, input_data: &str) -> Result<SandboxResult> {
        warn!("Using native execution mode - no security restrictions applied!");
        
        let parts: Vec<&str> = command.split_whitespace().collect();
        let program = parts[0];
        let args = &parts[1..];
        
        let mut cmd = Command::new(program);
        cmd.args(args)
            .current_dir(&self.work_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        
        self.execute_with_timeout(cmd, input_data).await
    }
    
    /// Common execution logic with timeout handling
    async fn execute_with_timeout(&self, mut cmd: Command, input_data: &str) -> Result<SandboxResult> {
        let mut child = cmd.spawn()
            .context("Failed to spawn process")?;
        
        // Write input to stdin
        if let Some(mut stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            let _ = stdin.write_all(input_data.as_bytes()).await;
            let _ = stdin.shutdown().await;
        }
        
        // Wait for completion with timeout
        let timeout = Duration::from_millis(self.config.max_execution_time_ms as u64 + 5000); // Extra buffer
        
        let output = match tokio::time::timeout(timeout, child.wait_with_output()).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                return Ok(SandboxResult {
                    exit_code: -1,
                    stdout: String::new(),
                    stderr: format!("Process error: {}", e),
                    time_ms: self.config.max_execution_time_ms,
                    memory_kb: 0,
                    verdict: SandboxVerdict::InternalError,
                    system_info: HashMap::new(),
                });
            }
            Err(_) => {
                // Timeout occurred - the child process has already been consumed by wait_with_output
                // So we can't kill it here, but the timeout should have handled the cleanup
                return Ok(SandboxResult {
                    exit_code: 124, // Standard timeout exit code
                    stdout: String::new(),
                    stderr: "Time limit exceeded".to_string(),
                    time_ms: self.config.max_execution_time_ms,
                    memory_kb: 0,
                    verdict: SandboxVerdict::TimeLimitExceeded,
                    system_info: HashMap::new(),
                });
            }
        };
        
        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        // Determine verdict based on exit code and output
        let verdict = match exit_code {
            0 => SandboxVerdict::OK,
            124 => SandboxVerdict::TimeLimitExceeded,
            137 => SandboxVerdict::MemoryLimitExceeded, // SIGKILL often indicates OOM
            _ => {
                if stderr.contains("Time limit exceeded") || stderr.contains("timeout") {
                    SandboxVerdict::TimeLimitExceeded
                } else if stderr.contains("Memory limit exceeded") || stderr.contains("out of memory") {
                    SandboxVerdict::MemoryLimitExceeded
                } else if stderr.contains("Security violation") || stderr.contains("forbidden") {
                    SandboxVerdict::SecurityViolation
                } else {
                    SandboxVerdict::RuntimeError
                }
            }
        };
        
        let mut system_info = HashMap::new();
        system_info.insert("exit_code".to_string(), serde_json::Value::Number(exit_code.into()));
        system_info.insert("mode".to_string(), serde_json::Value::String(format!("{:?}", self.mode)));
        
        Ok(SandboxResult {
            exit_code,
            stdout,
            stderr,
            time_ms: 0, // Will be filled by caller
            memory_kb: self.estimate_memory_usage(), // Rough estimate
            verdict,
            system_info,
        })
    }
    
    /// Initialize Docker environment
    async fn initialize_docker(&self) -> Result<()> {
        // Check if Docker is available
        let output = Command::new("docker")
            .args(&["version", "--format", "{{.Server.Version}}"])
            .output()
            .await?;
            
        if !output.status.success() {
            return Err(anyhow::anyhow!("Docker is not available or not running"));
        }
        
        debug!("Docker available: {}", String::from_utf8_lossy(&output.stdout).trim());
        Ok(())
    }
    
    /// Initialize isolate sandbox
    async fn initialize_isolate(&self, box_id: u32) -> Result<()> {
        // Check if isolate is available
        let output = Command::new("isolate")
            .args(&["--version"])
            .output()
            .await?;
            
        if !output.status.success() {
            return Err(anyhow::anyhow!("isolate is not available"));
        }
        
        debug!("isolate available: {}", String::from_utf8_lossy(&output.stdout).trim());
        
        // Clean up any existing box
        let _ = Command::new("isolate")
            .args(&["--box-id", &box_id.to_string(), "--cleanup"])
            .output()
            .await;
        
        Ok(())
    }
    
    /// Rough memory usage estimation
    fn estimate_memory_usage(&self) -> u32 {
        // This is a placeholder - in real implementation, memory usage would be
        // reported by isolate or read from cgroups
        1024 // 1MB baseline
    }
    
    /// Clean up sandbox resources
    pub async fn cleanup(&self) -> Result<()> {
        if self.work_dir.exists() {
            fs::remove_dir_all(&self.work_dir).await
                .context("Failed to clean up sandbox work directory")?;
        }
        
        match &self.mode {
            SandboxMode::Isolate { box_id } | SandboxMode::Layered { box_id, .. } => {
                let _ = Command::new("isolate")
                    .args(&["--box-id", &box_id.to_string(), "--cleanup"])
                    .output()
                    .await;
            }
            _ => {}
        }
        
        debug!("Sandbox cleaned up");
        Ok(())
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        // Ensure cleanup happens even if not called explicitly
        if self.work_dir.exists() {
            let work_dir = self.work_dir.clone();
            tokio::spawn(async move {
                let _ = fs::remove_dir_all(work_dir).await;
            });
        }
    }
}