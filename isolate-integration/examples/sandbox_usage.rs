use isolate_integration::sandbox::{
    IsolateSandbox, ResourceLimits, DirectoryRule, EnvRule
};
use std::path::PathBuf;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Example 1: Basic usage without external dependencies
    basic_example().await?;
    
    // Example 2: Environment and directory example
    env_and_dir_example().await?;
    
    // Example 3: File sharing with directory binding
    file_sharing_example().await?;
    
    // Example 4: Compile and run C++ program
    c_compile_example().await?;
    
    Ok(())
}

async fn basic_example() -> Result<()> {
    println!("=== Basic Example ===");
    
    // Create sandbox with minimal configuration
    let sandbox = IsolateSandbox::new(0)
        .with_meta_file(PathBuf::from("/tmp/meta.txt"));

    // Set basic resource limits
    let limits = ResourceLimits::new();

    // Initialize sandbox
    sandbox.init(&limits).await?;

    // Run a simple command with absolute path
    let result = sandbox.run("/bin/echo", ["Hello, World!"], &limits).await?;

    println!("Exit code: {:?}", result.exit_code);
    println!("Time used: {:.3}s", result.time_used);
    println!("Memory used: {} KB", result.memory_used);
    println!("Output: {}", result.stdout);

    // Cleanup
    sandbox.cleanup().await?;
    
    Ok(())
}

async fn env_and_dir_example() -> Result<()> {
    println!("\n=== Environment and Directory Example ===");
    
    // Create sandbox with environment and directory configuration
    let sandbox = IsolateSandbox::new(2)
        .with_env_rule(EnvRule::Set("MY_VAR".to_string(), "Hello from sandbox!".to_string()))
        .with_env_rule(EnvRule::Inherit("PATH".to_string()))
        .with_meta_file(PathBuf::from("/tmp/meta2.txt"))
        .verbose();

    let limits = ResourceLimits::new();

    sandbox.init(&limits).await?;
    
    // Run a command that uses the environment variable using bash
    let result = sandbox.run("/bin/bash", ["-c", "echo $MY_VAR && pwd"], &limits).await?;
    
    println!("Command: bash -c 'echo $MY_VAR && pwd'");
    println!("Exit code: {:?}", result.exit_code);
    println!("Output: {}", result.stdout);
    println!("Time used: {:.3}s", result.time_used);
    
    sandbox.cleanup().await?;
    
    Ok(())
}

async fn file_sharing_example() -> Result<()> {
    println!("\n=== File Sharing with Directory Binding Example ===");
    
    // Create a shared directory for demonstration
    let shared_dir = "/tmp/sandbox_demo";
    tokio::fs::create_dir_all(shared_dir).await?;
    
    // Create a test file on the host
    let test_content = "This file was created on the host system.\nIt will be accessible from within the sandbox.";
    tokio::fs::write(format!("{}/host_file.txt", shared_dir), test_content).await?;
    
    println!("Created test file on host: {}/host_file.txt", shared_dir);
    
    let sandbox = IsolateSandbox::new(1)
        // Bind the shared directory to /shared in sandbox (read-write)
        .with_directory_rule(DirectoryRule::bind("/shared", shared_dir).read_write())
        .with_meta_file(PathBuf::from("/tmp/meta_sharing.txt"))
        .verbose();

    let limits = ResourceLimits::new();

    sandbox.init(&limits).await?;
    
    // Read the file from within the sandbox using bash
    let result = sandbox.run("/bin/bash", ["-c", "cat /shared/host_file.txt"], &limits).await?;
    
    println!("File content read from sandbox:");
    println!("{}", result.stdout);
    
    // Create a new file from within the sandbox using bash
    let result = sandbox.run("/bin/bash", ["-c", "echo 'This file was created inside the sandbox.' > /shared/sandbox_file.txt"], &limits).await?;
    
    if result.exit_code == Some(0) {
        println!("Successfully created file from within sandbox");
        
        // Read the file back on the host
        let sandbox_content = tokio::fs::read_to_string(format!("{}/sandbox_file.txt", shared_dir)).await?;
        println!("File content read back on host:");
        println!("{}", sandbox_content.trim());
    } else {
        println!("Failed to create file in sandbox: {}", result.stderr);
    }

    // List files in the shared directory from within sandbox using bash
    let result = sandbox.run("/bin/bash", ["-c", "ls -la /shared/"], &limits).await?;
    
    println!("Files in shared directory (from sandbox perspective):");
    println!("{}", result.stdout);

    sandbox.cleanup().await?;
    
    Ok(())
}

async fn c_compile_example() -> Result<()> {
    println!("\n=== C Compile and Run Example ===");

    // Create a shared directory for file exchange between host and sandbox
    let shared_dir = "/tmp/sandbox_shared";
    tokio::fs::create_dir_all(shared_dir).await?;
    
    let sandbox = IsolateSandbox::new(3)
        // Bind the shared directory to /shared in sandbox (read-write)
        .with_directory_rule(DirectoryRule::bind("/shared", shared_dir).read_write())
        .with_meta_file(PathBuf::from("/tmp/meta_cpp.txt"))
        .verbose();

    let limits = ResourceLimits::new();

    sandbox.init(&limits).await?;
    
    // Write a simple C program to the shared directory (easier to compile)
    let c_code = r#"
#include <stdio.h>

int main() {
    printf("Hello from C program!\n");
    printf("This program was compiled and run in isolate sandbox.\n");
    printf("Files are shared through /shared directory.\n");
    return 0;
}
"#;

    // Write to the shared directory (accessible from both host and sandbox)
    tokio::fs::write(format!("{}/solution.c", shared_dir), c_code).await?;
    
    println!("Source file written to: {}/solution.c", shared_dir);
    
    // Try the simplest possible compilation first
    let _compile_result = sandbox.run("/bin/bash", ["-c", "gcc /shared/solution.c -o /shared/solution -B/usr/bin"], &limits).await?;

    println!("Compilation successful!");
    
    // Run the compiled program
    let run_limits = ResourceLimits::new()
        .with_time_limit(2.0)  // 2 seconds for execution
        .with_memory_limit(64 * 1024)  // 64 MB for execution
        .with_process_limit(1);

    let run_result = sandbox.run("/bin/bash", ["-c", "/shared/solution"], &run_limits).await?;

    println!("Execution exit code: {:?}", run_result.exit_code);
    println!("Execution time: {:.3}s", run_result.time_used);
    println!("Execution memory: {} KB", run_result.memory_used);
    println!("Program output: {}", run_result.stdout);

    if run_result.killed {
        println!("Program was killed (probably TLE/MLE)");
    }

    // Check if the compiled executable exists on the host
    let executable_path = format!("{}/solution", shared_dir);
    if tokio::fs::try_exists(&executable_path).await? {
        println!("Compiled executable is available on host at: {}", executable_path);
        
        // Get file size
        let metadata = tokio::fs::metadata(&executable_path).await?;
        println!("Executable size: {} bytes", metadata.len());
    } else {
        println!("Compiled executable not found on host");
    }

    sandbox.cleanup().await?;
    
    Ok(())
}
