/// Template generators for different plugin types

pub fn generate_plugin_toml(display_name: &str, plugin_type: &str) -> String {
    format!(
        r#"name = "{}"
version = "1.0.0"
{}

[backend]
entrypoint = "target/wasm32-wasi/release/{}.wasm"

[frontend]
module_name = "{}"
bundle_url = "./frontend/dist/bundle.js"

# Plugin UI routes
[[ui_routes]]
scope = "contest"
path = "/my-plugin"
component = "./MyPluginView"
required_permission = "participant"
nav_link = true
nav_text = "{}"

# Plugin backend API
[[http_routes]]
path = "/api/my-plugin"
handler_function = "handle_request"
"#,
        display_name,
        match plugin_type {
            "contest" => r#"contest_type_id = "custom-contest-v1""#,
            _ => "",
        },
        display_name.to_lowercase().replace(" ", "-"),
        display_name.to_lowercase().replace(" ", "_"),
        display_name
    )
}

pub fn generate_cargo_toml(plugin_name: &str) -> String {
    format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
plugin-sdk = {{ path = "../plugin-sdk" }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
tokio = {{ version = "1.0", features = ["macros", "rt"] }}
anyhow = "1.0"
uuid = {{ version = "1.0", features = ["v4", "serde"] }}

[dependencies.web-sys]
version = "0.3"
features = [
  "console",
  "Document",
  "Element",
  "Window",
]

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
"#,
        plugin_name
    )
}

pub fn generate_backend_code(plugin_name: &str, template_type: &str) -> String {
    let basic_template = format!(
        r#"use plugin_sdk::{{main, handler, event_handler, types::*}};
use serde::{{Deserialize, Serialize}};
use anyhow::Result;

#[derive(Serialize, Deserialize)]
struct PluginConfig {{
    // Add your configuration fields here
    api_endpoint: Option<String>,
}}

#[main]
async fn plugin_main() -> Result<()> {{
    // Plugin initialization
    println!("🚀 {} plugin started!");
    Ok(())
}}

#[handler("/api/my-plugin")]
async fn handle_request(req: HttpRequest) -> Result<HttpResponse, PluginError> {{
    // Handle HTTP requests to your plugin
    let response = HttpResponse {{
        status_code: 200,
        headers: std::collections::HashMap::new(),
        body: b"{{\"message\": \"Hello from plugin!\"}}".to_vec(),
    }};
    
    Ok(response)
}}

#[event_handler("submission.judged")]
async fn on_submission_judged(event: Event) -> Result<(), PluginError> {{
    // Handle submission judged events
    println!("Received submission.judged event: {{}}", event.id);
    Ok(())
}}

// Request the capabilities your plugin needs
plugin_sdk::request_capabilities![
    Capability::EmitEvent,
    Capability::WebSocketMessaging
];
"#,
        plugin_name
    );

    match template_type {
        "full" => format!(
            r#"{}

#[event_handler("contest.started")]
async fn on_contest_started(event: Event) -> Result<(), PluginError> {{
    // Handle contest started events
    println!("Contest started: {{}}", event.id);
    Ok(())
}}

#[handler("/api/my-plugin/status")]
async fn get_status(req: HttpRequest) -> Result<HttpResponse, PluginError> {{
    let response = HttpResponse {{
        status_code: 200,
        headers: std::collections::HashMap::new(),
        body: b"{{\"status\": \"active\", \"version\": \"1.0.0\"}}".to_vec(),
    }};
    
    Ok(response)
}}
"#,
            basic_template
        ),
        _ => basic_template,
    }
}

pub fn generate_frontend_code(plugin_name: &str) -> String {
    format!(
        r#"import React from 'react';
import {{ createRoot }} from 'react-dom/client';

interface {}Props {{
  // Define your component props here
}}

const {}: React.FC<{}Props> = (props) => {{
  return (
    <div className="plugin-container">
      <h2>{} Plugin</h2>
      <p>Welcome to your custom Judicia plugin!</p>
      
      <div className="plugin-content">
        {{/* Add your plugin UI components here */}}
        <button 
          onClick={{() => console.log('Plugin button clicked!')}}
          className="btn btn-primary"
        >
          Plugin Action
        </button>
      </div>
    </div>
  );
}};

// Export the component for plugin system
export default {};

// If this is a standalone plugin page, you can render it directly
if (typeof document !== 'undefined') {{
  const container = document.getElementById('plugin-root');
  if (container) {{
    const root = createRoot(container);
    root.render(<{} />);
  }}
}}
"#,
        plugin_name.replace("-", "").replace("_", ""),
        plugin_name.replace("-", "").replace("_", ""),
        plugin_name.replace("-", "").replace("_", ""),
        plugin_name,
        plugin_name.replace("-", "").replace("_", ""),
        plugin_name.replace("-", "").replace("_", "")
    )
}

pub fn generate_webpack_config(plugin_name: &str) -> String {
    format!(
        r#"const path = require('path');

module.exports = {{
  entry: './src/index.tsx',
  mode: 'production',
  module: {{
    rules: [
      {{
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      }},
    ],
  }},
  resolve: {{
    extensions: ['.tsx', '.ts', '.js'],
  }},
  output: {{
    filename: 'bundle.js',
    path: path.resolve(__dirname, 'dist'),
    library: '{}Plugin',
    libraryTarget: 'umd',
  }},
  externals: {{
    react: 'React',
    'react-dom': 'ReactDOM',
  }},
}};
"#,
        plugin_name.replace("-", "").replace("_", "")
    )
}

pub fn generate_readme(plugin_name: &str, plugin_type: &str) -> String {
    format!(
        r#"# {}

A {} plugin for the Judicia platform.

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
"#,
        plugin_name, plugin_type
    )
}