-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    hashed_password VARCHAR(255) NOT NULL,
    roles TEXT[] NOT NULL DEFAULT '{contestant}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Question types table
CREATE TABLE question_types (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT NOT NULL
);

-- Languages table
CREATE TABLE languages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    version VARCHAR(50) NOT NULL,
    compile_command TEXT,
    run_command TEXT NOT NULL,
    file_extension VARCHAR(10) NOT NULL
);

-- Contests table
CREATE TABLE contests (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    start_time TIMESTAMP WITH TIME ZONE NOT NULL,
    end_time TIMESTAMP WITH TIME ZONE NOT NULL,
    duration INTEGER NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id),
    participant_count INTEGER DEFAULT 0
);

-- Problems table
CREATE TABLE problems (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title VARCHAR(255) NOT NULL,
    author_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    statement TEXT NOT NULL,
    difficulty VARCHAR(20) NOT NULL CHECK (difficulty IN ('easy', 'medium', 'hard')),
    time_limit_ms INTEGER NOT NULL,
    memory_limit_kb INTEGER NOT NULL,
    question_type_id UUID NOT NULL REFERENCES question_types(id),
    metadata JSONB NOT NULL DEFAULT '{}',
    points INTEGER NOT NULL DEFAULT 100,
    contest_id UUID REFERENCES contests(id)
);

-- Test cases table
CREATE TABLE test_cases (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    problem_id UUID NOT NULL REFERENCES problems(id) ON DELETE CASCADE,
    input_data TEXT NOT NULL,
    output_data TEXT NOT NULL,
    is_sample BOOLEAN NOT NULL DEFAULT FALSE,
    order_index INTEGER NOT NULL
);

-- Submissions table
CREATE TABLE submissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id),
    problem_id UUID NOT NULL REFERENCES problems(id),
    language_id UUID NOT NULL REFERENCES languages(id),
    source_code TEXT NOT NULL,
    submitted_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    status VARCHAR(20) NOT NULL DEFAULT 'Queued',
    verdict VARCHAR(30),
    execution_time_ms INTEGER,
    execution_memory_kb INTEGER,
    contest_id UUID REFERENCES contests(id)
);

-- Submission results table
CREATE TABLE submission_results (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    submission_id UUID NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
    test_case_id UUID NOT NULL REFERENCES test_cases(id),
    verdict VARCHAR(30) NOT NULL,
    execution_time_ms INTEGER,
    execution_memory_kb INTEGER,
    stdout TEXT,
    stderr TEXT
);

-- Indexes for better performance
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_problems_contest_id ON problems(contest_id);
CREATE INDEX idx_problems_difficulty ON problems(difficulty);
CREATE INDEX idx_submissions_user_id ON submissions(user_id);
CREATE INDEX idx_submissions_problem_id ON submissions(problem_id);
CREATE INDEX idx_submissions_status ON submissions(status);
CREATE INDEX idx_test_cases_problem_id ON test_cases(problem_id);
CREATE INDEX idx_submission_results_submission_id ON submission_results(submission_id);

-- Insert default question types
INSERT INTO question_types (id, name, description) VALUES
    (uuid_generate_v4(), 'ioi-standard', 'Standard IOI-style problem with input/output files'),
    (uuid_generate_v4(), 'output-only', 'Output-only problem where contestants submit answers directly'),
    (uuid_generate_v4(), 'interactive', 'Interactive problem with real-time communication');

-- Insert default programming languages
INSERT INTO languages (id, name, version, compile_command, run_command, file_extension) VALUES
    (uuid_generate_v4(), 'C++17', '17', 'g++ -std=c++17 -O2 -o solution solution.cpp', './solution', 'cpp'),
    (uuid_generate_v4(), 'Python 3', '3.9', NULL, 'python3 solution.py', 'py'),
    (uuid_generate_v4(), 'Java', '11', 'javac Solution.java', 'java Solution', 'java'),
    (uuid_generate_v4(), 'JavaScript', 'Node 18', NULL, 'node solution.js', 'js');