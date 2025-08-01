# Judicia Platform - Development Guide

## 🏗️ Architecture Overview

Judicia Platform is a modern online judge system built as a monorepo with:

- **Backend**: Rust-based API server (minimal implementation)
- **Frontend**: React + TypeScript with modern tooling
- **Core SDK**: Plugin system foundation
- **Plugin System**: Contest type implementations (IOI, ICPC, etc.)

## 🚀 Quick Start

### Prerequisites
- Rust (1.70+)
- Node.js (18+)
- pnpm

### Development

1. **Start both services:**
   ```bash
   ./dev.sh
   ```

2. **Or start individually:**
   ```bash
   # Backend (Terminal 1)
   cd backend && cargo run

   # Frontend (Terminal 2)  
   cd frontend && pnpm dev
   ```

### URLs
- **Frontend**: http://localhost:5173
- **Backend API**: http://localhost:8080
- **Health Check**: http://localhost:8080/health

## 📁 Project Structure

```
judicia-platform/
├── backend/           # Rust API server
│   ├── src/main.rs   # Basic REST API with health, problems, contests
│   └── Cargo.toml    # Dependencies: actix-web, tokio, serde
├── frontend/          # React + TypeScript + Vite
│   ├── src/
│   │   ├── components/  # Layout, reusable components
│   │   ├── pages/      # Dashboard, Problems, Contests
│   │   ├── lib/        # API client, utilities
│   │   ├── types/      # TypeScript interfaces
│   │   └── hooks/      # Custom React hooks
│   ├── package.json   # Frontend dependencies
│   └── .env          # Environment variables
├── core-sdk/         # Plugin system foundation
└── plugins/          # Contest type implementations
```

## 🎨 Frontend Features

### Technologies
- **React 19** with TypeScript
- **Vite** for fast development
- **Tailwind CSS** for styling
- **React Query** for state management
- **React Router** for navigation
- **React Hook Form** + Zod for forms
- **Lucide React** for icons

### Pages Implemented
1. **Dashboard** - Overview with stats, recent submissions, active contests
2. **Problems** - Problem browser with search, filters, difficulty levels
3. **Contests** - Contest listing with running/upcoming/past tabs

### Components
- **Layout** - Responsive sidebar navigation with mobile support
- **Active link highlighting** based on current route
- **Mock data** for development (ready for API integration)

## 🔧 Backend API

### Endpoints
- `GET /health` - Service health check
- `GET /api/problems` - List problems
- `GET /api/contests` - List contests

### Features
- **CORS enabled** for frontend development
- **JSON responses** with mock data
- **Actix-web** framework
- **Extensible** for real database integration

## 🔌 Plugin System (Core SDK)

### Concepts
- **ContestType trait** - Define custom contest behaviors
- **Problem, Submission, Contest** - Core data types
- **Scoring systems** - IOI, ICPC, custom implementations
- **Plugin registry** - Dynamic loading and management

## 📝 Development Notes

### Frontend Focus
The frontend is the main development area with:
- Modern React patterns and TypeScript
- Responsive design with Tailwind CSS
- Ready for API integration
- Extensible component architecture

### Backend & SDK
Basic working implementation:
- Simple REST API with mock data
- Core SDK with trait definitions
- Ready for extension and real implementation

## 🚀 Next Steps

1. **Frontend Development**:
   - Add problem detail pages
   - Implement code editor for submissions
   - Add real-time leaderboard updates
   - Create contest participation flow

2. **Backend Integration**:
   - Connect to real database
   - Implement authentication
   - Add submission processing
   - Plugin loading system

3. **Plugin Development**:
   - Create IOI contest type plugin
   - Implement ICPC scoring
   - Add custom contest types

## 🛠️ Commands

```bash
# Frontend development
cd frontend
pnpm dev          # Start dev server
pnpm build        # Build for production
pnpm lint         # Run ESLint

# Backend development  
cd backend
cargo run         # Start API server
cargo test        # Run tests
cargo build       # Build binary

# Full stack
./dev.sh          # Start both services
```
