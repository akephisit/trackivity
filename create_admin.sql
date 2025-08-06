-- Delete existing admin user if exists
DELETE FROM admin_roles WHERE user_id IN (SELECT id FROM users WHERE email = 'admin@trackivity.local');
DELETE FROM users WHERE email = 'admin@trackivity.local';

-- Insert new admin user with password 'admin123'
INSERT INTO users (email, password_hash, first_name, last_name, student_id, qr_secret)
VALUES (
    'admin@trackivity.local',
    '$2b$12$0YUJPw9TyGJz0ZUoMDVHdOHGPgE8HKtUZnE1QiWQvhBvGZNdvLJQu', -- password: admin123
    'Admin',
    'User',
    '000000',
    'admin-qr-secret-unique'
);

-- Get the user ID and insert admin role
INSERT INTO admin_roles (user_id, admin_level, created_at, updated_at, permissions)
VALUES (
    (SELECT id FROM users WHERE email = 'admin@trackivity.local'),
    'super_admin',
    NOW(),
    NOW(),
    ARRAY['*']::text[]
);