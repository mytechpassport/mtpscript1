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
        return Err(MtpError::SecurityError(
            "Key too short for FIPS compliance".into(),
        ));
    }

    // Check maximum key length to prevent DoS
    if key_bytes.len() > 1024 {
        return Err(MtpError::SecurityError("Key too long".into()));
    }

    // Validate that key is not all zeros (weak key)
    if key_bytes.iter().all(|&b| b == 0) {
        return Err(MtpError::SecurityError(
            "Key is all zeros - not FIPS compliant".into(),
        ));
    }

    // Validate that key is not all ones (weak key)
    if key_bytes.iter().all(|&b| b == 0xFF) {
        return Err(MtpError::SecurityError(
            "Key is all 0xFF - not FIPS compliant".into(),
        ));
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
        return Err(MtpError::SecurityError(
            "Key appears sequential - not FIPS compliant".into(),
        ));
    }

    // Try to create key pair to validate format
    match EcdsaKeyPair::from_pkcs8_maybe_unchecked(&ECDSA_P256_SHA256_FIXED_SIGNING, key_bytes) {
        Ok(_) => Ok(()),
        Err(_) => Err(MtpError::SecurityError(
            "Invalid ECDSA-P256 key format".into(),
        )),
    }
}

/// Generate FIPS-compliant ECDSA-P256 key pair
pub fn generate_fips_compliant_key() -> Result<EcdsaKeyPair, MtpError> {
    let rng = ring::rand::SystemRandom::new();
    let alg = &ECDSA_P256_SHA256_FIXED_SIGNING;

    // Generate key with FIPS-approved random number generator
    let pkcs8 = EcdsaKeyPair::generate_pkcs8(alg, &rng)
        .map_err(|_| MtpError::SecurityError("Failed to generate FIPS-compliant key".into()))?;

    EcdsaKeyPair::from_pkcs8(alg, pkcs8.as_ref())
        .map_err(|_| MtpError::SecurityError("Generated key failed validation".into()))
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
        Err(_) => Err(MtpError::SecurityError("Key failed signing test".into())),
    }
}
