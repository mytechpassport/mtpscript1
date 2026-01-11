use mtpscript_core::effects::builtins::get_builtin_functions;
use mtpscript_core::json::Json;
use mtpscript_core::runtime::value::Value;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_parse() {
        let builtins = get_builtin_functions();
        let json_parse = builtins.get("Json.parse").unwrap();

        // Test valid JSON
        let input = Value::String(r#"{"a":1,"b":2}"#.to_string());
        let result = json_parse(input).unwrap();

        // Should return a Value representing the parsed JSON
        // The exact structure depends on implementation
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_json_stringify() {
        let builtins = get_builtin_functions();
        let json_stringify = builtins.get("Json.stringify").unwrap();

        // Test stringifying a simple value
        let input = Value::String("hello".to_string());
        let result = json_stringify(input).unwrap();

        assert_eq!(result, Value::String(r#""hello""#.to_string()));
    }

    #[test]
    fn test_decimal_from_string() {
        let builtins = get_builtin_functions();
        let decimal_from_string = builtins.get("Decimal.fromString").unwrap();

        // Test valid decimal string
        let input = Value::String("123.45".to_string());
        let result = decimal_from_string(input).unwrap();

        assert!(matches!(result, Value::Decimal(_)));
    }

    #[test]
    fn test_decimal_to_string() {
        let builtins = get_builtin_functions();
        let decimal_to_string = builtins.get("Decimal.toString").unwrap();

        // Test converting decimal back to string
        let input = Value::String("123.45".to_string());
        let parsed = builtins.get("Decimal.fromString").unwrap()(input).unwrap();
        let result = decimal_to_string(parsed).unwrap();

        assert_eq!(result, Value::String("123.45".to_string()));
    }

    #[test]
    fn test_fnv1a64() {
        let builtins = get_builtin_functions();
        let fnv1a64 = builtins.get("fnv1a64").unwrap();

        // Test hash function produces numbers
        let input = Value::String("hello".to_string());
        let result = fnv1a64(input).unwrap();
        assert!(matches!(result, Value::Number(_)));

        // Different strings give different hashes
        let input2 = Value::String("world".to_string());
        let result2 = fnv1a64(input2).unwrap();
        assert_ne!(result, result2);

        // Same string gives same hash
        let input3 = Value::String("hello".to_string());
        let result3 = fnv1a64(input3).unwrap();
        assert_eq!(result, result3);
    }

    #[test]
    fn test_fnv1a32() {
        let builtins = get_builtin_functions();
        let fnv1a32 = builtins.get("fnv1a32").unwrap();

        // Test hash of "hello"
        let input = Value::String("hello".to_string());
        let result = fnv1a32(input).unwrap();

        // Check it's a number
        assert!(matches!(result, Value::Number(_)));
    }

    #[test]
    fn test_cbor_encode() {
        let builtins = get_builtin_functions();
        let cbor_encode = builtins.get("cborEncode").unwrap();

        // Test encoding a simple value
        let input = Value::String("hello".to_string());
        let result = cbor_encode(input).unwrap();

        // Should return hex string
        assert!(matches!(result, Value::String(_)));
    }

    #[test]
    fn test_builtin_functions_registered() {
        let builtins = get_builtin_functions();

        let expected = vec![
            // JSON methods (both case variants)
            "Json.parse",
            "JSON.parse",
            "Json.stringify",
            "JSON.stringify",
            "JSON.stringifyCanonical",
            // Decimal methods
            "Decimal.fromString",
            "Decimal.toString",
            // Hash functions
            "fnv1a32",
            "fnv1a64",
            // CBOR
            "cborEncode",
        ];

        for name in &expected {
            assert!(builtins.contains_key(*name), "Missing builtin: {}", name);
        }

        assert_eq!(builtins.len(), expected.len());
    }
}
