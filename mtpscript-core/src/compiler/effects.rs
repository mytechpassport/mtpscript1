use crate::ir::nodes::IrExpr;

/// Compile effect calls to JS with whitelist validation
pub fn compile_effect_call(effect_name: &str, args: &[IrExpr]) -> Result<String, String> {
    // Validate arguments based on effect name
    validate_effect_args(effect_name, args)?;

    let mut js_args = Vec::new();
    for arg in args {
        let js_arg = ir_expr_to_js(arg)?;
        js_args.push(js_arg);
    }
    Ok(format!("{}({})", effect_name, js_args.join(", ")))
}

/// Convert an IR expression to JavaScript/JSON representation
fn ir_expr_to_js(expr: &IrExpr) -> Result<String, String> {
    match expr {
        IrExpr::String(s, _) => Ok(format!("\"{}\"", escape_json_string(s))),
        IrExpr::Number(n, _) => Ok(n.to_string()),
        IrExpr::Boolean(b, _) => Ok(b.to_string()),
        IrExpr::Array(elements, _) => {
            let items: Result<Vec<String>, String> = elements.iter().map(ir_expr_to_js).collect();
            Ok(format!("[{}]", items?.join(", ")))
        }
        IrExpr::Object(fields, _) => {
            let items: Result<Vec<String>, String> = fields
                .iter()
                .map(|(k, v)| {
                    let value = ir_expr_to_js(v)?;
                    Ok(format!("\"{}\": {}", escape_json_string(k), value))
                })
                .collect();
            Ok(format!("{{{}}}", items?.join(", ")))
        }
        IrExpr::Var(name, _) => Ok(name.clone()), // Variable reference
        IrExpr::Decimal(d, _) => Ok(format!("\"{}\"", d)), // Decimals as strings per spec
        _ => Err(format!("Unsupported effect argument type: {:?}", expr)),
    }
}

