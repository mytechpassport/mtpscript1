use mtpscript_core::effects::checker::check_program_effects;
use mtpscript_core::errors::compile::CompileError;
use mtpscript_core::lexer::scanner::Scanner;
use mtpscript_core::parser::Parser;

fn parse_and_check_effects(source: &str) -> Result<(), CompileError> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;
    let mut parser = Parser::new(&tokens);
    let program = parser.parse()?;
    check_program_effects(&program)
}

#[test]
fn test_declared_effects() {
    // This would test that declared effects are allowed
    // For now, placeholder - effects checking not fully implemented
    let source = r#"api POST "/users" uses { DbWrite } { respond json(true) }"#;
    let result = parse_and_check_effects(source);
    // Should pass once effects checking is implemented
    assert!(result.is_ok());
}

#[test]
fn test_undeclared_effects() {
    // Test that undeclared effects cause errors
    let source = r#"api POST "/users" uses { DbWrite } { DbRead("query") }"#;
    let result = parse_and_check_effects(source);
    // Should fail because DbRead is not declared
    assert!(result.is_err());
}

#[test]
fn test_lambda_no_effects() {
    // This would test that lambda expressions cannot use effects
    // For now, placeholder - no lambda parsing yet
    let source = r#"api GET "/test" { respond json(true) }"#;
    let result = parse_and_check_effects(source);
    assert!(result.is_ok());
}

#[test]
fn test_async_await() {
    // This would test async effect usage
    // For now, placeholder - async effects not implemented
    let source = r#"api GET "/async" uses { Async } { someAsyncCall() }"#;
    let result = parse_and_check_effects(source);
    // Should pass once async effects are implemented
    assert!(result.is_ok());
}
