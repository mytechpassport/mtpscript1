use super::Json;
use crate::errors::MtpError;

/// Deterministic CBOR encoding (RFC 7049 §3.9)
pub fn encode_cbor(json: &Json) -> Result<Vec<u8>, MtpError> {
    let mut output = Vec::new();
    encode_value(json, &mut output)?;
    Ok(output)
}

fn encode_value(json: &Json, output: &mut Vec<u8>) -> Result<(), MtpError> {
    match json {
        Json::Null => output.push(0xf6),        // null
        Json::Bool(true) => output.push(0xf5),  // true
        Json::Bool(false) => output.push(0xf4), // false
        Json::Int(n) => {
            if *n >= 0 {
                encode_uint(*n as u64, output);
            } else {
                encode_nint((-*n - 1) as u64, output);
            }
        }
        Json::Decimal(d) => {
            // Encode as string for determinism
            let s = d.to_string();
            encode_text_string(&s, output);
        }
        Json::String(s) => encode_text_string(s, output),
        Json::Array(arr) => {
            encode_array_header(arr.len() as u64, output);
            for item in arr {
                encode_value(item, output)?;
            }
        }
        Json::Object(obj) => {
            // Sort keys deterministically (UTF-8 byte order)
            let mut keys: Vec<_> = obj.keys().collect();
            keys.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
            encode_map_header(keys.len() as u64, output);
            for key in keys {
                encode_text_string(key, output);
                encode_value(&obj[key], output)?;
            }
        }
    }
    Ok(())
}

fn encode_uint(n: u64, output: &mut Vec<u8>) -> usize {
    let start = output.len();
    if n <= 23 {
        output.push(0x00 | n as u8);
    } else if n <= 0xff {
        output.push(0x18);
        output.push(n as u8);
    } else if n <= 0xffff {
        output.push(0x19);
        output.extend_from_slice(&(n as u16).to_be_bytes());
    } else if n <= 0xffffffff {
        output.push(0x1a);
        output.extend_from_slice(&(n as u32).to_be_bytes());
    } else {
        output.push(0x1b);
        output.extend_from_slice(&n.to_be_bytes());
    }
    output.len() - start
}

fn encode_nint(n: u64, output: &mut Vec<u8>) -> usize {
    let start = output.len();
    if n <= 23 {
        output.push(0x20 | n as u8);
    } else if n <= 0xff {
        output.push(0x38);
        output.push(n as u8);
    } else if n <= 0xffff {
        output.push(0x39);
        output.extend_from_slice(&(n as u16).to_be_bytes());
    } else if n <= 0xffffffff {
        output.push(0x3a);
        output.extend_from_slice(&(n as u32).to_be_bytes());
    } else {
        output.push(0x3b);
        output.extend_from_slice(&n.to_be_bytes());
    }
    output.len() - start
}

fn encode_text_string(s: &str, output: &mut Vec<u8>) {
    let bytes = s.as_bytes();
    let header_len = encode_uint(bytes.len() as u64, output);
    let header_index = output.len() - header_len;
    output.extend_from_slice(bytes);
    // Adjust major type
    output[header_index] |= 0x60;
}

fn encode_array_header(len: u64, output: &mut Vec<u8>) {
    let header_len = encode_uint(len, output);
    let header_index = output.len() - header_len;
    output[header_index] |= 0x80;
}

