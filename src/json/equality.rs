use crate::json::serialize::Json;
use crate::json::cbor::encode_cbor;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

impl PartialOrd for Json {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Json {
    fn cmp(&self, other: &Self) -> Ordering {
        // Per spec: only number and string are orderable
        // Other types compare by type tag first, then by value
        match (self, other) {
            // Numbers compare numerically
            (Json::Int(a), Json::Int(b)) => a.cmp(b),
            (Json::Decimal(a), Json::Decimal(b)) => {
                // Compare decimals as strings (lexicographically for same length, then by length)
                compare_decimal_strings(a, b)
            }
            (Json::Int(a), Json::Decimal(b)) => {
                compare_decimal_strings(&a.to_string(), b)
            }
            (Json::Decimal(a), Json::Int(b)) => {
                compare_decimal_strings(a, &b.to_string())
            }

            // Strings compare lexicographically
            (Json::String(a), Json::String(b)) => a.cmp(b),

            // Other types: compare by type tag, then by CBOR encoding
            _ => {
                let type_order_a = type_order(self);
                let type_order_b = type_order(other);
                if type_order_a != type_order_b {
                    type_order_a.cmp(&type_order_b)
                } else {
                    // Same type - compare by CBOR encoding
                    let cbor_a = encode_cbor(self);
                    let cbor_b = encode_cbor(other);
                    cbor_a.cmp(&cbor_b)
                }
            }
        }
    }
}

fn type_order(json: &Json) -> u8 {
    match json {
        Json::Null => 0,
        Json::Bool(_) => 1,
        Json::Int(_) => 2,
        Json::Decimal(_) => 2, // Same as Int for ordering purposes
        Json::String(_) => 3,
        Json::Array(_) => 4,
        Json::Object(_) => 5,
    }
}

/// Compare decimal strings properly handling sign, integer, and fractional parts
fn compare_decimal_strings(a: &str, b: &str) -> Ordering {
    // Parse into components for proper numeric comparison
    let a_neg = a.starts_with('-');
    let b_neg = b.starts_with('-');

    if a_neg != b_neg {
        return if a_neg { Ordering::Less } else { Ordering::Greater };
    }

    let a_trimmed = a.trim_start_matches('-');
    let b_trimmed = b.trim_start_matches('-');

    let (a_int, a_frac) = split_decimal(a_trimmed);
    let (b_int, b_frac) = split_decimal(b_trimmed);

    // Compare integer parts by length first, then lexicographically
    let int_cmp = if a_int.len() != b_int.len() {
        a_int.len().cmp(&b_int.len())
    } else {
        a_int.cmp(&b_int)
    };

    let result = if int_cmp != Ordering::Equal {
        int_cmp
    } else {
        // Compare fractional parts (pad with zeros)
        let max_len = a_frac.len().max(b_frac.len());
        let a_padded = format!("{:0<width$}", a_frac, width = max_len);
        let b_padded = format!("{:0<width$}", b_frac, width = max_len);
        a_padded.cmp(&b_padded)
    };

    // Reverse for negative numbers
    if a_neg {
        result.reverse()
    } else {
        result
    }
}

fn split_decimal(s: &str) -> (&str, &str) {
    if let Some(pos) = s.find('.') {
        (&s[..pos], &s[pos + 1..])
    } else {
        (s, "")
    }
}

impl Hash for Json {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash using CBOR encoding for consistency with equality
        let cbor = encode_cbor(self);
        cbor.hash(state);
    }
}

/// FNV-1a 64-bit hash of deterministic CBOR encoding
/// This is the canonical hash function per the spec
pub fn hash_json(json: &Json) -> u64 {
    fnv1a_64(&encode_cbor(json))
}

/// FNV-1a 32-bit hash
pub fn fnv1a_32(data: &[u8]) -> u32 {
    const FNV_OFFSET_32: u32 = 0x811c9dc5;
    const FNV_PRIME_32: u32 = 0x01000193;

    let mut hash = FNV_OFFSET_32;
    for byte in data {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(FNV_PRIME_32);
    }
    hash
}

/// FNV-1a 64-bit hash
pub fn fnv1a_64(data: &[u8]) -> u64 {
    const FNV_OFFSET_64: u64 = 0xcbf29ce484222325;
    const FNV_PRIME_64: u64 = 0x00000100000001B3;

    let mut hash = FNV_OFFSET_64;
    for byte in data {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME_64);
    }
    hash
}

/// Hash a string using FNV-1a 64-bit (for the fnv1a64 builtin)
pub fn fnv1a_64_string(s: &str) -> u64 {
    fnv1a_64(s.as_bytes())
}

/// Hash a string using FNV-1a 32-bit (for the fnv1a32 builtin)
pub fn fnv1a_32_string(s: &str) -> u32 {
    fnv1a_32(s.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_json_equality() {
        let a = Json::Object([("x".to_string(), Json::Int(1))].into_iter().collect());
        let b = Json::Object([("x".to_string(), Json::Int(1))].into_iter().collect());
        assert_eq!(a, b);
        assert_eq!(hash_json(&a), hash_json(&b));
    }

    #[test]
    fn test_fnv1a_64_known_value() {
        // Test against known FNV-1a value for "hello"
        let hash = fnv1a_64_string("hello");
        assert_eq!(hash, 0xa430d84680aabd0b);
    }

    #[test]
    fn test_number_ordering() {
        assert!(Json::Int(1) < Json::Int(2));
        assert!(Json::Int(-1) < Json::Int(0));
    }

    #[test]
    fn test_string_ordering() {
        assert!(Json::String("a".to_string()) < Json::String("b".to_string()));
        assert!(Json::String("aa".to_string()) > Json::String("a".to_string()));
    }

    #[test]
    fn test_decimal_comparison() {
        assert_eq!(
            compare_decimal_strings("1.5", "1.50"),
            Ordering::Equal
        );
        assert_eq!(
            compare_decimal_strings("1.5", "1.6"),
            Ordering::Less
        );
    }
}
