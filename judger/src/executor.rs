use anyhow::Result;
use std::{path::PathBuf, sync::Arc};
use tempfile::TempDir;
use shared::Language;

use crate::{config::Config, sandbox::Sandbox};

#[derive(Clone)]
pub struct Executor {
    config: Arc<Config>,
    sandbox: Sandbox,
}

pub struct CompileResult {
    pub success: bool,
    pub executable_path: PathBuf,
    #[allow(dead_code)]
    pub error_message: Option<String>,
}

pub struct RunResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub time_ms: i32,
    pub memory_kb: i32,
}

impl Executor {
    pub fn new(config: Arc<Config>) -> Result<Self> {
        let sandbox = Sandbox::new()?;
        
        Ok(Executor {
            config,
            sandbox,
        })
    }

    pub async fn compile(&self, source_code: &str, language: &Language) -> Result<CompileResult> {
        // Create temporary directory for compilation
        let temp_dir = TempDir::new()?;
        let source_file = temp_dir.path().join(format!("solution.{}", language.file_extension));
        let executable_path = temp_dir.path().join("solution");

        // Write source code to file
        tokio::fs::write(&source_file, source_code).await?;

        // If no compile command, return the source file as executable (for interpreted languages)
        let Some(compile_command) = &language.compile_command else {
            return Ok(CompileResult {
                success: true,
                executable_path: source_file,
                error_message: None,
            });
        };

        // Replace placeholders in compile command
        let compile_cmd = compile_command
            .replace("solution.cpp", source_file.to_str().unwrap())
            .replace("solution", executable_path.to_str().unwrap())
            .replace("Solution.java", source_file.to_str().unwrap());

        // Execute compile command in sandbox
        let result = self.sandbox.execute_command(
            &compile_cmd,
            "",
            5000, // 5 second compile timeout
            256 * 1024, // 256MB memory limit for compilation
            temp_dir.path(),
        ).await?;

        if result.exit_code == 0 {
            // Move executable to persistent location
            let persistent_path = PathBuf::from(&self.config.work_dir)
                .join(format!("executable_{}", uuid::Uuid::new_v4()));
            
            if executable_path.exists() {
                tokio::fs::copy(&executable_path, &persistent_path).await?;
            } else {
                // For interpreted languages, copy the source file
                tokio::fs::copy(&source_file, &persistent_path).await?;
            }

            Ok(CompileResult {
                success: true,
                executable_path: persistent_path,
                error_message: None,
            })
        } else {
            Ok(CompileResult {
                success: false,
                executable_path: PathBuf::new(),
                error_message: Some(result.stderr),
            })
        }
    }

    pub async fn run(
        &self,
        executable_path: &PathBuf,
        input_data: &str,
        time_limit_ms: i32,
        memory_limit_kb: i32,
    ) -> Result<RunResult> {
        // Create run command based on file extension
        let run_cmd = if executable_path.extension().map(|s| s.to_str()) == Some(Some("py")) {
            format!("python3 {}", executable_path.to_str().unwrap())
        } else if executable_path.extension().map(|s| s.to_str()) == Some(Some("js")) {
            format!("node {}", executable_path.to_str().unwrap())
        } else if executable_path.extension().map(|s| s.to_str()) == Some(Some("java")) {
            // For Java, we need to handle class name extraction
            format!("java -cp {} Solution", executable_path.parent().unwrap().to_str().unwrap())
        } else {
            // Assume it's a compiled executable
            executable_path.to_str().unwrap().to_string()
        };

        let result = self.sandbox.execute_command(
            &run_cmd,
            input_data,
            time_limit_ms,
            memory_limit_kb,
            executable_path.parent().unwrap(),
        ).await?;

        Ok(RunResult {
            exit_code: result.exit_code,
            stdout: result.stdout,
            stderr: result.stderr,
            time_ms: result.time_ms,
            memory_kb: result.memory_kb,
        })
    }
}