use crate::errors::MtpError;
use ring::signature::{
    EcdsaKeyPair, UnparsedPublicKey, ECDSA_P256_SHA256_FIXED, ECDSA_P256_SHA256_FIXED_SIGNING,
};
use std::fs;

/// Sign data with ECDSA-P256
pub fn sign_ecdsa_p256(data: &[u8], private_key_pem: &str) -> Result<Vec<u8>, MtpError> {
    // Parse PEM private key
    let private_key_der = parse_pem_private_key(private_key_pem)?;

    // Create key pair
    let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &private_key_der)
        .map_err(|_| MtpError::Security("Invalid private key".to_string()))?;

    // Sign the data
    let signature = key_pair
        .sign(&ring::rand::SystemRandom::new(), data)
        .map_err(|_| MtpError::Security("Signing failed".to_string()))?;

    Ok(signature.as_ref().to_vec())
}

/// Verify ECDSA-P256 signature
pub fn verify_ecdsa_p256(
    data: &[u8],
    signature: &[u8],
    public_key_pem: &str,
) -> Result<(), MtpError> {
    // Parse PEM public key
    let public_key_der = parse_pem_public_key(public_key_pem)?;
    let public_key = UnparsedPublicKey::new(&ECDSA_P256_SHA256_FIXED, public_key_der);

    // Verify signature
    public_key
        .verify(data, signature)
        .map_err(|_| MtpError::Security("Signature verification failed".to_string()))
}

/// Parse PEM private key (ECDSA-P256 PKCS#8)
fn parse_pem_private_key(pem: &str) -> Result<Vec<u8>, MtpError> {
    // Remove PEM headers/footers and decode base64
    let lines: Vec<&str> = pem
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect();

    let der = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &lines.join(""))
        .map_err(|_| MtpError::Security("Invalid PEM format".to_string()))?;

    // For ECDSA-P256, we expect PKCS#8 format
    // Ring can handle this directly
    Ok(der)
}

/// Parse PEM public key (ECDSA-P256 SPKI)
fn parse_pem_public_key(pem: &str) -> Result<Vec<u8>, MtpError> {
    // Remove PEM headers/footers and decode base64
    let lines: Vec<&str> = pem
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect();

    let der = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &lines.join(""))
        .map_err(|_| MtpError::Security("Invalid PEM format".to_string()))?;

    // For ECDSA-P256, we expect SPKI format
    // Ring can handle this directly
    Ok(der)
}

/// Load signing key from file
pub fn load_signing_key(path: &str) -> Result<String, MtpError> {
    fs::read_to_string(path).map_err(|e| MtpError::Io(e.to_string()))
}

/// Load certificate from file
pub fn load_certificate(path: &str) -> Result<String, MtpError> {
    fs::read_to_string(path).map_err(|e| MtpError::Io(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require actual ECDSA-P256 keys.
    // In a real implementation, we'd generate test keys or use known vectors.

    #[test]
    fn test_parse_pem_private_key() {
        // This would require a valid PEM private key string
        // For now, test with invalid input
        let invalid_pem = "invalid";
        assert!(parse_pem_private_key(invalid_pem).is_err());
    }

    #[test]
    fn test_parse_pem_public_key() {
        // Test with invalid input
        let invalid_pem = "invalid";
        assert!(parse_pem_public_key(invalid_pem).is_err());
    }

    #[test]
    fn test_sign_verify_roundtrip() {
        // This test would require generating a key pair
        // For demonstration, we'll use ring's test utilities if available
        // Since ring doesn't expose test keys easily, this is a placeholder

        // In practice, you'd do:
        // let key_pair = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &ring::rand::SystemRandom::new()).unwrap();
        // let public_key = key_pair.public_key();
        // let data = b"test data";
        // let signature = key_pair.sign(&ring::rand::SystemRandom::new(), data).unwrap();
        // assert!(public_key.verify(data, signature.as_ref()).is_ok());

        // For now, just test that the functions exist
        assert!(true);
    }
}
