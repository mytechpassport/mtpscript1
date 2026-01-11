use crate::errors::MtpError;
use ring::signature::{EcdsaKeyPair, KeyPair, ECDSA_P256_SHA256_FIXED_SIGNING};

/// Sign data with ECDSA-P256
pub fn sign_ecdsa_p256(data: &[u8], key_pair: &EcdsaKeyPair) -> Result<Vec<u8>, MtpError> {
    let rng = ring::rand::SystemRandom::new();
    key_pair
        .sign(&rng, data)
        .map(|sig| sig.as_ref().to_vec())
        .map_err(|_| MtpError::SecurityError {
            error: "SecurityError".to_string(),
            message: "Failed to sign data".to_string(),
        })
}

/// Verify ECDSA-P256 signature
pub fn verify_ecdsa_p256(data: &[u8], signature: &[u8], public_key: &[u8]) -> Result<(), MtpError> {
    let public_key =
        ring::signature::UnparsedPublicKey::new(&ECDSA_P256_SHA256_FIXED_SIGNING, public_key);

    public_key
        .verify(data, signature)
        .map_err(|_| MtpError::SecurityError {
            error: "SecurityError".to_string(),
            message: "Signature verification failed".to_string(),
        })
}

/// Generate a new ECDSA-P256 key pair
pub fn generate_ecdsa_keypair() -> Result<EcdsaKeyPair, MtpError> {
    let rng = ring::rand::SystemRandom::new();
    let pkcs8 =
        EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng).map_err(|_| {
            MtpError::SecurityError {
                error: "SecurityError".to_string(),
                message: "Failed to generate key pair".to_string(),
            }
        })?;

    EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8.as_ref()).map_err(|_| {
        MtpError::SecurityError {
            error: "SecurityError".to_string(),
            message: "Failed to parse generated key pair".to_string(),
        }
    })
}

/// Load ECDSA key pair from PKCS#8 bytes
pub fn load_ecdsa_keypair(pkcs8_bytes: &[u8]) -> Result<EcdsaKeyPair, MtpError> {
    EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8_bytes)
        .map_err(|_| MtpError::SecurityError("Failed to load key pair".into()))
}

/// Get public key bytes from key pair
pub fn get_public_key_bytes(key_pair: &EcdsaKeyPair) -> Vec<u8> {
    key_pair.public_key().as_ref().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation_and_signing() {
        let key_pair = generate_ecdsa_keypair().unwrap();
        let data = b"Hello, world!";

        let signature = sign_ecdsa_p256(data, &key_pair).unwrap();
        let public_key = get_public_key_bytes(&key_pair);

        // Verify the signature
        verify_ecdsa_p256(data, &signature, &public_key).unwrap();
    }

    #[test]
    fn test_signature_verification_failure() {
        let key_pair = generate_ecdsa_keypair().unwrap();
        let data = b"Hello, world!";
        let wrong_data = b"Goodbye, world!";

        let signature = sign_ecdsa_p256(data, &key_pair).unwrap();
        let public_key = get_public_key_bytes(&key_pair);

        // Should fail with wrong data
        assert!(verify_ecdsa_p256(wrong_data, &signature, &public_key).is_err());
    }
}
