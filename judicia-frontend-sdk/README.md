# @judicia/frontend-sdk

TypeScript SDK for developing Judicia Platform frontend components and micro frontends.

## Features

- 🎯 **Type-Safe Development**: Full TypeScript support with comprehensive type definitions
- 🧩 **Plugin Component System**: Easy registration and management of frontend components
- 🎨 **Theme System**: Built-in theming support with CSS variables
- 🔄 **State Management**: Zustand-based global state management
- 📡 **Event System**: Event-driven communication between plugins
- 🚀 **Micro Frontend Support**: Dynamic loading of remote components
- 🔧 **React Hooks**: Convenient hooks for common platform operations
- 🛡️ **Capability-Based Security**: Role and permission-based component rendering

## Installation

```bash
npm install @judicia/frontend-sdk
```

## Quick Start

```typescript
import { JudiciaFrontendSDK, useJudicia, usePlatformContext } from '@judicia/frontend-sdk';

// Initialize the SDK
const sdk = JudiciaFrontendSDK.getInstance({
  apiBaseUrl: 'https://api.judicia.dev/v1',
  enableDevTools: true,
});

// Initialize with platform context
await sdk.initialize({
  user: {
    id: '123',
    username: 'john_doe',
    email: 'john@example.com',
    displayName: 'John Doe',
    roles: ['contestant'],
    permissions: ['read_problems', 'submit_solutions'],
  },
  theme: 'auto',
  locale: 'en-US',
  timezone: 'America/New_York',
  isAdmin: false,
  isModerator: false,
  features: ['contests', 'problems', 'submissions'],
});
```

## Plugin Development

### Creating a Plugin Component

```typescript
import React from 'react';
import { useJudicia, usePlatformContext } from '@judicia/frontend-sdk';

function MyPluginComponent({ problemId }: { problemId: string }) {
  const sdk = useJudicia();
  const context = usePlatformContext();

  const handleSubmit = () => {
    sdk.emitPlatformEvent('submission.created', {
      problemId,
      userId: context.user?.id,
    });
  };

  return (
    <div>
      <h2>Problem Solver</h2>
      <button onClick={handleSubmit}>Submit Solution</button>
    </div>
  );
}

// Register the component
sdk.registerComponent({
  name: 'ProblemSolver',
  component: MyPluginComponent,
  props: {
    problemId: {
      type: 'string',
      required: true,
      description: 'The ID of the problem to solve'
    }
  },
  events: [
    {
      name: 'submit',
      description: 'Triggered when a solution is submitted',
      payload: {
        problemId: { type: 'string', required: true },
        solution: { type: 'string', required: true }
      }
    }
  ]
});
```

### Using React Hooks

```typescript
import React from 'react';
import { 
  useUser, 
  useCapabilities, 
  useTheme, 
  usePlatformEvent, 
  useNotifications 
} from '@judicia/frontend-sdk';

function UserDashboard() {
  const user = useUser();
  const { hasPermission } = useCapabilities();
  const { theme, setTheme } = useTheme();
  const { addNotification } = useNotifications();

  // Listen for platform events
  usePlatformEvent('contest.start', (event) => {
    addNotification({
      type: 'info',
      title: 'Contest Started',
      message: `Contest ${event.payload.contestId} has started!`
    });
  });

  if (!user) {
    return <div>Please log in</div>;
  }

  return (
    <div>
      <h1>Welcome, {user.displayName}</h1>
      
      {hasPermission('admin_operations') && (
        <button>Admin Panel</button>
      )}
      
      <button onClick={() => setTheme(theme === 'light' ? 'dark' : 'light')}>
        Toggle Theme
      </button>
    </div>
  );
}
```

### Event System

```typescript
import { usePlatformEvent, useEventEmitter } from '@judicia/frontend-sdk';

// Listen for events
usePlatformEvent('user.login', (event) => {
  console.log('User logged in:', event.payload.user);
});

// Emit events
const emit = useEventEmitter();

const handleAction = () => {
  emit('custom.action', { 
    data: 'some data',
    timestamp: new Date()
  }, {
    source: 'MyPlugin',
    category: 'user_interaction'
  });
};
```

### State Management

