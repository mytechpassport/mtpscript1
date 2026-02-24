use crate::errors::MtpError;
use crate::lexer::token::{Token, TokenKind};
use std::collections::HashMap;

pub struct Scanner<'a> {
    source: &'a str,
    chars: Vec<char>,
    start: usize,
    current: usize,
    line: usize,
    keywords: HashMap<&'static str, TokenKind>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        let chars: Vec<char> = source.chars().collect();
        let mut keywords = HashMap::new();

        // Initialize keywords
        keywords.insert("function", TokenKind::Function);
        keywords.insert("type", TokenKind::Type);
        keywords.insert("api", TokenKind::Api);
        keywords.insert("const", TokenKind::Const);
        keywords.insert("if", TokenKind::If);
        keywords.insert("else", TokenKind::Else);
        keywords.insert("then", TokenKind::Then);
        keywords.insert("match", TokenKind::Match);
        keywords.insert("await", TokenKind::Await);
        keywords.insert("uses", TokenKind::Uses);
        keywords.insert("import", TokenKind::Import);
        keywords.insert("respond", TokenKind::Respond);
        keywords.insert("true", TokenKind::True);
        keywords.insert("false", TokenKind::False);
        keywords.insert("Ok", TokenKind::Ok);
        keywords.insert("Err", TokenKind::Err);
        keywords.insert("Some", TokenKind::Some);
        keywords.insert("None", TokenKind::None);
        keywords.insert("GET", TokenKind::Get);
        keywords.insert("POST", TokenKind::Post);
        keywords.insert("PUT", TokenKind::Put);
        keywords.insert("DELETE", TokenKind::Delete);
        keywords.insert("PATCH", TokenKind::Patch);

        Scanner {
            source,
            chars,
            start: 0,
            current: 0,
            line: 1,
            keywords,
        }
    }

    pub fn scan_token(&mut self) -> Result<Token, MtpError> {
        self.skip_whitespace();

        self.start = self.current;

        if self.is_at_end() {
            return Ok(self.make_token(TokenKind::Eof));
        }

        let c = self.advance();

        match c {
            '(' => Ok(self.make_token(TokenKind::LeftParen)),
            ')' => Ok(self.make_token(TokenKind::RightParen)),
            '{' => Ok(self.make_token(TokenKind::LeftBrace)),
            '}' => Ok(self.make_token(TokenKind::RightBrace)),
            '[' => Ok(self.make_token(TokenKind::LeftBracket)),
            ']' => Ok(self.make_token(TokenKind::RightBracket)),
            ',' => Ok(self.make_token(TokenKind::Comma)),
            ':' => Ok(self.make_token(TokenKind::Colon)),
            ';' => Ok(self.make_token(TokenKind::Semicolon)),
            '.' => Ok(self.make_token(TokenKind::Dot)),
            '+' => Ok(self.make_token(TokenKind::Plus)),
            '-' => Ok(self.make_token(TokenKind::Minus)),
            '*' => Ok(self.make_token(TokenKind::Star)),
            '/' => Ok(self.make_token(TokenKind::Slash)),
            '!' => {
                if self.matches('=') {
                    Ok(self.make_token(TokenKind::BangEqual))
                } else {
                    Ok(self.make_token(TokenKind::Bang))
                }
            }
            '=' => {
                if self.matches('=') {
                    Ok(self.make_token(TokenKind::EqualEqual))
                } else if self.matches('>') {
                    Ok(self.make_token(TokenKind::EqualGreater))
                } else {
                    Ok(self.make_token(TokenKind::Equal))
                }
            }
            '<' => {
                if self.matches('=') {
                    Ok(self.make_token(TokenKind::LessEqual))
                } else {
                    Ok(self.make_token(TokenKind::Less))
                }
            }
            '>' => {
                if self.matches('=') {
                    Ok(self.make_token(TokenKind::GreaterEqual))
                } else {
                    Ok(self.make_token(TokenKind::Greater))
                }
            }
            '&' => {
                if self.matches('&') {
                    Ok(self.make_token(TokenKind::AmpAmp))
                } else {
                    return Err(MtpError::LexerError {
                        error: "LexerError".to_string(),
                        message: "Unexpected '&'".to_string(),
                    });
                }
            }
            '|' => {
                if self.matches('>') {
                    Ok(self.make_token(TokenKind::PipeGreater))
                } else if self.matches('|') {
                    Ok(self.make_token(TokenKind::PipePipe))
                } else {
                    return Err(MtpError::LexerError {
                        error: "LexerError".to_string(),
                        message: "Unexpected '|'".to_string(),
                    });
                }
            }
            '"' => self.string(),
            '0'..='9' => self.number(),
            '_' => {
                // Check if it's just a standalone underscore (wildcard) or part of an identifier
                if self.is_at_end() || !(self.peek().is_alphanumeric() || self.peek() == '_') {
                    Ok(self.make_token(TokenKind::Underscore))
                } else {
                    self.identifier()
                }
            }
            'a'..='z' | 'A'..='Z' => self.identifier(),
            _ => Err(MtpError::LexerError {
                error: "LexerError".to_string(),
                message: format!("Unexpected character: {}", c),
            }),
        }
    }

    fn string(&mut self) -> Result<Token, MtpError> {
        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(MtpError::LexerError {
                error: "LexerError".to_string(),
                message: "Unterminated string".to_string(),
            });
        }

        self.advance(); // consume closing quote

        let value = self.source[self.start + 1..self.current - 1].to_string();
        Ok(self.make_token(TokenKind::String(value)))
    }

    fn number(&mut self) -> Result<Token, MtpError> {
        while !self.is_at_end() && self.peek().is_ascii_digit() {
            self.advance();
        }

        // Look for decimal part
        if !self.is_at_end()
            && self.peek() == '.'
            && self.peek_next().map_or(false, |c| c.is_ascii_digit())
        {
            self.advance(); // consume '.'

            while !self.is_at_end() && self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let lexeme = &self.source[self.start..self.current];
        Ok(self.make_token(TokenKind::Number(lexeme.to_string())))
    }

    fn identifier(&mut self) -> Result<Token, MtpError> {
        while !self.is_at_end() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            self.advance();
        }

        let text = &self.source[self.start..self.current];

        let kind = self
            .keywords
            .get(text)
            .cloned()
            .unwrap_or_else(|| TokenKind::Identifier(text.to_string()));

        Ok(self.make_token(kind))
    }

    fn skip_whitespace(&mut self) {
        loop {
            if self.is_at_end() {
                break;
            }

            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '/' => {
                    if self.peek_next() == Some('/') {
                        // Line comment
                        while !self.is_at_end() && self.peek() != '\n' {
                            self.advance();
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        let lexeme = self.source[self.start..self.current].to_string();
        Token {
            kind,
            lexeme,
            line: self.line,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.chars.len()
    }

    fn advance(&mut self) -> char {
        let c = self.chars[self.current];
        self.current += 1;
        c
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.chars[self.current]
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.current + 1 >= self.chars.len() {
            None
        } else {
            Some(self.chars[self.current + 1])
        }
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.chars[self.current] != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }
}
