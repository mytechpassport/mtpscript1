use super::MtpError;
use serde_json::json;

#[derive(Debug)]
pub enum RuntimeError {
    GasExhausted { gas_limit: u64, gas_used: u64 },
    InvalidGasLimit,
    EffectNotFound(String),
    ValueError(String),
    TypeError(String),
}

impl From<crate::errors::compile::CompileError> for RuntimeError {
    fn from(err: crate::errors::compile::CompileError) -> Self {
        RuntimeError::ValueError(format!("Compile error: {:?}", err))
    }
}

impl From<RuntimeError> for MtpError {
    fn from(err: RuntimeError) -> Self {
        match err {
            RuntimeError::GasExhausted {
                gas_limit,
                gas_used,
            } => MtpError::GasExhausted {
                gas_limit,
                gas_used,
            },
            RuntimeError::InvalidGasLimit => {
                MtpError::Runtime("Invalid gas limit specified".to_string())
            }
            RuntimeError::EffectNotFound(name) => {
                MtpError::Runtime(format!("Effect '{}' not found", name))
            }
            RuntimeError::ValueError(msg) => MtpError::Runtime(format!("Value error: {}", msg)),
            RuntimeError::TypeError(msg) => MtpError::Runtime(format!("Type error: {}", msg)),
        }
    }
}
