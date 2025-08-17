-- Add is_enabled field to admin_roles table to separate login activity from account enabled status
DO $$
BEGIN
    BEGIN
        ALTER TABLE admin_roles ADD COLUMN is_enabled BOOLEAN NOT NULL DEFAULT TRUE;
    EXCEPTION WHEN duplicate_column THEN
        -- ignore
        NULL;
    END;
    
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_admin_roles_is_enabled') THEN
        CREATE INDEX idx_admin_roles_is_enabled ON admin_roles(is_enabled);
    END IF;
END $$;

-- Seed data will be inserted through the application or admin interface
