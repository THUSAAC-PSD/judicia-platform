# Event Bus

The event bus provides a distributed messaging system for the Judicia Platform, enabling loose coupling between components and plugins through asynchronous event-driven communication.

## Purpose

- **Decoupled Communication**: Allow components to communicate without direct dependencies
- **Event Distribution**: Broadcast events to interested subscribers across the system
- **Reliable Messaging**: Ensure message delivery with persistence and retry mechanisms
- **Plugin Integration**: Enable plugins to participate in platform-wide event flows
- **System Coordination**: Coordinate complex workflows across multiple services

## Key Components

### Event Manager
Central coordinator for event publishing and subscription management:
- Route events to appropriate subscribers
- Manage subscription lifecycles
- Handle event filtering and routing rules
- Provide event history and replay capabilities

### Event Types
Standardized event structures for different platform operations:
- **User Events**: Registration, login, profile changes
- **Contest Events**: Creation, updates, start/end notifications
- **Submission Events**: New submissions, evaluation results
- **System Events**: Plugin loading, errors, maintenance

### Subscription Management
Handle dynamic subscription and unsubscription:
- Pattern-based event filtering
- Priority-based delivery
- Dead letter handling for failed deliveries
- Subscription health monitoring

## Supported Backends

### RabbitMQ (Production)
Full-featured message broker with:
- Persistent message storage
- Complex routing patterns
- Clustering support
- Management interface

### Mock Implementation (Testing)
In-memory event bus for development and testing:
- Fast execution without external dependencies
- Complete event history tracking
- Deterministic behavior for tests

## Event Patterns

### Publish-Subscribe
Standard pub/sub pattern where multiple subscribers can receive the same event:
```
Publisher -> Event Bus -> [Subscriber 1, Subscriber 2, ...]
```

### Request-Response
Synchronous-style communication through correlated events:
```
Requester -> Request Event -> Event Bus -> Handler -> Response Event -> Event Bus -> Requester
```

### Event Sourcing
Store all system changes as a sequence of events for audit and replay:
```
Action -> Event -> Event Store -> [Subscribers + Audit Trail]
```

## Message Format

Events follow a standardized JSON format:
- **Event Type**: Hierarchical naming (e.g., `user.login`, `contest.started`)
- **Payload**: Event-specific data
- **Metadata**: Timestamps, correlation IDs, source information
- **Headers**: Routing hints and processing flags

## Reliability Features

- **At-least-once Delivery**: Messages are delivered at least once to each subscriber
- **Retry Logic**: Failed deliveries are retried with exponential backoff
- **Dead Letter Queues**: Permanently failed messages are preserved for analysis
- **Message Persistence**: Events survive system restarts

## Performance Characteristics

- **High Throughput**: Handle thousands of events per second
- **Low Latency**: Sub-millisecond event processing for local events
- **Scalable**: Horizontal scaling through message broker clustering
- **Efficient**: Minimal overhead for event publication and subscription