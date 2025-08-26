# Judicia SDK

The official SDK for developing plugins for the Judicia online judge platform. This SDK provides a powerful, type-safe way to create WebAssembly plugins with procedural macros that generate all necessary boilerplate code.

## Features

- **🚀 Declarative Plugin Definition**: Use `#[judicia_plugin]` to define your plugin with metadata
- **🔒 Automatic Capability Management**: Declare and validate platform capabilities at compile time
- **📘 Type-Safe API Bindings**: Strongly-typed interfaces to all platform services
- **⚡ Event-Driven Architecture**: React to platform events and emit custom events
- **🛠️ Resource Management**: Automatic cleanup and lifecycle management
- **🎨 Frontend Integration**: Create web components that integrate seamlessly
- **📦 WebAssembly Ready**: Compiles directly to WebAssembly for secure execution

## Quick Start

Add the SDK to your `Cargo.toml`:

```toml
[dependencies]
judicia-sdk = "0.1.0"

[lib]
crate-type = ["cdylib"]
```

Create a simple plugin:

```rust
use judicia_sdk::prelude::*;

#[judicia_plugin]
pub struct MyPlugin {
    name: "my-awesome-plugin",
    version: "1.0.0",
    author: "Your Name",
    description: "An awesome plugin for Judicia",
    capabilities: [
        Capability::SubscribeEvents,
        Capability::EmitEvent,
        Capability::SendNotifications
    ]
}

impl PluginMethods for MyPlugin {
    async fn on_initialize(&mut self, context: &PluginContext) -> PluginResult<()> {
        info!("Plugin {} initialized!", context.plugin_id);
        Ok(())
    }

    async fn on_event(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        match event.event_type.as_str() {
            "submission.created" => {
                info!("New submission: {}", event.payload);
                // Handle submission event
            }
            _ => {}
        }
        Ok(())
    }
}
```

Compile to WebAssembly:

```bash
cargo build --target wasm32-unknown-unknown --release
wasm-pack build --target web
```

## Core Concepts

### Plugin Capabilities

Plugins must declare their required capabilities, which are validated by the platform:

```rust
capabilities: [
    Capability::TriggerJudging,      // Trigger code evaluation
    Capability::ReadDatabase,        // Query contest data
    Capability::WriteDatabase,       // Modify contest data
    Capability::EmitEvent,           // Publish platform events
    Capability::SendNotifications,   // Send user notifications
    Capability::RegisterRoutes,      // Add HTTP API endpoints
    Capability::RegisterComponents,  // Add frontend components
]
```

### Event System

Plugins can subscribe to and emit platform events:

```rust
// Subscribe to events during initialization
let subscription = EventSubscription::new(context.plugin_id)
    .subscribe_to_all_submissions()
    .subscribe_to("contest.*")
    .subscribe_to("user.login");

subscription.register().await?;

// Emit custom events
platform.emit_event("custom.notification", serde_json::json!({
    "message": "Something happened!",
    "severity": "info"
})).await?;
```

### Database Integration

Query and modify contest data with type-safe database operations:

```rust
// Read data
let submissions = platform.query_database(
    "SELECT * FROM submissions WHERE user_id = $1 ORDER BY created_at DESC LIMIT 10",
    vec![serde_json::Value::String(user_id.to_string())]
).await?;

// Write data (requires WriteDatabase capability)
platform.query_database(
    "UPDATE submissions SET verdict = $1 WHERE id = $2",
    vec![
        serde_json::Value::String("AC".to_string()),
        serde_json::Value::String(submission_id.to_string())
    ]
).await?;
```

### HTTP API Routes

Register custom API endpoints for your plugin:

```rust
impl PluginMethods for MyPlugin {
    async fn on_initialize(&mut self, context: &PluginContext) -> PluginResult<()> {
        // Register API routes
        platform.register_route("GET", "/api/my-plugin/stats", "get_stats").await?;
        platform.register_route("POST", "/api/my-plugin/action", "handle_action").await?;
        Ok(())
    }

    async fn on_http_request(&mut self, request: &HttpRequest) -> PluginResult<HttpResponse> {
        match (request.method.as_str(), request.path.as_str()) {
            ("GET", "/api/my-plugin/stats") => {
                let stats = self.get_statistics().await?;
                crate::http::json_response(200, &stats)
            }
            ("POST", "/api/my-plugin/action") => {
                self.handle_custom_action(request).await
            }
            _ => Ok(crate::http::error_response(404, "Endpoint not found"))
        }
    }
}
```

