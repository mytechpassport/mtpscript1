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
