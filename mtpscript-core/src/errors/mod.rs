pub mod compile;
pub mod runtime;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MtpError {
    GasExhausted {
        error: String,
        gasLimit: u64,
        gasUsed: u64,
    },
    Security {
        error: String,
        message: String,
    },
    Runtime {
        error: String,
        message: String,
    },
    Build {
        error: String,
        message: String,
    },
    Io {
        error: String,
        message: String,
    },
    GasLimitOutOfRange {
        error: String,
        provided: u64,
        min: u64,
        max: u64,
    },
    IntegrityError {
        error: String,
        message: String,
    },
    RateLimitError {
        error: String,
        message: String,
    },
    ValidationError {
        error: String,
        message: String,
    },
    ModuleError {
        error: String,
        message: String,
    },
    TypeError {
        error: String,
        message: String,
    },
    ParseError {
        error: String,
        message: String,
    },
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
                error: _,
                gasLimit,
                gasUsed,
            } => write!(f, "Gas exhausted: used {} of {}", gasUsed, gasLimit),
            MtpError::Security { error: _, message } => write!(f, "Security error: {}", message),
            MtpError::Runtime { error: _, message } => write!(f, "Runtime error: {}", message),
            MtpError::Build { error: _, message } => write!(f, "Build error: {}", message),
            MtpError::Io { error: _, message } => write!(f, "IO error: {}", message),
            MtpError::GasLimitOutOfRange {
                error: _,
                provided,
                min,
                max,
            } => {
                write!(f, "Gas limit {} out of range [{}, {}]", provided, min, max)
            }
            MtpError::IntegrityError { error: _, message } => {
                write!(f, "Integrity error: {}", message)
            }
            MtpError::RateLimitError { error: _, message } => {
                write!(f, "Rate limit error: {}", message)
            }
            MtpError::ValidationError { error: _, message } => {
                write!(f, "Validation error: {}", message)
            }
            MtpError::ModuleError { error: _, message } => write!(f, "Module error: {}", message),
            MtpError::TypeError { error: _, message } => write!(f, "Type error: {}", message),
            MtpError::ParseError { error: _, message } => write!(f, "Parse error: {}", message),
        }
    }
}

impl std::error::Error for MtpError {}

impl From<std::io::Error> for MtpError {
    fn from(err: std::io::Error) -> Self {
        MtpError::Io {
            error: "Io".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for MtpError {
    fn from(err: serde_json::Error) -> Self {
        MtpError::Runtime {
            error: "Runtime".to_string(),
            message: format!("JSON error: {}", err),
        }
    }
}

impl From<std::string::FromUtf8Error> for MtpError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        MtpError::Runtime {
            error: "Runtime".to_string(),
            message: format!("UTF-8 error: {}", err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gas_exhausted_serialization() {
        let err = MtpError::GasExhausted {
            error: "GasExhausted".to_string(),
            gasLimit: 1000,
            gasUsed: 500,
        };
        let json = err.to_json();
        assert_eq!(
            json,
            r#"{"error":"GasExhausted","gasLimit":1000,"gasUsed":500}"#
        );
    }
}