### Frontend Components

Create reusable UI components that integrate with the platform:

```rust
#[frontend_component]
pub struct SubmissionDashboard {
    component_type: ComponentType::Widget,
    props: DashboardProps,
    events: ["refresh", "filter_changed"]
}

impl FrontendComponent for SubmissionDashboard {
    fn render(&self, props: &serde_json::Value) -> Result<String, JsValue> {
        let html = HtmlBuilder::new()
            .div(Some("submission-dashboard"), "")
            .div(Some("header"), "<h2>My Submissions</h2>")
            .table(
                vec!["Time", "Problem", "Verdict", "Score"],
                self.get_submission_rows(props)
            )
            .build();
        
        Ok(html)
    }
}
```

### Judging Integration

Trigger custom evaluation logic for submissions:

```rust
let judging_request = JudgingRequestBuilder::new(submission_id, problem_id, language_id)
    .source_code(&source_code)
    .time_limit(5000)
    .memory_limit(256 * 1024)
    .priority(1)
    .metadata("special_judge", serde_json::Value::Bool(true))
    .build();

let job_id = platform.trigger_judging(judging_request).await?;
info!("Judging job started: {}", job_id);
```

### Notifications

Send notifications to users:

```rust
let notification = NotificationBuilder::new(user_id)
    .title("Submission Judged")
    .message("Your submission to Problem A has been evaluated.")
    .notification_type(NotificationType::Success)
    .urgency(NotificationUrgency::Normal)
    .metadata("submission_id", serde_json::Value::String(submission_id.to_string()))
    .build();

platform.send_notification(notification).await?;
```

## Examples

The SDK includes comprehensive examples:

- **[Simple Plugin](examples/simple-plugin.rs)**: Basic event handling and notifications
- **[Judging Plugin](examples/judging-plugin.rs)**: Advanced judging logic with custom evaluation
- **[Frontend Plugin](examples/frontend-plugin.rs)**: UI components and API integration

## Plugin Types

### Contest Management Plugins
- Contest formats (ICPC, IOI, custom)
- Scoring systems
- Team management
- Time extensions

### Problem Type Plugins  
- Standard judging
- Special judge
- Interactive problems
- Output-only problems

### Utility Plugins
- Balloon notifications
- Announcements
- Statistics dashboards
- Export tools

### Integration Plugins
- External contest platforms
- Authentication providers
- Notification services
- Analytics platforms

## Security Model

All plugins run in a secure WebAssembly sandbox with:

- **Capability-based security**: Plugins can only access declared capabilities
- **Resource limits**: CPU, memory, and I/O restrictions
- **Network isolation**: No direct network access except through platform APIs
- **File system isolation**: Limited file access through platform storage APIs

## Development Workflow

1. **Create Plugin**: Use the SDK to define your plugin structure
2. **Implement Logic**: Add your business logic in the required trait methods
3. **Test Locally**: Use the SDK's testing utilities
4. **Build WebAssembly**: Compile to `.wasm` for deployment
5. **Deploy**: Upload to Judicia platform with plugin manifest

## Platform Integration

Plugins integrate with the Judicia platform through:

- **Core Kernel**: Plugin lifecycle management and capability enforcement
- **Event Bus**: Distributed event system for plugin communication  
- **Message Queue**: Reliable async processing for evaluation tasks
- **Database**: Shared contest data with transaction support
- **Frontend**: Component registration and micro-frontend architecture

## Best Practices

1. **Minimal Capabilities**: Only request capabilities you actually need
2. **Error Handling**: Use proper error types and context
3. **Resource Cleanup**: Implement cleanup logic in `on_cleanup`
4. **Event Filtering**: Subscribe only to relevant events to reduce overhead
5. **Async Design**: Use async/await for all platform interactions
6. **Testing**: Write comprehensive tests for your plugin logic

## API Reference

Full API documentation is available at [docs.judicia.dev/sdk](https://docs.judicia.dev/sdk).

Key modules:
- `judicia_sdk::prelude` - Common imports
- `judicia_sdk::types` - Core types and traits  
- `judicia_sdk::platform` - Platform API wrappers
- `judicia_sdk::frontend` - UI component utilities
- `judicia_sdk::error` - Error types and handling
- `judicia_sdk::utils` - Helper functions and utilities

## Contributing

We welcome contributions to the Judicia SDK! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.