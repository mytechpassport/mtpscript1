use crate::errors::compile::CompileError;

pub fn validate_number(_value: i64) -> Result<(), CompileError> {
    // i64 is always valid
    Ok(())
}

pub fn validate_boolean(_value: bool) -> Result<(), CompileError> {
    // Always valid
    Ok(())
}

pub fn validate_string(value: &str) -> Result<(), CompileError> {
    if std::str::from_utf8(value.as_bytes()).is_err() {
        return Err(CompileError::TypeError("Invalid UTF-8 string".to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Type, TypeContext};

    #[test]
    fn test_validate_primitives() {
        assert!(validate_number(42).is_ok());
        assert!(validate_boolean(true).is_ok());
        assert!(validate_string("hello").is_ok());
        assert!(validate_string("🚀").is_ok());
    }

    #[test]
    fn test_primitive_types_acceptance_criteria() {
        let ctx = TypeContext::with_builtins();

        assert!(Type::Number.is_primitive());
        assert!(Type::Boolean.is_primitive());
        assert!(Type::String.is_primitive());
        assert!(Type::Decimal.is_primitive());
        assert_eq!(Type::Number.size_bits(), 64);
        assert!(ctx.lookup("number").is_some());
    }
}
