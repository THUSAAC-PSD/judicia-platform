# Core Kernel

The core kernel is the heart of the Judicia Platform, responsible for managing the entire contest system. It provides a plugin-based architecture that allows for extensible functionality while maintaining a robust core.

## Purpose

- **Contest Management**: Handle contest creation, modification, and administration
- **User Authentication**: Secure user registration, login, and role management
- **Plugin Orchestration**: Load, manage, and route requests to plugins
- **Database Operations**: Centralized database access and migrations
- **Event System**: Coordinate system-wide events and notifications
- **API Gateway**: Unified API endpoints for all platform operations

## Key Features

### Time Extensions System
Administrators can grant time extensions to individual contestants or entire contests. Extensions are tracked with reasons and granted-by information for audit purposes.

### Flexible Scoring Methods
Support for three different scoring calculation methods:
- **Last Submission**: Uses the score from the most recent submission
- **Max Score**: Takes the highest score from all submissions
- **Subtask Sum**: Sums the highest scores from each subtask

### Enhanced Rejudge System
Comprehensive rejudging capabilities with three types:
- **Full Rejudge**: Complete recompilation and re-execution
- **Score Only**: Recalculate scores without re-execution  
- **Compile Only**: Re-compile submissions only

## Architecture

The kernel follows a plugin-based architecture where core functionality is extended through WebAssembly plugins. This allows for:
- **Modularity**: Features can be added without modifying core code
- **Security**: Plugins run in sandboxed environments
- **Flexibility**: Different contest formats can be supported through plugins

## Database

Uses PostgreSQL with SQLx for database operations. Supports both local PostgreSQL and cloud providers like Supabase. Migrations are automatically applied on startup to keep the schema up-to-date.

### Supported Database Providers
- **Supabase** (recommended): Managed PostgreSQL with built-in authentication and real-time features
- **Local PostgreSQL**: Traditional local database setup  
- **Neon, Railway, AWS RDS**: Other PostgreSQL-compatible services

## Configuration

Configured through environment variables:

### Database Configuration
- `DATABASE_URL`: PostgreSQL connection string (required)
- `SUPABASE_URL`: Supabase project URL (optional, for additional features)
- `SUPABASE_ANON_KEY`: Supabase anonymous key (optional)
- `SUPABASE_SERVICE_ROLE_KEY`: Supabase service role key (optional)

### Other Configuration
- `RABBITMQ_URL`: RabbitMQ connection for event bus
- `JWT_SECRET`: Secret key for JWT token generation
- `SERVER_ADDRESS`: Address and port to bind the server
- `TEST_MODE`: Skip database for plugin testing only

See [SUPABASE_SETUP.md](../SUPABASE_SETUP.md) for detailed database setup instructions.