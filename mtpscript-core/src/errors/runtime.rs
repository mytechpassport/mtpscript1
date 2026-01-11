use super::MtpError;
use serde_json::json;

#[derive(Debug)]
pub enum RuntimeError {
    GasExhausted { gas_limit: u64, gas_used: u64 },
    InvalidGasLimit,
    EffectNotFound(String),
    ValueError(String),
}

impl From<RuntimeError> for MtpError {
    fn from(err: RuntimeError) -> Self {
        match err {
            RuntimeError::GasExhausted {
                gas_limit,
                gas_used,
            } => MtpError::with_details(
                "RuntimeGasExhausted",
                "Gas limit exceeded",
                json!({
                    "gasLimit": gas_limit,
                    "gasUsed": gas_used
                }),
            ),
            RuntimeError::InvalidGasLimit => {
                MtpError::new("RuntimeInvalidGasLimit", "Invalid gas limit specified")
            }
            RuntimeError::EffectNotFound(name) => MtpError::new(
                "RuntimeEffectNotFound",
                &format!("Effect '{}' not found", name),
            ),
            RuntimeError::ValueError(msg) => MtpError::new("RuntimeValue", &msg),
        }
    }
}
