-- Cleanup activities schema for simplified fields used by the web UI
-- Remove legacy/unused columns and indexes

DO $$
BEGIN
    -- Drop foreign key dependent column department_id if exists
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'activities' AND column_name = 'department_id'
    ) THEN
        -- Drop indexes referencing department_id
        IF EXISTS (
            SELECT 1 FROM pg_indexes WHERE indexname = 'idx_activities_department_id'
        ) THEN
            DROP INDEX idx_activities_department_id;
        END IF;
        ALTER TABLE activities DROP COLUMN department_id;
        RAISE NOTICE 'Dropped activities.department_id';
    END IF;

    -- Drop legacy start_time/end_time timestamptz columns (we use date + time-only now)
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'activities' AND column_name = 'start_time'
    ) THEN
        IF EXISTS (
            SELECT 1 FROM pg_indexes WHERE indexname = 'idx_activities_start_time'
        ) THEN
            DROP INDEX idx_activities_start_time;
        END IF;
        ALTER TABLE activities DROP COLUMN start_time;
        RAISE NOTICE 'Dropped activities.start_time';
    END IF;

    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'activities' AND column_name = 'end_time'
    ) THEN
        ALTER TABLE activities DROP COLUMN end_time;
        RAISE NOTICE 'Dropped activities.end_time';
    END IF;
END $$;

