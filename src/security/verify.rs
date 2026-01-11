use ring::signature::{UnparsedPublicKey, ECDSA_P256_SHA256_FIXED};

pub fn verify_ecdsa_p256(data: &[u8], signature: &[u8], public_key: &[u8]) -> bool {
    let key = UnparsedPublicKey::new(&ECDSA_P256_SHA256_FIXED, public_key);
    key.verify(data, signature).is_ok()
}
