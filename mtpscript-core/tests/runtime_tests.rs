use mtpscript_core::runtime::interpreter::{Interpreter, JsExpr};
use mtpscript_core::runtime::value::{FunctionValue, Value};
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_execution() {
        let mut interp = Interpreter::new();

        // Test simple literal
        let expr = JsExpr::Literal(Value::Number(42));
        let result = interp.eval(&expr).unwrap();
        assert_eq!(result, Value::Number(42));

        // Test simple arithmetic
        let expr = JsExpr::BinOp(
            "+".to_string(),
            Box::new(JsExpr::Literal(Value::Number(2))),
            Box::new(JsExpr::Literal(Value::Number(3))),
        );
        let result = interp.eval(&expr).unwrap();
        assert_eq!(result, Value::Number(5));
    }

    #[test]
    fn test_variable_binding() {
        let mut interp = Interpreter::new();

        // Set a variable
        interp
            .global_scope
            .insert("x".to_string(), Value::Number(10));

        // Reference it
        let expr = JsExpr::Ident("x".to_string());
        let result = interp.eval(&expr).unwrap();
        assert_eq!(result, Value::Number(10));
    }

    #[test]
    fn test_gas_metering() {
        let mut interp = Interpreter::new();
        interp.set_gas_limit(10);

        let expr = JsExpr::Literal(Value::Number(42));
        let result = interp.eval(&expr);

        // Should succeed with gas limit 10
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Number(42));

        // Test gas exhaustion
        let mut interp2 = Interpreter::new();
        interp2.set_gas_limit(0); // No gas

        let result2 = interp2.eval(&expr);
        assert!(result2.is_err()); // Should fail due to gas exhaustion
    }

    #[test]
    fn test_effect_injection() {
        let mut interp = Interpreter::new();

        // Simulate injecting an effect as a global function
        // In real implementation, effects would be injected as globals
        interp.global_scope.insert(
            "mockEffect".to_string(),
            Value::Function(FunctionValue {
                name: Some("mockEffect".to_string()),
                params: vec!["arg".to_string()],
                closure: HashMap::new(),
            }),
        );

        // For this test, we can't actually call functions since the interpreter doesn't execute function bodies
        // Just test that the global is accessible
        let expr = JsExpr::Ident("mockEffect".to_string());
        let result = interp.eval(&expr).unwrap();
        assert!(matches!(result, Value::Function(_)));
    }

    #[test]
    fn test_deterministic_execution() {
        // Test that same expressions produce same results
        let expr = JsExpr::BinOp(
            "*".to_string(),
            Box::new(JsExpr::Literal(Value::Number(3))),
            Box::new(JsExpr::Literal(Value::Number(4))),
        );

        let mut results = Vec::new();

        for _ in 0..10 {
            let mut interp = Interpreter::new();
            let result = interp.eval(&expr).unwrap();
            results.push(result);
        }

        // All results should be identical
        let first = &results[0];
        for result in &results[1..] {
            assert_eq!(first, result);
        }

        assert_eq!(*first, Value::Number(12));
    }

    #[test]
    fn test_array_operations() {
        let mut interp = Interpreter::new();

        let expr = JsExpr::Array(vec![
            JsExpr::Literal(Value::Number(1)),
            JsExpr::Literal(Value::Number(2)),
            JsExpr::Literal(Value::Number(3)),
        ]);

        let result = interp.eval(&expr).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], Value::Number(1));
                assert_eq!(arr[1], Value::Number(2));
                assert_eq!(arr[2], Value::Number(3));
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_object_operations() {
        let mut interp = Interpreter::new();

        let expr = JsExpr::Object(vec![
            (
                "key1".to_string(),
                JsExpr::Literal(Value::String("value1".to_string())),
            ),
            ("key2".to_string(), JsExpr::Literal(Value::Number(42))),
        ]);

        let result = interp.eval(&expr).unwrap();
        match result {
            Value::Object(obj) => {
                assert_eq!(obj.get("key1"), Some(&Value::String("value1".to_string())));
                assert_eq!(obj.get("key2"), Some(&Value::Number(42)));
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_conditional_execution() {
        let mut interp = Interpreter::new();

        // Test true condition
        let expr = JsExpr::If(
            Box::new(JsExpr::Literal(Value::Boolean(true))),
            Box::new(JsExpr::Literal(Value::Number(1))),
            Some(Box::new(JsExpr::Literal(Value::Number(2)))),
        );

        let result = interp.eval(&expr).unwrap();
        assert_eq!(result, Value::Number(1));

        // Test false condition
        let expr2 = JsExpr::If(
            Box::new(JsExpr::Literal(Value::Boolean(false))),
            Box::new(JsExpr::Literal(Value::Number(1))),
            Some(Box::new(JsExpr::Literal(Value::Number(2)))),
        );

        let result2 = interp.eval(&expr2).unwrap();
        assert_eq!(result2, Value::Number(2));
    }

    #[test]
    fn test_error_handling() {
        let mut interp = Interpreter::new();

        // Test undefined variable
        let expr = JsExpr::Ident("undefined_var".to_string());
        let result = interp.eval(&expr);
        assert!(result.is_err());

        // Test type error in operations
        let expr2 = JsExpr::BinOp(
            "+".to_string(),
            Box::new(JsExpr::Literal(Value::Number(1))),
            Box::new(JsExpr::Literal(Value::Boolean(true))),
        );
        let result2 = interp.eval(&expr2);
        assert!(result2.is_err());
    }

    #[test]
    fn test_gas_cost_accuracy() {
        let mut interp = Interpreter::new();
        interp.set_gas_limit(100);

        let initial_gas = interp.gas_used();

        // Simple operation should consume some gas
        let expr = JsExpr::Literal(Value::Number(42));
        let _ = interp.eval(&expr).unwrap();

        let final_gas = interp.gas_used();
        assert!(final_gas > initial_gas);
    }
}
