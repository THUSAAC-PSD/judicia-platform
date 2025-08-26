-- Evaluation engine and event system tables

-- Judging queue for submission evaluation
CREATE TABLE judging_queue (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    submission_id UUID NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
    priority INTEGER NOT NULL DEFAULT 0, -- Higher numbers = higher priority
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    claimed_at TIMESTAMP WITH TIME ZONE,
    claimed_by VARCHAR(255), -- Worker node identifier
    max_retries INTEGER DEFAULT 3,
    retry_count INTEGER DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'claimed', 'completed', 'failed', 'retrying')),
    error_message TEXT,
    metadata JSONB DEFAULT '{}'
);

-- Worker nodes (evaluation engines) registry
CREATE TABLE worker_nodes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    node_id VARCHAR(100) UNIQUE NOT NULL,
    host_address VARCHAR(255) NOT NULL,
    port INTEGER NOT NULL,
    capabilities TEXT[] NOT NULL DEFAULT '{}', -- e.g., ['cpp', 'python', 'interactive']
    max_concurrent_jobs INTEGER DEFAULT 4,
    current_load INTEGER DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'offline' CHECK (status IN ('online', 'offline', 'maintenance', 'overloaded')),
    last_heartbeat TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    registered_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

-- Event log for system-wide event tracking
CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_type VARCHAR(100) NOT NULL, -- e.g., "submission.created", "contest.started"
    source_plugin_id UUID REFERENCES plugins(id),
    source_user_id UUID REFERENCES users(id),
    source_contest_id UUID REFERENCES contests(id),
    source_submission_id UUID REFERENCES submissions(id),
    payload JSONB NOT NULL DEFAULT '{}',
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    processed BOOLEAN DEFAULT FALSE
);

-- Event subscriptions for system components (not just plugins)
CREATE TABLE event_subscriptions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    subscriber_type VARCHAR(20) NOT NULL CHECK (subscriber_type IN ('plugin', 'service', 'webhook')),
    subscriber_id UUID, -- plugin_id for plugins, null for services
    subscriber_identifier VARCHAR(255), -- service name or webhook URL
    event_pattern VARCHAR(255) NOT NULL, -- e.g., "submission.*", "contest.problem.*"
    callback_url VARCHAR(500), -- For webhook subscriptions
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_triggered TIMESTAMP WITH TIME ZONE
);

-- Sandbox execution results (detailed judging information)
CREATE TABLE execution_results (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    submission_id UUID NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
    test_case_id UUID REFERENCES test_cases(id),
    worker_node_id UUID REFERENCES worker_nodes(id),
    execution_stage VARCHAR(20) NOT NULL CHECK (execution_stage IN ('compile', 'execute', 'compare')),
    verdict VARCHAR(30) NOT NULL, -- AC, WA, TLE, MLE, RE, CE, etc.
    execution_time_ms INTEGER,
    execution_memory_kb INTEGER,
    exit_code INTEGER,
    stdout TEXT,
    stderr TEXT,
    compiler_output TEXT,
    sandbox_info JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ABAC (Attribute-Based Access Control) policy definitions
CREATE TABLE abac_policies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) UNIQUE NOT NULL,
    description TEXT NOT NULL,
    resource_type VARCHAR(50) NOT NULL, -- e.g., 'problem', 'contest', 'submission'
    action VARCHAR(50) NOT NULL, -- e.g., 'read', 'write', 'execute', 'judge'
    policy_definition JSONB NOT NULL, -- The actual ABAC policy rules
    priority INTEGER DEFAULT 0, -- Higher numbers = higher priority
    is_active BOOLEAN DEFAULT TRUE,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_modified_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ABAC policy evaluations log (for auditing)
CREATE TABLE abac_evaluations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    policy_id UUID NOT NULL REFERENCES abac_policies(id),
    user_id UUID REFERENCES users(id),
    resource_type VARCHAR(50) NOT NULL,
    resource_id UUID,
    action VARCHAR(50) NOT NULL,
    decision VARCHAR(10) NOT NULL CHECK (decision IN ('permit', 'deny', 'error')),
    evaluation_time_ms INTEGER,
    context JSONB DEFAULT '{}', -- Request context used for evaluation
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- JPP (Judicia Problem Package) metadata
CREATE TABLE problem_packages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    problem_id UUID NOT NULL REFERENCES problems(id) ON DELETE CASCADE,
    package_version VARCHAR(50) NOT NULL DEFAULT '1.0.0',
    jpp_specification_version VARCHAR(10) NOT NULL DEFAULT '1.0',
    package_path TEXT NOT NULL, -- Path to the JPP directory
    problem_yaml_content TEXT NOT NULL, -- Content of problem.yaml
    validator_type VARCHAR(50), -- e.g., 'standard', 'special_judge', 'interactive'
    validator_path TEXT,
    generator_count INTEGER DEFAULT 0,
    solution_count INTEGER DEFAULT 0,
    test_group_count INTEGER DEFAULT 0,
    package_hash VARCHAR(64), -- SHA-256 hash of package contents
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_modified_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

-- Indexes for performance
CREATE INDEX idx_judging_queue_status ON judging_queue(status);
CREATE INDEX idx_judging_queue_priority ON judging_queue(priority DESC);
CREATE INDEX idx_judging_queue_created_at ON judging_queue(created_at);
CREATE INDEX idx_worker_nodes_status ON worker_nodes(status);
CREATE INDEX idx_worker_nodes_last_heartbeat ON worker_nodes(last_heartbeat);
CREATE INDEX idx_events_event_type ON events(event_type);
CREATE INDEX idx_events_timestamp ON events(timestamp DESC);
CREATE INDEX idx_events_processed ON events(processed);
CREATE INDEX idx_event_subscriptions_pattern ON event_subscriptions(event_pattern);
CREATE INDEX idx_event_subscriptions_active ON event_subscriptions(is_active);
CREATE INDEX idx_execution_results_submission_id ON execution_results(submission_id);
CREATE INDEX idx_execution_results_verdict ON execution_results(verdict);
CREATE INDEX idx_abac_policies_resource_type ON abac_policies(resource_type);
CREATE INDEX idx_abac_policies_active ON abac_policies(is_active);
CREATE INDEX idx_abac_evaluations_policy_id ON abac_evaluations(policy_id);
CREATE INDEX idx_abac_evaluations_timestamp ON abac_evaluations(timestamp);
CREATE INDEX idx_problem_packages_problem_id ON problem_packages(problem_id);
CREATE INDEX idx_problem_packages_hash ON problem_packages(package_hash);

-- Comments for documentation
COMMENT ON TABLE judging_queue IS 'Queue for submissions awaiting evaluation by worker nodes';
COMMENT ON TABLE worker_nodes IS 'Registry of distributed evaluation engine worker nodes';
COMMENT ON TABLE events IS 'System-wide event log for event-driven architecture';
COMMENT ON TABLE event_subscriptions IS 'Subscriptions to events by plugins, services, and webhooks';
COMMENT ON TABLE execution_results IS 'Detailed sandbox execution results for each test case';
COMMENT ON TABLE abac_policies IS 'Attribute-Based Access Control policy definitions';
COMMENT ON TABLE abac_evaluations IS 'Audit log of ABAC policy evaluations';
COMMENT ON TABLE problem_packages IS 'JPP (Judicia Problem Package) metadata and versioning';