use anyhow::{Result, anyhow};
// chrono imports removed as DateTime and Utc are unused in this module
use hmac::{Hmac, Mac};
use qrcode::QrCode;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

/// QR Code data structure ที่จะถูกเข้ารหัสใน QR Code (รุ่นใหม่)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrData {
    pub student_id: String,
    pub timestamp: u64,
    pub signature: String,
}

/// QR Code data structure สำหรับ client-side generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientQrData {
    pub user_id: uuid::Uuid,
    pub student_id: String,
    pub secret: String,
    pub timestamp: u64,
}

/// QR Code generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrGenerationResponse {
    pub qr_data: String,
    pub expires_at: u64,
}

/// ผลลัพธ์การตรวจสอบ QR Code
#[derive(Debug, Clone)]
pub struct QrValidationResult {
    pub student_id: String,
    pub is_valid: bool,
    pub error_message: Option<String>,
}

/// สร้าง QR Code data พร้อม enhanced HMAC signature verification
pub fn generate_qr_data(student_id: &str, secret_key: &str) -> Result<String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    
    // สร้าง message สำหรับ HMAC พร้อม additional security context
    let message = format!("{}:{}:qr_v2", student_id, timestamp);
    
    // สร้าง enhanced HMAC signature with SHA256
    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
        .map_err(|e| anyhow!("Invalid secret key: {}", e))?;
    mac.update(message.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());
    
    let qr_data = QrData {
        student_id: student_id.to_string(),
        timestamp,
        signature,
    };
    
    let json_data = serde_json::to_string(&qr_data)?;
    Ok(json_data)
}

/// ตรวจสอบ QR Code data และ signature
pub fn validate_qr_data(qr_json: &str, secret_key: &str, max_age_seconds: u64) -> QrValidationResult {
    // Parse JSON data
    let qr_data: QrData = match serde_json::from_str(qr_json) {
        Ok(data) => data,
        Err(_) => return QrValidationResult {
            student_id: String::new(),
            is_valid: false,
            error_message: Some("Invalid QR code format".to_string()),
        },
    };
    
    // ตรวจสอบ timestamp (replay attack prevention)
    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    if current_timestamp > qr_data.timestamp + max_age_seconds {
        return QrValidationResult {
            student_id: qr_data.student_id,
            is_valid: false,
            error_message: Some("QR code has expired".to_string()),
        };
    }
    
    // ตรวจสอบ enhanced HMAC signature with security context
    let message = format!("{}:{}:qr_v2", qr_data.student_id, qr_data.timestamp);
    let mut mac = match HmacSha256::new_from_slice(secret_key.as_bytes()) {
        Ok(mac) => mac,
        Err(_) => return QrValidationResult {
            student_id: qr_data.student_id,
            is_valid: false,
            error_message: Some("Server configuration error".to_string()),
        },
    };
    
    mac.update(message.as_bytes());
    let expected_signature = hex::encode(mac.finalize().into_bytes());
    
    if qr_data.signature != expected_signature {
        return QrValidationResult {
            student_id: qr_data.student_id,
            is_valid: false,
            error_message: Some("Invalid signature".to_string()),
        };
    }
    
    QrValidationResult {
        student_id: qr_data.student_id,
        is_valid: true,
        error_message: None,
    }
}

/// สร้าง QR Code แบบ text representation (สำหรับ demo)
pub fn generate_qr_code(data: &str) -> Result<String> {
    let code = QrCode::new(data)?;
    let image = code
        .render::<char>()
        .quiet_zone(false)
        .module_dimensions(2, 1)
        .build();

    Ok(image)
}

/// สร้าง unique identifier สำหรับ QR Code (UUID)
pub fn generate_qr_identifier() -> String {
    Uuid::new_v4().to_string()
}

/// สร้าง secret key ใหม่สำหรับ user
pub fn generate_secret_key() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    // สร้าง random bytes 32 bytes (256 bits)
    let bytes: [u8; 32] = rng.gen();
    hex::encode(bytes)
}

