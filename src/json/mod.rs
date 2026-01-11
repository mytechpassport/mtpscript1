use std::collections::HashMap;

/// JSON value representation
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Int(i64),
    String(String),
    Array(Vec<Json>),
    Object(HashMap<String, Json>),
}

impl Json {
    /// Parse JSON string
    pub fn parse(input: &str) -> Result<Self, crate::errors::MtpError> {
        let mut chars = input.chars().peekable();
        Self::parse_value(&mut chars)
    }

    fn parse_value(
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> Result<Self, crate::errors::MtpError> {
        Self::skip_whitespace(chars);

        match chars.peek() {
            Some('"') => Self::parse_string(chars),
            Some('0'..='9') | Some('-') => Self::parse_number(chars),
            Some('t') | Some('f') => Self::parse_bool(chars),
            Some('n') => Self::parse_null(chars),
            Some('{') => Self::parse_object(chars),
            Some('[') => Self::parse_array(chars),
            _ => Err(crate::errors::MtpError::JsonError {
                error: "JsonError".to_string(),
                message: "Invalid JSON token".to_string(),
            }),
        }
    }

    fn parse_string(
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> Result<Self, crate::errors::MtpError> {
        chars.next(); // consume opening quote
        let mut result = String::new();

        while let Some(c) = chars.next() {
            if c == '"' {
                break;
            } else if c == '\\' {
                if let Some(escaped) = chars.next() {
                    match escaped {
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
                                    return Err(crate::errors::MtpError::JsonError {
                                        error: "JsonError".to_string(),
                                        message: "Invalid Unicode escape sequence".to_string(),
                                    });
                                }
                            }
                            if let Some(ch) = char::from_u32(code) {
                                result.push(ch);
                            } else {
                                return Err(crate::errors::MtpError::JsonError {
                                    error: "JsonError".to_string(),
                                    message: "Invalid Unicode code point".to_string(),
                                });
                            }
                        }
                        _ => {
                            return Err(crate::errors::MtpError::JsonError { error: "JsonError".to_string(), message: "Invalid escape sequence".to_string() });
            }

            Self::skip_whitespace(chars);
            if chars.next() != Some(':') {
                return Err(crate::errors::MtpError::JsonError(
                    "Expected ':' after object key".into(),
                ));
            }

            // Parse value
            let value = Self::parse_value(chars)?;
            obj.insert(key, value);

            Self::skip_whitespace(chars);
            match chars.next() {
                Some(',') => continue,
                Some('}') => break,
                _ => {
                    return Err(crate::errors::MtpError::JsonError { error: "JsonError".to_string(), message: "Expected ',' or '}' in object".to_string() })
                }
            }
        }

        Ok(Json::Object(obj))
    }

    fn parse_array(
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> Result<Self, crate::errors::MtpError> {
        chars.next(); // consume '['
        let mut arr = Vec::new();

        loop {
            Self::skip_whitespace(chars);
            if chars.peek() == Some(&']') {
                chars.next();
                break;
            }

            let value = Self::parse_value(chars)?;
            arr.push(value);

            Self::skip_whitespace(chars);
            match chars.next() {
                Some(',') => continue,
                Some(']') => break,
                _ => {
                    return Err(crate::errors::MtpError::JsonError { error: "JsonError".to_string(), message: "Expected ',' or ']' in array".to_string() })
                }
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

    /// Serialize to canonical JSON (RFC 8785)
    pub fn to_canonical_string(&self) -> String {
        match self {
            Json::Object(obj) => {
                let mut keys: Vec<_> = obj.keys().collect();
                keys.sort();
                let mut result = "{".to_string();
                for (i, key) in keys.iter().enumerate() {
                    if i > 0 {
                        result.push(',');
                    }
                    result.push('"');
                    result.push_str(key);
                    result.push('"');
                    result.push(':');
                    result.push_str(&obj[*key].to_canonical_string());
                }
                result.push('}');
                result
            }
            Json::Array(arr) => {
                let mut result = "[".to_string();
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        result.push(',');
                    }
                    result.push_str(&item.to_canonical_string());
                }
                result.push(']');
                result
            }
            Json::String(s) => format!("\"{}\"", s.replace("\\", "\\\\").replace("\"", "\\\"")),
            Json::Int(n) => n.to_string(),
            Json::Bool(b) => b.to_string(),
            Json::Null => "null".to_string(),
        }
    }
}
