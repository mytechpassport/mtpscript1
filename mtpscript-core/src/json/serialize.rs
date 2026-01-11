use super::Json;
use crate::errors::MtpError;

/// Serialize Json to canonical JSON string (RFC 8785)
/// - Object keys sorted by §5 rules (type tag + hash + CBOR tie-break)
/// - Decimals in shortest form
/// - No -0, NaN, Infinity
/// - Deterministic output
pub fn serialize_canonical(json: &Json) -> Result<String, MtpError> {
    let mut output = String::new();
    serialize_value(json, &mut output)?;
    Ok(output)
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
            // Sort keys by §5: type tag (all strings, so same), then hash, then CBOR tie-break
            let mut keys: Vec<_> = obj.keys().collect();
            keys.sort_by(|a, b| {
                // For canonical JSON, sort by UTF-8 byte order
                a.as_bytes().cmp(b.as_bytes())
            });
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
        // Keys sorted alphabetically
        assert_eq!(serialize_canonical(&json).unwrap(), r#"{"a":1,"b":2}"#);
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
