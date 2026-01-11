use super::Type;

// Built-in pure functions available in MTPScript
pub struct BuiltinFunction {
    pub name: &'static str,
    pub signature: fn(Vec<Type>) -> Result<Type, String>,
}

// List of built-in functions
pub const BUILTINS: &[BuiltinFunction] = &[
    BuiltinFunction {
        name: "Json.parse",
        signature: |args| {
            if args.len() != 1 {
                return Err("Json.parse expects 1 argument".to_string());
            }
            if args[0] != super::Type::String {
                return Err("Json.parse expects string argument".to_string());
            }
            Ok(Type::Adt(Box::new(super::AdtType {
                name: "Json".to_string(),
                type_params: vec![],
                variants: vec![
                    super::AdtVariant::Tuple("JsonNull".to_string(), vec![]),
                    super::AdtVariant::Tuple("JsonBool".to_string(), vec![Type::Boolean]),
                    super::AdtVariant::Tuple("JsonInt".to_string(), vec![Type::Number]),
                    super::AdtVariant::Tuple("JsonDecimal".to_string(), vec![Type::Decimal]),
                    super::AdtVariant::Tuple("JsonString".to_string(), vec![Type::String]),
                    super::AdtVariant::Tuple(
                        "JsonArray".to_string(),
                        vec![Type::Adt(Box::new(super::AdtType {
                            name: "List".to_string(),
                            type_params: vec![],
                            variants: vec![], // Simplified
                        }))],
                    ),
                    super::AdtVariant::Tuple(
                        "JsonObject".to_string(),
                        vec![Type::Adt(Box::new(super::AdtType {
                            name: "Map".to_string(),
                            type_params: vec![],
                            variants: vec![], // Simplified
                        }))],
                    ),
                ],
            })))
        },
    },
    BuiltinFunction {
        name: "Json.stringify",
        signature: |args| {
            if args.len() != 1 {
                return Err("Json.stringify expects 1 argument".to_string());
            }
            Ok(Type::String)
        },
    },
    BuiltinFunction {
        name: "Decimal.fromString",
        signature: |args| {
            if args.len() != 1 {
                return Err("Decimal.fromString expects 1 argument".to_string());
            }
            Ok(Type::option(Type::Decimal))
        },
    },
    BuiltinFunction {
        name: "Decimal.toString",
        signature: |args| {
            if args.len() != 1 {
                return Err("Decimal.toString expects 1 argument".to_string());
            }
            Ok(Type::String)
        },
    },
    BuiltinFunction {
        name: "fnv1a32",
        signature: |args| {
            if args.len() != 1 {
                return Err("fnv1a32 expects 1 argument".to_string());
            }
            Ok(Type::Number)
        },
    },
    BuiltinFunction {
        name: "fnv1a64",
        signature: |args| {
            if args.len() != 1 {
                return Err("fnv1a64 expects 1 argument".to_string());
            }
            Ok(Type::Number)
        },
    },
    BuiltinFunction {
        name: "cborEncode",
        signature: |args| {
            if args.len() != 1 {
                return Err("cborEncode expects 1 argument".to_string());
            }
            Ok(Type::String)
        },
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_signatures() {
        let json_parse = &BUILTINS[0];
        assert_eq!(json_parse.name, "Json.parse");

        let result = (json_parse.signature)(vec![Type::String]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_option_result_types() {
        let ctx = super::super::TypeContext::with_builtins();
        // Option and Result are built-in types in the context
        assert!(ctx.lookup("Option").is_some());
        assert!(ctx.lookup("Result").is_some());
    }

    #[test]
    fn test_option_result_acceptance_criteria() {
        use super::super::{Type, TypeContext};

        let _ctx = TypeContext::with_builtins();

        // Test that Option<T> and Result<T,E> can be constructed
        let option_number = Type::option(Type::Number);
        let result_string_number = Type::result(Type::String, Type::Number);

        // Check that they are ADTs
        match option_number {
            Type::Adt(adt) => {
                assert_eq!(adt.name, "Option");
                assert_eq!(adt.variants.len(), 2);
            }
            _ => panic!("Expected ADT"),
        }

        match result_string_number {
            Type::Adt(adt) => {
                assert_eq!(adt.name, "Result");
                assert_eq!(adt.variants.len(), 2);
            }
            _ => panic!("Expected ADT"),
        }

        // Note: The full acceptance criteria would require a type checker
        // which we don't have implemented yet. This tests the type construction.
    }
}
