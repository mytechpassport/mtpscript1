use super::{cbor, Json};
use crate::errors::MtpError;
use std::hash::{Hash, Hasher};

/// FNV-1a 64-bit hash parameters
const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

/// Compute FNV-1a 64-bit hash of the given data
pub fn fnv1a_64(data: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Hash Json using FNV-1a 64-bit of deterministic CBOR
pub fn hash_json(json: &Json) -> Result<u64, MtpError> {
    let cbor_bytes = cbor::encode_cbor(json)?;
    Ok(fnv1a_64(&cbor_bytes))
}

impl Hash for Json {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the CBOR bytes using FNV-1a, but since Hasher is generic,
        // we compute the FNV hash and write it as u64
        if let Ok(cbor_bytes) = cbor::encode_cbor(self) {
            let hash_value = fnv1a_64(&cbor_bytes);
            state.write_u64(hash_value);
        } else {
            // If CBOR encoding fails (shouldn't happen), hash the debug string
            self.to_string().hash(state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn test_fnv1a_64() {
        // Test with empty string
        assert_eq!(fnv1a_64(&[]), FNV_OFFSET_BASIS);

        // Test with "hello"
        let hello = b"hello";
        let expected = 0xa430d84680aabd0b; // Known FNV-1a hash for "hello"
        assert_eq!(fnv1a_64(hello), expected);
    }

    #[test]
    fn test_hash_json_null() {
        let json = Json::Null;
        let hash = hash_json(&json).unwrap();
        assert_eq!(hash, fnv1a_64(&[0xf6])); // CBOR for null
    }

    #[test]
    fn test_hash_json_int() {
        let json = Json::Int(42);
        let hash = hash_json(&json).unwrap();
        let cbor = cbor::encode_cbor(&json).unwrap();
        assert_eq!(hash, fnv1a_64(&cbor));
    }

    #[test]
    fn test_hash_consistency() {
        let json1 = Json::Object(
            vec![("key".to_string(), Json::Int(1))]
                .into_iter()
                .collect(),
        );
        let json2 = Json::Object(
            vec![("key".to_string(), Json::Int(1))]
                .into_iter()
                .collect(),
        );

        let hash1 = hash_json(&json1).unwrap();
        let hash2 = hash_json(&json2).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_different() {
        let json1 = Json::Int(1);
        let json2 = Json::Int(2);

        let hash1 = hash_json(&json1).unwrap();
        let hash2 = hash_json(&json2).unwrap();

        assert_ne!(hash1, hash2);
    }
}
