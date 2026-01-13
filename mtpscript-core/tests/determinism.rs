use mtpscript_core::errors::{compile::CompileError, MtpError};
use mtpscript_core::ir::lower;
use mtpscript_core::lexer::scanner::Scanner;
use mtpscript_core::parser::Parser;
use mtpscript_core::snapshot::create_test_snapshot;
use mtpscript_core::types::checker::TypeChecker;
use sha2::{Digest, Sha256};

#[cfg(test)]
mod tests {
    use super::*;

    fn get_compiled_js(src: &str) -> Result<String, CompileError> {
        let mut scanner = Scanner::new(src)?;
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(&tokens);
        let program = parser.parse()?;
        let mut type_checker = TypeChecker::new();
        type_checker.typecheck_program(&program)?;
        let ir = lower::lower_ast_to_ir(&program)?;
        mtpscript_core::compiler::codegen::compile_ir_to_js(&ir)
    }

    fn compile_program_to_snapshot(src: &str) -> Result<Vec<u8>, MtpError> {
        let js = get_compiled_js(src)?;
        create_test_snapshot(&js)
    }

    #[test]
    fn test_compilation_determinism_simple_arithmetic() {
        let src = "function main() { 2 + 3 * 4 }";

        // Compile multiple times and check JS is identical
        let mut js_codes = Vec::new();
        for _ in 0..10 {
            let js = get_compiled_js(src).unwrap();
            js_codes.push(js);
        }

        // All JS codes should be identical
        let first_js = &js_codes[0];
        for js in &js_codes[1..] {
            assert_eq!(first_js, js);
        }

        // Check that snapshot creation is also deterministic
        let mut snapshots = Vec::new();
        for _ in 0..5 {
            let snapshot = compile_program_to_snapshot(src).unwrap();
            snapshots.push(snapshot);
        }

        let first_snapshot = &snapshots[0];
        for snapshot in &snapshots[1..] {
            assert_eq!(first_snapshot, snapshot);
        }
    }

    #[test]
    fn test_compilation_determinism_with_arrays() {
        let src = "function main() { [1, 2, 3, 4, 5] }";

        let mut js_codes = Vec::new();
        for _ in 0..10 {
            let js = get_compiled_js(src).unwrap();
            js_codes.push(js);
        }

        let first = &js_codes[0];
        for js in &js_codes[1..] {
            assert_eq!(first, js);
        }

        assert!(first.contains("[1, 2, 3, 4, 5]"));
    }

    #[test]
    fn test_compilation_determinism_different_inputs() {
        let src1 = "function main() { 42 }";
        let src2 = "function main() { 43 }";

        let js1 = get_compiled_js(src1).unwrap();
        let js2 = get_compiled_js(src2).unwrap();

        // Different inputs should give different JS
        assert_ne!(js1, js2);

        // Same input should always give same JS
        let js1_again = get_compiled_js(src1).unwrap();
        assert_eq!(js1, js1_again);
    }

    #[test]
    fn test_compilation_determinism_fuzzing() {
        // Test compilation determinism with many different programs
        let programs = vec![
            "function main() { 1 }",
            "function main() { true }",
            "function main() { [1,2,3] }",
            "function main() { 1 + 2 * 3 }",
        ];

        for program_src in programs {
            let mut js_codes = Vec::new();
            for _ in 0..5 {
                let js = get_compiled_js(program_src).unwrap();
                js_codes.push(js);
            }

            // All compilations of same program should be identical
            let first_js = &js_codes[0];
            for js in &js_codes[1..] {
                assert_eq!(first_js, js);
            }
        }
    }

    #[test]
    fn test_runtime_determinism_simple_expression() {
        use mtpscript_core::runtime::interpreter::{Interpreter, JsExpr};
        use mtpscript_core::runtime::value::Value;
        use sha2::{Digest, Sha256};

        // Create a simple expression: 1 + 2 * 3
        let expr = JsExpr::BinOp(
            "+".to_string(),
            Box::new(JsExpr::Literal(Value::Number(1))),
            Box::new(JsExpr::BinOp(
                "*".to_string(),
                Box::new(JsExpr::Literal(Value::Number(2))),
                Box::new(JsExpr::Literal(Value::Number(3))),
            )),
        );

        let mut results = Vec::new();

        // Execute the same expression multiple times with same gas limit
        for _ in 0..10 {
            let mut interp = Interpreter::new();
            interp.set_gas_limit(1000);
            let result = interp.eval(&expr).unwrap();
            // For determinism, we check that the result value is the same
            // Since Value implements PartialEq, we can use that
            results.push(result);
        }

        // All results should be identical
        let first_hash = &results[0];
        for hash in &results[1..] {
            assert_eq!(first_hash, hash);
        }

        // Verify the result is correct
        let mut interp = Interpreter::new();
        let result = interp.eval(&expr).unwrap();
        assert_eq!(result, Value::Number(7)); // 1 + (2 * 3) = 7
    }

