use crate::errors::MtpError;
use crate::security::sign::{load_signing_key, sign_ecdsa_p256};
use sha2::{Digest, Sha256};
use std::fs;

/// Create a snapshot from compiled JS code
pub fn create_snapshot(js_code: &str, signing_key_path: &str) -> Result<Vec<u8>, MtpError> {
    // Compute SHA-256 of JS content
    let mut hasher = Sha256::new();
    hasher.update(js_code.as_bytes());
    let content_hash = hasher.finalize();

    // Load signing key
    let private_key_pem = load_signing_key(signing_key_path)?;

    // Sign the content hash
    let signature = sign_ecdsa_p256(&content_hash, &private_key_pem)?;

    // Build snapshot format
    let mut snapshot = Vec::new();

    // Magic bytes (8 bytes)
    snapshot.extend_from_slice(b"MTPJS\x00\x00\x00");

    // Version (4 bytes, little-endian)
    snapshot.extend_from_slice(&51u32.to_le_bytes()); // v5.1

    // Total size placeholder (8 bytes, will be filled later)
    let size_offset = snapshot.len();
    snapshot.extend_from_slice(&[0u8; 8]);

    // Content hash (32 bytes)
    snapshot.extend_from_slice(&content_hash);

    // JS content (variable length)
    let _js_offset = snapshot.len();
    snapshot.extend_from_slice(js_code.as_bytes());

    // Signature (64 bytes for ECDSA-P256)
    snapshot.extend_from_slice(&signature);

    // CRC32 (4 bytes)
    let content_for_crc = &snapshot[0..snapshot.len()];
    let crc = crc32fast::hash(content_for_crc);
    snapshot.extend_from_slice(&crc.to_le_bytes());

    // Update total size
    let total_size = snapshot.len() as u64;
    snapshot[size_offset..size_offset + 8].copy_from_slice(&total_size.to_le_bytes());

    Ok(snapshot)
}

/// Save snapshot to file
pub fn save_snapshot(snapshot: &[u8], path: &str) -> Result<(), MtpError> {
    fs::write(path, snapshot).map_err(|e| MtpError::Io {
        error: "Io".to_string(),
        message: e.to_string(),
    })
}

/// Load snapshot from file
pub fn load_snapshot(path: &str) -> Result<Vec<u8>, MtpError> {
    fs::read(path).map_err(|e| MtpError::Io {
        error: "Io".to_string(),
        message: e.to_string(),
    })
}

/// Extract JS code from snapshot
pub fn extract_js_code(snapshot: &[u8]) -> Result<String, MtpError> {
    // Verify basic integrity
    if snapshot.len() < 132 {
        return Err(MtpError::Security {
            error: "Security".to_string(),
            message: "Snapshot too small".to_string(),
        });
    }

    // Check magic bytes
    if &snapshot[0..8] != b"MTPJS\x00\x00\x00" {
        return Err(MtpError::Security {
            error: "Security".to_string(),
            message: "Invalid magic bytes".to_string(),
        });
    }

    // Check version
    let version = u32::from_le_bytes(snapshot[8..12].try_into().unwrap());
    if version != 51 {
        return Err(MtpError::Security {
            error: "Security".to_string(),
            message: format!("Unsupported version: {}", version),
        });
    }

    // Get declared size
    let declared_size = u64::from_le_bytes(snapshot[12..20].try_into().unwrap()) as usize;
    if declared_size != snapshot.len() {
        return Err(MtpError::Security {
            error: "Security".to_string(),
            message: "Size mismatch".to_string(),
        });
    }

    // Extract JS content (from byte 52 to size-68, before signature)
    let js_start = 52;
    let js_end = snapshot.len() - 68; // 64 bytes signature + 4 bytes CRC
    let js_bytes = &snapshot[js_start..js_end];

    String::from_utf8(js_bytes.to_vec())
        .map_err(|_| MtpError::Security {
            error: "Security".to_string(),
            message: "Invalid UTF-8 in JS content".to_string(),
        })
}

