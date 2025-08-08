-- Add status field to faculties table
ALTER TABLE faculties ADD COLUMN status BOOLEAN NOT NULL DEFAULT TRUE;

-- Create index for status field
CREATE INDEX idx_faculties_status ON faculties(status);

-- Update existing faculties to be active
UPDATE faculties SET status = TRUE;

-- Add comment to describe the status field
COMMENT ON COLUMN faculties.status IS 'Faculty active status: true = active, false = inactive';