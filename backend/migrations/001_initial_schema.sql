-- Ensure required extensions
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Create custom types
CREATE TYPE admin_level AS ENUM ('super_admin', 'faculty_admin', 'regular_admin');
CREATE TYPE activity_status AS ENUM ('draft', 'published', 'ongoing', 'completed', 'cancelled');
CREATE TYPE participation_status AS ENUM ('registered', 'checked_in', 'checked_out', 'completed', 'no_show');
CREATE TYPE subscription_type AS ENUM ('basic', 'premium', 'enterprise');
-- Activity type used by admin UI
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'activity_type') THEN
        CREATE TYPE activity_type AS ENUM ('Academic', 'Sports', 'Cultural', 'Social', 'Other');
    END IF;
END $$;

-- Create faculties table
CREATE TABLE faculties (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    code VARCHAR(10) NOT NULL UNIQUE,
    description TEXT,
    status BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create departments table
CREATE TABLE departments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    code VARCHAR(10) NOT NULL,
    faculty_id UUID NOT NULL REFERENCES faculties(id) ON DELETE CASCADE,
    description TEXT,
    status BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(code, faculty_id)
);

-- Create users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id VARCHAR(20) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    qr_secret VARCHAR(255) NOT NULL UNIQUE,
    department_id UUID REFERENCES departments(id) ON DELETE RESTRICT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create admin_roles table
CREATE TABLE admin_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    admin_level admin_level NOT NULL,
    faculty_id UUID REFERENCES faculties(id) ON DELETE CASCADE,
    permissions TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id)
);

-- Create activities table
CREATE TABLE activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    location VARCHAR(255) NOT NULL,
    -- finalized schema for admin UI
    activity_type activity_type,
    academic_year VARCHAR(20) NOT NULL,
    organizer VARCHAR(255) NOT NULL,
    eligible_faculties JSONB NOT NULL DEFAULT '[]',
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    start_time_only TIME NOT NULL,
    end_time_only TIME NOT NULL,
    hours INTEGER NOT NULL,
    max_participants INTEGER CHECK (max_participants > 0),
    status activity_status NOT NULL DEFAULT 'draft',
    faculty_id UUID REFERENCES faculties(id) ON DELETE SET NULL,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CHECK (
        (end_date > start_date) OR 
        (end_date = start_date AND end_time_only > start_time_only)
    )
);

-- Create participations table
CREATE TABLE participations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    activity_id UUID NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    status participation_status NOT NULL DEFAULT 'registered',
    registered_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    checked_in_at TIMESTAMP WITH TIME ZONE,
    checked_out_at TIMESTAMP WITH TIME ZONE,
    notes TEXT,
    UNIQUE(user_id, activity_id),
    CHECK (checked_out_at IS NULL OR checked_in_at IS NOT NULL),
    CHECK (checked_out_at IS NULL OR checked_out_at > checked_in_at)
);

-- Create subscriptions table
CREATE TABLE subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    subscription_type subscription_type NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id)
);

-- Create indexes for better performance
CREATE INDEX idx_users_student_id ON users(student_id);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_department_id ON users(department_id);
CREATE INDEX idx_faculties_status ON faculties(status);
CREATE INDEX idx_admin_roles_user_id ON admin_roles(user_id);
CREATE INDEX idx_admin_roles_faculty_id ON admin_roles(faculty_id);
-- Enable/disable flag and last_session reference for admins
ALTER TABLE admin_roles ADD COLUMN is_enabled BOOLEAN NOT NULL DEFAULT TRUE;
ALTER TABLE admin_roles ADD COLUMN last_session_id VARCHAR(255);
CREATE INDEX idx_admin_roles_is_enabled ON admin_roles(is_enabled);
CREATE INDEX idx_admin_roles_last_session ON admin_roles(last_session_id);
CREATE INDEX idx_activities_faculty_id ON activities(faculty_id);
CREATE INDEX idx_activities_created_by ON activities(created_by);
CREATE INDEX idx_activities_status ON activities(status);
-- New indexes for finalized schema
CREATE INDEX idx_activities_academic_year ON activities(academic_year);
CREATE INDEX idx_activities_activity_type ON activities(activity_type);
CREATE INDEX idx_activities_start_date ON activities(start_date);
CREATE INDEX idx_activities_eligible_faculties ON activities USING GIN (eligible_faculties);
CREATE INDEX idx_participations_user_id ON participations(user_id);
CREATE INDEX idx_participations_activity_id ON participations(activity_id);
CREATE INDEX idx_participations_status ON participations(status);
CREATE INDEX idx_subscriptions_user_id ON subscriptions(user_id);
CREATE INDEX idx_subscriptions_expires_at ON subscriptions(expires_at);

-- Create functions to automatically update updated_at timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers for updated_at
CREATE TRIGGER update_faculties_updated_at BEFORE UPDATE ON faculties
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_departments_updated_at BEFORE UPDATE ON departments
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_admin_roles_updated_at BEFORE UPDATE ON admin_roles
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_activities_updated_at BEFORE UPDATE ON activities
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_subscriptions_updated_at BEFORE UPDATE ON subscriptions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Sessions table and triggers (session tracking)
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

CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);
CREATE INDEX idx_sessions_is_active ON sessions(is_active);
CREATE INDEX idx_sessions_last_accessed ON sessions(last_accessed);

CREATE OR REPLACE FUNCTION update_sessions_last_accessed_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_accessed = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_sessions_last_accessed BEFORE UPDATE ON sessions
    FOR EACH ROW EXECUTE FUNCTION update_sessions_last_accessed_column();

-- Analytics and notifications tables
CREATE TYPE notification_type AS ENUM ('subscription_expiry', 'system_alert', 'admin_notice', 'faculty_update');
CREATE TYPE notification_status AS ENUM ('pending', 'sent', 'failed', 'delivered');

CREATE TABLE faculty_analytics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    faculty_id UUID NOT NULL REFERENCES faculties(id) ON DELETE CASCADE,
    total_students INTEGER DEFAULT 0,
    active_students INTEGER DEFAULT 0,
    total_activities INTEGER DEFAULT 0,
    completed_activities INTEGER DEFAULT 0,
    average_participation_rate DECIMAL(5,2) DEFAULT 0.00,
    monthly_activity_count INTEGER DEFAULT 0,
    department_count INTEGER DEFAULT 0,
    calculated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT unique_faculty_analytics UNIQUE (faculty_id)
);

CREATE TABLE department_analytics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    department_id UUID NOT NULL REFERENCES departments(id) ON DELETE CASCADE,
    faculty_id UUID NOT NULL REFERENCES faculties(id) ON DELETE CASCADE,
    total_students INTEGER DEFAULT 0,
    active_students INTEGER DEFAULT 0,
    total_activities INTEGER DEFAULT 0,
    participation_rate DECIMAL(5,2) DEFAULT 0.00,
    calculated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE subscription_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES subscriptions(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notification_type notification_type NOT NULL,
    status notification_status NOT NULL DEFAULT 'pending',
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    days_until_expiry INTEGER,
    sent_at TIMESTAMP WITH TIME ZONE,
    email_sent BOOLEAN DEFAULT FALSE,
    sse_sent BOOLEAN DEFAULT FALSE,
    admin_notified BOOLEAN DEFAULT FALSE,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE system_analytics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    total_faculties INTEGER DEFAULT 0,
    total_departments INTEGER DEFAULT 0,
    total_users INTEGER DEFAULT 0,
    total_activities INTEGER DEFAULT 0,
    active_subscriptions INTEGER DEFAULT 0,
    expiring_subscriptions_7d INTEGER DEFAULT 0,
    expiring_subscriptions_1d INTEGER DEFAULT 0,
    system_uptime_hours DECIMAL(10,2) DEFAULT 0,
    avg_response_time_ms DECIMAL(8,2) DEFAULT 0,
    calculated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE email_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    to_email VARCHAR(255) NOT NULL,
    to_name VARCHAR(255),
    subject VARCHAR(500) NOT NULL,
    body_text TEXT NOT NULL,
    body_html TEXT,
    priority INTEGER DEFAULT 1,
    status notification_status NOT NULL DEFAULT 'pending',
    attempts INTEGER DEFAULT 0,
    max_attempts INTEGER DEFAULT 3,
    scheduled_for TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    sent_at TIMESTAMP WITH TIME ZONE,
    error_message TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE subscription_expiry_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES subscriptions(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    days_until_expiry INTEGER NOT NULL,
    notification_sent BOOLEAN DEFAULT FALSE,
    admin_alerted BOOLEAN DEFAULT FALSE,
    check_timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

-- Indexes for analytics/notifications
CREATE INDEX idx_faculty_analytics_faculty_id ON faculty_analytics(faculty_id);
CREATE INDEX idx_faculty_analytics_calculated_at ON faculty_analytics(calculated_at);
CREATE INDEX idx_department_analytics_department_id ON department_analytics(department_id);
CREATE INDEX idx_department_analytics_faculty_id ON department_analytics(faculty_id);
CREATE INDEX idx_subscription_notifications_subscription_id ON subscription_notifications(subscription_id);
CREATE INDEX idx_subscription_notifications_user_id ON subscription_notifications(user_id);
CREATE INDEX idx_subscription_notifications_status ON subscription_notifications(status);
CREATE INDEX idx_subscription_notifications_days_until_expiry ON subscription_notifications(days_until_expiry);
CREATE INDEX idx_email_queue_status ON email_queue(status);
CREATE INDEX idx_email_queue_priority ON email_queue(priority);
CREATE INDEX idx_email_queue_scheduled_for ON email_queue(scheduled_for);
CREATE INDEX idx_subscription_expiry_log_subscription_id ON subscription_expiry_log(subscription_id);
CREATE INDEX idx_subscription_expiry_log_check_timestamp ON subscription_expiry_log(check_timestamp);

-- updated_at triggers for analytics tables
CREATE TRIGGER update_faculty_analytics_updated_at BEFORE UPDATE ON faculty_analytics
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_department_analytics_updated_at BEFORE UPDATE ON department_analytics
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_subscription_notifications_updated_at BEFORE UPDATE ON subscription_notifications
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_email_queue_updated_at BEFORE UPDATE ON email_queue
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
