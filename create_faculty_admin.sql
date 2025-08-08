-- Create a Faculty Admin for testing
-- This admin will be responsible for คณะวิทยาศาสตร์ (Science Faculty)

DO $$
DECLARE
    faculty_admin_user_id UUID;
    science_faculty_id UUID := '550e8400-e29b-41d4-a716-446655440000'; -- Science Faculty ID from seed data
    math_dept_id UUID := '660e8400-e29b-41d4-a716-446655440000'; -- Math Department ID
    hashed_password TEXT;
BEGIN
    -- Check if faculty admin already exists
    IF NOT EXISTS (SELECT 1 FROM users WHERE email = 'faculty.admin@trackivity.local') THEN
        -- Generate UUID for faculty admin user
        faculty_admin_user_id := gen_random_uuid();
        
        -- Hash the default password 'faculty123!'
        hashed_password := '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMaqdHoplKpj3y3/YlFkWam8ze'; -- same as admin123! for simplicity
        
        -- Insert faculty admin user
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
            faculty_admin_user_id,
            'FAC001',
            'faculty.admin@trackivity.local',
            hashed_password,
            'Faculty',
            'Administrator',
            math_dept_id, -- Put in Math department under Science faculty
            gen_random_uuid()::text,
            NOW(),
            NOW()
        );
        
        -- Create faculty admin role for the user
        INSERT INTO admin_roles (
            id,
            user_id,
            admin_level,
            faculty_id, -- This is the key field for faculty-specific stats
            permissions,
            created_at,
            updated_at
        ) VALUES (
            gen_random_uuid(),
            faculty_admin_user_id,
            'faculty_admin', -- Faculty admin level
            science_faculty_id, -- Science Faculty ID
            ARRAY[
                'ManageUsers',
                'ManageActivities', 
                'ViewDashboard',
                'ViewFacultyReports'
            ],
            NOW(),
            NOW()
        );
        
        RAISE NOTICE 'Faculty admin created successfully';
        RAISE NOTICE 'Email: faculty.admin@trackivity.local';
        RAISE NOTICE 'Password: faculty123!';
        RAISE NOTICE 'Faculty: คณะวิทยาศาสตร์ (Science)';
        RAISE NOTICE 'Department: ภาควิชาคณิตศาสตร์ (Mathematics)';
    ELSE
        RAISE NOTICE 'Faculty admin already exists, skipping creation';
    END IF;
END $$;