```typescript
import { useJudiciaStore } from '@judicia/frontend-sdk';

function MyComponent() {
  // Access global state
  const platform = useJudiciaStore(state => state.platform);
  const notifications = useJudiciaStore(state => state.ui.notifications);
  
  // Update state
  const store = useJudiciaStore();
  
  const updateUser = (user) => {
    store.setState(state => ({
      ...state,
      platform: {
        ...state.platform,
        user
      }
    }));
  };

  return (
    <div>
      <p>Current user: {platform.user?.displayName}</p>
      <p>Notifications: {notifications.length}</p>
    </div>
  );
}
```

### API Integration

```typescript
import { useAPI, useLoading } from '@judicia/frontend-sdk';

function ProblemList() {
  const api = useAPI();
  const { loading } = useLoading('problems');
  const [problems, setProblems] = useState([]);

  useEffect(() => {
    loadProblems();
  }, []);

  const loadProblems = async () => {
    try {
      const response = await api.get('/problems', 'problems');
      setProblems(response.data);
    } catch (error) {
      console.error('Failed to load problems:', error);
    }
  };

  if (loading) {
    return <div>Loading problems...</div>;
  }

  return (
    <ul>
      {problems.map(problem => (
        <li key={problem.id}>{problem.title}</li>
      ))}
    </ul>
  );
}
```

### Micro Frontend Integration

```typescript
// Load a remote microfrontend
const microfrontendConfig = {
  name: 'ContestPlugin',
  url: 'https://cdn.example.com/contest-plugin/remoteEntry.js',
  scope: 'contestPlugin',
  module: './ContestDashboard',
  type: 'module' as const,
  fallback: () => <div>Contest plugin not available</div>
};

await sdk.loadMicrofrontend(microfrontendConfig);

// Use the remote component
function App() {
  const remoteComponent = sdk.getMicrofrontend('ContestPlugin');
  
  if (remoteComponent?.error) {
    return <div>Failed to load contest plugin</div>;
  }
  
  const ContestDashboard = remoteComponent?.component;
  
  return ContestDashboard ? <ContestDashboard /> : <div>Loading...</div>;
}
```

## API Reference

### JudiciaFrontendSDK

Main SDK class for platform integration.

```typescript
class JudiciaFrontendSDK {
  static getInstance(config?: SDKConfig): JudiciaFrontendSDK;
  
  async initialize(context: JudiciaPlatformContext): Promise<void>;
  async loadPlugin(pluginId: string): Promise<PluginMetadata>;
  async loadMicrofrontend(config: MicrofrontendConfig): Promise<RemoteComponent>;
  async registerComponent(registration: ComponentRegistration): Promise<void>;
  
  getComponent(name: string): ComponentRegistration | null;
  getMicrofrontend(name: string): RemoteComponent | null;
  getContext(): JudiciaPlatformContext | null;
  
  emitPlatformEvent<T>(type: string, payload: T, metadata?: Record<string, any>): void;
  onPlatformEvent<T>(type: string, handler: (event: PlatformEvent<T>) => void): () => void;
  
  updateContext(updates: Partial<JudiciaPlatformContext>): void;
  destroy(): void;
}
```

### Hooks

- `useJudicia()` - Access the SDK instance
- `usePlatformContext()` - Access platform context
- `useUser()` - Access current user
- `useCapabilities()` - Check user permissions and roles
- `useTheme()` - Manage theme state
- `usePlatformEvent()` - Subscribe to platform events
- `useEventEmitter()` - Emit platform events
- `useNotifications()` - Manage notifications
- `useLoading()` - Manage loading states
- `useDynamicComponent()` - Load components dynamically
- `useAPI()` - Make API calls with loading states

## Type Definitions

The SDK provides comprehensive TypeScript types for all platform entities:

- `JudiciaUser` - User information and permissions
- `JudiciaPlatformContext` - Global platform state
- `PluginMetadata` - Plugin configuration and capabilities
- `ComponentRegistration` - Component registration schema
- `PlatformEvent<T>` - Event system types
- `ThemeDefinition` - Theme configuration
- `APIResponse<T>` - API response wrapper
- And many more...

## Development

```bash
# Install dependencies
npm install

# Build the SDK
npm run build

# Run tests
npm test

# Start development mode
npm run dev

# Run Storybook
npm run storybook
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- 📖 [Documentation](https://docs.judicia.dev)
- 🐛 [Issue Tracker](https://github.com/judicia/judicia-platform/issues)
- 💬 [Discord Community](https://discord.gg/judicia)
- 📧 [Email Support](mailto:support@judicia.dev)