-- Plugin system tables for the microkernel architecture

-- Plugin registrations table
CREATE TABLE plugins (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) UNIQUE NOT NULL,
    version VARCHAR(50) NOT NULL,
    author VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    plugin_type VARCHAR(50) NOT NULL CHECK (plugin_type IN ('contest', 'problem', 'utility')),
    wasm_path TEXT NOT NULL,
    config_schema JSONB NOT NULL DEFAULT '{}',
    capabilities TEXT[] NOT NULL DEFAULT '{}',
    status VARCHAR(20) NOT NULL DEFAULT 'installed' CHECK (status IN ('installed', 'active', 'disabled', 'error')),
    installed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_loaded_at TIMESTAMP WITH TIME ZONE,
    metadata JSONB NOT NULL DEFAULT '{}'
);

-- Plugin permissions and capability access control
CREATE TABLE plugin_permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    plugin_id UUID NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    capability VARCHAR(100) NOT NULL,
    database_access_level VARCHAR(20) NOT NULL DEFAULT 'none' CHECK (database_access_level IN ('none', 'read_only', 'read_write', 'schema_admin')),
    rate_limit_requests_per_second INTEGER NOT NULL DEFAULT 10,
    rate_limit_db_queries_per_minute INTEGER NOT NULL DEFAULT 100,
    rate_limit_events_per_minute INTEGER NOT NULL DEFAULT 50,
    granted_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    granted_by UUID REFERENCES users(id),
    UNIQUE(plugin_id, capability)
);

-- Plugin private database schemas tracking
CREATE TABLE plugin_schemas (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    plugin_id UUID NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    schema_name VARCHAR(100) UNIQUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    size_estimate_kb INTEGER DEFAULT 0,
    last_accessed_at TIMESTAMP WITH TIME ZONE
);

-- Plugin UI route registrations
CREATE TABLE plugin_ui_routes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    plugin_id UUID NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    scope VARCHAR(50) NOT NULL CHECK (scope IN ('contest', 'problem', 'admin', 'global')),
    path VARCHAR(255) NOT NULL,
    component VARCHAR(255) NOT NULL,
    required_permission VARCHAR(50),
    nav_link BOOLEAN DEFAULT FALSE,
    nav_text VARCHAR(100),
    nav_order INTEGER DEFAULT 0,
    UNIQUE(plugin_id, path)
);

-- Plugin HTTP API route registrations
CREATE TABLE plugin_http_routes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    plugin_id UUID NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    path VARCHAR(255) NOT NULL,
    method VARCHAR(10) NOT NULL DEFAULT 'GET' CHECK (method IN ('GET', 'POST', 'PUT', 'DELETE', 'PATCH')),
    handler_function VARCHAR(100) NOT NULL,
    required_permission VARCHAR(50),
    rate_limit_override INTEGER,
    UNIQUE(plugin_id, path, method)
);

-- Plugin event subscriptions
CREATE TABLE plugin_event_subscriptions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    plugin_id UUID NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    event_pattern VARCHAR(255) NOT NULL, -- e.g., "submission.*", "contest.problem.*"
    handler_function VARCHAR(100) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(plugin_id, event_pattern)
);

-- Plugin runtime statistics
CREATE TABLE plugin_statistics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    plugin_id UUID NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    date DATE NOT NULL DEFAULT CURRENT_DATE,
    function_calls INTEGER DEFAULT 0,
    database_queries INTEGER DEFAULT 0,
    events_emitted INTEGER DEFAULT 0,
    events_received INTEGER DEFAULT 0,
    total_execution_time_ms BIGINT DEFAULT 0,
    memory_usage_kb INTEGER DEFAULT 0,
    error_count INTEGER DEFAULT 0,
    UNIQUE(plugin_id, date)
);

-- Contest plugins junction table (which plugins are active for which contests)
CREATE TABLE contest_plugins (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    contest_id UUID NOT NULL REFERENCES contests(id) ON DELETE CASCADE,
    plugin_id UUID NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
    enabled BOOLEAN DEFAULT TRUE,
    configuration JSONB NOT NULL DEFAULT '{}',
    added_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    added_by UUID REFERENCES users(id),
    UNIQUE(contest_id, plugin_id)
);

-- Indexes for plugin system performance
CREATE INDEX idx_plugins_name ON plugins(name);
CREATE INDEX idx_plugins_type ON plugins(plugin_type);
CREATE INDEX idx_plugins_status ON plugins(status);
CREATE INDEX idx_plugin_permissions_plugin_id ON plugin_permissions(plugin_id);
CREATE INDEX idx_plugin_permissions_capability ON plugin_permissions(capability);
CREATE INDEX idx_plugin_schemas_plugin_id ON plugin_schemas(plugin_id);
CREATE INDEX idx_plugin_ui_routes_plugin_id ON plugin_ui_routes(plugin_id);
CREATE INDEX idx_plugin_ui_routes_scope ON plugin_ui_routes(scope);
CREATE INDEX idx_plugin_http_routes_plugin_id ON plugin_http_routes(plugin_id);
CREATE INDEX idx_plugin_http_routes_path ON plugin_http_routes(path);
CREATE INDEX idx_plugin_event_subscriptions_plugin_id ON plugin_event_subscriptions(plugin_id);
CREATE INDEX idx_plugin_event_subscriptions_pattern ON plugin_event_subscriptions(event_pattern);
CREATE INDEX idx_plugin_statistics_plugin_id ON plugin_statistics(plugin_id);
CREATE INDEX idx_plugin_statistics_date ON plugin_statistics(date);
CREATE INDEX idx_contest_plugins_contest_id ON contest_plugins(contest_id);
CREATE INDEX idx_contest_plugins_plugin_id ON contest_plugins(plugin_id);

-- Comments for documentation
COMMENT ON TABLE plugins IS 'Registry of all installed WebAssembly plugins in the system';
COMMENT ON TABLE plugin_permissions IS 'Fine-grained capability permissions for each plugin';
COMMENT ON TABLE plugin_schemas IS 'Tracking of plugin-specific database schemas for data isolation';
COMMENT ON TABLE plugin_ui_routes IS 'Frontend route registrations for plugin UI components';
COMMENT ON TABLE plugin_http_routes IS 'Backend API route registrations for plugin handlers';
COMMENT ON TABLE plugin_event_subscriptions IS 'Event pattern subscriptions for plugins';
COMMENT ON TABLE plugin_statistics IS 'Runtime performance and usage statistics per plugin';
COMMENT ON TABLE contest_plugins IS 'Association of plugins with contests and their configurations';