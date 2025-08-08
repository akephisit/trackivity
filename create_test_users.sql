-- Create test users in Science Faculty for dashboard testing
-- These users will be counted in faculty-specific statistics

DO $$
DECLARE
    science_faculty_id UUID := '550e8400-e29b-41d4-a716-446655440000'; -- Science Faculty
    math_dept_id UUID := '660e8400-e29b-41d4-a716-446655440000'; -- Math Department
    physics_dept_id UUID := '660e8400-e29b-41d4-a716-446655440001'; -- Physics Department
    hashed_password TEXT;
    user_counter INTEGER := 1;
    new_user_id UUID;
BEGIN
    -- Hash a simple password for test users
    hashed_password := '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMaqdHoplKpj3y3/YlFkWam8ze'; -- admin123!
    
    -- Create 5 users in Math Department (Science Faculty)
    FOR user_counter IN 1..5 LOOP
        new_user_id := gen_random_uuid();
        
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
            new_user_id,
            'MATH' || LPAD(user_counter::text, 3, '0'),
            'math.student' || user_counter || '@trackivity.local',
            hashed_password,
            'Math Student',
            'Number ' || user_counter,
            math_dept_id,
            gen_random_uuid()::text,
            NOW() - (user_counter || ' days')::INTERVAL, -- Spread creation dates
            NOW()
        );
    END LOOP;
    
    -- Create 3 users in Physics Department (Science Faculty)
    FOR user_counter IN 1..3 LOOP
        new_user_id := gen_random_uuid();
        
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
            new_user_id,
            'PHYS' || LPAD(user_counter::text, 3, '0'),
            'physics.student' || user_counter || '@trackivity.local',
            hashed_password,
            'Physics Student',
            'Number ' || user_counter,
            physics_dept_id,
            gen_random_uuid()::text,
            NOW() - (user_counter || ' days')::INTERVAL,
            NOW()
        );
    END LOOP;
    
    -- Create 2 users in Engineering Faculty (for comparison)
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
        gen_random_uuid(),
        'CPE001',
        'eng.student1@trackivity.local',
        hashed_password,
        'Engineering Student',
        'Number 1',
        '660e8400-e29b-41d4-a716-446655440002', -- Computer Engineering
        gen_random_uuid()::text,
        NOW() - '1 day'::INTERVAL,
        NOW()
    ),
    (
        gen_random_uuid(),
        'CPE002',
        'eng.student2@trackivity.local',
        hashed_password,
        'Engineering Student',
        'Number 2',
        '660e8400-e29b-41d4-a716-446655440002', -- Computer Engineering
        gen_random_uuid()::text,
        NOW() - '2 days'::INTERVAL,
        NOW()
    );
    
    RAISE NOTICE 'Test users created successfully:';
    RAISE NOTICE '- 5 users in Math Department (Science Faculty)';
    RAISE NOTICE '- 3 users in Physics Department (Science Faculty)';
    RAISE NOTICE '- 2 users in Computer Engineering (Engineering Faculty)';
    RAISE NOTICE 'Faculty Admin should see 8 users (5+3) in Science Faculty';
    RAISE NOTICE 'Super Admin should see 10 users (5+3+2) plus existing admin accounts';
END $$;