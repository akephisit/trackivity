-- Create sessions table for session tracking and admin monitoring
CREATE TABLE sessions (
    id VARCHAR(255) PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_info JSONB DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_accessed TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    is_active BOOLEAN DEFAULT true
);

-- Create indexes for session management
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);
CREATE INDEX idx_sessions_is_active ON sessions(is_active);
CREATE INDEX idx_sessions_last_accessed ON sessions(last_accessed);

-- Create trigger for updating last_accessed
CREATE TRIGGER update_sessions_last_accessed BEFORE UPDATE ON sessions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Add session-related permissions to admin_roles if needed
-- This allows tracking which sessions are associated with admin operations
ALTER TABLE admin_roles ADD COLUMN IF NOT EXISTS last_session_id VARCHAR(255);
CREATE INDEX IF NOT EXISTS idx_admin_roles_last_session ON admin_roles(last_session_id);