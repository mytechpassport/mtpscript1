use crate::errors::MtpError;

pub fn create_test_snapshot(js: &str) -> Result<Vec<u8>, MtpError> {
    // Simple snapshot: just store the JS code as bytes
    Ok(js.as_bytes().to_vec())
}

pub fn load_snapshot(path: &str) -> Result<Vec<u8>, MtpError> {
    std::fs::read(path).map_err(|e| MtpError::RuntimeError {
        error: "IOError".to_string(),
        message: format!("Failed to load snapshot: {}", e),
    })
}

pub fn save_snapshot(snapshot: &[u8], path: &str) -> Result<(), MtpError> {
    std::fs::write(path, snapshot).map_err(|e| MtpError::RuntimeError {
        error: "IOError".to_string(),
        message: format!("Failed to save snapshot: {}", e),
    })
}

pub fn extract_js_code(snapshot: &[u8]) -> Result<String, MtpError> {
    String::from_utf8(snapshot.to_vec()).map_err(|e| MtpError::RuntimeError {
        error: "DecodeError".to_string(),
        message: format!("Failed to decode JS: {}", e),
    })
}
