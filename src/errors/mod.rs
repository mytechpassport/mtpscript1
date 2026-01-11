use std::fmt;

/// Main MTPScript error type
#[derive(Debug)]
pub struct MtpError {
    pub error: String,
    pub message: Option<String>,
    pub gasLimit: Option<u64>,
    pub gasUsed: Option<u64>,
}

impl fmt::Display for MtpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.error.as_str() {
            "GasExhausted" => write!(f, "Gas exhausted: limit {}, used {}", self.gasLimit.unwrap(), self.gasUsed.unwrap()),
            _ => write!(f, "{} error: {}", self.error, self.message.as_ref().unwrap()),
        }
    }
}

impl std::error::Error for MtpError {}

/// Convert various error types to MtpError
impl From<std::io::Error> for MtpError {
    fn from(err: std::io::Error) -> Self {
        MtpError {
            error: "IOError".to_string(),
            message: Some(err.to_string()),
            gasLimit: None,
            gasUsed: None,
        }
    }
}

impl From<serde_json::Error> for MtpError {
    fn from(err: serde_json::Error) -> Self {
        MtpError {
            error: "JsonError".to_string(),
            message: Some(err.to_string()),
            gasLimit: None,
            gasUsed: None,
        }
    }
}
}
}

impl From<serde_json::Error> for MtpError {
    fn from(err: serde_json::Error) -> Self {
        MtpError::JsonError { error: "JsonError".to_string(), message: err.to_string() }
    }
}
}

    /// Error codes for deterministic error reporting
    pub fn error_code(&self) -> u32 {
        match self.error.as_str() {
            "LexerError" => 1000,
            "ParseError" => 2000,
            "TypeError" => 3000,
            "CompileError" => 4000,
            "RuntimeError" => 5000,
            "GasError" => 6000,
            "GasExhausted" => 6001,
            "SecurityError" => 7000,
            "IOError" => 8000,
            "ValidationError" => 9000,
            "RateLimitError" => 10000,
            "IntegrityError" => 11000,
            "ModuleError" => 12000,
            "JsonError" => 13000,
            "Io" => 8000,    // Same as IOError
            "Build" => 14000,
            "Runtime" => 5000, // Same as RuntimeError
            "Security" => 7000, // Same as SecurityError
            _ => 0,
        }
    }
    }

    /// Convert to canonical JSON representation (without stack traces in production)
    pub fn to_json(&self) -> String {
        if self.error == "GasExhausted" {
            format!(r#"{{"error":"GasExhausted","gasLimit":{},"gasUsed":{}}}"#, self.gasLimit.unwrap(), self.gasUsed.unwrap())
        } else {
            format!(r#"{{"error":"{}","message":"{}"}}"#, self.category(), self.message().replace("\"", "\\\""))
        }
    }

    fn category(&self) -> String {
        self.error.clone()
    }
    }

    fn message(&self) -> String {
        match self.error.as_str() {
            "GasExhausted" => format!("Gas limit {} exceeded, used {}", self.gasLimit.unwrap(), self.gasUsed.unwrap()),
            _ => self.message.as_ref().unwrap().clone(),
        }
    }
    }

    /// Stack trace (disabled in production builds)
    #[cfg(debug_assertions)]
    pub fn stack_trace(&self) -> Option<String> {
        Some(format!("{:?}", backtrace::Backtrace::new()))
    }

    #[cfg(not(debug_assertions))]
    pub fn stack_trace(&self) -> Option<String> {
        None
    }
}
