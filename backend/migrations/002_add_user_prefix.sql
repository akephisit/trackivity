-- Add prefix field to users table
-- This migration adds the Thai prefix (คำนำหน้า) support to the user system

-- Create enum type for common Thai prefixes
CREATE TYPE user_prefix AS ENUM (
    'นาย',         -- Mr.
    'นาง',         -- Mrs.
    'นางสาว',      -- Miss/Ms.
    'ดร.',         -- Dr.
    'ศาสตราจารย์', -- Professor
    'รองศาสตราจารย์', -- Associate Professor
    'ผู้ช่วยศาสตราจารย์', -- Assistant Professor
    'อาจารย์',     -- Lecturer/Teacher
    'คุณ'          -- Generic honorific
);

-- Add prefix column to users table
ALTER TABLE users 
ADD COLUMN prefix user_prefix DEFAULT 'นาย';

-- Create index for prefix field for better query performance
CREATE INDEX idx_users_prefix ON users(prefix);

-- Update existing users with appropriate defaults based on common Thai naming patterns
-- This is a basic assumption and should be reviewed/updated based on actual data
UPDATE users 
SET prefix = CASE 
    WHEN first_name LIKE '%ดร%' OR last_name LIKE '%ดร%' THEN 'ดร.'::user_prefix
    WHEN first_name LIKE '%อาจารย์%' OR last_name LIKE '%อาจารย์%' THEN 'อาจารย์'::user_prefix
    ELSE 'นาย'::user_prefix
END;

-- Add comment to document the prefix field
COMMENT ON COLUMN users.prefix IS 'Thai prefix/title for the user (คำนำหน้า)';
COMMENT ON TYPE user_prefix IS 'Enumeration of common Thai prefixes/titles';