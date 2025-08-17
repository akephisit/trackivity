-- Migration 006: Add missing fields to activities table for enhanced admin functionality
-- Created: 2025-08-16
-- This migration is idempotent and safe to run multiple times

-- Add academic_year field (if not exists)
DO $$ 
BEGIN 
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='activities' AND column_name='academic_year') THEN
        ALTER TABLE activities ADD COLUMN academic_year VARCHAR(20);
        RAISE NOTICE 'Added academic_year column to activities table';
    ELSE
        RAISE NOTICE 'academic_year column already exists in activities table';
    END IF;
END $$;

-- Add hours field (if not exists)
DO $$ 
BEGIN 
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='activities' AND column_name='hours') THEN
        ALTER TABLE activities ADD COLUMN hours INTEGER;
        RAISE NOTICE 'Added hours column to activities table';
    ELSE
        RAISE NOTICE 'hours column already exists in activities table';
    END IF;
END $$;

-- Add organizer field (if not exists)
DO $$ 
BEGIN 
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='activities' AND column_name='organizer') THEN
        ALTER TABLE activities ADD COLUMN organizer VARCHAR(255);
        RAISE NOTICE 'Added organizer column to activities table';
    ELSE
        RAISE NOTICE 'organizer column already exists in activities table';
    END IF;
END $$;

-- Add eligible_faculties field (if not exists)
DO $$ 
BEGIN 
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='activities' AND column_name='eligible_faculties') THEN
        ALTER TABLE activities ADD COLUMN eligible_faculties JSONB;
        RAISE NOTICE 'Added eligible_faculties column to activities table';
    ELSE
        RAISE NOTICE 'eligible_faculties column already exists in activities table';
    END IF;
END $$;

-- Create activity_type enum and add column (if not exists)
DO $$ 
BEGIN 
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'activity_type') THEN
        CREATE TYPE activity_type AS ENUM ('Academic', 'Sports', 'Cultural', 'Social', 'Other');
        RAISE NOTICE 'Created activity_type enum';
    ELSE
        RAISE NOTICE 'activity_type enum already exists';
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='activities' AND column_name='activity_type') THEN
        ALTER TABLE activities ADD COLUMN activity_type activity_type;
        RAISE NOTICE 'Added activity_type column to activities table';
    ELSE
        RAISE NOTICE 'activity_type column already exists in activities table';
    END IF;
END $$;

-- Add start_date field (if not exists)
DO $$ 
BEGIN 
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='activities' AND column_name='start_date') THEN
        ALTER TABLE activities ADD COLUMN start_date DATE;
        RAISE NOTICE 'Added start_date column to activities table';
    ELSE
        RAISE NOTICE 'start_date column already exists in activities table';
    END IF;
END $$;

-- Add end_date field (if not exists)
DO $$ 
BEGIN 
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='activities' AND column_name='end_date') THEN
        ALTER TABLE activities ADD COLUMN end_date DATE;
        RAISE NOTICE 'Added end_date column to activities table';
    ELSE
        RAISE NOTICE 'end_date column already exists in activities table';
    END IF;
END $$;

-- Add start_time_only field (if not exists)
DO $$ 
BEGIN 
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='activities' AND column_name='start_time_only') THEN
        ALTER TABLE activities ADD COLUMN start_time_only TIME;
        RAISE NOTICE 'Added start_time_only column to activities table';
    ELSE
        RAISE NOTICE 'start_time_only column already exists in activities table';
    END IF;
END $$;

-- Add end_time_only field (if not exists)
DO $$ 
BEGIN 
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='activities' AND column_name='end_time_only') THEN
        ALTER TABLE activities ADD COLUMN end_time_only TIME;
        RAISE NOTICE 'Added end_time_only column to activities table';
    ELSE
        RAISE NOTICE 'end_time_only column already exists in activities table';
    END IF;
END $$;

-- Add indexes for new fields (if not exists)
DO $$ 
BEGIN 
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_activities_academic_year') THEN
        CREATE INDEX idx_activities_academic_year ON activities(academic_year);
        RAISE NOTICE 'Created index idx_activities_academic_year';
    ELSE
        RAISE NOTICE 'Index idx_activities_academic_year already exists';
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_activities_activity_type') THEN
        CREATE INDEX idx_activities_activity_type ON activities(activity_type);
        RAISE NOTICE 'Created index idx_activities_activity_type';
    ELSE
        RAISE NOTICE 'Index idx_activities_activity_type already exists';
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_activities_start_date') THEN
        CREATE INDEX idx_activities_start_date ON activities(start_date);
        RAISE NOTICE 'Created index idx_activities_start_date';
    ELSE
        RAISE NOTICE 'Index idx_activities_start_date already exists';
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_activities_eligible_faculties') THEN
        CREATE INDEX idx_activities_eligible_faculties ON activities USING GIN (eligible_faculties);
        RAISE NOTICE 'Created GIN index idx_activities_eligible_faculties';
    ELSE
        RAISE NOTICE 'Index idx_activities_eligible_faculties already exists';
    END IF;
