use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Int(i64),
    Decimal(String), // Fixed-point decimal as string
    String(String),
    Array(Vec<Json>),
    Object(HashMap<String, Json>),
}

impl Eq for Json {}

/// Serialize JSON to canonical form per RFC 8785 (JCS)
/// - Object keys sorted lexicographically by UTF-16 code units
/// - No whitespace
/// - Numbers in shortest form
/// - Strings with minimal escaping
pub fn serialize_canonical(json: &Json) -> String {
    match json {
        Json::Null => "null".to_string(),
        Json::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        Json::Int(n) => {
            // Use shortest representation, no -0
            if *n == 0 {
                "0".to_string()
            } else {
                n.to_string()
            }
        }
        Json::Decimal(d) => {
            // Decimal stored as string, output in shortest form
            let trimmed = d.trim_end_matches('0').trim_end_matches('.');
            if trimmed.is_empty() || trimmed == "-" {
                "0".to_string()
            } else {
                trimmed.to_string()
            }
        }
        Json::String(s) => serialize_string(s),
        Json::Array(arr) => {
            let mut result = String::from("[");
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    result.push(',');
                }
                result.push_str(&serialize_canonical(item));
            }
            result.push(']');
            result
        }
        Json::Object(obj) => {
            // Sort keys lexicographically (RFC 8785 uses ES6 ordering)
            let mut keys: Vec<&String> = obj.keys().collect();
            keys.sort_by(|a, b| {
                // Compare by UTF-16 code units (ES6 ordering)
                let a_utf16: Vec<u16> = a.encode_utf16().collect();
                let b_utf16: Vec<u16> = b.encode_utf16().collect();
                a_utf16.cmp(&b_utf16)
            });

            let mut result = String::from("{");
            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    result.push(',');
                }
                result.push_str(&serialize_string(key));
                result.push(':');
                result.push_str(&serialize_canonical(&obj[*key]));
            }
            result.push('}');
            result
        }
    }
}

/// Serialize a string with proper JSON escaping per RFC 8785
fn serialize_string(s: &str) -> String {
    let mut result = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c < '\x20' => {
                // Control characters must be escaped as \uXXXX
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result.push('"');
    result
}

/// Parse JSON from string
pub fn parse_json(input: &str) -> Result<Json, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("Empty input".to_string());
    }

    let (json, rest) = parse_value(input)?;
    if !rest.trim().is_empty() {
        return Err(format!("Unexpected trailing content: {}", rest));
    }
    Ok(json)
}

fn parse_value(input: &str) -> Result<(Json, &str), String> {
    let input = input.trim_start();

    if input.starts_with("null") {
        return Ok((Json::Null, &input[4..]));
    }
    if input.starts_with("true") {
        return Ok((Json::Bool(true), &input[4..]));
    }
    if input.starts_with("false") {
        return Ok((Json::Bool(false), &input[5..]));
    }
    if input.starts_with('"') {
        return parse_string(input);
    }
    if input.starts_with('[') {
        return parse_array(input);
    }
    if input.starts_with('{') {
        return parse_object(input);
    }
    if input.starts_with('-') || input.chars().next().map_or(false, |c| c.is_ascii_digit()) {
        return parse_number(input);
    }

    Err(format!("Unexpected character: {}", input.chars().next().unwrap_or(' ')))
}

fn parse_string(input: &str) -> Result<(Json, &str), String> {
    if !input.starts_with('"') {
        return Err("Expected string".to_string());
    }

    let mut result = String::new();
    let mut chars = input[1..].chars().peekable();
    let mut consumed = 1;

    loop {
        match chars.next() {
            None => return Err("Unterminated string".to_string()),
            Some('"') => {
                consumed += 1;
                break;
            }
            Some('\\') => {
                consumed += 1;
                match chars.next() {
                    Some('n') => { result.push('\n'); consumed += 1; }
                    Some('r') => { result.push('\r'); consumed += 1; }
                    Some('t') => { result.push('\t'); consumed += 1; }
                    Some('\\') => { result.push('\\'); consumed += 1; }
                    Some('"') => { result.push('"'); consumed += 1; }
                    Some('/') => { result.push('/'); consumed += 1; }
                    Some('b') => { result.push('\x08'); consumed += 1; }
                    Some('f') => { result.push('\x0c'); consumed += 1; }
                    Some('u') => {
                        consumed += 1;
                        let mut hex = String::new();
                        for _ in 0..4 {
                            match chars.next() {
                                Some(c) if c.is_ascii_hexdigit() => {
                                    hex.push(c);
                                    consumed += 1;
                                }
                                _ => return Err("Invalid unicode escape".to_string()),
                            }
                        }
                        let code = u16::from_str_radix(&hex, 16)
                            .map_err(|_| "Invalid unicode escape")?;
                        if let Some(c) = char::from_u32(code as u32) {
                            result.push(c);
                        } else {
                            // Handle surrogate pairs
                            result.push_str(&format!("\\u{}", hex));
                        }
                    }
                    Some(c) => return Err(format!("Invalid escape: \\{}", c)),
                    None => return Err("Unterminated escape".to_string()),
                }
            }
            Some(c) => {
                result.push(c);
                consumed += c.len_utf8();
            }
        }
    }

    Ok((Json::String(result), &input[consumed..]))
}

