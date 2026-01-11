use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Int(i64),
    // Decimal as string for now
    String(String),
    Array(Vec<Json>),
    Object(HashMap<String, Json>),
}

pub fn serialize_canonical(json: &Json) -> String {
    // Placeholder implementation - needs proper canonical JSON
    match json {
        Json::Object(obj) => {
            let mut keys: Vec<_> = obj.keys().collect();
            keys.sort();
            let mut result = "{".to_string();
            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    result.push(',');
                }
                result.push('"');
                result.push_str(key);
                result.push('"');
                result.push(':');
                result.push_str(&serialize_canonical(&obj[*key]));
            }
            result.push('}');
            result
        }
        Json::Array(arr) => {
            let mut result = "[".to_string();
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    result.push(',');
                }
                result.push_str(&serialize_canonical(item));
            }
            result.push(']');
            result
        }
        Json::String(s) => format!("\"{}\"", s.replace("\"", "\\\"").replace("\\", "\\\\")),
        Json::Int(n) => n.to_string(),
        Json::Bool(b) => b.to_string(),
        Json::Null => "null".to_string(),
        // No separate Decimal variant for now
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_canonical() {
        let json = Json::Object([("a".to_string(), Json::Int(1))].into());
        let serialized = serialize_canonical(&json);
        assert!(serialized.contains("a"));
    }
}
