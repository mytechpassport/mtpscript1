use crate::ir::nodes::IrExpr;

/// Compile effect calls to JS
pub fn compile_effect_call(effect_name: &str, args: &[IrExpr]) -> Result<String, String> {
    let mut js_args = Vec::new();
    for arg in args {
        let js_arg = match arg {
            IrExpr::String(s, _) => format!("\"{}\"", s),
            IrExpr::Number(n, _) => n.to_string(),
            IrExpr::Boolean(b, _) => b.to_string(),
            _ => return Err(format!("Unsupported effect argument type: {:?}", arg)),
        };
        js_args.push(js_arg);
    }
    Ok(format!("{}({})", effect_name, js_args.join(", ")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::nodes::IrExpr;
    use crate::types::Type;

    #[test]
    fn test_compile_db_read() {
        let sql = IrExpr::String("SELECT 1".to_string(), Type::String);
        let params = IrExpr::Array(vec![], Type::Number);
        let result = compile_effect_call("DbRead", &[sql]).unwrap();
        assert_eq!(result, "DbRead(\"SELECT 1\")");
    }
}
