-- Create default system admin account
-- This migration creates a default Super Admin account for initial system setup

DO $$
DECLARE
    admin_user_id UUID;
    hashed_password TEXT;
BEGIN
    -- Check if default admin already exists
    IF NOT EXISTS (SELECT 1 FROM users WHERE email = 'admin@trackivity.local') THEN
        -- Generate UUID for admin user
        admin_user_id := gen_random_uuid();
        
        -- Hash the default password 'admin123!'
        -- Note: In production, this should be changed immediately
        hashed_password := '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMaqdHoplKpj3y3/YlFkWam8ze'; -- admin123!
        
        -- Insert default admin user
        INSERT INTO users (
            id,
            student_id,
            email,
            password_hash,
            first_name,
            last_name,
            department_id,
            qr_secret,
            created_at,
            updated_at
        ) VALUES (
            admin_user_id,
            'ADMIN001',
            'admin@trackivity.local',
            hashed_password,
            'System',
            'Administrator',
            NULL,
            gen_random_uuid()::text,
            NOW(),
            NOW()
        );
        
        -- Create admin role for the user
        INSERT INTO admin_roles (
            id,
            user_id,
            admin_level,
            faculty_id,
            permissions,
            created_at,
            updated_at
        ) VALUES (
            gen_random_uuid(),
            admin_user_id,
            'super_admin',
            NULL,
            ARRAY[
                'ViewSystemReports',
                'ManageAllFaculties', 
                'ManageUsers',
                'ManageActivities',
                'ManageAdmins',
                'ManageSessions',
                'ViewAllReports'
            ],
            NOW(),
            NOW()
        );
        
        RAISE NOTICE 'Default system admin created successfully';
        RAISE NOTICE 'Email: admin@trackivity.local';
        RAISE NOTICE 'Password: admin123!';
        RAISE NOTICE 'IMPORTANT: Change this password immediately after first login!';
    ELSE
        RAISE NOTICE 'Default admin already exists, skipping creation';
    END IF;
END $$;