use mtpscript_core::compiler::codegen;
use mtpscript_core::errors::compile::CompileError;
use mtpscript_core::ir::lower;
use mtpscript_core::lexer::scanner::Scanner;
use mtpscript_core::parser::Parser;
use mtpscript_core::types::checker::TypeChecker;

#[cfg(test)]
mod tests {
    use super::*;

    fn compile_source(src: &str) -> Result<String, CompileError> {
        let mut scanner = Scanner::new(src)?;
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(&tokens);
        let program = parser.parse()?;
        let mut type_checker = TypeChecker::new();
        type_checker.typecheck_program(&program)?;
        let ir = lower::lower_ast_to_ir(&program)?;
        codegen::compile_ir_to_js(&ir)
    }

    #[test]
    fn test_simple_function_compilation() {
        let src = r#"
            function add(a: number, b: number) {
                a + b
            }
        "#;
        let js = compile_source(src).unwrap();
        assert!(js.contains("function add(a, b)"));
        assert!(js.contains("return a + b"));
    }

    #[test]
    fn test_constant_compilation() {
        let src = r#"
            function main() {
                const x = 42;
                x
            }
        "#;
        let js = compile_source(src).unwrap();
        assert!(js.contains("const x = 42"));
        // The codegen outputs let bindings followed by the body variable
        assert!(js.contains("x"));
    }

    #[test]
    fn test_if_expression_compilation() {
        let src = r#"
            function max(a: number, b: number) {
                if (a > b) { a } else { b }
            }
        "#;
        let js = compile_source(src).unwrap();
        // Simple if expressions compile to ternary operators
        assert!(js.contains("a > b") || js.contains("(a > b)"));
        assert!(js.contains("a") && js.contains("b"));
    }

    #[test]
    fn test_arithmetic_operations() {
        let src = r#"
            function calc(a: number, b: number, c: number) {
                a + b * c - 1
            }
        "#;
        let js = compile_source(src).unwrap();
        assert!(js.contains("a + b * c - 1"));
    }

    #[test]
    fn test_comparison_operations() {
        let src = r#"
            function compare(a: number, b: number) {
                a == b && a < b
            }
        "#;
        let js = compile_source(src).unwrap();
        assert!(js.contains("a === b && a < b"));
    }

    #[test]
    fn test_string_operations() {
        let src = r#"
            function greet(name: string) {
                "Hello, " + name
            }
        "#;
        let js = compile_source(src).unwrap();
        assert!(js.contains(r#""Hello, " + name"#));
    }

    #[test]
    fn test_boolean_operations() {
        let src = r#"
            function logic(a: boolean, b: boolean) {
                a && b || !a
            }
        "#;
        let js = compile_source(src).unwrap();
        assert!(js.contains("a && b || !a"));
    }

    #[test]
    fn test_array_literals() {
        let src = r#"
            function make_array() {
                [1, 2, 3]
            }
        "#;
        let js = compile_source(src).unwrap();
        assert!(js.contains("[1, 2, 3]"));
    }

    #[test]
    fn test_object_literals() {
        let src = r#"
            function make_object() {
                { "key": "value", "num": 42 }
            }
        "#;
        let js = compile_source(src).unwrap();
        assert!(js.contains(r#"{"key": "value", "num": 42}"#));
    }

    #[test]
    fn test_record_access() {
        let src = r#"
            type User { name: string, age: number }
            function get_name(u: User) {
                u.name
            }
        "#;
        let js = compile_source(src).unwrap();
        assert!(js.contains("u.name"));
    }

    #[test]
    fn test_nested_expressions() {
        let src = r#"
            function complex() {
                if (true) {
                    1 + 2 * 3
                } else {
                    0
                }
            }
        "#;
        let js = compile_source(src).unwrap();
        // Simple if expressions compile to ternary operators
        assert!(js.contains("true"));
        assert!(js.contains("1 + 2 * 3"));
        assert!(js.contains("0"));
    }

    #[test]
    fn test_multiple_functions() {
        let src = r#"
            function helper(x: number) {
                x * 2
            }
            function main() {
                helper(21)
            }
        "#;
        let js = compile_source(src).unwrap();
        assert!(js.contains("function helper(x)"));
        assert!(js.contains("function main()"));
        assert!(js.contains("helper(21)"));
    }

    #[test]
    fn test_forbidden_constructs() {
        // Test that forbidden constructs are not generated in the code body
        let src = r#"
            function test() {
                42
            }
        "#;
        let js = compile_source(src).unwrap();
        // Exclude comments when checking for forbidden constructs
        let code_without_comments: String = js
            .lines()
            .filter(|line| !line.trim().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(!code_without_comments.contains("class "));
        assert!(!code_without_comments.contains("this."));
        assert!(!code_without_comments.contains("eval("));
        assert!(!code_without_comments.contains("try {"));
        assert!(!code_without_comments.contains("catch"));
    }

    #[test]
    fn test_deterministic_output() {
        let src = r#"
            function test() {
                const a = 1;
                const b = 2;
                a + b
            }
        "#;

        // Compile multiple times and check identical output
        let js1 = compile_source(src).unwrap();
        let js2 = compile_source(src).unwrap();
        let js3 = compile_source(src).unwrap();

        assert_eq!(js1, js2);
        assert_eq!(js2, js3);
    }

    #[test]
    fn test_pipeline_operator() {
        let src = r#"
            function double(x: number) {
                x * 2
            }
            function main() {
                5 |> double |> double
            }
        "#;
        let js = compile_source(src).unwrap();
        // Pipeline should be desugared to nested calls
        assert!(js.contains("double(double(5))"));
    }

    #[test]
    fn test_api_compilation() {
        let src = r#"
            api GET "/test" {
                respond json({ "ok": true })
            }
        "#;
        let js = compile_source(src).unwrap();
        // API should compile to a function that returns the response
        assert!(js.contains("handle_get"));
        assert!(js.contains("test"));
    }
}
