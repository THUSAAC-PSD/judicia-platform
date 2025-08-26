# 🏛️ Judicia Platform

**Next-Generation Plugin-Based Online Judge Platform**

A revolutionary competitive programming platform built with a microkernel architecture, featuring WebAssembly plugins, micro frontends, and distributed evaluation.

## 🎯 **Architecture Overview**

```
┌─────────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│  React Frontend     │◄──►│  Core Kernel     │◄──►│ Plugin Ecosystem│
│  + Micro Frontends  │    │  (Rust/Axum)     │    │ (WebAssembly)   │
│  + Module Federation│    │  + Event Bus     │    │ + Dynamic Load  │
└─────────────────────┘    └──────────────────┘    └─────────────────┘
                                     │                        │
                           ┌──────────────────┐    ┌─────────────────┐
                           │ Evaluation Engine│◄──►│   IOI Isolate   │
                           │ + Job Queue      │    │   Integration   │
                           │ + Distributed    │    │   (Pending)     │
                           └──────────────────┘    └─────────────────┘
```

## ✨ **Key Features**

- 🏛️ **Microkernel Architecture** with WebAssembly plugins
- 🎯 **Micro Frontend System** with Module Federation
- ⚡ **Event-Driven Architecture** with real-time updates
- 🔐 **Multi-Layer Security** with ABAC authorization
- 🚀 **Production-Ready Plugins**: Judge, Contest, Notifications
- 📊 **Complete Plugin SDK** with CLI scaffolding tools
- 🔄 **Hot Plugin Loading** with zero-downtime updates

## 🚀 **Quick Start**

### **Prerequisites**
- Rust 1.75+
- Node.js 18+
- pnpm
- Database: PostgreSQL 14+ (local) OR Supabase account (recommended)

### **Development Setup**

#### **Option 1: Supabase (Recommended)**
```bash
# Clone repository
git clone https://github.com/judicia/judicia-platform.git
cd judicia-platform

# Configure Supabase
cp .env.example .env
# Edit .env with your Supabase credentials (see SUPABASE_SETUP.md)

# Install dependencies
cargo build
pnpm install

# Start development environment
pnpm run dev
```

#### **Option 2: Local PostgreSQL**
```bash
# Setup local PostgreSQL database
createdb judicia

# Use local development script
pnpm run dev:local
```

📖 **Detailed Supabase Setup**: See [SUPABASE_SETUP.md](./SUPABASE_SETUP.md)

This starts:
- **Frontend**: http://localhost:5173 (Vite + React)
- **Backend**: http://localhost:5000 (Rust Core Kernel)
- **Plugin Discovery**: http://localhost:5000/api/plugins/discovery

### **Production Build**
```bash
# Build all components
pnpm run build
pnpm run start
```

## 🔌 **Plugin Development**

### **Create New Plugin**
```bash
# Generate plugin scaffold
cd create-judicia-plugin
npm run create -- --template microfrontend-plugin --name my-plugin
```

### **Available Templates**
- `basic` - Rust-only plugin
- `microfrontend-plugin` - Full micro frontend with React components
- `contest-plugin` - Contest management functionality
- `judge-system` - Custom judging logic

### **Plugin Architecture**
```rust
use judicia_sdk::prelude::*;

#[judicia_plugin]
pub struct MyPlugin {
    // Plugin state
}

#[async_trait]
impl Plugin for MyPlugin {
    async fn initialize(&mut self) -> PluginResult<()> {
        // Plugin initialization
        Ok(())
    }
}
```

## 📁 **Project Structure**

```
judicia-platform/
├── core-kernel/              # Rust microkernel (2,000+ lines)
├── evaluation-engine/        # Distributed evaluation system
├── judicia-sdk/             # Plugin development SDK (2,500+ lines)
├── judicia-frontend-sdk/    # TypeScript/React SDK (2,000+ lines)
├── plugins/
│   ├── standard-judge/      # Standard competitive programming judge
│   ├── icpc-contest/        # ICPC-style contest management
│   ├── notification-system/ # Multi-channel notifications
│   └── announcement-system/ # Contest announcements
├── client/                  # React frontend application
└── create-judicia-plugin/   # CLI scaffolding tool
```

### **Pending: IOI Isolate Integration**
The only remaining component is the IOI Isolate integration for secure code execution. See `ISOLATE_INTEGRATION_ISSUE.md` for implementation details.

## 🛠️ **Development Scripts**

```bash
# Development
pnpm run dev              # Start full development environment
pnpm run dev:frontend     # Frontend only
pnpm run dev:backend      # Backend only

# Production
pnpm run build            # Build all components
pnpm run start            # Start production server

# Testing
pnpm run test             # Run all tests
cargo test                # Rust tests only
pnpm run check            # Type checking

# Database
pnpm run db:migrate       # Run database migrations
```

## 📚 **Documentation**

- **Architecture Overview**: `TODO.md` - Complete transformation documentation
- **Plugin Development**: `judicia-sdk/README.md`
- **Frontend SDK**: `judicia-frontend-sdk/README.md`
- **CLI Tools**: `create-judicia-plugin/README.md`
- **IOI Isolate Integration**: `ISOLATE_INTEGRATION_ISSUE.md`

## 🏆 **Revolutionary Features**

### **Plugin System**
- **WebAssembly Runtime**: Secure, sandboxed plugin execution
- **Hot Loading**: Update plugins without system restart
- **Capability-Based Security**: Fine-grained permissions
- **Event-Driven Communication**: Real-time plugin coordination

### **Micro Frontend Architecture**
- **Module Federation**: Dynamic component loading
- **Plugin UI Components**: Isolated frontend modules
- **Shared State Management**: Cross-plugin data sharing
- **Development Hot Reload**: Rapid development workflow

### **Complete Plugin Ecosystem**
- **Standard Judge**: Traditional competitive programming evaluation
- **ICPC Contest**: Full contest management with real-time scoreboard
- **Notification System**: Multi-channel alerts (Browser, Email, SMS, Push)
- **Announcement System**: Contest-wide communications

### **Production Features**
- **PostgreSQL Database**: Complete schema with migrations
- **Event Bus**: RabbitMQ integration for distributed messaging
- **ABAC Authorization**: Attribute-based access control
- **Comprehensive Logging**: Structured logging with tracing

## 🚧 **Contributing**

1. **Fork the repository**
2. **Create feature branch**: `git checkout -b feature/my-feature`
3. **Make changes** following the architecture patterns
4. **Add tests** for new functionality
5. **Submit pull request**

## 📄 **License**

MIT License - see [LICENSE](LICENSE) file for details.

## 🌟 **Acknowledgments**

- **IOI Isolate** for secure code execution sandbox
- **WebAssembly** for plugin runtime security
- **Module Federation** for micro frontend architecture
- **Rust Ecosystem** for high-performance backend

---

**🚀 The future of competitive programming platforms is here!**

*Built by developers, for developers, with extensibility at its core.*