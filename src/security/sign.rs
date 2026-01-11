use ring::signature::{EcdsaKeyPair, ECDSA_P256_SHA256_FIXED_SIGNING};

pub fn sign_ecdsa_p256(data: &[u8], key_pair: &EcdsaKeyPair) -> Vec<u8> {
    let rng = ring::rand::SystemRandom::new();
    key_pair.sign(&rng, data).unwrap().as_ref().to_vec()
}
