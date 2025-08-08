-- Create test activities for faculty-specific dashboard testing

DO $$
DECLARE
    science_faculty_id UUID := '550e8400-e29b-41d4-a716-446655440000'; -- Science Faculty
    eng_faculty_id UUID := '550e8400-e29b-41d4-a716-446655440001'; -- Engineering Faculty
    admin_user_id UUID;
    faculty_admin_user_id UUID;
    activity_id UUID;
    counter INTEGER;
BEGIN
    -- Get user IDs for activity creators
    SELECT id INTO admin_user_id FROM users WHERE email = 'admin@trackivity.local';
    SELECT id INTO faculty_admin_user_id FROM users WHERE email = 'faculty.admin@trackivity.local';
    
    -- Create 3 activities in Science Faculty
    FOR counter IN 1..3 LOOP
        activity_id := gen_random_uuid();
        
        INSERT INTO activities (
            id,
            title,
            description,
            location,
            start_time,
            end_time,
            max_participants,
            created_by,
            faculty_id,
            department_id,
            status,
            created_at,
            updated_at
        ) VALUES (
            activity_id,
            'Science Activity ' || counter,
            'This is a science faculty activity number ' || counter,
            'Science Building Room ' || (counter * 100),
            NOW() + (counter || ' days')::INTERVAL,
            NOW() + (counter || ' days')::INTERVAL + '2 hours'::INTERVAL,
            CASE counter 
                WHEN 1 THEN 30
                WHEN 2 THEN 25  
                WHEN 3 THEN 40
            END,
            COALESCE(faculty_admin_user_id, admin_user_id),
            science_faculty_id,
            '660e8400-e29b-41d4-a716-446655440000', -- Math Department
            CASE counter 
                WHEN 1 THEN 'ongoing'
                WHEN 2 THEN 'scheduled'
                WHEN 3 THEN 'ongoing'
            END,
            NOW() - (counter || ' hours')::INTERVAL,
            NOW()
        );
        
        -- Create some participations for these activities
        INSERT INTO participations (
            id,
            user_id,
            activity_id,
            participated_at,
            created_at,
            updated_at
        )
        SELECT 
            gen_random_uuid(),
            u.id,
            activity_id,
            NOW() - (RANDOM() * counter || ' hours')::INTERVAL,
            NOW(),
            NOW()
        FROM users u
        JOIN departments d ON u.department_id = d.id
        WHERE d.faculty_id = science_faculty_id
        LIMIT (counter * 2); -- Varying number of participants
    END LOOP;
    
    -- Create 2 activities in Engineering Faculty (for comparison)
    FOR counter IN 1..2 LOOP
        activity_id := gen_random_uuid();
        
        INSERT INTO activities (
            id,
            title,
            description,
            location,
            start_time,
            end_time,
            max_participants,
            created_by,
            faculty_id,
            department_id,
            status,
            created_at,
            updated_at
        ) VALUES (
            activity_id,
            'Engineering Activity ' || counter,
            'This is an engineering faculty activity number ' || counter,
            'Engineering Building Room ' || (counter * 200),
            NOW() + (counter || ' days')::INTERVAL,
            NOW() + (counter || ' days')::INTERVAL + '3 hours'::INTERVAL,
            20,
            admin_user_id,
            eng_faculty_id,
            '660e8400-e29b-41d4-a716-446655440002', -- Computer Engineering
            'scheduled',
            NOW() - (counter || ' hours')::INTERVAL,
            NOW()
        );
        
        -- Add participations from engineering students
        INSERT INTO participations (
            id,
            user_id,
            activity_id,
            participated_at,
            created_at,
            updated_at
        )
        SELECT 
            gen_random_uuid(),
            u.id,
            activity_id,
            NOW() - (RANDOM() * counter || ' hours')::INTERVAL,
            NOW(),
            NOW()
        FROM users u
        JOIN departments d ON u.department_id = d.id
        WHERE d.faculty_id = eng_faculty_id
        LIMIT counter;
    END LOOP;
    
    RAISE NOTICE 'Test activities created successfully:';
    RAISE NOTICE '- 3 activities in Science Faculty (2 ongoing, 1 scheduled)';
    RAISE NOTICE '- 2 activities in Engineering Faculty (2 scheduled)';
    RAISE NOTICE 'Faculty Admin should see 3 activities and their participations';
    RAISE NOTICE 'Super Admin should see all 5 activities';
END $$;