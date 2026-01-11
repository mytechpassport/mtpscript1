use crate::errors::MtpError;
use crate::security::sign::{load_certificate, verify_ecdsa_p256};
use std::fs;

/// Verify snapshot signature
pub fn verify_snapshot(snapshot_path: &str, cert_path: &str) -> Result<(), MtpError> {
    // Read snapshot
    let snapshot = fs::read(snapshot_path).map_err(|e| MtpError::Io(e.to_string()))?;

    // Read certificate
    let cert_pem = load_certificate(cert_path)?;

    // Extract signature from snapshot (last 64 bytes before CRC)
    if snapshot.len() < 132 {
        return Err(MtpError::Security("Snapshot too small".to_string()));
    }

    let sig_start = snapshot.len() - 132;
    let sig_end = snapshot.len() - 4; // Before CRC32
    let signature = &snapshot[sig_start..sig_end];

    // Extract JS content hash (bytes 20-51)
    let content_hash = &snapshot[20..52];

    // Verify signature
    verify_ecdsa_p256(content_hash, signature, &cert_pem)?;

    // Verify CRC32
    verify_crc32(&snapshot)?;

    Ok(())
}

/// Verify CRC32 checksum
fn verify_crc32(data: &[u8]) -> Result<(), MtpError> {
    if data.len() < 4 {
        return Err(MtpError::Security("Data too small for CRC".to_string()));
    }

    let content = &data[..data.len() - 4];
    let expected_crc = &data[data.len() - 4..];

    let computed_crc = crc32fast::hash(content);
    let expected_crc_u32 = u32::from_le_bytes(expected_crc.try_into().unwrap());

    if computed_crc != expected_crc_u32 {
        return Err(MtpError::Security("CRC32 verification failed".to_string()));
    }

    Ok(())
}

/// Verify snapshot data integrity
pub fn verify_snapshot_integrity(data: &[u8]) -> Result<(), MtpError> {
    if data.len() < 52 {
        return Err(MtpError::Security("Snapshot too small".to_string()));
    }

    // Check magic bytes
    if &data[0..8] != b"MTPJS\x00\x00\x00" {
        return Err(MtpError::Security("Invalid magic bytes".to_string()));
    }

    // Check version
    let version = u32::from_le_bytes(data[8..12].try_into().unwrap());
    if version != 51 {
        return Err(MtpError::Security(format!(
            "Unsupported version: {}",
            version
        )));
    }

    // Verify size
    let declared_size = u64::from_le_bytes(data[12..20].try_into().unwrap()) as usize;
    if declared_size != data.len() {
        return Err(MtpError::Security("Size mismatch".to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_crc32_verification() {
        let mut data = vec![1, 2, 3, 4];
        let crc = crc32fast::hash(&data);
        data.extend_from_slice(&crc.to_le_bytes());

        assert!(verify_crc32(&data).is_ok());

        // Corrupt data
        data[0] = 99;
        assert!(verify_crc32(&data).is_err());
    }

    #[test]
    fn test_snapshot_integrity() {
        // Create a proper snapshot format
        let js_content = b"console.log('test');";
        let mut data = b"MTPJS\x00\x00\x00".to_vec(); // magic
        data.extend_from_slice(&51u32.to_le_bytes()); // version

        // Calculate total size: magic(8) + version(4) + size(8) + hash(32) + content + sig(64) + crc(4)
        let total_size = 8 + 4 + 8 + 32 + js_content.len() + 64 + 4;
        data.extend_from_slice(&(total_size as u64).to_le_bytes()); // size
        data.extend_from_slice(&[0u8; 32]); // hash placeholder
        data.extend_from_slice(js_content); // JS content
        data.extend_from_slice(&[0u8; 64]); // signature placeholder

        // Add CRC of everything except the CRC itself
        let crc = crc32fast::hash(&data);
        data.extend_from_slice(&crc.to_le_bytes());

        assert!(verify_snapshot_integrity(&data).is_ok());

        // Wrong magic
        let mut bad_data = data.clone();
        bad_data[0] = b'X';
        assert!(verify_snapshot_integrity(&bad_data).is_err());
    }
}
