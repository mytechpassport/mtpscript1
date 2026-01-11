use mtpscript_core::errors::compile::CompileError;
use mtpscript_core::lexer::scanner::Scanner;
use mtpscript_core::parser::ast::*;
use mtpscript_core::parser::Parser;

fn parse_source(source: &str) -> Result<Program, CompileError> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;
    let mut parser = Parser::new(&tokens);
    parser.parse()
}

#[test]
fn test_api_decl() {
    let source = r#"api POST "/users" uses { DbWrite, Log } { respond json(true) }"#;

    let result = parse_source(source);
    assert!(result.is_ok());

    let program = result.unwrap();
    assert_eq!(program.decls.len(), 1);

    match &program.decls[0] {
        ModuleDecl::ApiDecl(api) => {
            assert_eq!(api.method, HttpMethod::Post);
            assert_eq!(api.path, "/users");
            assert_eq!(api.effects, vec!["DbWrite".to_string(), "Log".to_string()]);
            // The body parsing is tested in expressions
        }
    }
}

#[test]
fn test_expressions() {
    // Test basic expressions
    let source = r#"api GET "/test" { true }"#;
    let program = parse_source(source).unwrap();
    match &program.decls[0] {
        ModuleDecl::ApiDecl(api) => {
            assert_eq!(api.body, Expr::Boolean(true));
        }
    }

    // Test string expression
    let source = r#"api GET "/test" { "hello" }"#;
    let program = parse_source(source).unwrap();
    match &program.decls[0] {
        ModuleDecl::ApiDecl(api) => {
            assert_eq!(api.body, Expr::String("hello".to_string()));
        }
    }

    // Test number expression
    let source = r#"api GET "/test" { 42 }"#;
    let program = parse_source(source).unwrap();
    match &program.decls[0] {
        ModuleDecl::ApiDecl(api) => {
            assert_eq!(api.body, Expr::Number(42));
        }
    }

    // Test identifier expression
    let source = r#"api GET "/test" { result }"#;
    let program = parse_source(source).unwrap();
    match &program.decls[0] {
        ModuleDecl::ApiDecl(api) => {
            assert_eq!(api.body, Expr::Ident("result".to_string()));
        }
    }

    // Test function call
    let source = r#"api GET "/test" { add(a, b) }"#;
    let program = parse_source(source).unwrap();
    match &program.decls[0] {
        ModuleDecl::ApiDecl(api) => match &api.body {
            Expr::Call { func, args } => {
                assert_eq!(func, "add");
                assert_eq!(args.len(), 2);
                assert_eq!(args[0], Expr::Ident("a".to_string()));
                assert_eq!(args[1], Expr::Ident("b".to_string()));
            }
            _ => panic!("Expected function call"),
        },
    }

    // Test respond json expression
    let source = r#"api GET "/test" { respond json(true) }"#;
    let program = parse_source(source).unwrap();
    match &program.decls[0] {
        ModuleDecl::ApiDecl(api) => match &api.body {
            Expr::RespondJson(inner) => {
                assert_eq!(**inner, Expr::Boolean(true));
            }
            _ => panic!("Expected RespondJson"),
        },
    }
}

#[test]
fn test_api_decl_without_effects() {
    let source = r#"api GET "/status" { respond json("ok") }"#;

    let result = parse_source(source);
    assert!(result.is_ok());

    let program = result.unwrap();
    assert_eq!(program.decls.len(), 1);

    match &program.decls[0] {
        ModuleDecl::ApiDecl(api) => {
            assert_eq!(api.method, HttpMethod::Get);
            assert_eq!(api.path, "/status");
            assert_eq!(api.effects.len(), 0); // No effects specified
        }
    }
}

#[test]
fn test_multiple_api_decls() {
    let source = r#"
        api GET "/users" { respond json("list") }
        api POST "/users" uses { DbWrite } { respond json("created") }
    "#;

    let result = parse_source(source);
    assert!(result.is_ok());

    let program = result.unwrap();
    assert_eq!(program.decls.len(), 2);

    match &program.decls[0] {
        ModuleDecl::ApiDecl(api) => {
            assert_eq!(api.method, HttpMethod::Get);
            assert_eq!(api.path, "/users");
        }
    }

    match &program.decls[1] {
        ModuleDecl::ApiDecl(api) => {
            assert_eq!(api.method, HttpMethod::Post);
            assert_eq!(api.path, "/users");
            assert_eq!(api.effects, vec!["DbWrite".to_string()]);
        }
    }
}
