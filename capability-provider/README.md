# Capability Provider

The capability provider implements a security framework that grants controlled access to platform resources based on declared plugin capabilities. It ensures plugins can only access the resources they explicitly request and are authorized to use.

## Purpose

- **Security Enforcement**: Control access to sensitive platform operations
- **Capability-Based Access**: Grant permissions based on declared capabilities
- **Resource Protection**: Prevent unauthorized access to system resources
- **API Gateway**: Centralized access point for all plugin-to-platform interactions
- **Audit Trail**: Track and log all capability usage for security monitoring

## Key Components

### Judicia Capability Provider
Main implementation that provides controlled access to:
- **Database Operations**: Query and modify contest data with appropriate permissions
- **Event System**: Publish and subscribe to platform events
- **Job Queue**: Submit evaluation jobs and monitor progress
- **WebSocket Communications**: Real-time updates and notifications
- **Messaging System**: Send notifications and communicate with users

### Mock Capability Provider
Testing implementation for development and unit tests:
- Simulates all platform capabilities without external dependencies
- Provides predictable responses for automated testing
- Enables offline development and testing

## Supported Capabilities

### Data Access
- **ReadDatabase**: Query contest data, user information, submissions
- **WriteDatabase**: Modify contest state, update user records
- **ReadFiles**: Access uploaded files and test cases
- **WriteFiles**: Store generated files and outputs

### Communication
- **EmitEvent**: Publish events to the platform event bus
- **SubscribeEvents**: Subscribe to and receive platform events
- **SendNotifications**: Send notifications to users
- **WebSocketAccess**: Real-time communication with frontend clients

### Execution
- **TriggerJudging**: Submit code for evaluation
- **ManageJobs**: Monitor and control evaluation jobs
- **SystemAccess**: Limited access to system resources

### User Interface
- **RegisterRoutes**: Add custom HTTP API endpoints
- **RegisterComponents**: Add frontend UI components
- **ModifyUI**: Alter existing user interface elements

## Security Model

### Capability Declaration
Plugins must declare required capabilities at compile time:
```rust
capabilities: [
    Capability::ReadDatabase,
    Capability::EmitEvent,
    Capability::SendNotifications
]
```

### Runtime Verification
All API calls are validated against declared capabilities:
- Requests without proper capabilities are rejected
- Capability usage is logged for audit purposes
- Resource limits are enforced per capability

### Principle of Least Privilege
Plugins receive only the minimum capabilities needed:
- No default access to any resources
- Explicit grant required for each capability
- Granular permissions within capabilities

## API Surface

The capability provider exposes a controlled API surface that wraps:
- **Database Pool**: PostgreSQL connection pool with query validation
- **Event Bus**: Message publishing and subscription
- **Job Queue**: Evaluation job management
- **WebSocket Manager**: Real-time communication channels

## Resource Management

### Connection Pooling
Efficiently manage database connections across all plugin instances:
- Shared connection pool prevents resource exhaustion
- Connection lifecycle management
- Query optimization and caching

### Rate Limiting
Prevent resource abuse through rate limiting:
- Per-plugin API call limits
- Resource-specific quotas
- Fair sharing across plugins

### Memory Management
Control memory usage of plugin operations:
- Limit query result sizes
- Prevent memory leaks in long-running operations
- Garbage collection coordination

## Integration Points

The capability provider integrates with:
- **Core Kernel**: Plugin capability validation and enforcement
- **Plugin Runtime**: Secure API access for WebAssembly plugins
- **Database**: Controlled access to persistent data
- **Event Bus**: Secure event publishing and subscription
- **Evaluation Engine**: Job submission and monitoring