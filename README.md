# Judicia Platform

A modern online judge system with plugin-based architecture for different contest types.

## Structure

```
judicia-platform/
├── backend/           # Rust API server (basic structure)
├── core-sdk/          # Core SDK for plugins (minimal)
├── frontend/          # React + TypeScript (main focus)
├── plugins/           # Contest type plugins
└── docker/            # Docker configuration
```

## Quick Start

1. **Frontend Development**:
   ```bash
   cd frontend
   npm install
   npm run dev
   ```

2. **Backend** (basic):
   ```bash
   cd backend
   cargo run
   ```

## Architecture

- **Frontend**: React + TypeScript + Vite + Tailwind CSS
- **Backend**: Rust with basic REST API
- **Plugins**: Contest types (IOI, ICPC, etc.)
