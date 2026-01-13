use super::hash::fnv1a_64;
use super::Json;
use crate::errors::MtpError;
use std::cmp::Ordering;

/// Serialize Json to canonical JSON string (RFC 8785)
/// - Object keys sorted by §5 rules (type tag + FNV-1a hash + CBOR tie-break)
/// - Decimals in shortest form
/// - No -0, NaN, Infinity
/// - Deterministic output
pub fn serialize_canonical(json: &Json) -> Result<String, MtpError> {
    let mut output = String::new();
    serialize_value(json, &mut output)?;
    Ok(output)
}

/// Compare two string keys per §5 ordering rules:
/// 1. Type tag (all strings, so same)
/// 2. FNV-1a 64-bit hash of key
/// 3. CBOR byte-wise tie-break (for hash collisions)
fn compare_keys_section5(a: &str, b: &str) -> Ordering {
    // Step 1: Type tag - all keys are strings, so same type (skip)

    // Step 2: Compare by FNV-1a hash of the key
    let hash_a = fnv1a_64(a.as_bytes());
    let hash_b = fnv1a_64(b.as_bytes());

    match hash_a.cmp(&hash_b) {
        Ordering::Equal => {
            // Step 3: CBOR byte-wise tie-break
            // For strings, CBOR encoding is: length-prefixed UTF-8 bytes
            // Shorter strings come first, then lexicographic byte order
            let cbor_a = encode_cbor_string(a);
            let cbor_b = encode_cbor_string(b);
            cbor_a.cmp(&cbor_b)
        }
        other => other,
    }
}

/// Encode a string to CBOR format for comparison
/// (simplified version for key comparison only)
fn encode_cbor_string(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    let mut output = Vec::with_capacity(bytes.len() + 9);

    // CBOR text string encoding
    let len = bytes.len() as u64;
    if len <= 23 {
        output.push(0x60 | len as u8);
    } else if len <= 0xff {
        output.push(0x78);
        output.push(len as u8);
    } else if len <= 0xffff {
        output.push(0x79);
        output.extend_from_slice(&(len as u16).to_be_bytes());
    } else if len <= 0xffffffff {
        output.push(0x7a);
        output.extend_from_slice(&(len as u32).to_be_bytes());
    } else {
        output.push(0x7b);
        output.extend_from_slice(&len.to_be_bytes());
    }
    output.extend_from_slice(bytes);
    output
}

