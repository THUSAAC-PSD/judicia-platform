use anyhow::{Context, Result, bail};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::process::Command;

/// Directory binding rule
#[derive(Debug, Clone)]
pub struct DirectoryRule {
    pub inside_path: PathBuf,
    pub outside_path: Option<PathBuf>,
    pub options: DirectoryOptions,
}

/// Directory binding options
/// NOTE: Unless --no-default-dirs is specified, the default set of directory rules binds /bin, /dev (with devices allowed), /lib, /lib64 (if it exists), and /usr.
/// It also binds the working directory to /box (read-write), mounts the proc filesystem at /proc, and creates a temporary directory /tmp.
#[derive(Debug, Clone, Default)]
pub struct DirectoryOptions {
    pub read_write: bool,
    pub allow_devices: bool,
    pub no_exec: bool,
    pub maybe: bool,
    pub is_filesystem: bool,
    pub is_tmp: bool,
    pub no_recursive: bool,
}

/// Environment variable rule
#[derive(Debug, Clone)]
pub enum EnvRule {
    Inherit(String),
    Set(String, String),
    FullEnv,
}

/// Resource limits for isolate sandbox
/// All size-related items are in kilobytes (kB), time-related items are in seconds (s).
/// NOTE: use cgroups-related options first to control memory precisely.
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub time_limit: Option<f64>,
    pub wall_time_limit: Option<f64>,
    pub extra_time: Option<f64>,
    pub memory_limit: Option<u32>,
    pub cg_memory_limit: Option<u32>,
    pub stack_limit: Option<u32>,
    pub open_files_limit: Option<u32>,
    pub file_size_limit: Option<u32>,
    pub core_limit: Option<u32>,
    pub process_limit: Option<u32>,
    pub quota: Option<(u32, u32)>,
}

/// Special options for isolate
#[derive(Debug, Clone, Default)]
pub struct SpecialOptions {
    pub share_net: bool,
    pub inherit_fds: bool,
    pub tty_hack: bool,
    pub special_files: bool,
    pub use_cgroups: bool,
    pub no_default_dirs: bool,
    pub verbose: bool,
    pub silent: bool,
    pub wait: bool,
    pub as_uid: Option<u32>,
    pub as_gid: Option<u32>,
}

/// Execution result from isolate
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub exit_code: Option<i32>,
    pub signal: Option<i32>,
    pub time_used: f64,
    pub wall_time_used: f64,
    pub memory_used: u32,
    pub cg_memory_used: Option<u32>,
    pub killed: bool,
    pub cg_oom_killed: bool,
    pub status: String,
    pub message: String,
    pub stdout: String,
    pub stderr: String,
}

/// Main isolate sandbox implementation
pub struct IsolateSandbox {
    pub box_id: u32,
    pub isolate_bin: String,
    pub directory_rules: Vec<DirectoryRule>,
    pub env_rules: Vec<EnvRule>,
    pub stdin_file: Option<String>,
    pub stdout_file: Option<String>,
    pub stderr_file: Option<String>,
    pub stderr_to_stdout: bool,
    pub chdir: Option<String>,
    pub meta_file: Option<PathBuf>,
    pub special_options: SpecialOptions,
}

impl ResourceLimits {
    pub fn new() -> Self {
        Self {
            time_limit: None,
            wall_time_limit: None,
            extra_time: None,
            memory_limit: None,
            cg_memory_limit: None,
            stack_limit: None,
            open_files_limit: None,
            file_size_limit: None,
            core_limit: None,
            process_limit: None,
            quota: None,
        }
    }

    pub fn with_time_limit(mut self, seconds: f64) -> Self {
        self.time_limit = Some(seconds);
        self
    }

    pub fn with_memory_limit(mut self, kilobytes: u32) -> Self {
        self.memory_limit = Some(kilobytes);
        self
    }

    pub fn with_wall_time_limit(mut self, seconds: f64) -> Self {
        self.wall_time_limit = Some(seconds);
        self
    }

    pub fn with_cg_memory_limit(mut self, kilobytes: u32) -> Self {
        self.cg_memory_limit = Some(kilobytes);
        self
    }

    pub fn with_process_limit(mut self, count: u32) -> Self {
        self.process_limit = Some(count);
        self
    }
}

