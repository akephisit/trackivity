pub mod qr;
pub mod validation;

pub use qr::*;
pub use validation::*;

pub fn get_client_info() -> (Option<String>, Option<String>) {
    // In a real implementation, this would extract IP and User-Agent from request
    // For now, return None values
    (None, None)
}