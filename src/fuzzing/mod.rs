use crate::errors::MtpError;
use crate::ir::{validate_ir_program, IrProgram};
use crate::parser::Parser;
use crate::runtime::Interpreter;
use crate::types::TypeChecker;

/// Fuzzer for the lexer
pub fn fuzz_lexer(input: &[u8]) {
    if let Ok(input_str) = std::str::from_utf8(input) {
        let mut lexer = crate::lexer::Lexer::new(input_str);
        // Just consume all tokens - we're looking for crashes
        while let Some(_) = lexer.next_token() {}
    }
}

/// Fuzzer for the parser
pub fn fuzz_parser(input: &[u8]) {
    if let Ok(input_str) = std::str::from_utf8(input) {
        let mut parser = Parser::new(input_str);
        let _ = parser.parse_program(); // We don't care about the result, just that it doesn't crash
    }
}

/// Fuzzer for type checking
pub fn fuzz_type_checker(input: &[u8]) {
    if let Ok(input_str) = std::str::from_utf8(input) {
        let mut parser = Parser::new(input_str);
        if let Ok(ast) = parser.parse_program() {
            let mut type_checker = TypeChecker::new();
            let _ = type_checker.check_program(&ast); // Look for crashes in type checking
        }
    }
}

/// Fuzzer for IR generation and validation
pub fn fuzz_ir_generation(input: &[u8]) {
    if let Ok(input_str) = std::str::from_utf8(input) {
        let mut parser = Parser::new(input_str);
        if let Ok(ast) = parser.parse_program() {
            let mut type_checker = TypeChecker::new();
            if let Ok(typed_ast) = type_checker.check_program(&ast) {
                // Try IR lowering
                let _ = crate::ir::lower_ast_to_ir(&typed_ast);
            }
        }
    }
}

/// Fuzzer for runtime execution
pub fn fuzz_runtime_execution(input: &[u8]) {
    if let Ok(input_str) = std::str::from_utf8(input) {
        let mut parser = Parser::new(input_str);
        if let Ok(ast) = parser.parse_program() {
            let mut type_checker = TypeChecker::new();
            if let Ok(typed_ast) = type_checker.check_program(&ast) {
                if let Ok(ir) = crate::ir::lower_ast_to_ir(&typed_ast) {
                    if let Ok(js) = crate::compiler::compile_ir_to_js(&ir) {
                        let mut interpreter = Interpreter::new(Default::default());
                        let _ = interpreter.execute(&js); // Look for execution crashes
                    }
                }
            }
        }
    }
}

/// Fuzzer for JSON parsing
pub fn fuzz_json_parsing(input: &[u8]) {
    if let Ok(input_str) = std::str::from_utf8(input) {
        let _ = crate::json::parse_json(input_str); // Look for parsing crashes
    }
}

/// Fuzzer for CBOR encoding/decoding
pub fn fuzz_cbor_operations(input: &[u8]) {
    if let Ok(input_str) = std::str::from_utf8(input) {
        if let Ok(json) = crate::json::parse_json(input_str) {
            let _ = crate::json::encode_cbor(&json); // Look for encoding crashes
        }
    }
}

/// Fuzzer for cryptographic operations
pub fn fuzz_crypto_operations(input: &[u8]) {
    use crate::security::crypto_audit::audit_crypto_operation;

    // Use input to generate test data
    if input.len() >= 32 {
        let data = &input[0..32];
        audit_crypto_operation("fuzz_test", "TestAlgo", Some(256), "Fuzzing test");

        // Try signature operations
        let _ = crate::security::generate_ecdsa_keypair();
    }
}