/// Escape special characters for JSON string
fn escape_json_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn validate_effect_args(effect_name: &str, args: &[IrExpr]) -> Result<(), String> {
    match effect_name {
        "DbRead" => {
            if args.len() != 2 {
                return Err(
                    "DbRead expects 2 arguments: sql (string) and params (object)".to_string(),
                );
            }
            if !matches!(args[0], IrExpr::String(_, _)) {
                return Err("DbRead first argument must be string (SQL)".to_string());
            }
            if !matches!(args[1], IrExpr::Object(_, _) | IrExpr::Array(_, _)) {
                return Err("DbRead second argument must be object or array (params)".to_string());
            }
        }
        "DbWrite" => {
            if args.len() != 2 {
                return Err(
                    "DbWrite expects 2 arguments: sql (string) and params (object)".to_string(),
                );
            }
            if !matches!(args[0], IrExpr::String(_, _)) {
                return Err("DbWrite first argument must be string (SQL)".to_string());
            }
            if !matches!(args[1], IrExpr::Object(_, _) | IrExpr::Array(_, _)) {
                return Err("DbWrite second argument must be object or array (params)".to_string());
            }
        }
        "HttpOut" => {
            if args.len() != 2 {
                return Err(
                    "HttpOut expects 2 arguments: method (string) and url (string)".to_string(),
                );
            }
            if !matches!(args[0], IrExpr::String(_, _)) {
                return Err("HttpOut first argument must be string (method)".to_string());
            }
            if !matches!(args[1], IrExpr::String(_, _)) {
                return Err("HttpOut second argument must be string (url)".to_string());
            }
        }
        "Log" => {
            if args.len() != 1 {
                return Err("Log expects 1 argument: message (string)".to_string());
            }
            if !matches!(args[0], IrExpr::String(_, _)) {
                return Err("Log argument must be string (message)".to_string());
            }
        }
        "Async" => {
            if args.len() != 3 {
                return Err("Async expects 3 arguments: promiseHash (string), contId (string), args (array)".to_string());
            }
            if !matches!(args[0], IrExpr::String(_, _)) {
                return Err("Async first argument must be string (promiseHash)".to_string());
            }
            if !matches!(args[1], IrExpr::String(_, _)) {
                return Err("Async second argument must be string (contId)".to_string());
            }
            if !matches!(args[2], IrExpr::Array(_, _) | IrExpr::Object(_, _)) {
                return Err("Async third argument must be array or object (args)".to_string());
            }
        }
        _ => return Err(format!("Unknown effect: {}", effect_name)),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::nodes::IrExpr;
    use crate::types::Type;

    #[test]
    fn test_compile_db_read() {
        let sql = IrExpr::String("SELECT 1".to_string(), Type::String);
        let params = IrExpr::Object(vec![], Type::Number);
        let result = compile_effect_call("DbRead", &[sql, params]).unwrap();
        assert_eq!(result, "DbRead(\"SELECT 1\", {})");
    }

    #[test]
    fn test_invalid_effect_args() {
        let invalid_sql = IrExpr::Number(42, Type::Number);
        let params = IrExpr::Object(vec![], Type::Number);
        let result = compile_effect_call("DbRead", &[invalid_sql, params]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("first argument must be string"));
    }

    // Comprehensive validation tests

    #[test]
    fn test_db_read_wrong_arg_count() {
        let sql = IrExpr::String("SELECT 1".to_string(), Type::String);

        // Too few args
        let result = compile_effect_call("DbRead", &[sql.clone()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expects 2 arguments"));

        // Too many args
        let params = IrExpr::Object(vec![], Type::Number);
        let extra = IrExpr::Number(1, Type::Number);
        let result = compile_effect_call("DbRead", &[sql, params, extra]);
        assert!(result.is_err());
    }

    #[test]
    fn test_db_read_wrong_second_arg_type() {
        let sql = IrExpr::String("SELECT 1".to_string(), Type::String);
        let bad_params = IrExpr::Number(42, Type::Number);
        let result = compile_effect_call("DbRead", &[sql, bad_params]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("second argument must be"));
    }

    #[test]
    fn test_db_read_with_array_params() {
        let sql = IrExpr::String("SELECT ?".to_string(), Type::String);
        let params = IrExpr::Array(vec![IrExpr::Number(1, Type::Number)], Type::Number);
        let result = compile_effect_call("DbRead", &[sql, params]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_db_write_validation() {
        let sql = IrExpr::String("INSERT INTO t VALUES (?)".to_string(), Type::String);
        let params = IrExpr::Object(vec![], Type::Number);

        // Valid call
        let result = compile_effect_call("DbWrite", &[sql.clone(), params.clone()]);
        assert!(result.is_ok());

        // Wrong first arg type
        let bad_sql = IrExpr::Boolean(true, Type::Boolean);
        let result = compile_effect_call("DbWrite", &[bad_sql, params.clone()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("first argument must be string"));

        // Wrong second arg type
        let bad_params = IrExpr::String("bad".to_string(), Type::String);
        let result = compile_effect_call("DbWrite", &[sql, bad_params]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("second argument must be"));
    }

    #[test]
    fn test_http_out_validation() {
        let method = IrExpr::String("GET".to_string(), Type::String);
        let url = IrExpr::String("https://example.com".to_string(), Type::String);

        // Valid call
        let result = compile_effect_call("HttpOut", &[method.clone(), url.clone()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "HttpOut(\"GET\", \"https://example.com\")");

        // Wrong number of args
        let result = compile_effect_call("HttpOut", &[method.clone()]);
        assert!(result.is_err());

        // Wrong first arg type
        let bad_method = IrExpr::Number(1, Type::Number);
        let result = compile_effect_call("HttpOut", &[bad_method, url.clone()]);
        assert!(result.is_err());

        // Wrong second arg type
        let bad_url = IrExpr::Boolean(false, Type::Boolean);
        let result = compile_effect_call("HttpOut", &[method, bad_url]);
        assert!(result.is_err());
    }

    #[test]
    fn test_log_validation() {
        let msg = IrExpr::String("test message".to_string(), Type::String);

        // Valid call
        let result = compile_effect_call("Log", &[msg.clone()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Log(\"test message\")");

        // Wrong number of args
        let result = compile_effect_call("Log", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expects 1 argument"));

        let extra = IrExpr::String("extra".to_string(), Type::String);
        let result = compile_effect_call("Log", &[msg.clone(), extra]);
        assert!(result.is_err());

        // Wrong arg type
        let bad_msg = IrExpr::Number(42, Type::Number);
        let result = compile_effect_call("Log", &[bad_msg]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be string"));
    }

    #[test]
    fn test_async_validation() {
        let hash = IrExpr::String("abc123".to_string(), Type::String);
        let cont_id = IrExpr::String("cont_1".to_string(), Type::String);
        let args = IrExpr::Array(vec![], Type::Number);

        // Valid call
        let result = compile_effect_call("Async", &[hash.clone(), cont_id.clone(), args.clone()]);
        assert!(result.is_ok());

        // Wrong number of args
        let result = compile_effect_call("Async", &[hash.clone(), cont_id.clone()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expects 3 arguments"));

        // Wrong first arg type
        let bad_hash = IrExpr::Number(123, Type::Number);
        let result = compile_effect_call("Async", &[bad_hash, cont_id.clone(), args.clone()]);
        assert!(result.is_err());

        // Wrong second arg type
        let bad_cont = IrExpr::Boolean(true, Type::Boolean);
        let result = compile_effect_call("Async", &[hash.clone(), bad_cont, args.clone()]);
        assert!(result.is_err());

        // Wrong third arg type
        let bad_args = IrExpr::String("bad".to_string(), Type::String);
        let result = compile_effect_call("Async", &[hash, cont_id, bad_args]);
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_effect_rejected() {
        let arg = IrExpr::String("test".to_string(), Type::String);
        let result = compile_effect_call("UnknownEffect", &[arg]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown effect"));
    }

    #[test]
    fn test_empty_effect_name() {
        let arg = IrExpr::String("test".to_string(), Type::String);
        let result = compile_effect_call("", &[arg]);
        assert!(result.is_err());
    }

    #[test]
    fn test_unsupported_arg_type_rejected() {
        // Try using a Lambda as an argument (not supported)
        let sql = IrExpr::String("SELECT 1".to_string(), Type::String);
        let lambda = IrExpr::Lambda {
            params: vec![],
            body: Box::new(IrExpr::Number(1, Type::Number)),
            result_type: Type::Number,
        };
        let result = compile_effect_call("DbRead", &[sql, lambda]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("second argument must be"));
    }

    #[test]
    fn test_compile_with_various_arg_types() {
        // Test that different valid arg types compile correctly
        let str_arg = IrExpr::String("hello".to_string(), Type::String);
        let num_arg = IrExpr::Number(42, Type::Number);
        let bool_arg = IrExpr::Boolean(true, Type::Boolean);

        // Log takes a string
        let result = compile_effect_call("Log", &[str_arg]);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("\"hello\""));

        // Test that unsupported arg types in allowed positions fail gracefully
        let result = compile_effect_call("Log", &[num_arg]);
        assert!(result.is_err());

        let result = compile_effect_call("Log", &[bool_arg]);
        assert!(result.is_err());
    }

    #[test]
    fn test_sql_with_special_characters() {
        // Ensure SQL strings are passed through correctly
        let sql = IrExpr::String("SELECT * FROM users WHERE name = 'O\\'Brien'".to_string(), Type::String);
        let params = IrExpr::Object(vec![], Type::Number);
        let result = compile_effect_call("DbRead", &[sql, params]);
        assert!(result.is_ok());
        // The compiled result should contain the SQL
        assert!(result.unwrap().contains("SELECT"));
    }
}
