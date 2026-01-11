use super::MtpError;

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
                error: "GasExhausted".to_string(),
                gasLimit: gas_limit,
                gasUsed: gas_used,
            },
            RuntimeError::InvalidGasLimit => MtpError::Runtime {
                error: "Runtime".to_string(),
                message: "Invalid gas limit specified".to_string(),
            },
            RuntimeError::EffectNotFound(name) => MtpError::Runtime {
                error: "Runtime".to_string(),
                message: format!("Effect '{}' not found", name),
            },
            RuntimeError::ValueError(msg) => MtpError::Runtime {
                error: "Runtime".to_string(),
                message: format!("Value error: {}", msg),
            },
            RuntimeError::TypeError(msg) => MtpError::Runtime {
                error: "Runtime".to_string(),
                message: format!("Type error: {}", msg),
            },
        }
    }
}