/// สร้าง QR data สำหรับ client-side generation
pub fn generate_client_qr_data(user_id: &Uuid, student_id: &str, secret: &str) -> Result<QrGenerationResponse> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    
    let client_data = ClientQrData {
        user_id: *user_id,
        student_id: student_id.to_string(),
        secret: secret.to_string(),
        timestamp,
    };
    
    let json_data = serde_json::to_string(&client_data)?;
    let expires_at = timestamp + (5 * 60); // 5 minutes expiry
    
    Ok(QrGenerationResponse {
        qr_data: json_data,
        expires_at,
    })
}

/// ตรวจสอบ client-side QR code data
pub fn validate_client_qr_data(qr_json: &str, expected_secret: &str, max_age_seconds: u64) -> QrValidationResult {
    // Parse JSON data
    let client_data: ClientQrData = match serde_json::from_str(qr_json) {
        Ok(data) => data,
        Err(_) => return QrValidationResult {
            student_id: String::new(),
            is_valid: false,
            error_message: Some("Invalid QR code format".to_string()),
        },
    };
    
    // ตรวจสอบ timestamp (replay attack prevention)
    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    if current_timestamp > client_data.timestamp + max_age_seconds {
        return QrValidationResult {
            student_id: client_data.student_id,
            is_valid: false,
            error_message: Some("QR code has expired".to_string()),
        };
    }
    
    // ตรวจสอบ secret
    if client_data.secret != expected_secret {
        return QrValidationResult {
            student_id: client_data.student_id,
            is_valid: false,
            error_message: Some("Invalid QR code secret".to_string()),
        };
    }
    
    QrValidationResult {
        student_id: client_data.student_id,
        is_valid: true,
        error_message: None,
    }
}

/// Validate student_id format
pub fn validate_student_id(student_id: &str) -> bool {
    // ตรวจสอบว่า student_id เป็นตัวอักษรและตัวเลขเท่านั้น และมีความยาวเหมาะสม
    student_id.len() >= 3 && student_id.len() <= 20 && 
    student_id.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_generate_and_validate_qr_data() {
        let student_id = "STU001";
        let secret_key = generate_secret_key();
        
        // สร้าง QR data
        let qr_json = generate_qr_data(student_id, &secret_key).unwrap();
        
        // ตรวจสอบ QR data
        let result = validate_qr_data(&qr_json, &secret_key, 300); // 5 minutes
        
        assert!(result.is_valid);
        assert_eq!(result.student_id, student_id);
        assert!(result.error_message.is_none());
    }
    
    #[test]
    fn test_invalid_signature() {
        let student_id = "STU002";
        let secret_key = generate_secret_key();
        let wrong_key = generate_secret_key();
        
        // สร้าง QR data ด้วย key หนึ่ง
        let qr_json = generate_qr_data(student_id, &secret_key).unwrap();
        
        // ตรวจสอบด้วยอีก key หนึ่ง
        let result = validate_qr_data(&qr_json, &wrong_key, 300);
        
        assert!(!result.is_valid);
        assert!(result.error_message.is_some());
    }
    
    #[test]
    fn test_expired_qr_code() {
        let student_id = "STU003";
        let secret_key = generate_secret_key();
        
        // สร้าง QR data
        let qr_json = generate_qr_data(student_id, &secret_key).unwrap();
        
        // รอให้ QR หมดอายุ (ใช้ max_age = 0 เพื่อทดสอบ)
        thread::sleep(Duration::from_millis(10));
        let result = validate_qr_data(&qr_json, &secret_key, 0);
        
        assert!(!result.is_valid);
        assert!(result.error_message.unwrap().contains("expired"));
    }
    
    #[test]
    fn test_validate_student_id() {
        assert!(validate_student_id("STU001"));
        assert!(validate_student_id("2567123456"));
        assert!(!validate_student_id("AB"));  // too short
        assert!(!validate_student_id(""));    // empty
        assert!(!validate_student_id("STU@001")); // invalid character
    }
}