fn serialize_value(json: &Json, output: &mut String) -> Result<(), MtpError> {
    match json {
        Json::Null => output.push_str("null"),
        Json::Bool(b) => output.push_str(if *b { "true" } else { "false" }),
        Json::Int(n) => output.push_str(&n.to_string()),
        Json::Decimal(d) => output.push_str(&d.to_string()),
        Json::String(s) => {
            output.push('"');
            for c in s.chars() {
                match c {
                    '"' => output.push_str("\\\""),
                    '\\' => output.push_str("\\\\"),
                    '\x08' => output.push_str("\\b"),
                    '\x0c' => output.push_str("\\f"),
                    '\n' => output.push_str("\\n"),
                    '\r' => output.push_str("\\r"),
                    '\t' => output.push_str("\\t"),
                    c if c.is_control() => {
                        // Unicode escape for control chars
                        output.push_str(&format!("\\u{:04x}", c as u32));
                    }
                    c => output.push(c),
                }
            }
            output.push('"');
        }
        Json::Array(arr) => {
            output.push('[');
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    output.push(',');
                }
                serialize_value(item, output)?;
            }
            output.push(']');
        }
        Json::Object(obj) => {
            output.push('{');
            // Sort keys by §5: type tag (all strings) -> FNV-1a hash -> CBOR tie-break
            let mut keys: Vec<_> = obj.keys().collect();
            keys.sort_by(|a, b| compare_keys_section5(a, b));
            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    output.push(',');
                }
                // Key is always string
                serialize_value(&Json::String((*key).clone()), output)?;
                output.push(':');
                serialize_value(&obj[*key], output)?;
            }
            output.push('}');
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_null() {
        let json = Json::Null;
        assert_eq!(serialize_canonical(&json).unwrap(), "null");
    }

    #[test]
    fn test_canonical_bool() {
        assert_eq!(serialize_canonical(&Json::Bool(true)).unwrap(), "true");
        assert_eq!(serialize_canonical(&Json::Bool(false)).unwrap(), "false");
    }

    #[test]
    fn test_canonical_int() {
        assert_eq!(serialize_canonical(&Json::Int(42)).unwrap(), "42");
    }

    #[test]
    fn test_canonical_string() {
        assert_eq!(
            serialize_canonical(&Json::String("hello".to_string())).unwrap(),
            r#""hello""#
        );
        assert_eq!(
            serialize_canonical(&Json::String("he\"llo".to_string())).unwrap(),
            r#""he\"llo""#
        );
    }

    #[test]
    fn test_canonical_array() {
        let arr = Json::Array(vec![Json::Int(1), Json::Int(2)]);
        assert_eq!(serialize_canonical(&arr).unwrap(), "[1,2]");
    }

    #[test]
    fn test_canonical_object() {
        let mut obj = std::collections::HashMap::new();
        obj.insert("b".to_string(), Json::Int(2));
        obj.insert("a".to_string(), Json::Int(1));
        let json = Json::Object(obj);
        // Keys sorted by §5: FNV-1a hash ordering (deterministic)
        let result = serialize_canonical(&json).unwrap();
        // Verify it's valid JSON with both keys present
        assert!(result.contains("\"a\":1"));
        assert!(result.contains("\"b\":2"));
        // Verify determinism - same input produces same output
        assert_eq!(serialize_canonical(&json).unwrap(), result);
    }

    #[test]
    fn test_section5_key_ordering() {
        // Test that keys are sorted by FNV-1a hash, not alphabetically
        let hash_a = fnv1a_64("a".as_bytes());
        let hash_b = fnv1a_64("b".as_bytes());

        // The order depends on which hash is smaller
        let expected_order = if hash_a < hash_b { ("a", "b") } else { ("b", "a") };

        let mut obj = std::collections::HashMap::new();
        obj.insert("a".to_string(), Json::Int(1));
        obj.insert("b".to_string(), Json::Int(2));
        let json = Json::Object(obj);

        let result = serialize_canonical(&json).unwrap();

        // Verify the order matches FNV-1a hash order
        let first_key_pos = result.find(&format!("\"{}\":", expected_order.0)).unwrap();
        let second_key_pos = result.find(&format!("\"{}\":", expected_order.1)).unwrap();
        assert!(first_key_pos < second_key_pos, "Keys should be in FNV-1a hash order");
    }

    #[test]
    fn test_cbor_tiebreak() {
        // Test CBOR tie-break when hashes collide (unlikely but possible)
        // For this test, just verify that identical keys produce identical output
        let mut obj1 = std::collections::HashMap::new();
        obj1.insert("key1".to_string(), Json::Int(1));
        obj1.insert("key2".to_string(), Json::Int(2));

        let mut obj2 = std::collections::HashMap::new();
        obj2.insert("key2".to_string(), Json::Int(2));
        obj2.insert("key1".to_string(), Json::Int(1));

        let result1 = serialize_canonical(&Json::Object(obj1)).unwrap();
        let result2 = serialize_canonical(&Json::Object(obj2)).unwrap();

        // Same keys/values should produce identical canonical output regardless of insertion order
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_multi_run_determinism() {
        let mut obj = std::collections::HashMap::new();
        obj.insert("z".to_string(), Json::Int(3));
        obj.insert("a".to_string(), Json::Int(1));
        obj.insert(
            "b".to_string(),
            Json::Array(vec![Json::Bool(true), Json::Null]),
        );
        let json = Json::Object(obj);

        let expected = serialize_canonical(&json).unwrap();
        for _ in 0..100 {
            let canonical = serialize_canonical(&json).unwrap();
            assert_eq!(canonical, expected);
        }
    }
}
