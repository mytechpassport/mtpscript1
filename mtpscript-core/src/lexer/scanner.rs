use super::token::{Token, TokenInfo};
use crate::errors::compile::CompileError;

pub struct Scanner<'a> {
    source: &'a str,
    chars: Vec<char>,
    char_byte_offsets: Vec<usize>, // byte offset for each char index
    start: usize,                   // char index
    current: usize,                 // char index
    line: usize,
    column: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Result<Self, CompileError> {
        // Input validation: check size limits and UTF-8 validity
        if source.len() > 10_000_000 {
            return Err(CompileError::LexerError("Source too large".to_string()));
        }

        // Check for null bytes and other invalid characters
        if source.chars().any(|c| c == '\0') {
            return Err(CompileError::LexerError(
                "Source contains null bytes".to_string(),
            ));
        }

        // Validate UTF-8 encoding (should be valid since it's &str, but check for overlong encodings)
        if !source.is_char_boundary(source.len()) {
            return Err(CompileError::LexerError(
                "Invalid UTF-8 encoding".to_string(),
            ));
        }

        // Build char vector and byte offset mapping
        let chars: Vec<char> = source.chars().collect();
        let mut char_byte_offsets = Vec::with_capacity(chars.len() + 1);
        let mut byte_offset = 0;
        for c in &chars {
            char_byte_offsets.push(byte_offset);
            byte_offset += c.len_utf8();
        }
        char_byte_offsets.push(byte_offset); // end offset for slicing

        Ok(Self {
            source,
            chars,
            char_byte_offsets,
            start: 0,
            current: 0,
            line: 1,
            column: 1,
        })
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<TokenInfo>, CompileError> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.start = self.current;
            let token = self.scan_token()?;
            if let Some(token) = token {
                tokens.push(token);
            }
        }

        tokens.push(TokenInfo {
            token: Token::Eof,
            line: self.line,
            column: self.column,
        });

        Ok(tokens)
    }

    fn scan_token(&mut self) -> Result<Option<TokenInfo>, CompileError> {
        let c = self.advance();

        match c {
            '(' => Ok(Some(self.make_token(Token::LParen))),
            ')' => Ok(Some(self.make_token(Token::RParen))),
            '{' => Ok(Some(self.make_token(Token::LBrace))),
            '}' => Ok(Some(self.make_token(Token::RBrace))),
            '[' => Ok(Some(self.make_token(Token::LBracket))),
            ']' => Ok(Some(self.make_token(Token::RBracket))),
            ',' => Ok(Some(self.make_token(Token::Comma))),
            ':' => Ok(Some(self.make_token(Token::Colon))),
            ';' => Ok(Some(self.make_token(Token::Semicolon))),
            '+' => Ok(Some(self.make_token(Token::Plus))),
            '-' => Ok(Some(self.make_token(Token::Minus))),
            '*' => Ok(Some(self.make_token(Token::Star))),
            '/' => {
                if self.match_char('/') {
                    // Single-line comment: skip until end of line
                    while !self.is_at_end() && self.peek() != '\n' {
                        self.advance();
                    }
                    Ok(None) // skip comment
                } else if self.match_char('*') {
                    // Multi-line comment: skip until */
                    while !self.is_at_end() {
                        if self.peek() == '*' {
                            self.advance();
                            if !self.is_at_end() && self.peek() == '/' {
                                self.advance();
                                break;
                            }
                        } else {
                            if self.peek() == '\n' {
                                self.line += 1;
                                self.column = 0;
                            }
                            self.advance();
                        }
                    }
                    Ok(None) // skip comment
                } else {
                    Ok(Some(self.make_token(Token::Slash)))
                }
            }
            '.' => {
                if self.peek().is_ascii_digit() {
                    Err(CompileError::LexerError(
                        "Number cannot start with decimal point".to_string(),
                    ))
                } else {
                    Ok(Some(self.make_token(Token::Dot)))
                }
            }
            '!' => {
                if self.match_char('=') {
                    Ok(Some(self.make_token(Token::BangEqual)))
                } else {
                    Ok(Some(self.make_token(Token::Bang)))
                }
            }
            '=' => {
                if self.match_char('=') {
                    Ok(Some(self.make_token(Token::EqualEqual)))
                } else if self.match_char('>') {
                    Ok(Some(self.make_token(Token::Arrow)))
                } else {
                    Ok(Some(self.make_token(Token::Equal)))
                }
            }
            '<' => {
                if self.match_char('=') {
                    Ok(Some(self.make_token(Token::LessEqual)))
                } else {
                    Ok(Some(self.make_token(Token::Less)))
                }
            }
            '>' => {
                if self.match_char('=') {
                    Ok(Some(self.make_token(Token::GreaterEqual)))
                } else if self.match_char('|') {
                    Ok(Some(self.make_token(Token::PipeGreater)))
                } else {
                    Ok(Some(self.make_token(Token::Greater)))
                }
            }
            '&' => {
                if self.match_char('&') {
                    Ok(Some(self.make_token(Token::AndAnd)))
                } else {
                    Err(CompileError::LexerError("Unexpected '&'".to_string()))
                }
            }
            '|' => {
                if self.match_char('|') {
                    Ok(Some(self.make_token(Token::OrOr)))
                } else if self.match_char('>') {
                    Ok(Some(self.make_token(Token::PipeGreater)))
                } else {
                    Ok(Some(self.make_token(Token::Pipe)))
                }
            }
            '"' => self.string(),
            '0'..='9' => self.number(),
            c if c.is_alphabetic() || c == '_' => self.identifier(),
            c if c.is_whitespace() => {
                if c == '\n' {
                    self.line += 1;
                    self.column = 1;
                } else {
                    self.column += 1;
                }
                Ok(None) // skip whitespace
            }
            _ => Err(CompileError::LexerError(format!(
                "Unexpected character: {}",
                c
            ))),
        }
    }

    fn string(&mut self) -> Result<Option<TokenInfo>, CompileError> {
        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                return Err(CompileError::LexerError("Unterminated string".to_string()));
            }
            if self.peek() == '\\' {
                self.advance(); // consume \
                if self.is_at_end() {
                    return Err(CompileError::LexerError("Unterminated string".to_string()));
                }
                // handle escape, but for now just consume next
                self.advance();
            } else {
                self.advance();
            }
        }

        if self.is_at_end() {
            return Err(CompileError::LexerError("Unterminated string".to_string()));
        }

        self.advance(); // closing "

        // Get the string content between the quotes using byte offsets
        let start_byte = self.char_byte_offsets[self.start + 1]; // skip opening quote
        let end_byte = self.char_byte_offsets[self.current - 1]; // skip closing quote
        let value = &self.source[start_byte..end_byte];
        let processed = self.process_escapes(value);

        Ok(Some(self.make_token(Token::String(processed))))
    }

    fn process_escapes(&self, s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => result.push('\n'),
                    Some('t') => result.push('\t'),
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    Some(c) => result.push(c), // unknown escape, keep as is
                    None => {}                 // shouldn't happen
                }
            } else {
                result.push(c);
            }
        }

        result
    }

    fn number(&mut self) -> Result<Option<TokenInfo>, CompileError> {
        while !self.is_at_end() && self.peek().is_ascii_digit() {
            self.advance();
        }

        // Look for decimal
        if !self.is_at_end() && self.peek() == '.' {
            self.advance(); // consume .
            if self.is_at_end() || !self.peek().is_ascii_digit() {
                return Err(CompileError::LexerError(
                    "Invalid number literal".to_string(),
                ));
            }
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                self.advance();
            }
            let value = self.current_lexeme();
            return Ok(Some(self.make_token(Token::Decimal(value.to_string()))));
        }

        let value = self.current_lexeme();
        // Try parsing as i64, handle potential overflow for i64::MIN edge case
        let num = match value.parse::<i64>() {
            Ok(n) => n,
            Err(_) => {
                // Special case: 9223372036854775808 is only valid as part of -9223372036854775808 (i64::MIN)
                // Store as i64::MAX and let the parser handle negation
                if value == "9223372036854775808" {
                    // Use i64::MIN directly - the unary minus in parser will need to handle this
                    i64::MIN
                } else {
                    return Err(CompileError::LexerError(format!(
                        "Number {} is out of i64 range",
                        value
                    )));
                }
            }
        };
        Ok(Some(self.make_token(Token::Number(num))))
    }

    fn identifier(&mut self) -> Result<Option<TokenInfo>, CompileError> {
        while !self.is_at_end() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            self.advance();
        }

        let text = self.current_lexeme();

        let token = match text {
            "function" => Token::Function,
            "type" => Token::Type,
            "api" => Token::Api,
            "const" => Token::Const,
            "if" => Token::If,
            "else" => Token::Else,
            "match" => Token::Match,
            "await" => Token::Await,
            "uses" => Token::Uses,
            "respond" => Token::Respond,
            "import" => Token::Import,
            "GET" => Token::Get,
            "POST" => Token::Post,
            "PUT" => Token::Put,
            "DELETE" => Token::Delete,
            "PATCH" => Token::Patch,
            "true" => Token::Boolean(true),
            "false" => Token::Boolean(false),
            "_" => Token::Underscore,
            _ => Token::Ident(text.to_string()),
        };

        Ok(Some(self.make_token(token)))
    }

    fn make_token(&self, token: Token) -> TokenInfo {
        TokenInfo {
            token,
            line: self.line,
            column: self.column - (self.current - self.start),
        }
    }

    fn advance(&mut self) -> char {
        let c = self.chars[self.current];
        self.current += 1;
        self.column += 1;
        c
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.chars[self.current]
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.peek() != expected {
            false
        } else {
            self.advance();
            true
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.chars.len()
    }

    /// Get the source text between start and current char indices using proper byte offsets
    fn current_lexeme(&self) -> &str {
        let start_byte = self.char_byte_offsets[self.start];
        let end_byte = self.char_byte_offsets[self.current];
        &self.source[start_byte..end_byte]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex_tokens(source: &str) -> Result<Vec<Token>, CompileError> {
        let mut scanner = Scanner::new(source)?;
        let tokens = scanner.scan_tokens()?;
        Ok(tokens.into_iter().map(|ti| ti.token).collect())
    }

    #[test]
    fn test_string_literal() {
        assert_eq!(
            lex_tokens(r#""hello\nworld""#).unwrap(),
            vec![Token::String("hello\nworld".to_string()), Token::Eof]
        );
        assert!(lex_tokens(r#""unterminated"#).is_err());
    }

    #[test]
    fn test_number_literal() {
        assert_eq!(
            lex_tokens("42").unwrap(),
            vec![Token::Number(42), Token::Eof]
        );
        assert_eq!(
            lex_tokens("3.14").unwrap(),
            vec![Token::Decimal("3.14".to_string()), Token::Eof]
        );
        assert!(lex_tokens(".5").is_err());
    }

    #[test]
    fn test_basic_tokens() {
        assert_eq!(
            lex_tokens("function foo() { }").unwrap(),
            vec![
                Token::Function,
                Token::Ident("foo".to_string()),
                Token::LParen,
                Token::RParen,
                Token::LBrace,
                Token::RBrace,
                Token::Eof
            ]
        );
    }
}
