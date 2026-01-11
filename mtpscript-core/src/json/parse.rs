use super::{Json, ParseError};
use crate::types::Decimal;
use std::collections::HashMap;

pub fn parse_json(input: &str) -> Result<Json, ParseError> {
    parse_json_with_limits(input, 1000, 10_000_000) // default limits: depth 1000, size 10M
}

pub fn parse_json_with_limits(
    input: &str,
    max_depth: usize,
    max_size: usize,
) -> Result<Json, ParseError> {
    let mut parser = JsonParser::new(input, max_depth, max_size);
    let result = parser.parse_value(0)?;
    if !parser.is_at_end() {
        return Err(ParseError::TrailingCharacters);
    }
    Ok(result)
}

struct JsonParser<'a> {
    input: &'a str,
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    pos: usize,
    max_depth: usize,
    max_size: usize,
}

impl<'a> JsonParser<'a> {
    fn new(input: &'a str, max_depth: usize, max_size: usize) -> Self {
        Self {
            input,
            chars: input.chars().peekable(),
            pos: 0,
            max_depth,
            max_size,
        }
    }

    fn parse_value(&mut self, depth: usize) -> Result<Json, ParseError> {
        if depth > self.max_depth {
            return Err(ParseError::InvalidNumber(
                "Maximum depth exceeded".to_string(),
            ));
        }
        if self.pos > self.max_size {
            return Err(ParseError::InvalidNumber(
                "Maximum size exceeded".to_string(),
            ));
        }
        self.skip_whitespace();
        match self.peek() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string().map(Json::String),
            Some('0'..='9') | Some('-') => self.parse_number(),
            Some('[') => self.parse_array(depth + 1),
            Some('{') => self.parse_object(depth + 1),
            Some(c) => Err(ParseError::UnexpectedChar(c, self.pos)),
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    fn parse_null(&mut self) -> Result<Json, ParseError> {
        self.expect_str("null")?;
        Ok(Json::Null)
    }

    fn parse_bool(&mut self) -> Result<Json, ParseError> {
        if self.peek() == Some('t') {
            self.expect_str("true")?;
            Ok(Json::Bool(true))
        } else {
            self.expect_str("false")?;
            Ok(Json::Bool(false))
        }
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        self.expect('"')?;
        let mut result = String::new();
        while let Some(c) = self.next() {
            match c {
                '"' => return Ok(result),
                '\\' => match self.next() {
                    Some('"') => result.push('"'),
                    Some('\\') => result.push('\\'),
                    Some('/') => result.push('/'),
                    Some('b') => result.push('\x08'),
                    Some('f') => result.push('\x0c'),
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some('u') => {
                        let mut code = 0u32;
                        for _ in 0..4 {
                            let c = self.next().ok_or(ParseError::UnexpectedEnd)?;
                            let digit = c.to_digit(16).ok_or(ParseError::InvalidEscape(c))?;
                            code = code * 16 + digit;
                        }
                        let ch = char::from_u32(code).ok_or(ParseError::InvalidEscape('u'))?;
                        result.push(ch);
                    }
                    Some(c) => return Err(ParseError::InvalidEscape(c)),
                    None => return Err(ParseError::UnexpectedEnd),
                },
                c => result.push(c),
            }
        }
        Err(ParseError::UnexpectedEnd)
    }

    fn parse_number(&mut self) -> Result<Json, ParseError> {
        let start = self.pos;
        // Simple number parsing for now
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' || c == '-' || c == 'e' || c == 'E' || c == '+' {
                self.next();
            } else {
                break;
            }
        }
        let num_str = &self.input[start..self.pos];
        if num_str.contains('.') || num_str.contains('e') || num_str.contains('E') {
            // Decimal
            match Decimal::from_str(num_str) {
                Ok(d) => Ok(Json::Decimal(d)),
                Err(_) => Err(ParseError::InvalidNumber(num_str.to_string())),
            }
        } else {
            // Int
            match num_str.parse::<i64>() {
                Ok(n) => Ok(Json::Int(n)),
                Err(_) => Err(ParseError::InvalidNumber(num_str.to_string())),
            }
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<Json, ParseError> {
        self.expect('[')?;
        let mut elements = Vec::new();
        loop {
            self.skip_whitespace();
            if self.peek() == Some(']') {
                self.next();
                break;
            }
            elements.push(self.parse_value(depth)?);
            self.skip_whitespace();
            if self.peek() == Some(',') {
                self.next();
            } else if self.peek() == Some(']') {
                // allow trailing comma
            } else {
                return Err(ParseError::UnexpectedChar(self.peek().unwrap(), self.pos));
            }
        }
        Ok(Json::Array(elements))
    }

    fn parse_object(&mut self, depth: usize) -> Result<Json, ParseError> {
        self.expect('{')?;
        let mut object = HashMap::new();
        loop {
            self.skip_whitespace();
            if self.peek() == Some('}') {
                self.next();
                break;
            }
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.expect(':')?;
            let value = self.parse_value(depth)?;
            if object.contains_key(&key) {
                return Err(ParseError::DuplicateKey(key));
            }
            object.insert(key, value);
            self.skip_whitespace();
            if self.peek() == Some(',') {
                self.next();
            } else if self.peek() == Some('}') {
                // allow trailing comma
            } else {
                return Err(ParseError::UnexpectedChar(self.peek().unwrap(), self.pos));
            }
        }
        Ok(Json::Object(object))
    }

    fn expect(&mut self, expected: char) -> Result<(), ParseError> {
        match self.next() {
            Some(c) if c == expected => Ok(()),
            Some(c) => Err(ParseError::UnexpectedChar(c, self.pos)),
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    fn expect_str(&mut self, expected: &str) -> Result<(), ParseError> {
        for c in expected.chars() {
            self.expect(c)?;
        }
        Ok(())
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn next(&mut self) -> Option<char> {
        let c = self.chars.next();
        if let Some(c) = c {
            self.pos += c.len_utf8();
        }
        c
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.next();
            } else {
                break;
            }
        }
    }

    fn is_at_end(&mut self) -> bool {
        self.peek().is_none()
    }
}