fn parse_number(input: &str) -> Result<(Json, &str), String> {
    let mut end = 0;
    let mut has_decimal = false;
    let mut has_exponent = false;
    let chars: Vec<char> = input.chars().collect();

    // Optional negative sign
    if end < chars.len() && chars[end] == '-' {
        end += 1;
    }

    // Integer part
    if end < chars.len() && chars[end] == '0' {
        end += 1;
    } else {
        while end < chars.len() && chars[end].is_ascii_digit() {
            end += 1;
        }
    }

    // Decimal part
    if end < chars.len() && chars[end] == '.' {
        has_decimal = true;
        end += 1;
        while end < chars.len() && chars[end].is_ascii_digit() {
            end += 1;
        }
    }

    // Exponent part
    if end < chars.len() && (chars[end] == 'e' || chars[end] == 'E') {
        has_exponent = true;
        end += 1;
        if end < chars.len() && (chars[end] == '+' || chars[end] == '-') {
            end += 1;
        }
        while end < chars.len() && chars[end].is_ascii_digit() {
            end += 1;
        }
    }

    let num_str: String = chars[..end].iter().collect();
    let byte_len: usize = chars[..end].iter().map(|c| c.len_utf8()).sum();

    if has_decimal || has_exponent {
        // Parse as decimal
        Ok((Json::Decimal(num_str), &input[byte_len..]))
    } else {
        // Parse as integer
        let n: i64 = num_str.parse().map_err(|_| format!("Invalid number: {}", num_str))?;
        Ok((Json::Int(n), &input[byte_len..]))
    }
}

fn parse_array(input: &str) -> Result<(Json, &str), String> {
    if !input.starts_with('[') {
        return Err("Expected array".to_string());
    }

    let mut rest = &input[1..];
    let mut items = Vec::new();

    rest = rest.trim_start();
    if rest.starts_with(']') {
        return Ok((Json::Array(items), &rest[1..]));
    }

    loop {
        let (item, new_rest) = parse_value(rest)?;
        items.push(item);
        rest = new_rest.trim_start();

        if rest.starts_with(']') {
            return Ok((Json::Array(items), &rest[1..]));
        }
        if rest.starts_with(',') {
            rest = &rest[1..];
        } else {
            return Err("Expected ',' or ']' in array".to_string());
        }
    }
}

fn parse_object(input: &str) -> Result<(Json, &str), String> {
    if !input.starts_with('{') {
        return Err("Expected object".to_string());
    }

    let mut rest = &input[1..];
    let mut obj = HashMap::new();

    rest = rest.trim_start();
    if rest.starts_with('}') {
        return Ok((Json::Object(obj), &rest[1..]));
    }

    loop {
        rest = rest.trim_start();

        // Parse key
        let (key_json, new_rest) = parse_string(rest)?;
        let key = match key_json {
            Json::String(s) => s,
            _ => return Err("Object key must be string".to_string()),
        };

        // Check for duplicate keys (RFC 8259 recommends unique keys)
        if obj.contains_key(&key) {
            return Err(format!("Duplicate key: {}", key));
        }

        rest = new_rest.trim_start();

        // Expect colon
        if !rest.starts_with(':') {
            return Err("Expected ':' in object".to_string());
        }
        rest = &rest[1..];

        // Parse value
        let (value, new_rest) = parse_value(rest)?;
        obj.insert(key, value);
        rest = new_rest.trim_start();

        if rest.starts_with('}') {
            return Ok((Json::Object(obj), &rest[1..]));
        }
        if rest.starts_with(',') {
            rest = &rest[1..];
        } else {
            return Err("Expected ',' or '}' in object".to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_canonical() {
        let json = Json::Object([("a".to_string(), Json::Int(1))].into());
        let serialized = serialize_canonical(&json);
        assert_eq!(serialized, r#"{"a":1}"#);
    }

    #[test]
    fn test_canonical_key_ordering() {
        let mut obj = HashMap::new();
        obj.insert("z".to_string(), Json::Int(1));
        obj.insert("a".to_string(), Json::Int(2));
        let json = Json::Object(obj);
        let serialized = serialize_canonical(&json);
        assert_eq!(serialized, r#"{"a":2,"z":1}"#);
    }

    #[test]
    fn test_string_escaping() {
        let json = Json::String("hello\nworld".to_string());
        let serialized = serialize_canonical(&json);
        assert_eq!(serialized, r#""hello\nworld""#);
    }

    #[test]
    fn test_parse_roundtrip() {
        let input = r#"{"a":1,"b":"test","c":[1,2,3]}"#;
        let parsed = parse_json(input).unwrap();
        let serialized = serialize_canonical(&parsed);
        assert_eq!(serialized, input);
    }

    #[test]
    fn test_reject_duplicate_keys() {
        let input = r#"{"a":1,"a":2}"#;
        assert!(parse_json(input).is_err());
    }
}
