use crate::json::Json;
use crate::runtime::value::Value;
use crate::types::Decimal;
use std::collections::HashMap;

// Built-in pure functions

pub fn json_parse(s: Value) -> Result<Value, String> {
    match s {
        Value::String(s) => {
            // Input validation: check string length and content
            if s.len() > 10_000_000 {
                return Err("JSON string too large".to_string());
            }
            if s.chars()
                .any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t')
            {
                return Err("JSON string contains invalid control characters".to_string());
            }
            match Json::parse(&s) {
                Ok(json) => Ok(json.to_value()),
                Err(e) => Err(format!("JSON parse error: {}", e)),
            }
        }
        _ => Err("Json.parse expects string".to_string()),
    }
}

pub fn json_stringify(v: Value) -> Result<Value, String> {
    // Input validation: check for reasonable object/array depth and size
    validate_value_depth(&v, 100)?;
    let json = value_to_json(v);
    match json.to_canonical_string() {
        Ok(s) => {
            if s.len() > 10_000_000 {
                return Err("JSON output too large".to_string());
            }
            Ok(Value::String(s))
        }
        Err(e) => Err(format!("JSON stringify error: {:?}", e)),
    }
}

pub fn value_to_json(v: Value) -> Json {
    match v {
        Value::Null => Json::Null,
        Value::Boolean(b) => Json::Bool(b),
        Value::Number(n) => Json::Int(n),
        Value::Decimal(d) => Json::Decimal(d),
        Value::String(s) => Json::String(s),
        Value::Array(arr) => Json::Array(arr.into_iter().map(value_to_json).collect()),
        Value::Object(obj) => Json::Object(
            obj.into_iter()
                .map(|(k, v)| (k, value_to_json(v)))
                .collect(),
        ),
        Value::Function(_) => Json::String("<function>".to_string()), // Functions not serializable
    }
}

pub fn decimal_from_string(s: Value) -> Result<Value, String> {
    match s {
        Value::String(s) => {
            // Input validation: check string length and basic format
            if s.len() > 100 {
                return Err("Decimal string too long".to_string());
            }
            if !s
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '+')
            {
                return Err("Invalid characters in decimal string".to_string());
            }
            Decimal::from_str(&s)
                .map(Value::Decimal)
                .map_err(|_| "Invalid decimal string".to_string())
        }
        _ => Err("Decimal.fromString expects string".to_string()),
    }
}

pub fn decimal_to_string(d: Value) -> Result<Value, String> {
    match d {
        Value::Decimal(d) => Ok(Value::String(d.to_string())),
        _ => Err("Decimal.toString expects Decimal".to_string()),
    }
}

pub fn fnv1a32(data: Value) -> Result<Value, String> {
    match data {
        Value::String(s) => {
            // Input validation: check string length
            if s.len() > 1_000_000 {
                return Err("Input string too large for hashing".to_string());
            }
            let hash = fnv1a_32(s.as_bytes());
            Ok(Value::Number(hash as i64))
        }
        _ => Err("fnv1a32 expects string".to_string()),
    }
}

pub fn fnv1a64(data: Value) -> Result<Value, String> {
    match data {
        Value::String(s) => {
            // Input validation: check string length
            if s.len() > 1_000_000 {
                return Err("Input string too large for hashing".to_string());
            }
            let hash = fnv1a_64(s.as_bytes());
            Ok(Value::Number(hash as i64))
        }
        _ => Err("fnv1a64 expects string".to_string()),
    }
}

pub fn cbor_encode(v: Value) -> Result<Value, String> {
    // Input validation: check depth and size
    validate_value_depth(&v, 50)?;
    // Encode to CBOR and return hex string
    let json = value_to_json(v);
    match json.to_cbor_hex() {
        Ok(hex) => {
            if hex.len() > 2_000_000 {
                return Err("CBOR output too large".to_string());
            }
            Ok(Value::String(hex))
        }
        Err(e) => Err(format!("CBOR encode error: {:?}", e)),
    }
}

// Input validation functions

fn validate_value_depth(v: &Value, max_depth: usize) -> Result<(), String> {
    fn check_depth(v: &Value, current_depth: usize, max_depth: usize) -> Result<(), String> {
        if current_depth > max_depth {
            return Err("Value exceeds maximum nesting depth".to_string());
        }
        match v {
            Value::Array(arr) => {
                if arr.len() > 10_000 {
                    return Err("Array too large".to_string());
                }
                for item in arr {
                    check_depth(item, current_depth + 1, max_depth)?;
                }
            }
            Value::Object(obj) => {
                if obj.len() > 1_000 {
                    return Err("Object too large".to_string());
                }
                for (k, v) in obj {
                    if k.len() > 1_000 {
                        return Err("Object key too long".to_string());
                    }
                    check_depth(v, current_depth + 1, max_depth)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    check_depth(v, 0, max_depth)
}

// FNV-1a hash implementations

fn fnv1a_32(data: &[u8]) -> u32 {
    const FNV_OFFSET: u32 = 0x811c9dc5;
    const FNV_PRIME: u32 = 0x01000193;

    let mut hash = FNV_OFFSET;
    for &byte in data {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn fnv1a_64(data: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

// Function registry

pub type BuiltinFn = fn(Value) -> Result<Value, String>;

pub fn get_builtin_functions() -> HashMap<String, BuiltinFn> {
    let mut map = HashMap::new();

    // JSON methods (both case variants for compatibility)
    map.insert("Json.parse".to_string(), json_parse as BuiltinFn);
    map.insert("JSON.parse".to_string(), json_parse as BuiltinFn);
    map.insert("Json.stringify".to_string(), json_stringify as BuiltinFn);
    map.insert("JSON.stringify".to_string(), json_stringify as BuiltinFn);
    map.insert(
        "JSON.stringifyCanonical".to_string(),
        json_stringify as BuiltinFn, // Same as stringify - both are canonical
    );

    // Decimal methods
    map.insert(
        "Decimal.fromString".to_string(),
        decimal_from_string as BuiltinFn,
    );
    map.insert(
        "Decimal.toString".to_string(),
        decimal_to_string as BuiltinFn,
    );

    // Hash functions
    map.insert("fnv1a32".to_string(), fnv1a32 as BuiltinFn);
    map.insert("fnv1a64".to_string(), fnv1a64 as BuiltinFn);

    // CBOR encoding
    map.insert("cborEncode".to_string(), cbor_encode as BuiltinFn);

    map
}