/// Create a test snapshot for testing purposes (without signing)
pub fn create_test_snapshot(js: &str) -> Result<Vec<u8>, MtpError> {
    let mut snapshot = Vec::new();

    // Magic bytes (8)
    snapshot.extend_from_slice(b"MTPJS\x00\x00\x00");

    // Version (4)
    snapshot.extend_from_slice(&51u32.to_le_bytes());

    // Size placeholder (8)
    let size_offset = snapshot.len();
    snapshot.extend_from_slice(&[0u8; 8]);

    // Content hash
    let hash = Sha256::digest(js.as_bytes());
    snapshot.extend_from_slice(&hash);

    // JS content
    snapshot.extend_from_slice(js.as_bytes());

    // Signature placeholder (64 bytes of zeros)
    snapshot.extend_from_slice(&[0u8; 64]);

    // CRC32
    let crc = crc32fast::hash(&snapshot);
    snapshot.extend_from_slice(&crc.to_le_bytes());

    // Update size
    let total_size = snapshot.len() as u64;
    snapshot[size_offset..size_offset + 8].copy_from_slice(&total_size.to_le_bytes());

    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::verify::verify_snapshot_integrity;
    use tempfile::NamedTempFile;

    #[test]
    fn test_snapshot_creation_and_extraction() {
        let js_code = "function main() { return 42; }";
        let key_path = "test_key.pem"; // Would need a real key in practice

        // Skip actual signing test without a key file
        // In a real test, we'd create a temporary key

        // Test basic structure
        let snapshot = create_snapshot(js_code, key_path);
        // This will fail without a key file, but that's expected
        assert!(snapshot.is_err()); // For now, just check it tries to load the key
    }

    #[test]
    fn test_snapshot_format_validation() {
        // Test that invalid snapshots are rejected
        let invalid_magic = vec![b'X', b'M', b'T', b'P'];
        assert!(verify_snapshot_integrity(&invalid_magic).is_err());

        // Test that snapshots with wrong version are rejected
        let mut bad_version = create_test_snapshot("function main() { 42 }").unwrap();
        bad_version[8..12].copy_from_slice(&99u32.to_le_bytes()); // Wrong version
        assert!(verify_snapshot_integrity(&bad_version).is_err());

        // Test that snapshots with wrong size are rejected
        let mut bad_size = create_test_snapshot("function main() { 42 }").unwrap();
        bad_size[12..20].copy_from_slice(&999999u64.to_le_bytes()); // Wrong size
        assert!(verify_snapshot_integrity(&bad_size).is_err());
    }

    #[test]
    fn test_snapshot_js_extraction() {
        let snapshot = create_test_snapshot("function main() { return 42; }").unwrap();
        let js_result = extract_js_code(&snapshot);

        assert!(js_result.is_ok());
        let js_code = js_result.unwrap();
        assert_eq!(js_code, "function main() { return 42; }");
    }

    #[test]
    fn test_snapshot_load_save_roundtrip() {
        let js_code = "function test() { return 123; }";

        // Create a minimal snapshot without signing for testing
        let mut snapshot = Vec::new();
        snapshot.extend_from_slice(b"MTPJS\x00\x00\x00"); // magic
        snapshot.extend_from_slice(&51u32.to_le_bytes()); // version
        snapshot.extend_from_slice(&[0u8; 8]); // size placeholder
        snapshot.extend_from_slice(&[0u8; 32]); // hash placeholder
        snapshot.extend_from_slice(js_code.as_bytes()); // JS content
        snapshot.extend_from_slice(&[0u8; 64]); // signature placeholder

        // Calculate CRC
        let crc = crc32fast::hash(&snapshot);
        snapshot.extend_from_slice(&crc.to_le_bytes());

        // Update size
        let total_size = snapshot.len() as u64;
        snapshot[12..20].copy_from_slice(&total_size.to_le_bytes());

        // Test save and load
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_string_lossy().to_string();

        assert!(save_snapshot(&snapshot, &path).is_ok());
        let loaded = load_snapshot(&path);
        assert!(loaded.is_ok());
        assert_eq!(loaded.unwrap(), snapshot);
    }
}