impl DirectoryRule {
    pub fn bind(inside: impl Into<PathBuf>, outside: impl Into<PathBuf>) -> Self {
        Self {
            inside_path: inside.into(),
            outside_path: Some(outside.into()),
            options: DirectoryOptions::default(),
        }
    }

    pub fn bind_same(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        Self {
            inside_path: path.clone(),
            outside_path: Some(path),
            options: DirectoryOptions::default(),
        }
    }

    pub fn tmp(inside: impl Into<PathBuf>) -> Self {
        Self {
            inside_path: inside.into(),
            outside_path: None,
            options: DirectoryOptions {
                is_tmp: true,
                read_write: true,
                ..Default::default()
            },
        }
    }

    pub fn filesystem(name: impl Into<PathBuf>) -> Self {
        Self {
            inside_path: name.into(),
            outside_path: None,
            options: DirectoryOptions {
                is_filesystem: true,
                ..Default::default()
            },
        }
    }

    pub fn read_write(mut self) -> Self {
        self.options.read_write = true;
        self
    }

    pub fn allow_devices(mut self) -> Self {
        self.options.allow_devices = true;
        self
    }

    pub fn no_exec(mut self) -> Self {
        self.options.no_exec = true;
        self
    }

    pub fn maybe(mut self) -> Self {
        self.options.maybe = true;
        self
    }

    pub fn no_recursive(mut self) -> Self {
        self.options.no_recursive = true;
        self
    }
}

impl IsolateSandbox {
    pub fn new(box_id: u32) -> Self {
        Self {
            box_id,
            isolate_bin: std::env::var("ISOLATE_BIN").unwrap_or_else(|_| "isolate".to_string()),
            directory_rules: Vec::new(),
            env_rules: vec![EnvRule::Set(
                "LIBC_FATAL_STDERR_".to_string(), // send fatal errors to stderr by default
                "1".to_string(),
            )],
            stdin_file: None,
            stdout_file: None,
            stderr_file: None,
            stderr_to_stdout: false,
            chdir: None,
            meta_file: None,
            special_options: SpecialOptions {
                use_cgroups: true, // NOTE: enable cgroup by default
                ..Default::default()
            },
        }
    }

    /// Initialize the sandbox
    pub async fn init(&self, limits: &ResourceLimits) -> Result<()> {
        let mut cmd = Command::new(&self.isolate_bin);

        cmd.arg("--box-id").arg(self.box_id.to_string());
        cmd.arg("--init");

        if let Some((blocks, inodes)) = limits.quota {
            cmd.arg("--quota").arg(format!("{},{}", blocks, inodes));
        }

        // Add special options
        if self.special_options.use_cgroups {
            cmd.arg("--cg");
        }
        if self.special_options.verbose {
            cmd.arg("--verbose");
        }
        if self.special_options.silent {
            cmd.arg("--silent");
        }
        if self.special_options.wait {
            cmd.arg("--wait");
        }
        if let Some(uid) = self.special_options.as_uid {
            cmd.arg("--as-uid").arg(uid.to_string());
        }
        if let Some(gid) = self.special_options.as_gid {
            cmd.arg("--as-gid").arg(gid.to_string());
        }

        let output = cmd
            .output()
            .await
            .context("Failed to execute isolate --init")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("isolate --init failed: {}", stderr);
        }