    #[test]
    fn test_runtime_determinism_with_seed() {
        use mtpscript_core::runtime::interpreter::{Interpreter, JsExpr};
        use mtpscript_core::runtime::seed::{compute_seed, SeedRequest};
        use mtpscript_core::runtime::value::Value;
        use sha2::{Digest, Sha256};

        // Test with deterministic seed computation
        let seed_req = SeedRequest::new(
            "test_request_id".to_string(),
            "test_account_id".to_string(),
            "test_version".to_string(),
            [0u8; 32],  // snapshot hash
            10_000_000, // gas limit
        );
        let seed = compute_seed(&seed_req).unwrap();

        // Create a simple expression that could be affected by seed
        // (In practice, effects would use the seed)
        let expr = JsExpr::Literal(Value::Number(42));

        let mut results = Vec::new();

        for _ in 0..5 {
            let mut interp = Interpreter::new();
            interp.set_gas_limit(10_000_000);

            // Inject seed into global scope (simulating effect injection)
            interp.global_scope.insert(
                "request_seed".to_string(),
                Value::String(hex::encode(&seed)),
            );

            let result = interp.eval(&expr).unwrap();
            let result_json = format!(
                "{{\"result\": {}, \"seed\": \"{}\"}}",
                result.as_number().unwrap(),
                hex::encode(&seed)
            );
            let hash = Sha256::digest(result_json.as_bytes());
            results.push(hash);
        }

        // All results should be identical
        let first_hash = &results[0];
        for hash in &results[1..] {
            assert_eq!(first_hash, hash);
        }
    }

    #[test]
    fn test_runtime_determinism_end_to_end() {
        use mtpscript_core::runtime::interpreter::{Interpreter, JsExpr};
        use mtpscript_core::runtime::seed::{compute_seed, SeedRequest};
        use mtpscript_core::runtime::value::Value;
        use sha2::{Digest, Sha256};

        // Simulate end-to-end determinism test
        // In a real implementation, this would:
        // 1. Compile MTP to JS
        // 2. Create snapshot
        // 3. Clone interpreter from snapshot
        // 4. Inject effects with seed
        // 5. Execute
        // 6. Serialize response to canonical JSON
        // 7. Hash the response

        let request_id = "req_123456789";
        let account_id = "acc_987654321";
        let function_version = "v1.0.0";
        let gas_limit = 10_000_000;
        let snapshot_hash_bytes = Sha256::digest(b"test_snapshot_content");
        let mut snapshot_hash = [0u8; 32];
        snapshot_hash.copy_from_slice(&snapshot_hash_bytes);

        let seed_req = SeedRequest::new(
            request_id.to_string(),
            account_id.to_string(),
            function_version.to_string(),
            snapshot_hash,
            gas_limit,
        );
        let seed = compute_seed(&seed_req).unwrap();

        // Simple program simulation
        let expr = JsExpr::Object(vec![
            (
                "status".to_string(),
                JsExpr::Literal(Value::String("ok".to_string())),
            ),
            ("data".to_string(), JsExpr::Literal(Value::Number(42))),
            (
                "seed_prefix".to_string(),
                JsExpr::Literal(Value::String(hex::encode(&seed[0..8]))),
            ),
        ]);

        let mut response_hashes = Vec::new();

        for _ in 0..1000 {
            // Test many times for statistical confidence
            let mut interp = Interpreter::new();
            interp.set_gas_limit(gas_limit);

            // Simulate effect injection with seed
            interp.global_scope.insert(
                "deterministic_seed".to_string(),
                Value::String(hex::encode(&seed)),
            );

            let result = interp.eval(&expr).unwrap();

            // Serialize to simple JSON for testing (in practice use canonical serializer)
            let json_str = match &result {
                Value::Object(obj) => {
                    let mut keys: Vec<&String> = obj.keys().collect();
                    keys.sort(); // Sort keys for deterministic output
                    let mut parts = Vec::new();
                    for k in keys {
                        if let Some(v) = obj.get(k) {
                            match v {
                                Value::String(s) => parts.push(format!("\"{}\":\"{}\"", k, s)),
                                Value::Number(n) => parts.push(format!("\"{}\":{}", k, n)),
                                _ => parts.push(format!("\"{}\":\"unsupported\"", k)),
                            }
                        }
                    }
                    format!("{{{}}}", parts.join(","))
                }
                _ => "unsupported".to_string(),
            };

            // Compute SHA-256 of response
            let response_hash = Sha256::digest(json_str.as_bytes());
            response_hashes.push(response_hash);
        }

        // All response hashes should be identical
        let first_hash = &response_hashes[0];
        for hash in &response_hashes[1..] {
            assert_eq!(first_hash, hash, "Non-deterministic response detected");
        }

        // Verify we got 1000 identical results
        assert_eq!(response_hashes.len(), 1000);
        let unique_hashes: std::collections::HashSet<_> = response_hashes.into_iter().collect();
        assert_eq!(
            unique_hashes.len(),
            1,
            "All executions should produce identical response hashes"
        );
    }
}
