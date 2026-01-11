use crate::ir::nodes::IrExpr;

/// Compile effect calls to JS with whitelist validation
pub fn compile_effect_call(effect_name: &str, args: &[IrExpr]) -> Result<String, String> {
    // Validate arguments based on effect name
    validate_effect_args(effect_name, args)?;

    let mut js_args = Vec::new();
    for arg in args {
        let js_arg = match arg {
            IrExpr::String(s, _) => format!("\"{}\"", s),
            IrExpr::Number(n, _) => n.to_string(),
            IrExpr::Boolean(b, _) => b.to_string(),
            IrExpr::Array(_, _) => {
                // For arrays like params
                // Simplified: assume JSON serialization
                "{}".to_string() // Placeholder
            }
            IrExpr::Object(_, _) => {
                // For objects
                "{}".to_string() // Placeholder
            }
            _ => return Err(format!("Unsupported effect argument type: {:?}", arg)),
        };
        js_args.push(js_arg);
    }
    Ok(format!("{}({})", effect_name, js_args.join(", ")))
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
}
