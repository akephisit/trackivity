-- Migration 007: Add hours column to activities for student accumulation tracking
-- Adds an integer hours field representing credited hours for participating students

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'activities' AND column_name = 'hours'
    ) THEN
        ALTER TABLE activities ADD COLUMN hours INTEGER;
        RAISE NOTICE 'Added hours column to activities table';
    ELSE
        RAISE NOTICE 'hours column already exists on activities table';
    END IF;
END $$;

