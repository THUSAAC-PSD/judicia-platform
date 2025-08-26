-- Migration: Contest Extensions and Scoring Methods
-- Adds support for time extensions and configurable scoring methods

-- Add scoring method to contests table
ALTER TABLE contests ADD COLUMN scoring_method VARCHAR(50) DEFAULT 'last_submission';
ALTER TABLE contests ADD COLUMN extra_time_minutes INTEGER DEFAULT 0;

-- Contest time extensions table (for individual users)
CREATE TABLE contest_extensions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    contest_id UUID NOT NULL REFERENCES contests(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    extra_minutes INTEGER NOT NULL,
    reason TEXT NOT NULL,
    granted_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    
    -- Ensure one extension per user per contest
    UNIQUE(contest_id, user_id)
);

-- Subtask support for problems
CREATE TABLE subtasks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    problem_id UUID NOT NULL REFERENCES problems(id) ON DELETE CASCADE,
    subtask_number INTEGER NOT NULL,
    name VARCHAR(100) NOT NULL,
    max_score DECIMAL(10,2) NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    
    -- Ensure unique subtask numbers per problem
    UNIQUE(problem_id, subtask_number)
);

-- Link test cases to subtasks
ALTER TABLE test_cases ADD COLUMN subtask_id UUID REFERENCES subtasks(id);

-- Submission scoring history for tracking max scores
CREATE TABLE submission_scores (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    submission_id UUID NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
    subtask_id UUID REFERENCES subtasks(id),
    score DECIMAL(10,2) NOT NULL,
    max_score DECIMAL(10,2) NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    
    -- Index for efficient score queries
    INDEX(submission_id, subtask_id)
);

-- Rejudge requests tracking
CREATE TABLE rejudge_requests (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    contest_id UUID REFERENCES contests(id),
    problem_id UUID REFERENCES problems(id),
    submission_ids JSON NOT NULL, -- Array of submission UUIDs
    rejudge_type VARCHAR(30) NOT NULL, -- 'full', 'score_only', 'compile_only'
    reason TEXT NOT NULL,
    status VARCHAR(30) DEFAULT 'pending', -- 'pending', 'in_progress', 'completed', 'failed'
    requested_by UUID NOT NULL REFERENCES users(id),
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW(),
    
    -- Indexes for admin queries
    INDEX(contest_id, created_at),
    INDEX(status, created_at)
);

-- Update submissions table to track scoring method used
ALTER TABLE submissions ADD COLUMN scoring_method VARCHAR(50);
ALTER TABLE submissions ADD COLUMN subtask_scores JSON; -- Store per-subtask scores

-- Indexes for performance
CREATE INDEX idx_contest_extensions_contest_id ON contest_extensions(contest_id);
CREATE INDEX idx_contest_extensions_user_id ON contest_extensions(user_id);
CREATE INDEX idx_subtasks_problem_id ON subtasks(problem_id);
CREATE INDEX idx_submission_scores_submission_id ON submission_scores(submission_id);
CREATE INDEX idx_submission_scores_subtask_id ON submission_scores(subtask_id);

-- Add some example scoring methods as enum check
ALTER TABLE contests ADD CONSTRAINT check_scoring_method 
    CHECK (scoring_method IN ('last_submission', 'max_score', 'subtask_sum'));

ALTER TABLE rejudge_requests ADD CONSTRAINT check_rejudge_type
    CHECK (rejudge_type IN ('full', 'score_only', 'compile_only'));

ALTER TABLE rejudge_requests ADD CONSTRAINT check_rejudge_status
    CHECK (status IN ('pending', 'in_progress', 'completed', 'failed'));

-- Insert some example subtasks for demonstration (optional)
-- These would normally be created when problems are imported