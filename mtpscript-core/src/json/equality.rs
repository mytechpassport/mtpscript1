use super::Json;
use std::cmp::Ordering;

/// Implement PartialOrd for Json with special rules:
/// - Only numbers and strings can be compared
/// - Other types return None (incomparable)
impl PartialOrd for Json {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Json::Int(a), Json::Int(b)) => Some(a.cmp(b)),
            (Json::Decimal(a), Json::Decimal(b)) => a.partial_cmp(b),
            (Json::String(a), Json::String(b)) => Some(a.cmp(b)),
            (Json::Int(a), Json::Decimal(b)) => {
                // Compare int as decimal
                let a_str = a.to_string();
                let a_dec = crate::types::Decimal::from_str(&a_str).ok()?;
                a_dec.partial_cmp(b)
            }
            (Json::Decimal(a), Json::Int(b)) => {
                let b_str = b.to_string();
                let b_dec = crate::types::Decimal::from_str(&b_str).ok()?;
                a.partial_cmp(&b_dec)
            }
            _ => None, // Incomparable types
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_partial_cmp_int() {
        assert_eq!(
            Json::Int(1).partial_cmp(&Json::Int(2)),
            Some(Ordering::Less)
        );
        assert_eq!(
            Json::Int(2).partial_cmp(&Json::Int(1)),
            Some(Ordering::Greater)
        );
        assert_eq!(
            Json::Int(1).partial_cmp(&Json::Int(1)),
            Some(Ordering::Equal)
        );
    }

    #[test]
    fn test_partial_cmp_string() {
        assert_eq!(
            Json::String("a".to_string()).partial_cmp(&Json::String("b".to_string())),
            Some(Ordering::Less)
        );
        assert_eq!(
            Json::String("b".to_string()).partial_cmp(&Json::String("a".to_string())),
            Some(Ordering::Greater)
        );
        assert_eq!(
            Json::String("a".to_string()).partial_cmp(&Json::String("a".to_string())),
            Some(Ordering::Equal)
        );
    }

    #[test]
    fn test_partial_cmp_incomparable() {
        assert_eq!(Json::Null.partial_cmp(&Json::Int(1)), None);
        assert_eq!(
            Json::Bool(true).partial_cmp(&Json::String("true".to_string())),
            None
        );
        assert_eq!(
            Json::Array(vec![]).partial_cmp(&Json::Object(HashMap::new())),
            None
        );
    }

    #[test]
    fn test_partial_cmp_decimal() {
        let d1 = crate::types::Decimal::from_str("1.0").unwrap();
        let d2 = crate::types::Decimal::from_str("2.0").unwrap();
        assert_eq!(
            Json::Decimal(d1).partial_cmp(&Json::Decimal(d2)),
            Some(Ordering::Less)
        );
    }
}
