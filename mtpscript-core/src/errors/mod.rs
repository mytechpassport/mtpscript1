pub mod compile;
pub mod runtime;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtpError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl MtpError {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    pub fn with_details(code: &str, message: &str, details: serde_json::Value) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            details: Some(details),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    #[cfg(not(debug_assertions))]
    pub fn stack_trace(&self) -> Option<String> {
        None
    }

    #[cfg(debug_assertions)]
    pub fn stack_trace(&self) -> Option<String> {
        Some(format!("{:?}", std::backtrace::Backtrace::capture()))
    }
}
