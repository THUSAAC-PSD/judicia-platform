# create-judicia-plugin

🚀 CLI tool for scaffolding Judicia Platform plugins

## Quick Start

```bash
# Create a new plugin
npx create-judicia-plugin my-awesome-plugin

# Or install globally
npm install -g create-judicia-plugin
create-judicia-plugin my-awesome-plugin
```

## Usage

### Create a New Plugin

```bash
create-judicia-plugin [name] [options]

# Interactive mode
create-judicia-plugin

# With options
create-judicia-plugin my-plugin --template=contest-plugin --no-install
```

**Options:**
- `-t, --template <template>` - Plugin template (default: basic)
- `-d, --directory <directory>` - Output directory (default: .)
- `--no-install` - Skip npm install
- `--no-git` - Skip git initialization

### Add Components

```bash
# Add a new component to your plugin
create-judicia-plugin add-component MyComponent

# Specify component type
create-judicia-plugin add-component MyWidget --type react
```

### Available Templates

- **basic** - Minimal plugin with basic structure
- **contest-plugin** - Contest management functionality
- **problem-solver** - Problem-solving interfaces and tools
- **judge-system** - Custom judging system with evaluation logic
- **frontend-only** - TypeScript frontend-only plugin
- **admin-dashboard** - Administrative dashboard and tools
- **notification-system** - Notifications and alerts handling
- **analytics** - Analytics and reporting functionality

## Plugin Structure

### Rust Plugin (WebAssembly)

```
my-plugin/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   └── components/
│       ├── mod.rs
│       └── hello_world.rs
├── pkg/              # Generated WebAssembly output
├── www/              # Frontend assets
└── README.md
```

### TypeScript Plugin (Frontend)

```
my-plugin/
├── package.json
├── tsconfig.json
├── src/
│   ├── index.ts
│   ├── components/
│   │   └── HelloWorld.tsx
│   └── styles/
│       └── main.css
├── dist/             # Built output
└── README.md
```

## Development Workflow

1. **Create Plugin**
   ```bash
   create-judicia-plugin my-plugin
   cd my-plugin
   ```

2. **Add Components**
   ```bash
   create-judicia-plugin add-component ProblemList
   create-judicia-plugin add-component SubmissionForm --type react
   ```

3. **Development**
   ```bash
   npm run dev        # Start development server
   cargo run          # For Rust plugins
   ```

4. **Build**
   ```bash
   npm run build      # Build for production
   cargo build --release --target wasm32-unknown-unknown  # For Rust
   ```

## Plugin Configuration

### Capabilities

Plugins can request various platform capabilities:

- **Data Access**
  - `read_problems` - Read problem data
  - `write_problems` - Create/modify problems
  - `read_submissions` - Read submission data
  - `write_submissions` - Create/modify submissions
  - `read_contests` - Read contest data
  - `write_contests` - Create/modify contests

- **UI Integration**
  - `register_components` - Register UI components
  - `register_routes` - Register custom routes
  - `emit_events` - Emit platform events
  - `subscribe_events` - Subscribe to events

- **Platform Services**
  - `notifications` - Send notifications
  - `file_storage` - Access file storage
  - `admin_operations` - Administrative operations

### Plugin Metadata

```rust
#[judicia_plugin]
pub struct MyPlugin {
    name: "my-plugin",
    version: "1.0.0",
    author: "Your Name",
    description: "My awesome plugin",
    capabilities: [
        Capability::ReadProblems,
        Capability::RegisterComponents,
        Capability::EmitEvents
    ]
}
```

## Examples

### Basic Event Handler

```rust
impl PluginMethods for MyPlugin {
    async fn on_event(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        match event.event_type.as_str() {
            "submission.created" => {
                // Handle new submission
                let submission_id = event.payload.get("submission_id");
                println!("New submission: {:?}", submission_id);
            }
            _ => {}
        }
        Ok(())
    }
}
```

### React Component

```typescript
import { useJudicia, usePlatformEvent } from '@judicia/frontend-sdk';

export function ProblemList() {
  const sdk = useJudicia();
  const [problems, setProblems] = useState([]);

  // Listen for problem updates
  usePlatformEvent('problem.updated', (event) => {
    // Refresh problem list
    loadProblems();
  });

  return (
    <div className="problem-list">
      {problems.map(problem => (
        <div key={problem.id}>{problem.title}</div>
      ))}
    </div>
  );
}
```

## CLI Commands

- `create-judicia-plugin create [name]` - Create new plugin
- `create-judicia-plugin add-component <name>` - Add component
- `create-judicia-plugin add-route <path>` - Add route
- `create-judicia-plugin build` - Build plugin
- `create-judicia-plugin dev` - Start development server

## Requirements

- Node.js 16+
- For Rust plugins: Rust 1.70+, wasm-pack
- For TypeScript plugins: TypeScript 5+

## License

MIT

## Support

- 📖 [Documentation](https://docs.judicia.dev)
- 🐛 [Issue Tracker](https://github.com/judicia/judicia-platform/issues)
- 💬 [Discord Community](https://discord.gg/judicia)