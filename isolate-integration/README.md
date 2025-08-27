# Isolate Integration

这个 crate 提供了一个完整的 Rust 接口来使用 [ioi/isolate](https://github.com/ioi/isolate) 沙箱程序。

## 特性

- **完整的 isolate 支持**：支持 isolate 的所有选项和功能
- **类型安全**：使用 Rust 类型系统确保配置正确性
- **异步支持**：基于 tokio 的异步 API
- **详细结果**：解析 isolate 的元数据文件获取精确的执行信息
- **默认启用 cgroup**：自动使用 Linux 控制组进行更好的资源管理

## 前提条件

确保系统已安装 isolate：

```bash
# Ubuntu/Debian
sudo apt-get install isolate

# 或从源码编译
git clone https://github.com/ioi/isolate.git
cd isolate
make && sudo make install
```

同时请确保系统支持 cgroup v2。

## 基本用法

```rust
use isolate_integration::{IsolateSandbox, ResourceLimits};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建沙箱（默认启用 cgroup）
    let sandbox = IsolateSandbox::new(0)
        .with_stdin("input.txt")
        .with_stdout("output.txt")
        .with_stderr("error.txt");

    // 如果需要禁用 cgroup
    let sandbox_no_cg = IsolateSandbox::new(1)
        .disable_cgroups()
        .with_stdin("input.txt");

    // 设置资源限制
    let limits = ResourceLimits::new()
        .with_time_limit(1.0)      // 1 秒时间限制
        .with_cg_memory_limit(64 * 1024)  // 64 MB 内存限制
        .with_process_limit(1);        // 只允许 1 个进程

    // 初始化沙箱
    sandbox.init(&limits).await?;

    // 运行命令
    let result = sandbox.run("echo", ["Hello, World!"], &limits).await?;

    println!("退出码: {:?}", result.exit_code);
    println!("执行时间: {:.3}s", result.time_used);
    println!("内存使用: {} KB", result.cg_memory_used);
    println!("输出: {}", result.stdout);

    // 清理沙箱
    sandbox.cleanup().await?;
    
    Ok(())
}
```

## 高级配置

### 目录绑定

```rust
use isolate_integration::DirectoryRule;

let sandbox = IsolateSandbox::new(1)
    // 绑定外部目录到沙箱内部
    .with_directory_rule(DirectoryRule::bind("/tmp/work", "/tmp/sandbox_work").read_write())
    // 创建临时目录
    .with_directory_rule(DirectoryRule::tmp("/tmp/temp"))
    // 绑定系统目录（只读，禁止执行）
    .with_directory_rule(DirectoryRule::bind_same("/usr/bin").no_exec())
    // 挂载文件系统
    .with_directory_rule(DirectoryRule::filesystem("proc"));
```

### 环境变量

```rust
use isolate_integration::EnvRule;

let sandbox = IsolateSandbox::new(2)
    // 继承环境变量
    .with_env_rule(EnvRule::Inherit("HOME".to_string()))
    // 设置环境变量
    .with_env_rule(EnvRule::Set("PATH".to_string(), "/usr/bin:/bin".to_string()))
    // 继承所有环境变量
    .with_env_rule(EnvRule::FullEnv);
```

### 资源限制

```rust
let limits = ResourceLimits::new()
    .with_time_limit(2.0)              // CPU 时间限制
    .with_wall_time_limit(10.0)        // 墙钟时间限制
    .with_memory_limit(128 * 1024)     // 内存限制
    .with_cg_memory_limit(256 * 1024)  // 控制组内存限制
    .with_open_files_limit(64)         // 打开文件数限制
    .with_file_size_limit(1024)        // 单文件大小限制
    .with_process_limit(5);            // 进程数限制
```

### 特殊选项

```rust
let sandbox = IsolateSandbox::new(3)
    .use_cgroups()          // 启用控制组（默认已启用）
    .disable_cgroups()      // 禁用控制组
    .share_network()        // 共享网络命名空间
    .no_default_dirs()      // 不绑定默认目录
    .verbose();             // 详细输出
```

## 编译和运行示例

参见 `examples/sandbox_usage.rs` 中的完整示例，包括：

1. **基本用法**：简单的命令执行
2. **高级配置**：复杂的沙箱设置
3. **C++ 编译运行**：完整的编译和执行流程

运行示例：

```bash
cargo run --example sandbox_usage
```

## API 参考

### IsolateSandbox

主要的沙箱类，提供以下方法：

- `new(box_id: u32)` - 创建新的沙箱实例（默认启用 cgroup）
- `init(&self, limits: &ResourceLimits)` - 初始化沙箱
- `run<I, S>(&self, program: &str, args: I, limits: &ResourceLimits)` - 运行命令
- `cleanup(&self)` - 清理沙箱
- `disable_cgroups(self)` - 禁用控制组（如果不需要）

### ResourceLimits

资源限制配置：

- `with_time_limit(seconds: f64)` - 设置 CPU 时间限制
- `with_memory_limit(kb: u32)` - 设置内存限制
- `with_wall_time_limit(seconds: f64)` - 设置墙上时间限制
- `with_process_limit(count: u32)` - 设置进程数限制

### DirectoryRule

目录绑定规则：

- `bind(inside, outside)` - 绑定外部目录
- `bind_same(path)` - 绑定到相同路径
- `tmp(path)` - 创建临时目录
- `filesystem(name)` - 挂载文件系统

### ExecutionResult

执行结果包含：

- `exit_code: Option<i32>` - 退出码
- `signal: Option<i32>` - 信号
- `time_used: f64` - CPU 时间
- `wall_time_used: f64` - 墙上时间
- `memory_used: u32` - 内存使用
- `killed: bool` - 是否被杀死
- `stdout: String` - 标准输出
- `stderr: String` - 标准错误
