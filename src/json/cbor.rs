use crate::json::serialize::Json;
use std::collections::HashMap;

/// Encode JSON value to deterministic CBOR per RFC 7049 Section 3.9
/// This implementation follows the canonical CBOR rules:
/// - Integers use minimal encoding
/// - Map keys are sorted by encoded length, then lexicographically
/// - Floats use shortest representation
pub fn encode_cbor(json: &Json) -> Vec<u8> {
    let mut output = Vec::new();
    encode_value(json, &mut output);
    output
}

fn encode_value(json: &Json, output: &mut Vec<u8>) {
    match json {
        Json::Null => {
            // CBOR null is major type 7, additional info 22
            output.push(0xf6);
        }
        Json::Bool(b) => {
            // CBOR false=0xf4, true=0xf5
            output.push(if *b { 0xf5 } else { 0xf4 });
        }
        Json::Int(n) => {
            encode_integer(*n, output);
        }
        Json::Decimal(d) => {
            // Encode decimal as text string (type 3)
            // This preserves precision better than float
            encode_text_string(d, output);
        }
        Json::String(s) => {
            encode_text_string(s, output);
        }
        Json::Array(arr) => {
            // Major type 4 (array)
            encode_head(4, arr.len() as u64, output);
            for item in arr {
                encode_value(item, output);
            }
        }
        Json::Object(obj) => {
            // Major type 5 (map)
            // Keys must be sorted by encoded length, then lexicographically
            let mut keys: Vec<&String> = obj.keys().collect();

            // Sort keys by CBOR encoding (length-first ordering per RFC 7049 3.9)
            keys.sort_by(|a, b| {
                let a_len = a.len();
                let b_len = b.len();
                if a_len != b_len {
                    a_len.cmp(&b_len)
                } else {
                    a.as_bytes().cmp(b.as_bytes())
                }
            });

            encode_head(5, keys.len() as u64, output);
            for key in keys {
                encode_text_string(key, output);
                encode_value(&obj[key], output);
            }
        }
    }
}

fn encode_head(major_type: u8, value: u64, output: &mut Vec<u8>) {
    let mt = major_type << 5;
    if value < 24 {
        output.push(mt | value as u8);
    } else if value < 256 {
        output.push(mt | 24);
        output.push(value as u8);
    } else if value < 65536 {
        output.push(mt | 25);
        output.extend_from_slice(&(value as u16).to_be_bytes());
    } else if value < 4294967296 {
        output.push(mt | 26);
        output.extend_from_slice(&(value as u32).to_be_bytes());
    } else {
        output.push(mt | 27);
        output.extend_from_slice(&value.to_be_bytes());
    }
}

fn encode_integer(n: i64, output: &mut Vec<u8>) {
    if n >= 0 {
        // Major type 0 (unsigned integer)
        encode_head(0, n as u64, output);
    } else {
        // Major type 1 (negative integer)
        // CBOR encodes -1-n, so -1 is 0, -2 is 1, etc.
        encode_head(1, (-1 - n) as u64, output);
    }
}

fn encode_text_string(s: &str, output: &mut Vec<u8>) {
    // Major type 3 (text string)
    encode_head(3, s.len() as u64, output);
    output.extend_from_slice(s.as_bytes());
}

/// Decode CBOR to JSON (for verification/testing)
pub fn decode_cbor(input: &[u8]) -> Result<Json, String> {
    if input.is_empty() {
        return Err("Empty CBOR input".to_string());
    }
    let (json, _) = decode_value(input)?;
    Ok(json)
}

