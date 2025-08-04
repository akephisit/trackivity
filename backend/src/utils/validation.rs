use regex::Regex;

pub fn validate_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

pub fn validate_student_id(student_id: &str) -> bool {
    // Validate Thai university student ID format (usually 10-13 digits)
    let student_id_regex = Regex::new(r"^[0-9]{10,13}$").unwrap();
    student_id_regex.is_match(student_id)
}

pub fn validate_password(password: &str) -> bool {
    // At least 8 characters, contains at least one letter and one number
    password.len() >= 8 && 
    password.chars().any(|c| c.is_alphabetic()) && 
    password.chars().any(|c| c.is_numeric())
}

pub fn validate_name(name: &str) -> bool {
    // Name should be 1-100 characters and not just whitespace
    !name.trim().is_empty() && name.trim().len() <= 100
}