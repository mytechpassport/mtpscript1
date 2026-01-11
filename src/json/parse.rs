use crate::errors::MtpError;
use crate::json::Json;
use std::collections::HashMap;

/// Configuration for JSON parsing limits
#[derive(Debug, Clone)]
pub struct JsonParseConfig {
    pub max_depth: usize,
    pub max_size: usize,
}

impl Default for JsonParseConfig {
    fn default() -> Self {
        JsonParseConfig {
            max_depth: 32,              // Reasonable depth limit
            max_size: 10 * 1024 * 1024, // 10MB limit
        }
    }
}

impl Json {
    /// Parse JSON string with configurable limits
    pub fn parse_with_config(input: &str, config: &JsonParseConfig) -> Result<Self, MtpError> {
        if input.len() > config.max_size {
            return Err(MtpError::JsonError(
                "Input exceeds maximum size limit".into(),
            ));
        }

        let mut chars = input.chars().peekable();
        let mut depth = 0;

        Self::parse_value(&mut chars, &mut depth, config)
    }

    /// Parse JSON string with default limits
    pub fn parse(input: &str) -> Result<Self, MtpError> {
        Self::parse_with_config(input, &JsonParseConfig::default())
    }

    fn parse_value(
        chars: &mut std::iter::Peekable<std::str::Chars>,
        depth: &mut usize,
        config: &JsonParseConfig,
    ) -> Result<Self, MtpError> {
        *depth += 1;
        if *depth > config.max_depth {
            return Err(MtpError::JsonError("Maximum nesting depth exceeded".into()));
        }

        Self::skip_whitespace(chars);

        let result = match chars.peek() {
            Some('"') => Self::parse_string(chars),
            Some('0'..='9') | Some('-') => Self::parse_number(chars),
            Some('t') | Some('f') => Self::parse_bool(chars),
            Some('n') => Self::parse_null(chars),
            Some('{') => Self::parse_object(chars, depth, config),
            Some('[') => Self::parse_array(chars, depth, config),
            _ => Err(MtpError::JsonError("Invalid JSON token".into())),
        };

        *depth -= 1;
        result
    }

    fn parse_string(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<Self, MtpError> {
        chars.next(); // consume opening quote
        let mut result = String::new();
        let mut escaped = false;

        while let Some(c) = chars.next() {
            if escaped {
                match c {
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    '/' => result.push('/'),
                    'b' => result.push('\x08'),
                    'f' => result.push('\x0C'),
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    'u' => {
                        // Parse 4 hex digits for Unicode escape
                        let mut code = 0u32;
                        for _ in 0..4 {
                            if let Some(d) = chars.next().and_then(|c| c.to_digit(16)) {
                                code = code * 16 + d;
                            } else {
                                return Err(MtpError::JsonError(
                                    "Invalid Unicode escape sequence".into(),
                                ));
                            }
                        }
                        if let Some(ch) = char::from_u32(code) {
                            result.push(ch);
                        } else {
                            return Err(MtpError::JsonError("Invalid Unicode code point".into()));
                        }
                    }
                    _ => return Err(MtpError::JsonError("Invalid escape sequence".into())),
                }
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                break;
            } else {
                result.push(c);
            }
        }

        Ok(Json::String(result))
    }

    fn parse_number(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<Self, MtpError> {
        let mut num_str = String::new();

        while let Some(&c) = chars.peek() {
            if c.is_numeric() || c == '-' || c == '.' || c == 'e' || c == 'E' || c == '+' {
                num_str.push(c);
                chars.next();
            } else {
                break;
            }
        }

        // Try parsing as i64 first, then as f64
        if let Ok(int_val) = num_str.parse::<i64>() {
            Ok(Json::Int(int_val))
        } else if let Ok(float_val) = num_str.parse::<f64>() {
            // Check for special values that should be rejected
            if float_val.is_infinite() || float_val.is_nan() {
                return Err(MtpError::JsonError(
                    "Invalid number: infinity or NaN not allowed".into(),
                ));
            }
            // For now, store as string - proper Decimal implementation needed
            Ok(Json::String(num_str))
        } else {
            Err(MtpError::JsonError("Invalid number format".into()))
        }
    }

    fn parse_bool(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<Self, MtpError> {
        if chars.take(4).collect::<String>() == "true" {
            Ok(Json::Bool(true))
        } else if chars.take(5).collect::<String>() == "false" {
            Ok(Json::Bool(false))
        } else {
            Err(MtpError::JsonError("Invalid boolean value".into()))
        }
    }

    fn parse_null(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<Self, MtpError> {
        if chars.take(4).collect::<String>() == "null" {
            Ok(Json::Null)
        } else {
            Err(MtpError::JsonError("Invalid null value".into()))
        }
    }

    fn parse_object(
        chars: &mut std::iter::Peekable<std::str::Chars>,
        depth: &mut usize,
        config: &JsonParseConfig,
    ) -> Result<Self, MtpError> {
        chars.next(); // consume '{'
        let mut obj = HashMap::new();

        loop {
            Self::skip_whitespace(chars);
            if chars.peek() == Some(&'}') {
                chars.next();
                break;
            }

            // Parse key
            let key = match Self::parse_string(chars)? {
                Json::String(s) => s,
                _ => return Err(MtpError::JsonError("Object key must be string".into())),
            };

            // Check for duplicate keys
            if obj.contains_key(&key) {
                return Err(MtpError::JsonError("Duplicate object key".into()));
            }

            Self::skip_whitespace(chars);
            if chars.next() != Some(':') {
                return Err(MtpError::JsonError("Expected ':' after object key".into()));
            }

            // Parse value
            let value = Self::parse_value(chars, depth, config)?;
            obj.insert(key, value);

            Self::skip_whitespace(chars);
            match chars.next() {
                Some(',') => continue,
                Some('}') => break,
                _ => return Err(MtpError::JsonError("Expected ',' or '}' in object".into())),
            }
        }

        Ok(Json::Object(obj))
    }

    fn parse_array(
        chars: &mut std::iter::Peekable<std::str::Chars>,
        depth: &mut usize,
        config: &JsonParseConfig,
    ) -> Result<Self, MtpError> {
        chars.next(); // consume '['
        let mut arr = Vec::new();

        loop {
            Self::skip_whitespace(chars);
            if chars.peek() == Some(&']') {
                chars.next();
                break;
            }

            let value = Self::parse_value(chars, depth, config)?;
            arr.push(value);

            Self::skip_whitespace(chars);
            match chars.next() {
                Some(',') => continue,
                Some(']') => break,
                _ => return Err(MtpError::JsonError("Expected ',' or ']' in array".into())),
            }
        }

        Ok(Json::Array(arr))
    }

    fn skip_whitespace(chars: &mut std::iter::Peekable<std::str::Chars>) {
        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }
    }
}
