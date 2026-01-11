pub mod scanner;
pub mod token;

pub use scanner::*;
pub use token::*;

/// Lex source code into tokens
pub fn lex(source: &str) -> Result<Vec<Token>, crate::errors::MtpError> {
    let mut scanner = Scanner::new(source);
    let mut tokens = Vec::new();

    loop {
        let token = scanner.scan_token()?;
        let should_break = matches!(token.kind, TokenKind::Eof);
        tokens.push(token);
        if should_break {
            break;
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_keywords() {
        let tokens = lex("function type api").unwrap();
        assert_eq!(tokens.len(), 4); // 3 tokens + EOF
        assert_eq!(tokens[0].kind, TokenKind::Function);
        assert_eq!(tokens[1].kind, TokenKind::Type);
        assert_eq!(tokens[2].kind, TokenKind::Api);
    }

    #[test]
    fn test_lex_identifiers() {
        let tokens = lex("foo bar").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Identifier("foo".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Identifier("bar".to_string()));
    }

    #[test]
    fn test_lex_strings() {
        let tokens = lex("\"hello\"").unwrap();
        match &tokens[0].kind {
            TokenKind::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected string token"),
        }
    }

    #[test]
    fn test_lex_numbers() {
        let tokens = lex("42 3.14").unwrap();
        match &tokens[0].kind {
            TokenKind::Number(s) => assert_eq!(s, "42"),
            _ => panic!("Expected number token"),
        }
        match &tokens[1].kind {
            TokenKind::Number(s) => assert_eq!(s, "3.14"),
            _ => panic!("Expected number token"),
        }
    }
}
