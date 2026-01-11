pub mod cbor;
pub mod equality;
pub mod hash;
pub mod parse;
pub mod serialize;

use crate::types::Decimal;
use std::collections::HashMap;

/// MTPScript JSON ADT as per §9
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Int(i64),
    Decimal(Decimal),
    String(String),
    Array(Vec<Json>),
    Object(HashMap<String, Json>),
}

impl Json {
    /// Parse JSON string to Json ADT
    /// Rejects duplicate keys, returns Result
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        parse::parse_json(input)
    }

    /// Serialize to canonical JSON string (RFC 8785)
    pub fn to_canonical_string(&self) -> Result<String, crate::errors::MtpError> {
        serialize::serialize_canonical(self)
    }

    /// Encode to deterministic CBOR (RFC 7049 §3.9) as hex string
    pub fn to_cbor_hex(&self) -> Result<String, crate::errors::MtpError> {
        let cbor_bytes = cbor::encode_cbor(self)?;
        Ok(hex::encode(cbor_bytes))
    }
}

impl std::fmt::Display for Json {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // For fallback in hash, use canonical string
        match serialize::serialize_canonical(self) {
            Ok(s) => write!(f, "{}", s),
            Err(_) => write!(f, "{:?}", self),
        }
    }
}

impl Json {
    /// Convert Json to runtime Value
    pub fn to_value(&self) -> crate::runtime::value::Value {
        match self {
            Json::Null => crate::runtime::value::Value::Null,
            Json::Bool(b) => crate::runtime::value::Value::Boolean(*b),
            Json::Int(n) => crate::runtime::value::Value::Number(*n),
            Json::Decimal(d) => crate::runtime::value::Value::Decimal(d.clone()),
            Json::String(s) => crate::runtime::value::Value::String(s.clone()),
            Json::Array(arr) => {
                crate::runtime::value::Value::Array(arr.iter().map(|j| j.to_value()).collect())
            }
            Json::Object(obj) => crate::runtime::value::Value::Object(
                obj.iter().map(|(k, v)| (k.clone(), v.to_value())).collect(),
            ),
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedChar(char, usize),
    UnexpectedEnd,
    InvalidNumber(String),
    DuplicateKey(String),
    TrailingCharacters,
    InvalidEscape(char),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedChar(c, pos) => {
                write!(f, "Unexpected character '{}' at position {}", c, pos)
            }
            ParseError::UnexpectedEnd => write!(f, "Unexpected end of input"),
            ParseError::InvalidNumber(s) => write!(f, "Invalid number: {}", s),
            ParseError::DuplicateKey(key) => write!(f, "Duplicate key: {}", key),
            ParseError::TrailingCharacters => write!(f, "Trailing characters after JSON value"),
            ParseError::InvalidEscape(c) => write!(f, "Invalid escape sequence: \\{}", c),
        }
    }
}

impl std::error::Error for ParseError {}
