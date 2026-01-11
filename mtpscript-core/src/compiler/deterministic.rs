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
}
