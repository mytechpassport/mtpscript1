pub mod compile;
pub mod runtime;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "error", content = "details")]
pub enum MtpError {
    #[serde(rename = "GasExhausted")]
    GasExhausted { gas_limit: u64, gas_used: u64 },
    #[serde(rename = "Security")]
    Security(String),
    #[serde(rename = "Runtime")]
    Runtime(String),
    #[serde(rename = "Build")]
    Build(String),
    #[serde(rename = "Io")]
    Io(String),
    #[serde(rename = "GasLimitOutOfRange")]
    GasLimitOutOfRange { provided: u64, min: u64, max: u64 },
}

impl MtpError {
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

impl std::fmt::Display for MtpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MtpError::GasExhausted {
                gas_limit,
                gas_used,
            } => write!(f, "Gas exhausted: used {} of {}", gas_used, gas_limit),
            MtpError::Security(msg) => write!(f, "Security error: {}", msg),
            MtpError::Runtime(msg) => write!(f, "Runtime error: {}", msg),
            MtpError::Build(msg) => write!(f, "Build error: {}", msg),
            MtpError::Io(err) => write!(f, "IO error: {}", err),
            MtpError::GasLimitOutOfRange { provided, min, max } => {
                write!(f, "Gas limit {} out of range [{}, {}]", provided, min, max)
            }
        }
    }
}

impl std::error::Error for MtpError {}

impl From<std::io::Error> for MtpError {
    fn from(err: std::io::Error) -> Self {
        MtpError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for MtpError {
    fn from(err: serde_json::Error) -> Self {
        MtpError::Runtime(format!("JSON error: {}", err))
    }
}

impl From<std::string::FromUtf8Error> for MtpError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        MtpError::Runtime(format!("UTF-8 error: {}", err))
    }
}
