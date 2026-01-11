use mtpscript_core::parser::mod::parse;
use mtpscript_core::ir::lower::lower_ast_to_ir;
use mtpscript_core::compiler::codegen::compile_ir_to_js;

fn main() {
    // Test simple function compilation
    let source = r#"
        function add(a: number, b: number): number {
            a + b
        }
    "#;
    
    let ast = parse(source).expect("Failed to parse");
    let ir = lower_ast_to_ir(&ast).expect("Failed to lower to IR");
    let js = compile_ir_to_js(&ir).expect("Failed to compile to JS");
    
    println!("Generated JS:");
    println!("{}", js);
    
    // Check that it contains expected elements
    assert!(js.contains("function add(a, b)"));
    assert!(js.contains("return a + b"));
    assert!(!js.contains("class"));
    assert!(!js.contains("eval"));
    
    println!("✅ Code generation test passed!");
}
