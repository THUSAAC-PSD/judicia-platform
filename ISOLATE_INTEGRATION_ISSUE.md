# 🏗️ 实现请求：IOI Isolate 集成模块

## 📋 概述

需要实现与 [IOI Isolate](https://github.com/ioi/isolate) 的集成来替换当前的占位符评测系统。除了代码执行沙箱外，Judicia Platform 架构已经完成。

## 🎯 需要实现的功能

### **核心模块：`isolate-integration/`**
创建一个新的 Rust crate，在评测引擎和 IOI isolate 之间建立接口。

**位置**：`D:\judicia-platform\isolate-integration\`

### **核心功能**

1. **Isolate 包装器**：Rust 接口调用 isolate 命令行工具
2. **安全管理**：沙箱配置和清理
3. **资源监控**：内存、时间和进程限制
4. **文件管理**：沙箱内输入输出文件处理
5. **错误处理**：完整的错误报告和恢复

### **集成点**

### **需要替换的接口**

```rust
// 文件: evaluation-engine/src/executor.rs (已存在)
// 替换 compile 和 run 方法的实现

impl Executor {
    pub async fn compile(&self, source_code: &str, language: &Language) -> Result<CompileResult> {
        // TODO: 在这里集成 isolate 编译功能
    }
    
    pub async fn run(&self, ...) -> Result<RunResult> {
        // TODO: 在这里集成 isolate 执行功能  
    }
}
```

## 🛠️ 技术要求

### **需要创建的核心结构**

```rust
// 沙箱管理
pub struct IsolateSandbox {
    box_id: u32,
    work_dir: PathBuf,
}

impl IsolateSandbox {
    pub async fn create(box_id: u32) -> Result<Self>;
    pub async fn cleanup(&self) -> Result<()>;
    pub async fn compile(&self, source_code: &str, language: &Language) -> Result<CompileResult>;
    pub async fn run(&self, executable: &Path, input: &str, limits: &ResourceLimits) -> Result<RunResult>;
}
```

### **资源限制配置**
```rust
pub struct ResourceLimits {
    pub time_limit_ms: u32,     // 时间限制(毫秒)
    pub memory_limit_kb: u32,   // 内存限制(KB)
    pub file_size_limit: usize, // 文件大小限制
}
```

## 📁 文件结构

```
isolate-integration/
├── Cargo.toml
├── src/
│   ├── lib.rs       # 主要导出
│   ├── sandbox.rs   # IsolateSandbox 实现
│   ├── compiler.rs  # 编译逻辑
│   └── error.rs     # 错误处理
└── tests/           # 集成测试
```

## 🔗 集成步骤

1. **创建 isolate-integration crate**
2. **实现 IsolateSandbox 基本功能**（创建、清理沙箱）
3. **添加编译支持**（C++, Python, Java）
4. **添加执行支持**（时间、内存限制）
5. **替换 evaluation-engine/src/executor.rs 中的占位符**
6. **编写测试用例**
7. **集成到工作区**

## 📚 参考资料

### **IOI Isolate 基本命令**
```bash
# 初始化沙箱
isolate --box-id=1 --init

# 编译程序
isolate --box-id=1 --run -- g++ -o solution solution.cpp

# 执行程序（1秒时限，256MB内存）
isolate --box-id=1 --time=1 --mem=262144 --run -- ./solution

# 清理沙箱
isolate --box-id=1 --cleanup
```

### **现有代码位置**
- **待替换接口**: `evaluation-engine/src/executor.rs`
- **数据结构**: `shared/src/types.rs` 
- **插件集成**: `plugins/standard-judge/src/lib.rs`

## ✅ 完成标准

- [ ] 支持多语言编译执行（C++, Python, Java）
- [ ] 正确的资源限制（时间、内存）
- [ ] 完整的错误处理和日志
- [ ] 集成测试覆盖主要场景
- [ ] 替换 executor.rs 中的占位符代码
- [ ] 与现有评测系统完整集成

**优先级：高** - 这是平台投入生产的最后一个组件。其他所有系统（19,300+ 行代码）已完成并等待此集成。