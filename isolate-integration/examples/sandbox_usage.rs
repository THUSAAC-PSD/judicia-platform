use isolate_integration::sandbox::{
    IsolateSandbox, ResourceLimits, DirectoryRule, EnvRule
};
use std::path::PathBuf;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Example 1: Basic usage
    basic_example().await?;
    
    // Example 2: Advanced configuration
    advanced_example().await?;
    
    // Example 3: Compile and run a C++ program
    cpp_example().await?;
    
    Ok(())
}

async fn basic_example() -> Result<()> {
    println!("=== Basic Example ===");
    
    // Create sandbox with box ID 0
    let sandbox = IsolateSandbox::new(0)
        .with_stdin("input.txt")
        .with_stdout("output.txt")
        .with_stderr("error.txt")
        .with_meta_file(PathBuf::from("/tmp/meta.txt"));

    // Set resource limits
    let limits = ResourceLimits::new()
        .with_time_limit(1.0)      // 1 second
        .with_memory_limit(64 * 1024)  // 64 MB
        .with_wall_time_limit(5.0)     // 5 seconds wall time
        .with_process_limit(1);        // 1 process only

    // Initialize sandbox
    sandbox.init(&limits).await?;

    // Run a simple command
    let result = sandbox.run("echo", ["Hello, World!"], &limits).await?;

    println!("Exit code: {:?}", result.exit_code);
    println!("Time used: {:.3}s", result.time_used);
    println!("Memory used: {} KB", result.memory_used);
    println!("Output: {}", result.stdout);

    // Cleanup
    sandbox.cleanup().await?;
    
    Ok(())
}

async fn advanced_example() -> Result<()> {
    println!("\n=== Advanced Example ===");
    
    // Create sandbox with advanced configuration
    let sandbox = IsolateSandbox::new(1)
        .with_directory_rule(DirectoryRule::bind("/tmp/work", "/tmp/sandbox_work").read_write())
        .with_directory_rule(DirectoryRule::tmp("/tmp/temp"))
        .with_directory_rule(DirectoryRule::bind_same("/usr/bin").no_exec())
        .with_env_rule(EnvRule::Set("PATH".to_string(), "/usr/bin:/bin".to_string()))
        .with_env_rule(EnvRule::Inherit("HOME".to_string()))
        .with_stdin("input.txt")
        .with_stdout("output.txt")
        .with_chdir("/tmp/work")
        .with_meta_file(PathBuf::from("/tmp/meta1.txt"))
        .use_cgroups()
        .no_default_dirs()
        .verbose();

    // Set comprehensive resource limits
    let limits = ResourceLimits::new()
        .with_time_limit(2.0)
        .with_wall_time_limit(10.0)
        .with_memory_limit(128 * 1024)  // 128 MB
        .with_cg_memory_limit(256 * 1024)  // 256 MB for control group
        .with_process_limit(5);

    // Initialize and run
    sandbox.init(&limits).await?;
    
    let result = sandbox.run("ls", ["-la", "/tmp"], &limits).await?;
    
    println!("Status: {}", result.status);
    println!("Message: {}", result.message);
    println!("Time used: {:.3}s", result.time_used);
    println!("Wall time: {:.3}s", result.wall_time_used);
    println!("Memory used: {} KB", result.memory_used);
    if let Some(cg_mem) = result.cg_memory_used {
        println!("CG Memory used: {} KB", cg_mem);
    }
    
    sandbox.cleanup().await?;
    
    Ok(())
}

async fn cpp_example() -> Result<()> {
    println!("\n=== C++ Compile and Run Example ===");
    
    // Prepare source code
    let source_code = r#"
#include <iostream>
#include <vector>
#include <algorithm>

int main() {
    std::vector<int> numbers = {5, 2, 8, 1, 9};
    std::sort(numbers.begin(), numbers.end());
    
    for (int n : numbers) {
        std::cout << n << " ";
    }
    std::cout << std::endl;
    
    return 0;
}
"#;

    // Write source to file
    tokio::fs::write("/tmp/solution.cpp", source_code).await?;

    // Step 1: Compile
    let compile_sandbox = IsolateSandbox::new(2)
        .with_directory_rule(DirectoryRule::bind("/tmp", "/tmp").read_write())
        .with_stderr("compile_error.txt")
        .with_meta_file(PathBuf::from("/tmp/compile_meta.txt"));

    let compile_limits = ResourceLimits::new()
        .with_time_limit(10.0)  // 10 seconds for compilation
        .with_memory_limit(512 * 1024)  // 512 MB for compilation
        .with_process_limit(10);  // Allow multiple processes for g++

    compile_sandbox.init(&compile_limits).await?;
    
    let compile_result = compile_sandbox.run(
        "g++", 
        ["-O2", "-std=c++17", "/tmp/solution.cpp", "-o", "/tmp/solution"], 
        &compile_limits
    ).await?;

    println!("Compilation exit code: {:?}", compile_result.exit_code);
    println!("Compilation time: {:.3}s", compile_result.time_used);
    
    if compile_result.exit_code != Some(0) {
        println!("Compilation failed: {}", compile_result.stderr);
        compile_sandbox.cleanup().await?;
        return Ok(());
    }

    compile_sandbox.cleanup().await?;

    // Step 2: Run the compiled program
    let run_sandbox = IsolateSandbox::new(3)
        .with_directory_rule(DirectoryRule::bind("/tmp/solution", "/tmp/solution"))
        .with_stdout("program_output.txt")
        .with_stderr("program_error.txt")
        .with_meta_file(PathBuf::from("/tmp/run_meta.txt"));

    let run_limits = ResourceLimits::new()
        .with_time_limit(1.0)  // 1 second for execution
        .with_memory_limit(64 * 1024)  // 64 MB for execution
        .with_process_limit(1);

    run_sandbox.init(&run_limits).await?;
    
    let run_result = run_sandbox.run("/tmp/solution", Vec::<String>::new(), &run_limits).await?;

    println!("Execution exit code: {:?}", run_result.exit_code);
    println!("Execution time: {:.3}s", run_result.time_used);
    println!("Execution memory: {} KB", run_result.memory_used);
    println!("Program output: {}", run_result.stdout);

    if run_result.killed {
        println!("Program was killed (probably TLE/MLE)");
    }

    run_sandbox.cleanup().await?;

    Ok(())
}