END $$;

-- Update existing activities with default values if they don't have the new fields
UPDATE activities SET 
    academic_year = '2567',
    organizer = COALESCE(
        (SELECT first_name || ' ' || last_name FROM users WHERE id = activities.created_by), 
        'Unknown Organizer'
    ),
    activity_type = 'Other',
    start_date = COALESCE(start_time::date, CURRENT_DATE),
    end_date = COALESCE(end_time::date, CURRENT_DATE),
    start_time_only = COALESCE(start_time::time, '09:00:00'),
    end_time_only = COALESCE(end_time::time, '17:00:00'),
    eligible_faculties = '[]'::jsonb
WHERE academic_year IS NULL 
   OR organizer IS NULL 
   OR activity_type IS NULL 
   OR start_date IS NULL 
   OR end_date IS NULL 
   OR start_time_only IS NULL 
   OR end_time_only IS NULL 
   OR eligible_faculties IS NULL;

-- Enforce NOT NULL constraints on required fields (after backfilling)
DO $$ 
BEGIN 
    -- academic_year NOT NULL
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='activities' AND column_name='academic_year' AND is_nullable='YES') THEN
        ALTER TABLE activities ALTER COLUMN academic_year SET NOT NULL;
        RAISE NOTICE 'Set NOT NULL on academic_year';
    END IF;

    -- organizer NOT NULL
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='activities' AND column_name='organizer' AND is_nullable='YES') THEN
        ALTER TABLE activities ALTER COLUMN organizer SET NOT NULL;
        RAISE NOTICE 'Set NOT NULL on organizer';
    END IF;

    -- eligible_faculties NOT NULL
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='activities' AND column_name='eligible_faculties' AND is_nullable='YES') THEN
        ALTER TABLE activities ALTER COLUMN eligible_faculties SET NOT NULL;
        RAISE NOTICE 'Set NOT NULL on eligible_faculties';
    END IF;

    -- activity_type NOT NULL
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='activities' AND column_name='activity_type' AND is_nullable='YES') THEN
        ALTER TABLE activities ALTER COLUMN activity_type SET NOT NULL;
        RAISE NOTICE 'Set NOT NULL on activity_type';
    END IF;

    -- start_date NOT NULL
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='activities' AND column_name='start_date' AND is_nullable='YES') THEN
        ALTER TABLE activities ALTER COLUMN start_date SET NOT NULL;
        RAISE NOTICE 'Set NOT NULL on start_date';
    END IF;

    -- end_date NOT NULL
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='activities' AND column_name='end_date' AND is_nullable='YES') THEN
        ALTER TABLE activities ALTER COLUMN end_date SET NOT NULL;
        RAISE NOTICE 'Set NOT NULL on end_date';
    END IF;

    -- start_time_only NOT NULL
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='activities' AND column_name='start_time_only' AND is_nullable='YES') THEN
        ALTER TABLE activities ALTER COLUMN start_time_only SET NOT NULL;
        RAISE NOTICE 'Set NOT NULL on start_time_only';
    END IF;

    -- end_time_only NOT NULL
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='activities' AND column_name='end_time_only' AND is_nullable='YES') THEN
        ALTER TABLE activities ALTER COLUMN end_time_only SET NOT NULL;
        RAISE NOTICE 'Set NOT NULL on end_time_only';
    END IF;

    -- hours NOT NULL
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='activities' AND column_name='hours' AND is_nullable='YES') THEN
        -- Default hours to 1 if NULL before enforcing
        UPDATE activities SET hours = 1 WHERE hours IS NULL;
        ALTER TABLE activities ALTER COLUMN hours SET NOT NULL;
        RAISE NOTICE 'Set NOT NULL on hours';
    END IF;
END $$;

-- Final verification
DO $$
DECLARE
    missing_columns TEXT[] := '{}';
BEGIN
    -- Check for all required columns
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='activities' AND column_name='academic_year') THEN
        missing_columns := array_append(missing_columns, 'academic_year');
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='activities' AND column_name='organizer') THEN
        missing_columns := array_append(missing_columns, 'organizer');
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='activities' AND column_name='eligible_faculties') THEN
        missing_columns := array_append(missing_columns, 'eligible_faculties');
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='activities' AND column_name='activity_type') THEN
        missing_columns := array_append(missing_columns, 'activity_type');
    END IF;
    
    IF array_length(missing_columns, 1) > 0 THEN
        RAISE EXCEPTION 'Migration incomplete. Missing columns: %', array_to_string(missing_columns, ', ');
    ELSE
        RAISE NOTICE 'All required columns exist. Migration completed successfully!';
    END IF;
END $$;
