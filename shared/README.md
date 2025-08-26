# Shared

The shared library contains common data structures, types, and utilities used across all components of the Judicia Platform. It ensures consistency and reduces code duplication throughout the system.

## Purpose

- **Data Model Consistency**: Shared data structures across all components
- **Type Safety**: Common type definitions for database entities
- **API Contracts**: Standardized request/response types for APIs
- **Serialization**: Consistent JSON serialization across the platform
- **Database Mapping**: SQLx-compatible types for database operations

## Key Components

### Models (`models.rs`)
Core data structures representing platform entities:
- **User**: User accounts, authentication, and profile information
- **Contest**: Contest definitions, timing, and configuration
- **Problem**: Problem statements, constraints, and metadata
- **Submission**: Code submissions and evaluation results
- **Language**: Programming language definitions and execution settings
- **TestCase**: Input/output data for problem validation

### Types (`types.rs`)
Common type definitions and enums:
- **Verdict Types**: Accepted, Wrong Answer, Time Limit Exceeded, etc.
- **Difficulty Levels**: Easy, Medium, Hard
- **Contest States**: Draft, Active, Finished
- **User Roles**: Contestant, Admin, Judge
- **Submission Status**: Queued, Running, Completed

### Request/Response Types
Standardized API communication structures:
- **Authentication**: Login, register, password reset requests
- **Contest Operations**: Contest creation, updates, participant management
- **Problem Management**: Problem creation, test case uploads
- **Submission Handling**: Code submission, result retrieval

## Database Integration

All models include:
- **SQLx Compatibility**: Direct mapping to/from database rows
- **Serde Serialization**: JSON serialization for APIs
- **UUID Primary Keys**: Consistent identifier format
- **Timestamp Tracking**: Created/updated time tracking

### Example Model Structure
```rust
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub hashed_password: String,
    pub roles: Vec<String>,
    pub created_at: DateTime<Utc>,
}
```

## Serialization Standards

### JSON Format
All data structures use consistent JSON formatting:
- Snake_case field names for external APIs
- ISO 8601 timestamps
- UUID strings for identifiers
- Null handling for optional fields

### Database Compatibility
Types are designed to work seamlessly with PostgreSQL:
- UUID native support
- JSONB for flexible metadata
- Array types for collections
- Timestamp with timezone

## Validation

Common validation patterns:
- **Email Format**: RFC-compliant email validation
- **Username Rules**: Alphanumeric with specific length constraints
- **Password Strength**: Configurable password requirements
- **Time Constraints**: Logical validation for contest timings

## Error Handling

Standardized error types across the platform:
- **Validation Errors**: Input format and constraint violations
- **Authentication Errors**: Login and permission failures
- **Database Errors**: Connection and query failures
- **Business Logic Errors**: Contest rules and state violations

## Extensibility

The shared library supports:
- **Custom Fields**: JSONB metadata fields for extensible data
- **Role Extensions**: Flexible role-based permission system
- **Contest Types**: Pluggable contest format definitions
- **Problem Types**: Extensible problem format support

## Dependencies

Minimal external dependencies to reduce coupling:
- **chrono**: Date/time handling
- **serde**: Serialization framework
- **sqlx**: Database integration
- **uuid**: Unique identifier generation

## Usage Across Components

Every platform component depends on shared types:
- **Core Kernel**: Database operations and API types
- **Evaluation Engine**: Submission and result types
- **Plugin System**: Common data structures for plugin APIs
- **Frontend**: Type-safe API communication
- **CLI Tools**: Consistent data handling