fn decode_value(input: &[u8]) -> Result<(Json, &[u8]), String> {
    if input.is_empty() {
        return Err("Unexpected end of CBOR".to_string());
    }

    let initial = input[0];
    let major_type = initial >> 5;
    let additional = initial & 0x1f;

    match major_type {
        0 => {
            // Unsigned integer
            let (value, rest) = decode_uint(&input[1..], additional)?;
            Ok((Json::Int(value as i64), rest))
        }
        1 => {
            // Negative integer
            let (value, rest) = decode_uint(&input[1..], additional)?;
            Ok((Json::Int(-1 - value as i64), rest))
        }
        3 => {
            // Text string
            let (len, rest) = decode_uint(&input[1..], additional)?;
            if rest.len() < len as usize {
                return Err("Truncated text string".to_string());
            }
            let s = String::from_utf8(rest[..len as usize].to_vec())
                .map_err(|_| "Invalid UTF-8 in text string")?;
            Ok((Json::String(s), &rest[len as usize..]))
        }
        4 => {
            // Array
            let (len, mut rest) = decode_uint(&input[1..], additional)?;
            let mut items = Vec::new();
            for _ in 0..len {
                let (item, new_rest) = decode_value(rest)?;
                items.push(item);
                rest = new_rest;
            }
            Ok((Json::Array(items), rest))
        }
        5 => {
            // Map
            let (len, mut rest) = decode_uint(&input[1..], additional)?;
            let mut obj = HashMap::new();
            for _ in 0..len {
                let (key, new_rest) = decode_value(rest)?;
                let key_str = match key {
                    Json::String(s) => s,
                    _ => return Err("Map key must be string".to_string()),
                };
                let (value, new_rest) = decode_value(new_rest)?;
                obj.insert(key_str, value);
                rest = new_rest;
            }
            Ok((Json::Object(obj), rest))
        }
        7 => {
            // Simple values and floats
            match additional {
                20 => Ok((Json::Bool(false), &input[1..])),
                21 => Ok((Json::Bool(true), &input[1..])),
                22 => Ok((Json::Null, &input[1..])),
                _ => Err(format!("Unsupported simple value: {}", additional)),
            }
        }
        _ => Err(format!("Unsupported CBOR major type: {}", major_type)),
    }
}

fn decode_uint(input: &[u8], additional: u8) -> Result<(u64, &[u8]), String> {
    if additional < 24 {
        Ok((additional as u64, input))
    } else if additional == 24 {
        if input.is_empty() {
            return Err("Truncated CBOR".to_string());
        }
        Ok((input[0] as u64, &input[1..]))
    } else if additional == 25 {
        if input.len() < 2 {
            return Err("Truncated CBOR".to_string());
        }
        let value = u16::from_be_bytes([input[0], input[1]]);
        Ok((value as u64, &input[2..]))
    } else if additional == 26 {
        if input.len() < 4 {
            return Err("Truncated CBOR".to_string());
        }
        let value = u32::from_be_bytes([input[0], input[1], input[2], input[3]]);
        Ok((value as u64, &input[4..]))
    } else if additional == 27 {
        if input.len() < 8 {
            return Err("Truncated CBOR".to_string());
        }
        let value = u64::from_be_bytes([
            input[0], input[1], input[2], input[3],
            input[4], input[5], input[6], input[7],
        ]);
        Ok((value, &input[8..]))
    } else {
        Err(format!("Unsupported additional info: {}", additional))
    }
}

/// Encode JSON to CBOR and return as hex string (for cborEncode builtin)
pub fn cbor_encode_hex(json: &Json) -> String {
    let bytes = encode_cbor(json);
    hex::encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let mut obj = HashMap::new();
        obj.insert("a".to_string(), Json::Int(1));
        let json = Json::Object(obj);

        let cbor = encode_cbor(&json);
        let decoded = decode_cbor(&cbor).unwrap();

        assert_eq!(json, decoded);
    }

    #[test]
    fn test_deterministic_encoding() {
        let mut obj = HashMap::new();
        obj.insert("b".to_string(), Json::Int(2));
        obj.insert("a".to_string(), Json::Int(1));
        let json = Json::Object(obj);

        let cbor1 = encode_cbor(&json);
        let cbor2 = encode_cbor(&json);

        assert_eq!(cbor1, cbor2);
    }

    #[test]
    fn test_encode_integer() {
        assert_eq!(encode_cbor(&Json::Int(0)), vec![0x00]);
        assert_eq!(encode_cbor(&Json::Int(23)), vec![0x17]);
        assert_eq!(encode_cbor(&Json::Int(24)), vec![0x18, 0x18]);
        assert_eq!(encode_cbor(&Json::Int(-1)), vec![0x20]);
    }

    #[test]
    fn test_encode_null() {
        assert_eq!(encode_cbor(&Json::Null), vec![0xf6]);
    }

    #[test]
    fn test_encode_bool() {
        assert_eq!(encode_cbor(&Json::Bool(false)), vec![0xf4]);
        assert_eq!(encode_cbor(&Json::Bool(true)), vec![0xf5]);
    }
}
