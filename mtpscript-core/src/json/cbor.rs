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
}
