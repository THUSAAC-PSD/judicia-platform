# hello-world

A utility plugin for the Judicia platform.

## Description

This plugin provides [describe your plugin's functionality here].

## Features

- [Feature 1]
- [Feature 2]
- [Feature 3]

## Configuration

Edit `plugin.toml` to configure:

- `name`: Display name of the plugin
- `version`: Plugin version
- UI routes and backend API endpoints

## Development

### Backend (Rust)

The plugin backend is written in Rust and compiled to WebAssembly.

```bash
# Build the plugin
cargo build --target wasm32-wasi --release

# Run tests
cargo test
```

### Frontend (React/TypeScript)

The plugin frontend is a React component.

```bash
cd frontend
npm install
npm run build
```

## Installation

1. Build both backend and frontend components
2. Copy the plugin directory to your Judicia plugins folder
3. Restart the Judicia platform to load the plugin

## API Endpoints

- `GET /api/my-plugin` - Main plugin endpoint
- [Add more endpoints as needed]

## Events

This plugin listens to:

- `submission.judged` - Handles judged submissions
- [Add more events as needed]

## Permissions

This plugin requires:

- Event emission capability
- WebSocket messaging capability
- [Add more permissions as needed]

## License

[Add your license information here]
