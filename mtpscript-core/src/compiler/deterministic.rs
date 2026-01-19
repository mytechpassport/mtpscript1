use std::collections::HashMap;

/// Ensure deterministic code generation
/// - Sort declarations by name
/// - Use consistent variable naming
pub fn make_deterministic(js: &str) -> String {
    // For now, assume input is already deterministic
    // In full implementation, parse JS AST and reorder/sort
    js.to_string()
}

/// Generate unique variable names deterministically
pub struct NameGenerator {
    counter: HashMap<String, usize>,
}

impl NameGenerator {
    pub fn new() -> Self {
        Self {
            counter: HashMap::new(),
        }
    }

    pub fn fresh_name(&mut self, base: &str) -> String {
        let count = self.counter.entry(base.to_string()).or_insert(0);
        *count += 1;
        format!("{}_{}", base, count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_generator() {
        let mut gen = NameGenerator::new();
        assert_eq!(gen.fresh_name("var"), "var_1");
        assert_eq!(gen.fresh_name("var"), "var_2");
        assert_eq!(gen.fresh_name("tmp"), "tmp_1");
    }

    #[test]
    fn test_name_generator_deterministic_across_instances() {
        // Multiple generators should produce the same sequence for the same inputs
        let mut gen1 = NameGenerator::new();
        let mut gen2 = NameGenerator::new();

        // Same sequence of calls should produce same outputs
        let names1: Vec<String> = (0..10).map(|_| gen1.fresh_name("x")).collect();
        let names2: Vec<String> = (0..10).map(|_| gen2.fresh_name("x")).collect();

        assert_eq!(names1, names2);
    }

    #[test]
    fn test_name_generator_different_bases() {
        let mut gen = NameGenerator::new();

        // Different bases should have independent counters
        assert_eq!(gen.fresh_name("a"), "a_1");
        assert_eq!(gen.fresh_name("b"), "b_1");
        assert_eq!(gen.fresh_name("a"), "a_2");
        assert_eq!(gen.fresh_name("c"), "c_1");
        assert_eq!(gen.fresh_name("b"), "b_2");
    }

    #[test]
    fn test_name_generator_special_characters() {
        let mut gen = NameGenerator::new();

        // Base names with various characters
        assert_eq!(gen.fresh_name("var_x"), "var_x_1");
        assert_eq!(gen.fresh_name("_private"), "_private_1");
        assert_eq!(gen.fresh_name("CamelCase"), "CamelCase_1");
    }

    #[test]
    fn test_make_deterministic_idempotent() {
        let js = "function test() { return 42; }";

        // Applying make_deterministic multiple times should give same result
        let result1 = make_deterministic(js);
        let result2 = make_deterministic(&result1);
        let result3 = make_deterministic(&result2);

        assert_eq!(result1, result2);
        assert_eq!(result2, result3);
    }

    #[test]
    fn test_deterministic_output_consistency() {
        // Run the same transformation 100 times to ensure consistency
        let js = r#"
            function add(a, b) { return a + b; }
            function mul(a, b) { return a * b; }
        "#;

        let first_result = make_deterministic(js);

        for _ in 0..100 {
            let result = make_deterministic(js);
            assert_eq!(
                result, first_result,
                "Output must be deterministic across runs"
            );
        }
    }

    #[test]
    fn test_name_generator_large_count() {
        let mut gen = NameGenerator::new();

        // Generate many names and verify they're sequential
        for i in 1..=1000 {
            let name = gen.fresh_name("test");
            assert_eq!(name, format!("test_{}", i));
        }
    }

    #[test]
    fn test_name_generator_empty_base() {
        let mut gen = NameGenerator::new();

        // Empty base should still work
        assert_eq!(gen.fresh_name(""), "_1");
        assert_eq!(gen.fresh_name(""), "_2");
    }

    #[test]
    fn test_deterministic_with_various_inputs() {
        // Test with different types of JS code
        let inputs = vec![
            "42;",
            "function f() {}",
            "const x = 1; const y = 2;",
            "if (true) { return 1; } else { return 2; }",
            r#"{"key": "value"}"#,
        ];

        for input in inputs {
            let result1 = make_deterministic(input);
            let result2 = make_deterministic(input);
            assert_eq!(
                result1, result2,
                "make_deterministic should be consistent for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_name_generator_concurrent_simulation() {
        // Simulate what would happen with multiple "threads" generating names
        // All should produce deterministic results based on order of calls

        let mut gen = NameGenerator::new();
        let mut results = Vec::new();

        // Interleaved calls to different bases
        for i in 0..10 {
            if i % 2 == 0 {
                results.push(gen.fresh_name("even"));
            } else {
                results.push(gen.fresh_name("odd"));
            }
        }

        // Verify the expected interleaved pattern
        assert_eq!(results[0], "even_1");
        assert_eq!(results[1], "odd_1");
        assert_eq!(results[2], "even_2");
        assert_eq!(results[3], "odd_2");
        // etc.

        // A new generator with the same call sequence should produce same results
        let mut gen2 = NameGenerator::new();
        let mut results2 = Vec::new();

        for i in 0..10 {
            if i % 2 == 0 {
                results2.push(gen2.fresh_name("even"));
            } else {
                results2.push(gen2.fresh_name("odd"));
            }
        }

        assert_eq!(results, results2);
    }
}
