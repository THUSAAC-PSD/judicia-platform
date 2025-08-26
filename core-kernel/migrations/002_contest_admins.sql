-- Add contest_admins table for managing contest admin assignments
CREATE TABLE contest_admins (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    contest_id UUID NOT NULL REFERENCES contests(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    assigned_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(contest_id, user_id)
);

-- Create index for faster lookups
CREATE INDEX idx_contest_admins_contest_id ON contest_admins(contest_id);
CREATE INDEX idx_contest_admins_user_id ON contest_admins(user_id);

-- Update the comment for the users table roles column to include contest_admin
COMMENT ON COLUMN users.roles IS 'User roles array: contestant, contest_admin, admin, superadmin';
