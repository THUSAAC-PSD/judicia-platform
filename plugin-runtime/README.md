# Plugin Runtime

The plugin runtime provides the WebAssembly execution environment for Judicia plugins. It handles the secure loading, execution, and management of plugin code within sandboxed environments.

## Purpose

- **WebAssembly Execution**: Run compiled plugin code in secure WASM environments
- **Host Function Binding**: Provide platform APIs accessible from plugin code
- **Memory Management**: Manage memory allocation and deallocation for plugin instances
- **Security Sandboxing**: Enforce capability-based security and resource limits
- **Plugin Lifecycle**: Handle plugin initialization, execution, and cleanup

## Key Components

### Plugin Manager
Manages the loading and lifecycle of plugin instances. Handles:
- Loading WASM modules from disk
- Plugin metadata validation
- Instance creation and destruction
- Capability verification

### Host Functions
Implements the host-side functions that plugins can call to interact with the platform:
- Database operations
- Event emission and subscription
- HTTP request handling
- Notification services
- File system operations (limited)

### Sandbox Environment
Provides isolated execution contexts for plugins with:
- Memory limits
- CPU time restrictions
- Capability-based API access
- Resource monitoring

## WebAssembly Integration

Uses the Wasmtime runtime to execute WebAssembly modules compiled from Rust, providing:
- Just-in-time compilation for performance
- Memory safety guarantees
- Cross-platform compatibility
- Async/await support

## Security Features

- **Capability Enforcement**: Only plugins with proper capabilities can access restricted APIs
- **Resource Quotas**: Enforce CPU and memory limits per plugin
- **API Surface Control**: Limit which platform functions plugins can access
- **Isolation**: Prevent plugins from interfering with each other or the host system

## Plugin Communication

Plugins communicate with the platform through:
- Serialized function calls (JSON over WASM boundary)
- Event subscription and emission
- Shared memory regions (controlled)
- HTTP-like request/response patterns

## Performance Considerations

The runtime is optimized for:
- Fast plugin startup times
- Low memory overhead per plugin instance
- Efficient serialization/deserialization
- Minimal host function call overhead