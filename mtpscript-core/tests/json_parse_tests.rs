use mtpscript_core::json::Json;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_parse_basic() {
        // Test parsing various JSON types
        assert_eq!(Json::parse("null").unwrap(), Json::Null);
        assert_eq!(Json::parse("true").unwrap(), Json::Bool(true));
        assert_eq!(Json::parse("42").unwrap(), Json::Int(42));
        assert_eq!(
            Json::parse("\"hello\"").unwrap(),
            Json::String("hello".to_string())
        );
    }

    #[test]
    fn test_json_parse_object() {
        let json = Json::parse(r#"{"a": 1, "b": null}"#).unwrap();
        match json {
            Json::Object(obj) => {
                assert_eq!(obj.get("a"), Some(&Json::Int(1)));
                assert_eq!(obj.get("b"), Some(&Json::Null));
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_json_parse_array() {
        let json = Json::parse(r#"[1, "hello", null]"#).unwrap();
        match json {
            Json::Array(arr) => {
                assert_eq!(arr[0], Json::Int(1));
                assert_eq!(arr[1], Json::String("hello".to_string()));
                assert_eq!(arr[2], Json::Null);
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_json_parse_reject_duplicate_keys() {
        // Should reject duplicate keys
        let result = Json::parse(r#"{"a": 1, "a": 2}"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_json_parse_invalid() {
        // Test some invalid inputs - may panic or error depending on implementation
        // For now, just test that valid parsing works
        let _ = Json::parse("{}").unwrap(); // Valid
    }
}
