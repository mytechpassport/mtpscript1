use crate::errors::MtpError;
use ring::signature::{EcdsaKeyPair, ECDSA_P256_SHA256_FIXED_SIGNING};

pub fn sign_ecdsa_p256(data: &[u8], key_pair: &EcdsaKeyPair) -> Vec<u8> {
    let rng = ring::rand::SystemRandom::new();
    key_pair.sign(&rng, data).unwrap().as_ref().to_vec()
}

/// Validate ECDSA-P256 key for FIPS compliance
pub fn validate_ecdsa_key(key_bytes: &[u8]) -> Result<(), MtpError> {
    // Check minimum key length (ECDSA-P256 should be 32 bytes for private key)
    if key_bytes.len() < 32 {
        return Err(MtpError {
            error: "SecurityError".to_string(),
            message: Some("Key too short for FIPS compliance".to_string()),
            gasLimit: None,
            gasUsed: None,
        });
    }

    // Check maximum key length to prevent DoS
    if key_bytes.len() > 1024 {
        return Err(MtpError {
            error: "SecurityError".to_string(),
            message: Some("Key too long".to_string()),
            gasLimit: None,
            gasUsed: None,
        });
    }

    // Validate that key is not all zeros (weak key)
    if key_bytes.iter().all(|&b| b == 0) {
        return Err(MtpError {
            error: "SecurityError".to_string(),
            message: Some("Key is all zeros - not FIPS compliant".to_string()),
            gasLimit: None,
            gasUsed: None,
        });
    }

    // Validate that key is not all ones (weak key)
    if key_bytes.iter().all(|&b| b == 0xFF) {
        return Err(MtpError {
            error: "SecurityError".to_string(),
            message: Some("Key is all 0xFF - not FIPS compliant".to_string()),
            gasLimit: None,
            gasUsed: None,
        });
    }

    // Check for predictable patterns (simple check)
    let mut sequential = true;
    for i in 1..key_bytes.len() {
        if key_bytes[i] != key_bytes[i - 1].wrapping_add(1) {
            sequential = false;
            break;
        }
    }
    if sequential {
        return Err(MtpError {
            error: "SecurityError".to_string(),
            message: Some("Key appears sequential - not FIPS compliant".to_string()),
            gasLimit: None,
            gasUsed: None,
        });
    }

    // Try to create key pair to validate format
    match EcdsaKeyPair::from_pkcs8_maybe_unchecked(&ECDSA_P256_SHA256_FIXED_SIGNING, key_bytes) {
        Ok(_) => Ok(()),
        Err(_) => Err(MtpError {
            error: "SecurityError".to_string(),
            message: Some("Invalid ECDSA-P256 key format".to_string()),
            gasLimit: None,
            gasUsed: None,
        }),
    }
}

/// Generate FIPS-compliant ECDSA-P256 key pair
pub fn generate_fips_compliant_key() -> Result<EcdsaKeyPair, MtpError> {
    let rng = ring::rand::SystemRandom::new();
    let alg = &ECDSA_P256_SHA256_FIXED_SIGNING;

    // Generate key with FIPS-approved random number generator
    let pkcs8 = EcdsaKeyPair::generate_pkcs8(alg, &rng).map_err(|_| MtpError {
        error: "SecurityError".to_string(),
        message: Some("Failed to generate FIPS-compliant key".to_string()),
        gasLimit: None,
        gasUsed: None,
    })?;

    EcdsaKeyPair::from_pkcs8(alg, pkcs8.as_ref()).map_err(|_| MtpError {
        error: "SecurityError".to_string(),
        message: Some("Generated key failed validation".to_string()),
        gasLimit: None,
        gasUsed: None,
    })
}

/// Validate key strength according to FIPS requirements
pub fn validate_key_strength(key: &EcdsaKeyPair) -> Result<(), MtpError> {
    // For ECDSA-P256, the key is considered strong by definition
    // since it's a standard NIST curve. We mainly validate it's not corrupted.

    // Try signing with a test message to ensure key works
    let test_data = b"FIPS key validation test";
    let rng = ring::rand::SystemRandom::new();

    match key.sign(&rng, test_data) {
        Ok(_) => Ok(()),
        Err(_) => Err(MtpError {
            error: "SecurityError".to_string(),
            message: Some("Key failed signing test".to_string()),
            gasLimit: None,
            gasUsed: None,
        }),
    }
}