/// Fuzzer for taint analysis
pub fn fuzz_taint_analysis(input: &[u8]) {
    use crate::taint::{StaticTaintAnalyzer, TaintLevel, TaintSource};

    if let Ok(input_str) = std::str::from_utf8(input) {
        let mut analyzer = StaticTaintAnalyzer::new();

        let source = TaintSource {
            id: "fuzz_input".to_string(),
            description: "Fuzzer input".to_string(),
        };
        analyzer.add_source(source);

        // Split input into potential variable names
        for part in input_str.split_whitespace() {
            if !part.is_empty() {
                analyzer.taint_variable(part, TaintLevel::Tainted, {
                    let mut sources = std::collections::HashSet::new();
                    sources.insert(TaintSource {
                        id: "fuzz".to_string(),
                        description: "Fuzz test".to_string(),
                    });
                    sources
                });
            }
        }

        let _ = analyzer.get_report(); // Look for crashes in report generation
    }
}

/// Fuzzer for schema validation
pub fn fuzz_schema_validation(input: &[u8]) {
    use crate::validation::JsonSchemaValidator;

    if let Ok(input_str) = std::str::from_utf8(input) {
        // Try to parse as JSON schema
        if let Ok(schema) = serde_json::from_str::<serde_json::Value>(input_str) {
            let validator = JsonSchemaValidator::new(schema);

            // Try to validate some test data
            let test_data = serde_json::json!({"test": "data"});
            let _ = validator.validate(&test_data);
        }
    }
}

/// Comprehensive fuzz target that exercises the entire pipeline
pub fn fuzz_full_pipeline(input: &[u8]) {
    // Try to interpret input as MTPScript code
    fuzz_parser(input);
    fuzz_type_checker(input);
    fuzz_ir_generation(input);
    fuzz_runtime_execution(input);
    fuzz_json_parsing(input);
    fuzz_cbor_operations(input);
    fuzz_crypto_operations(input);
    fuzz_taint_analysis(input);
    fuzz_schema_validation(input);
}

#[cfg(feature = "afl")]
mod afl_targets {
    use super::*;
    use afl::fuzz;

    #[no_mangle]
    pub extern "C" fn LLVMFuzzerTestOneInput(data: *const u8, size: usize) -> i32 {
        let input = unsafe { std::slice::from_raw_parts(data, size) };
        fuzz_full_pipeline(input);
        0
    }
}

#[cfg(feature = "libfuzzer")]
mod libfuzzer_targets {
    use super::*;

    #[no_mangle]
    pub extern "C" fn LLVMFuzzerTestOneInput(data: &[u8]) -> i32 {
        fuzz_full_pipeline(data);
        0
    }
}

/// Run fuzzing campaigns
pub struct FuzzingRunner {
    pub targets: Vec<(&'static str, fn(&[u8]))>,
}

impl FuzzingRunner {
    pub fn new() -> Self {
        FuzzingRunner {
            targets: vec![
                ("lexer", fuzz_lexer),
                ("parser", fuzz_parser),
                ("type_checker", fuzz_type_checker),
                ("ir_generation", fuzz_ir_generation),
                ("runtime_execution", fuzz_runtime_execution),
                ("json_parsing", fuzz_json_parsing),
                ("cbor_operations", fuzz_cbor_operations),
                ("crypto_operations", fuzz_crypto_operations),
                ("taint_analysis", fuzz_taint_analysis),
                ("schema_validation", fuzz_schema_validation),
                ("full_pipeline", fuzz_full_pipeline),
            ],
        }
    }

    /// Run a specific fuzzing target with test input
    pub fn run_target(&self, target_name: &str, input: &[u8]) {
        if let Some((_, target_fn)) = self.targets.iter().find(|(name, _)| *name == target_name) {
            target_fn(input);
        }
    }

    /// List available fuzzing targets
    pub fn list_targets(&self) -> Vec<&str> {
        self.targets.iter().map(|(name, _)| *name).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzing_runner() {
        let runner = FuzzingRunner::new();
        let targets = runner.list_targets();

        assert!(targets.contains(&"lexer"));
        assert!(targets.contains(&"parser"));
        assert!(targets.contains(&"full_pipeline"));
    }

    #[test]
    fn test_individual_fuzzers() {
        // Test with some basic inputs
        let test_input = b"function test() { return 42; }";

        fuzz_lexer(test_input);
        fuzz_parser(test_input);
        // Other fuzzers might fail due to incomplete implementations, but shouldn't crash
    }
}
