use anyhow::Result;
use base64::{Engine as _, engine::general_purpose};
use qrcode::QrCode;

pub fn generate_qr_code(data: &str) -> Result<String> {
    let code = QrCode::new(data)?;
    let image = code.render::<char>()
        .quiet_zone(false)
        .module_dimensions(2, 1)
        .build();
    
    // For simplicity, we'll return the text representation
    // In a real application, you might want to generate an actual image
    Ok(image)
}

pub fn generate_user_qr_data(user_id: &str, qr_secret: &str) -> String {
    format!("{}:{}", user_id, qr_secret)
}