fn encode_map_header(len: u64, output: &mut Vec<u8>) {
    let header_len = encode_uint(len, output);
    let header_index = output.len() - header_len;
    output[header_index] |= 0xa0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_cbor_null() {
        let json = Json::Null;
        let cbor = encode_cbor(&json).unwrap();
        assert_eq!(cbor, vec![0xf6]);
    }

    #[test]
    fn test_cbor_bool() {
        assert_eq!(encode_cbor(&Json::Bool(true)).unwrap(), vec![0xf5]);
        assert_eq!(encode_cbor(&Json::Bool(false)).unwrap(), vec![0xf4]);
    }

    #[test]
    fn test_cbor_int() {
        assert_eq!(encode_cbor(&Json::Int(42)).unwrap(), vec![0x18, 42]);
        assert_eq!(encode_cbor(&Json::Int(-1)).unwrap(), vec![0x20]);
    }

    #[test]
    fn test_cbor_string() {
        let json = Json::String("hello".to_string());
        let cbor = encode_cbor(&json).unwrap();
        assert_eq!(cbor, vec![0x65, b'h', b'e', b'l', b'l', b'o']);
    }

    #[test]
    fn test_cbor_array() {
        let json = Json::Array(vec![Json::Int(1), Json::Int(2)]);
        let cbor = encode_cbor(&json).unwrap();
        assert_eq!(cbor, vec![0x82, 0x01, 0x02]); // array of 2 elements: 1, 2
    }

    #[test]
    fn test_cbor_object() {
        use std::collections::HashMap;
        let mut obj = HashMap::new();
        obj.insert("b".to_string(), Json::Int(2));
        obj.insert("a".to_string(), Json::Int(1));
        let json = Json::Object(obj);
        let cbor = encode_cbor(&json).unwrap();
        // Map with 2 pairs, keys sorted: "a" -> 1, "b" -> 2
        assert_eq!(cbor, vec![0xa2, 0x61, b'a', 0x01, 0x61, b'b', 0x02]);
    }

    // Comprehensive CBOR encoder validation tests (#26)

    #[test]
    fn test_cbor_small_integers() {
        // RFC 7049: integers 0-23 encoded in single byte
        for i in 0..=23 {
            let cbor = encode_cbor(&Json::Int(i)).unwrap();
            assert_eq!(cbor.len(), 1, "Integer {} should encode in 1 byte", i);
            assert_eq!(cbor[0], i as u8, "Integer {} has wrong encoding", i);
        }
    }

    #[test]
    fn test_cbor_boundary_integers() {
        // Test encoding boundaries
        // 24 requires 2 bytes (0x18 prefix)
        assert_eq!(encode_cbor(&Json::Int(24)).unwrap(), vec![0x18, 24]);

        // 255 (max 1-byte additional)
        assert_eq!(encode_cbor(&Json::Int(255)).unwrap(), vec![0x18, 255]);

        // 256 requires 3 bytes (0x19 prefix + 2 bytes)
        assert_eq!(encode_cbor(&Json::Int(256)).unwrap(), vec![0x19, 0x01, 0x00]);

        // 65535 (max 2-byte additional)
        assert_eq!(
            encode_cbor(&Json::Int(65535)).unwrap(),
            vec![0x19, 0xff, 0xff]
        );

        // 65536 requires 5 bytes
        assert_eq!(
            encode_cbor(&Json::Int(65536)).unwrap(),
            vec![0x1a, 0x00, 0x01, 0x00, 0x00]
        );

        // Max 32-bit value
        assert_eq!(
            encode_cbor(&Json::Int(u32::MAX as i64)).unwrap(),
            vec![0x1a, 0xff, 0xff, 0xff, 0xff]
        );

        // Requires 64-bit encoding
        assert_eq!(
            encode_cbor(&Json::Int(u32::MAX as i64 + 1)).unwrap(),
            vec![0x1b, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00]
        );
    }

    #[test]
    fn test_cbor_negative_integers() {
        // -1 encoded as 0x20
        assert_eq!(encode_cbor(&Json::Int(-1)).unwrap(), vec![0x20]);

        // -24 still fits in single byte (0x20 | 23 = 0x37)
        assert_eq!(encode_cbor(&Json::Int(-24)).unwrap(), vec![0x37]);

        // -25 requires additional byte
        assert_eq!(encode_cbor(&Json::Int(-25)).unwrap(), vec![0x38, 24]);

        // -256
        assert_eq!(encode_cbor(&Json::Int(-256)).unwrap(), vec![0x38, 255]);

        // -257 requires 2 additional bytes
        assert_eq!(encode_cbor(&Json::Int(-257)).unwrap(), vec![0x39, 0x01, 0x00]);

        // Large negative
        assert_eq!(
            encode_cbor(&Json::Int(-65537)).unwrap(),
            vec![0x3a, 0x00, 0x01, 0x00, 0x00]
        );
    }

    #[test]
    fn test_cbor_zero() {
        let cbor = encode_cbor(&Json::Int(0)).unwrap();
        assert_eq!(cbor, vec![0x00]);
    }

    #[test]
    fn test_cbor_empty_string() {
        let cbor = encode_cbor(&Json::String(String::new())).unwrap();
        assert_eq!(cbor, vec![0x60]); // Text string of length 0
    }

    #[test]
    fn test_cbor_unicode_string() {
        // Test UTF-8 encoding is preserved
        let json = Json::String("日本語".to_string());
        let cbor = encode_cbor(&json).unwrap();

        // "日本語" is 9 bytes in UTF-8
        assert_eq!(cbor[0], 0x69); // text string of length 9
        assert_eq!(cbor.len(), 10); // header + 9 bytes
    }

    #[test]
    fn test_cbor_long_string() {
        // String longer than 23 bytes needs 2-byte header
        let long_str = "x".repeat(30);
        let json = Json::String(long_str);
        let cbor = encode_cbor(&json).unwrap();

        assert_eq!(cbor[0], 0x78); // text string with 1-byte length
        assert_eq!(cbor[1], 30); // length = 30
        assert_eq!(cbor.len(), 32); // 2-byte header + 30 bytes
    }

    #[test]
    fn test_cbor_empty_array() {
        let json = Json::Array(vec![]);
        let cbor = encode_cbor(&json).unwrap();
        assert_eq!(cbor, vec![0x80]); // Array of length 0
    }

    #[test]
    fn test_cbor_empty_object() {
        let json = Json::Object(std::collections::HashMap::new());
        let cbor = encode_cbor(&json).unwrap();
        assert_eq!(cbor, vec![0xa0]); // Map of length 0
    }

    #[test]
    fn test_cbor_nested_structure() {
        use std::collections::HashMap;
        let mut inner_obj = HashMap::new();
        inner_obj.insert("x".to_string(), Json::Int(1));

        let json = Json::Array(vec![
            Json::Object(inner_obj),
            Json::Array(vec![Json::Bool(true), Json::Null]),
        ]);

        let cbor = encode_cbor(&json).unwrap();

        // Verify structure is valid
        assert!(cbor.len() > 0);
        assert_eq!(cbor[0], 0x82); // Array of 2 elements
    }

    #[test]
    fn test_cbor_decimal_as_string() {
        // Decimals are encoded as strings for determinism
        let decimal = Decimal::from_str("123.456").unwrap();
        let json = Json::Decimal(decimal);
        let cbor = encode_cbor(&json).unwrap();

        // Should be a text string
        assert!(cbor[0] & 0xe0 == 0x60, "Decimal should encode as text string");
    }

    #[test]
    fn test_cbor_determinism_map_key_order() {
        use std::collections::HashMap;

        // Insert keys in different orders
        let mut obj1 = HashMap::new();
        obj1.insert("zebra".to_string(), Json::Int(1));
        obj1.insert("alpha".to_string(), Json::Int(2));
        obj1.insert("beta".to_string(), Json::Int(3));

        let mut obj2 = HashMap::new();
        obj2.insert("alpha".to_string(), Json::Int(2));
        obj2.insert("beta".to_string(), Json::Int(3));
        obj2.insert("zebra".to_string(), Json::Int(1));

        let cbor1 = encode_cbor(&Json::Object(obj1)).unwrap();
        let cbor2 = encode_cbor(&Json::Object(obj2)).unwrap();

        // Should produce identical output
        assert_eq!(cbor1, cbor2, "Map key order must be deterministic");

        // Verify keys are sorted alphabetically (UTF-8 byte order)
        // First key should be "alpha"
        let expected_alpha_pos = cbor1.iter().position(|&b| b == b'a').unwrap();
        let expected_beta_pos = cbor1.iter().position(|&b| b == b'b').unwrap();
        let expected_zebra_pos = cbor1.iter().position(|&b| b == b'z').unwrap();

        assert!(
            expected_alpha_pos < expected_beta_pos,
            "alpha should come before beta"
        );
        assert!(
            expected_beta_pos < expected_zebra_pos,
            "beta should come before zebra"
        );
    }

    #[test]
    fn test_cbor_determinism_multiple_runs() {
        use std::collections::HashMap;

        let mut obj = HashMap::new();
        obj.insert("key1".to_string(), Json::Int(100));
        obj.insert("key2".to_string(), Json::String("value".to_string()));
        obj.insert("key3".to_string(), Json::Array(vec![Json::Bool(true)]));

        let json = Json::Object(obj);
        let first_result = encode_cbor(&json).unwrap();

        // Run 100 times
        for _ in 0..100 {
            let result = encode_cbor(&json).unwrap();
            assert_eq!(result, first_result, "CBOR encoding must be deterministic");
        }
    }

    #[test]
    fn test_cbor_large_array() {
        // Test array with > 23 elements
        let items: Vec<Json> = (0..30).map(Json::Int).collect();
        let json = Json::Array(items);
        let cbor = encode_cbor(&json).unwrap();

        // Array header should have 1-byte length (0x98 prefix)
        assert_eq!(cbor[0], 0x98); // array with 1-byte length
        assert_eq!(cbor[1], 30); // 30 elements
    }

    #[test]
    fn test_cbor_all_types_combined() {
        use std::collections::HashMap;

        let decimal = Decimal::from_str("3.14").unwrap();
        let mut obj = HashMap::new();
        obj.insert("null".to_string(), Json::Null);
        obj.insert("bool".to_string(), Json::Bool(true));
        obj.insert("int".to_string(), Json::Int(42));
        obj.insert("decimal".to_string(), Json::Decimal(decimal));
        obj.insert("string".to_string(), Json::String("test".to_string()));
        obj.insert(
            "array".to_string(),
            Json::Array(vec![Json::Int(1), Json::Int(2)]),
        );

        let json = Json::Object(obj);
        let cbor = encode_cbor(&json).unwrap();

        // Verify it encodes successfully
        assert!(cbor.len() > 0);

        // Map header
        assert_eq!(cbor[0] & 0xe0, 0xa0); // Major type 5 (map)
    }

    #[test]
    fn test_cbor_special_string_characters() {
        // Test strings with special characters
        let test_cases = vec![
            "hello\nworld", // newline
            "tab\there",    // tab
            "quote\"test",  // quote
            "back\\slash",  // backslash
            "🎉🎊🎈",       // emoji
            "\x00\x01\x02", // control characters
        ];

        for s in test_cases {
            let json = Json::String(s.to_string());
            let cbor = encode_cbor(&json).unwrap();

            // Should encode successfully
            assert!(cbor.len() > 0);
            // First byte should indicate text string
            assert!(cbor[0] & 0xe0 == 0x60, "Should be text string type");
        }
    }

    #[test]
    fn test_cbor_deeply_nested() {
        // Create 10 levels of nesting
        let mut value = Json::Int(42);
        for _ in 0..10 {
            value = Json::Array(vec![value]);
        }

        let cbor = encode_cbor(&value).unwrap();
        assert!(cbor.len() > 10); // At least 10 array headers + value
    }

    #[test]
    fn test_cbor_max_i64() {
        let json = Json::Int(i64::MAX);
        let cbor = encode_cbor(&json).unwrap();

        // Should be 9 bytes: 1 header + 8 bytes
        assert_eq!(cbor.len(), 9);
        assert_eq!(cbor[0], 0x1b); // 64-bit unsigned integer
    }

    #[test]
    fn test_cbor_min_safe_i64() {
        // Test minimum negative that doesn't overflow
        let json = Json::Int(-9223372036854775807); // i64::MIN + 1
        let cbor = encode_cbor(&json).unwrap();

        // Should encode as 64-bit negative
        assert_eq!(cbor.len(), 9);
        assert_eq!(cbor[0], 0x3b); // 64-bit negative integer
    }

    #[test]
    fn test_cbor_string_length_boundaries() {
        // Test length encoding boundaries

        // 23 bytes - fits in single byte header
        let s23 = "x".repeat(23);
        let cbor23 = encode_cbor(&Json::String(s23)).unwrap();
        assert_eq!(cbor23[0], 0x77); // 0x60 | 23 = 0x77

        // 24 bytes - needs 2-byte header
        let s24 = "x".repeat(24);
        let cbor24 = encode_cbor(&Json::String(s24)).unwrap();
        assert_eq!(cbor24[0], 0x78); // text string with 1-byte length

        // 255 bytes
        let s255 = "x".repeat(255);
        let cbor255 = encode_cbor(&Json::String(s255)).unwrap();
        assert_eq!(cbor255[0], 0x78);
        assert_eq!(cbor255[1], 255);

        // 256 bytes - needs 3-byte header
        let s256 = "x".repeat(256);
        let cbor256 = encode_cbor(&Json::String(s256)).unwrap();
        assert_eq!(cbor256[0], 0x79); // text string with 2-byte length
    }

    #[test]
    fn test_cbor_output_size_reasonable() {
        // Verify encoding doesn't produce unreasonably large output
        use std::collections::HashMap;

        let mut obj = HashMap::new();
        for i in 0..100 {
            obj.insert(format!("k{}", i), Json::Int(i));
        }

        let json = Json::Object(obj);
        let cbor = encode_cbor(&json).unwrap();

        // Rough estimate: 100 keys * ~5 bytes average = ~500 bytes
        // Should be less than 1KB for 100 small entries
        assert!(
            cbor.len() < 1024,
            "Output size {} seems too large",
            cbor.len()
        );
    }
}