        Ok(())
    }

    /// Run a command in the sandbox
    pub async fn run<I, S>(
        &self,
        program: &str,
        args: I,
        limits: &ResourceLimits,
    ) -> Result<ExecutionResult>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut cmd = Command::new(&self.isolate_bin);

        cmd.arg("--box-id").arg(self.box_id.to_string());
        cmd.arg("--run");

        // Add resource limits
        if let Some(time) = limits.time_limit {
            cmd.arg("--time").arg(time.to_string());
        }
        if let Some(wall_time) = limits.wall_time_limit {
            cmd.arg("--wall-time").arg(wall_time.to_string());
        }
        if let Some(extra_time) = limits.extra_time {
            cmd.arg("--extra-time").arg(extra_time.to_string());
        }
        if let Some(memory) = limits.memory_limit {
            cmd.arg("--mem").arg(memory.to_string());
        }
        if let Some(cg_memory) = limits.cg_memory_limit {
            cmd.arg("--cg-mem").arg(cg_memory.to_string());
        }
        if let Some(stack) = limits.stack_limit {
            cmd.arg("--stack").arg(stack.to_string());
        }
        if let Some(open_files) = limits.open_files_limit {
            cmd.arg("--open-files").arg(open_files.to_string());
        }
        if let Some(file_size) = limits.file_size_limit {
            cmd.arg("--fsize").arg(file_size.to_string());
        }
        if let Some(core) = limits.core_limit {
            cmd.arg("--core").arg(core.to_string());
        }
        if let Some(processes) = limits.process_limit {
            if processes == 0 {
                cmd.arg("--processes");
            } else {
                cmd.arg("--processes").arg(processes.to_string());
            }
        }

        // Add I/O redirection
        if let Some(ref stdin) = self.stdin_file {
            cmd.arg("--stdin").arg(stdin);
        }
        if let Some(ref stdout) = self.stdout_file {
            cmd.arg("--stdout").arg(stdout);
        }
        if let Some(ref stderr) = self.stderr_file {
            cmd.arg("--stderr").arg(stderr);
        }
        if self.stderr_to_stdout {
            cmd.arg("--stderr-to-stdout");
        }

        // Add directory change
        if let Some(ref chdir) = self.chdir {
            cmd.arg("--chdir").arg(chdir);
        }

        // Add meta file
        if let Some(ref meta) = self.meta_file {
            cmd.arg("--meta").arg(meta);
        }

        // Add directory rules
        if self.special_options.no_default_dirs {
            cmd.arg("--no-default-dirs");
        }
        for rule in &self.directory_rules {
            let mut dir_arg = if rule.options.is_filesystem {
                format!("{}:fs", rule.inside_path.display())
            } else if rule.options.is_tmp {
                format!("{}:tmp", rule.inside_path.display())
            } else if let Some(ref outside) = rule.outside_path {
                format!("{}={}", rule.inside_path.display(), outside.display())
            } else {
                rule.inside_path.display().to_string()
            };

            let mut options = Vec::new();
            if rule.options.read_write {
                options.push("rw");
            }
            if rule.options.allow_devices {
                options.push("dev");
            }
            if rule.options.no_exec {
                options.push("noexec");
            }
            if rule.options.maybe {
                options.push("maybe");
            }
            if rule.options.no_recursive {
                options.push("norec");
            }

            if !options.is_empty() {
                dir_arg.push(':');
                dir_arg.push_str(&options.join(","));
            }

            cmd.arg("--dir").arg(dir_arg);
        }

        // Add environment rules
        for rule in &self.env_rules {
            match rule {
                EnvRule::Inherit(var) => {
                    cmd.arg("--env").arg(var);
                }
                EnvRule::Set(var, value) => {
                    cmd.arg("--env").arg(format!("{}={}", var, value));
                }
                EnvRule::FullEnv => {
                    cmd.arg("--full-env");
                }
            }
        }

        // Add special options
        if self.special_options.use_cgroups {
            cmd.arg("--cg");
        }
        if self.special_options.share_net {
            cmd.arg("--share-net");
        }
        if self.special_options.inherit_fds {
            cmd.arg("--inherit-fds");
        }
        if self.special_options.tty_hack {
            cmd.arg("--tty-hack");
        }
        if self.special_options.special_files {
            cmd.arg("--special-files");
        }
        if self.special_options.verbose {
            cmd.arg("--verbose");
        }
        if self.special_options.silent {
            cmd.arg("--silent");
        }

        // Add the command to execute
        cmd.arg("--").arg(program);
        for arg in args {
            cmd.arg(arg.as_ref());
        }

        let output = cmd
            .output()
            .await
            .context("Failed to execute isolate --run")?;

        // Read output files if specified
        let stdout = if let Some(ref stdout_file) = self.stdout_file {
            fs::read_to_string(stdout_file).await.unwrap_or_default()
        } else {
            String::from_utf8_lossy(&output.stdout).to_string()
        };

        let stderr = if let Some(ref stderr_file) = self.stderr_file {
            fs::read_to_string(stderr_file).await.unwrap_or_default()
        } else {
            String::from_utf8_lossy(&output.stderr).to_string()
        };

        // Parse metadata if available
        let metadata = if let Some(ref meta_file) = self.meta_file {
            self.parse_metadata(meta_file).await.unwrap_or_default()
        } else {
            HashMap::new()
        };

        Ok(ExecutionResult {
            exit_code: metadata.get("exitcode").and_then(|s| s.parse().ok()),
            signal: metadata.get("exitsig").and_then(|s| s.parse().ok()),
            time_used: metadata
                .get("time")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0),
            wall_time_used: metadata
                .get("time-wall")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0),
            memory_used: metadata
                .get("max-rss")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            cg_memory_used: metadata.get("cg-mem").and_then(|s| s.parse().ok()),
            killed: metadata.get("killed").map(|s| s == "1").unwrap_or(false),
            cg_oom_killed: metadata.get("cg-oom-killed").is_some(),
            status: metadata.get("status").cloned().unwrap_or_default(),
            message: metadata.get("message").cloned().unwrap_or_default(),
            stdout,
            stderr,
        })
    }

    /// Cleanup the sandbox
    pub async fn cleanup(&self) -> Result<()> {
        let mut cmd = Command::new(&self.isolate_bin);

        cmd.arg("--box-id").arg(self.box_id.to_string());
        cmd.arg("--cleanup");

        if self.special_options.use_cgroups {
            cmd.arg("--cg");
        }

        let output = cmd
            .output()
            .await
            .context("Failed to execute isolate --cleanup")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("isolate --cleanup warning: {}", stderr);
        }

        Ok(())
    }

    async fn parse_metadata(&self, meta_file: &PathBuf) -> Result<HashMap<String, String>> {
        let content = fs::read_to_string(meta_file)
            .await
            .context("Failed to read metadata file")?;

        let mut metadata = HashMap::new();
        for line in content.lines() {
            if let Some((key, value)) = line.split_once(':') {
                metadata.insert(key.to_string(), value.to_string());
            }
        }

        Ok(metadata)
    }

    /// The following are builder options.

    pub fn with_directory_rule(mut self, rule: DirectoryRule) -> Self {
        self.directory_rules.push(rule);
        self
    }

    pub fn with_env_rule(mut self, rule: EnvRule) -> Self {
        self.env_rules.push(rule);
        self
    }

    pub fn with_stdin(mut self, file: impl Into<String>) -> Self {
        self.stdin_file = Some(file.into());
        self
    }

    pub fn with_stdout(mut self, file: impl Into<String>) -> Self {
        self.stdout_file = Some(file.into());
        self
    }

    pub fn with_stderr(mut self, file: impl Into<String>) -> Self {
        self.stderr_file = Some(file.into());
        self
    }

    pub fn with_stderr_to_stdout(mut self) -> Self {
        self.stderr_to_stdout = true;
        self
    }

    pub fn with_chdir(mut self, dir: impl Into<String>) -> Self {
        self.chdir = Some(dir.into());
        self
    }

    pub fn with_meta_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.meta_file = Some(file.into());
        self
    }

    pub fn with_special_options(mut self, options: SpecialOptions) -> Self {
        self.special_options = options;
        self
    }

    pub fn use_cgroups(mut self) -> Self {
        self.special_options.use_cgroups = true;
        self
    }

    pub fn disable_cgroups(mut self) -> Self {
        self.special_options.use_cgroups = false;
        self
    }

    pub fn share_network(mut self) -> Self {
        self.special_options.share_net = true;
        self
    }

    pub fn no_default_dirs(mut self) -> Self {
        self.special_options.no_default_dirs = true;
        self
    }

    pub fn verbose(mut self) -> Self {
        self.special_options.verbose = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_cgroups_enabled() {
        let sandbox = IsolateSandbox::new(0);
        assert!(
            sandbox.special_options.use_cgroups,
            "Cgroups should be enabled by default"
        );
    }

    #[test]
    fn test_disable_cgroups() {
        let sandbox = IsolateSandbox::new(0).disable_cgroups();
        assert!(
            !sandbox.special_options.use_cgroups,
            "Cgroups should be disabled after calling disable_cgroups"
        );
    }

    #[test]
    fn test_use_cgroups() {
        let sandbox = IsolateSandbox::new(0).disable_cgroups().use_cgroups();
        assert!(
            sandbox.special_options.use_cgroups,
            "Cgroups should be enabled after calling use_cgroups"
        );
    }
}
