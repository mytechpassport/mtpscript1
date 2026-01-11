use mtpscript_core::errors::compile::CompileError;
use mtpscript_core::lexer::scanner::Scanner;
use mtpscript_core::lexer::token::Token;

fn lex_tokens(source: &str) -> Result<Vec<Token>, CompileError> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;
    Ok(tokens.into_iter().map(|ti| ti.token).collect())
}

#[test]
fn test_keywords() {
    assert_eq!(
        lex_tokens("function type api const if else match await uses respond import").unwrap(),
        vec![
            Token::Function,
            Token::Type,
            Token::Api,
            Token::Const,
            Token::If,
            Token::Else,
            Token::Match,
            Token::Await,
            Token::Uses,
            Token::Respond,
            Token::Import,
            Token::Eof
        ]
    );
}

#[test]
fn test_operators() {
    assert_eq!(
        lex_tokens("+ - * / == != < > <= >= && || ! . |> =>").unwrap(),
        vec![
            Token::Plus,
            Token::Minus,
            Token::Star,
            Token::Slash,
            Token::EqualEqual,
            Token::BangEqual,
            Token::Less,
            Token::Greater,
            Token::LessEqual,
            Token::GreaterEqual,
            Token::AndAnd,
            Token::OrOr,
            Token::Bang,
            Token::Dot,
            Token::PipeGreater,
            Token::Arrow,
            Token::Eof
        ]
    );
}

#[test]
fn test_literals() {
    assert_eq!(
        lex_tokens("42 3.14 \"hello\" true false").unwrap(),
        vec![
            Token::Number(42),
            Token::Decimal("3.14".to_string()),
            Token::String("hello".to_string()),
            Token::Boolean(true),
            Token::Boolean(false),
            Token::Eof
        ]
    );
}

#[test]
fn test_identifiers() {
    assert_eq!(
        lex_tokens("foo bar_baz _private camelCase").unwrap(),
        vec![
            Token::Ident("foo".to_string()),
            Token::Ident("bar_baz".to_string()),
            Token::Ident("_private".to_string()),
            Token::Ident("camelCase".to_string()),
            Token::Eof
        ]
    );
}

#[test]
fn test_http_methods() {
    assert_eq!(
        lex_tokens("GET POST PUT DELETE PATCH").unwrap(),
        vec![
            Token::Get,
            Token::Post,
            Token::Put,
            Token::Delete,
            Token::Patch,
            Token::Eof
        ]
    );
}

#[test]
fn test_delimiters() {
    assert_eq!(
        lex_tokens("( ) { } [ ] , : ;").unwrap(),
        vec![
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::RBrace,
            Token::LBracket,
            Token::RBracket,
            Token::Comma,
            Token::Colon,
            Token::Semicolon,
            Token::Eof
        ]
    );
}

#[test]
fn test_error_cases() {
    // Invalid number starting with decimal
    assert!(lex_tokens(".5").is_err());

    // Unterminated string
    assert!(lex_tokens("\"unterminated").is_err());

    // Unexpected characters
    assert!(lex_tokens("@").is_err());

    // Invalid operators
    assert!(lex_tokens("=").is_err());
    assert!(lex_tokens("&").is_err());
    assert!(lex_tokens("|").is_err());
}
