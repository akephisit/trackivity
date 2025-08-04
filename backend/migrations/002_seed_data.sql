-- Insert sample faculties
INSERT INTO faculties (id, name, code, description) VALUES
    ('550e8400-e29b-41d4-a716-446655440000', 'คณะวิทยาศาสตร์', 'SCI', 'Faculty of Science'),
    ('550e8400-e29b-41d4-a716-446655440001', 'คณะวิศวกรรมศาสตร์', 'ENG', 'Faculty of Engineering'),
    ('550e8400-e29b-41d4-a716-446655440002', 'คณะศิลปศาสตร์', 'ART', 'Faculty of Liberal Arts'),
    ('550e8400-e29b-41d4-a716-446655440003', 'คณะบริหารธุรกิจ', 'BUS', 'Faculty of Business Administration');

-- Insert sample departments
INSERT INTO departments (id, name, code, faculty_id, description) VALUES
    ('660e8400-e29b-41d4-a716-446655440000', 'ภาควิชาคณิตศาสตร์', 'MATH', '550e8400-e29b-41d4-a716-446655440000', 'Department of Mathematics'),
    ('660e8400-e29b-41d4-a716-446655440001', 'ภาควิชาฟิสิกส์', 'PHYS', '550e8400-e29b-41d4-a716-446655440000', 'Department of Physics'),
    ('660e8400-e29b-41d4-a716-446655440002', 'ภาควิชาวิศวกรรมคอมพิวเตอร์', 'CPE', '550e8400-e29b-41d4-a716-446655440001', 'Department of Computer Engineering'),
    ('660e8400-e29b-41d4-a716-446655440003', 'ภาควิชาภาษาอังกฤษ', 'ENG', '550e8400-e29b-41d4-a716-446655440002', 'Department of English'),
    ('660e8400-e29b-41d4-a716-446655440004', 'ภาควิชาการจัดการ', 'MGT', '550e8400-e29b-41d4-a716-446655440003', 'Department of Management');