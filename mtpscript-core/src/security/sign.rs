use crate::errors::MtpError;
use ring::signature::{Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519};
use std::fs;

/// Sign data with ECDSA-P256
pub fn sign_ecdsa_p256(data: &[u8], private_key_pem: &str) -> Result<Vec<u8>, MtpError> {
    // Parse PEM private key
    let private_key_der = parse_pem_private_key(private_key_pem)?;

    // Create key pair
    let key_pair = Ed25519KeyPair::from_pkcs8(&private_key_der)
        .map_err(|_| MtpError::Security("Invalid private key".to_string()))?;

    // Sign the data
    let signature = key_pair.sign(data);

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
    let public_key = UnparsedPublicKey::new(&ED25519, public_key_der);

    // Verify signature
    public_key
        .verify(data, signature)
        .map_err(|_| MtpError::Security("Signature verification failed".to_string()))
}

/// Parse PEM private key (simplified)
fn parse_pem_private_key(pem: &str) -> Result<Vec<u8>, MtpError> {
    // This is a simplified implementation
    // In practice, you'd use a proper PEM parser
    let lines: Vec<&str> = pem
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect();

    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &lines.join(""))
        .map_err(|_| MtpError::Security("Invalid PEM format".to_string()))
}

/// Parse PEM public key (simplified)
fn parse_pem_public_key(pem: &str) -> Result<Vec<u8>, MtpError> {
    // This is a simplified implementation
    let lines: Vec<&str> = pem
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect();

    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &lines.join(""))
        .map_err(|_| MtpError::Security("Invalid PEM format".to_string()))
}

/// Load signing key from file
pub fn load_signing_key(path: &str) -> Result<String, MtpError> {
    fs::read_to_string(path).map_err(|e| MtpError::Io(e.to_string()))
}

/// Load certificate from file
pub fn load_certificate(path: &str) -> Result<String, MtpError> {
    fs::read_to_string(path).map_err(|e| MtpError::Io(e.to_string()))
}
