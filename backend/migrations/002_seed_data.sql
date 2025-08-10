-- Add is_enabled field to admin_roles table to separate login activity from account enabled status
ALTER TABLE admin_roles 
ADD COLUMN is_enabled BOOLEAN NOT NULL DEFAULT TRUE;

-- Add index for better performance when querying by enabled status
CREATE INDEX idx_admin_roles_is_enabled ON admin_roles(is_enabled);

-- Seed data will be inserted through the application or admin interface