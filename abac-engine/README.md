# ABAC Engine

The Attribute-Based Access Control (ABAC) engine provides fine-grained authorization for the Judicia Platform. It evaluates access requests based on attributes of users, resources, actions, and environmental conditions.

## Purpose

- **Fine-Grained Authorization**: Control access based on multiple attributes rather than simple roles
- **Dynamic Policy Evaluation**: Make access decisions based on current context and conditions
- **Flexible Permission Model**: Support complex authorization scenarios beyond role-based access
- **Policy Management**: Define and enforce access policies across the platform
- **Audit and Compliance**: Maintain detailed records of authorization decisions

## Key Components

### Policy Engine
Core component that evaluates access requests against defined policies:
- Load and compile authorization policies
- Evaluate attribute-based rules
- Cache policy decisions for performance
- Handle policy updates dynamically

### Attribute Evaluation
Extract and evaluate attributes from different sources:
- **Subject Attributes**: User roles, groups, permissions, profile data
- **Resource Attributes**: Contest status, problem difficulty, submission ownership
- **Action Attributes**: Operation type, request method, API endpoint
- **Environment Attributes**: Time of day, IP location, system load

### Decision Framework
Structured decision-making process:
- **Permit**: Explicitly allow the requested action
- **Deny**: Explicitly deny the requested action  
- **Not Applicable**: Policy doesn't apply to this request
- **Indeterminate**: Unable to evaluate due to missing information

## Authorization Model

### Access Request Structure
Each authorization request contains:
```rust
AccessRequest {
    subject: Subject {        // Who is making the request?
        user_id: Uuid,
        roles: Vec<String>,
        attributes: HashMap<String, AttributeValue>
    },
    resource: Resource {      // What resource is being accessed?
        type: String,
        id: Option<Uuid>,
        attributes: HashMap<String, AttributeValue>
    },
    action: Action {          // What action is being performed?
        operation: String,
        attributes: HashMap<String, AttributeValue>
    },
    environment: Environment { // Under what conditions?
        timestamp: DateTime<Utc>,
        attributes: HashMap<String, AttributeValue>
    }
}
```

### Policy Language
Policies are defined using a domain-specific language that supports:
- Attribute comparisons and logical operators
- Set operations and pattern matching
- Temporal conditions and constraints
- Custom functions and evaluators

### Default Policies
The engine comes with default policies for common scenarios:
- **Admin Access**: Full platform access for administrators
- **Contest Participation**: Users can only access contests they're registered for
- **Submission Ownership**: Users can only view their own submissions
- **Time-Based Access**: Contest access restricted to contest duration

## Attribute Types

### Supported Data Types
- **String**: Text values, roles, usernames
- **Number**: Numeric values, IDs, scores
- **Boolean**: True/false conditions
- **DateTime**: Temporal values and ranges
- **Set**: Collections of values for membership testing

### Attribute Sources
Attributes can be sourced from:
- **User Profile**: Stored user information and preferences
- **Session Context**: Current login session and state
- **Resource Metadata**: Properties of the accessed resource
- **System State**: Current system status and configuration
- **External Systems**: Third-party attribute providers

## Performance Optimization

### Policy Caching
- Compiled policies are cached to avoid repeated parsing
- Decision results are cached for frequently accessed resources
- Cache invalidation on policy updates

### Attribute Caching
- User attributes cached per session
- Resource attributes cached with TTL
- Environment attributes computed on demand

### Evaluation Optimization
- Short-circuit evaluation for simple policies
- Lazy attribute loading for complex evaluations
- Parallel evaluation of independent policy components

## Integration Points

The ABAC engine integrates with:
- **Core Kernel**: Authorization checks for all API endpoints
- **Plugin System**: Capability-based access control for plugins
- **Database**: Attribute retrieval and policy storage
- **Event System**: Authorization for event publishing and subscription
- **Frontend**: UI element visibility and interaction permissions

## Policy Examples

### Contest Access Policy
```
PERMIT IF (
    subject.roles CONTAINS "contestant" AND
    resource.type = "contest" AND
    resource.status = "active" AND
    environment.current_time WITHIN resource.duration AND
    subject.user_id IN resource.participants
)
```

### Admin Override Policy
```
PERMIT IF (
    subject.roles CONTAINS "admin"
) UNLESS (
    resource.type = "system" AND
    action.operation = "delete" AND
    environment.maintenance_mode = false
)
```

## Extensibility

The ABAC engine supports:
- **Custom Attribute Providers**: Add new sources of attribute data
- **Policy Extensions**: Define domain-specific policy constructs
- **Decision Handlers**: Custom logic for specific authorization scenarios
- **Audit Extensions**: Custom logging and monitoring integrations