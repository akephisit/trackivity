-- Migration for Faculty Analytics and Subscription Notifications
-- Created for Trackivity Platform Faculty Management System

-- Create notification types
CREATE TYPE notification_type AS ENUM ('subscription_expiry', 'system_alert', 'admin_notice', 'faculty_update');
CREATE TYPE notification_status AS ENUM ('pending', 'sent', 'failed', 'delivered');

-- Faculty Analytics table
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
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Department Analytics table
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

-- Subscription Notifications table (tracking-only, no enforcement)
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

-- System wide analytics for Super Admin Dashboard
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

-- Email notification queue for async processing
CREATE TABLE email_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    to_email VARCHAR(255) NOT NULL,
    to_name VARCHAR(255),
    subject VARCHAR(500) NOT NULL,
    body_text TEXT NOT NULL,
    body_html TEXT,
    priority INTEGER DEFAULT 1, -- 1=low, 2=normal, 3=high, 4=critical
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

-- Subscription expiry tracking log
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

-- Create indexes for performance
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

-- Create updated_at triggers
CREATE TRIGGER update_faculty_analytics_updated_at BEFORE UPDATE ON faculty_analytics
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_department_analytics_updated_at BEFORE UPDATE ON department_analytics
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_subscription_notifications_updated_at BEFORE UPDATE ON subscription_notifications
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_email_queue_updated_at BEFORE UPDATE ON email_queue
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function to calculate faculty analytics
CREATE OR REPLACE FUNCTION calculate_faculty_analytics(faculty_uuid UUID)
RETURNS VOID AS $$
DECLARE
    student_count INTEGER;
    active_student_count INTEGER;
    activity_count INTEGER;
    completed_activity_count INTEGER;
    participation_rate DECIMAL(5,2);
    monthly_activities INTEGER;
    dept_count INTEGER;
BEGIN
    -- Get total students in faculty
    SELECT COUNT(DISTINCT u.id) INTO student_count
    FROM users u
    JOIN departments d ON u.department_id = d.id
    WHERE d.faculty_id = faculty_uuid;

    -- Get active students (participated in activities in last 30 days)
    SELECT COUNT(DISTINCT u.id) INTO active_student_count
    FROM users u
    JOIN departments d ON u.department_id = d.id
    JOIN participations p ON u.id = p.user_id
    JOIN activities a ON p.activity_id = a.id
    WHERE d.faculty_id = faculty_uuid
    AND p.created_at >= NOW() - INTERVAL '30 days'
    AND p.status IN ('checked_in', 'checked_out', 'completed');

    -- Get total activities
    SELECT COUNT(*) INTO activity_count
    FROM activities
    WHERE faculty_id = faculty_uuid;

    -- Get completed activities
    SELECT COUNT(*) INTO completed_activity_count
    FROM activities
    WHERE faculty_id = faculty_uuid
    AND status = 'completed';

    -- Calculate participation rate
    IF student_count > 0 THEN
        participation_rate := (active_student_count::DECIMAL / student_count::DECIMAL) * 100;
    ELSE
        participation_rate := 0;
    END IF;

    -- Get monthly activity count
    SELECT COUNT(*) INTO monthly_activities
    FROM activities
    WHERE faculty_id = faculty_uuid
    AND created_at >= NOW() - INTERVAL '30 days';

    -- Get department count
    SELECT COUNT(*) INTO dept_count
    FROM departments
    WHERE faculty_id = faculty_uuid;

    -- Insert or update analytics
    INSERT INTO faculty_analytics (
        faculty_id, total_students, active_students, total_activities, 
        completed_activities, average_participation_rate, 
        monthly_activity_count, department_count, calculated_at
    ) VALUES (
        faculty_uuid, student_count, active_student_count, activity_count,
        completed_activity_count, participation_rate,
        monthly_activities, dept_count, NOW()
    )
    ON CONFLICT (faculty_id) DO UPDATE SET
        total_students = EXCLUDED.total_students,
        active_students = EXCLUDED.active_students,
        total_activities = EXCLUDED.total_activities,
        completed_activities = EXCLUDED.completed_activities,
        average_participation_rate = EXCLUDED.average_participation_rate,
        monthly_activity_count = EXCLUDED.monthly_activity_count,
        department_count = EXCLUDED.department_count,
        calculated_at = NOW(),
        updated_at = NOW();
END;
$$ LANGUAGE plpgsql;

-- Add unique constraint to prevent duplicate analytics per faculty
ALTER TABLE faculty_analytics ADD CONSTRAINT unique_faculty_analytics UNIQUE (faculty_id);

-- Function to calculate system-wide analytics
CREATE OR REPLACE FUNCTION calculate_system_analytics()
RETURNS VOID AS $$
DECLARE
    faculty_count INTEGER;
    dept_count INTEGER;
    user_count INTEGER;
    activity_count INTEGER;
    active_subs INTEGER;
    expiring_7d INTEGER;
    expiring_1d INTEGER;
BEGIN
    SELECT COUNT(*) INTO faculty_count FROM faculties;
    SELECT COUNT(*) INTO dept_count FROM departments;
    SELECT COUNT(*) INTO user_count FROM users;
    SELECT COUNT(*) INTO activity_count FROM activities;
    
    SELECT COUNT(*) INTO active_subs 
    FROM subscriptions 
    WHERE is_active = true AND expires_at > NOW();
    
    SELECT COUNT(*) INTO expiring_7d
    FROM subscriptions 
    WHERE is_active = true 
    AND expires_at > NOW() 
    AND expires_at <= NOW() + INTERVAL '7 days';
    
    SELECT COUNT(*) INTO expiring_1d
    FROM subscriptions 
    WHERE is_active = true 
    AND expires_at > NOW() 
    AND expires_at <= NOW() + INTERVAL '1 day';

    INSERT INTO system_analytics (
        total_faculties, total_departments, total_users, total_activities,
        active_subscriptions, expiring_subscriptions_7d, expiring_subscriptions_1d,
        calculated_at
    ) VALUES (
        faculty_count, dept_count, user_count, activity_count,
        active_subs, expiring_7d, expiring_1d, NOW()
    );
END;
$$ LANGUAGE plpgsql;