use super::MtpError;
use serde_json::json;

#[derive(Debug)]
pub enum CompileError {
    LexerError(String),
    ParserError(String),
    TypeError(String),
}

impl From<CompileError> for MtpError {
    fn from(err: CompileError) -> Self {
        match err {
            CompileError::LexerError(msg) => MtpError::new("CompileLexer", &msg),
            CompileError::ParserError(msg) => MtpError::new("CompileParser", &msg),
            CompileError::TypeError(msg) => MtpError::new("CompileType", &msg),
        }
    }
}
