use crate::errors::runtime::RuntimeError;
use std::env;

const DEFAULT_GAS_LIMIT: u64 = 10_000_000;
const MAX_GAS_LIMIT: u64 = 2_000_000_000;

#[derive(Debug, Clone)]
pub struct GasCounter {
    limit: u64,
    used: u64,
}

impl GasCounter {
    pub fn new(limit: u64) -> Self {
        Self { limit, used: 0 }
    }

    pub fn from_env() -> Result<Self, RuntimeError> {
        let limit_str = env::var("MTP_GAS_LIMIT").unwrap_or_else(|_| DEFAULT_GAS_LIMIT.to_string());
        let limit: u64 = limit_str
            .parse()
            .map_err(|_| RuntimeError::InvalidGasLimit)?;

        if limit < 1 || limit > MAX_GAS_LIMIT {
            return Err(RuntimeError::InvalidGasLimit);
        }

        Ok(Self::new(limit))
    }

    pub fn consume(&mut self, amount: u64) -> Result<(), RuntimeError> {
        if self.used + amount > self.limit {
            self.used = self.limit + 1; // Mark as exhausted
            return Err(RuntimeError::GasExhausted {
                gas_limit: self.limit,
                gas_used: self.used,
            });
        }
        self.used += amount;
        Ok(())
    }

    pub fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.used)
    }

    pub fn used(&self) -> u64 {
        self.used
    }

    pub fn is_exhausted(&self) -> bool {
        self.used > self.limit
    }

    pub fn error(&self) -> Option<RuntimeError> {
        if self.is_exhausted() {
            Some(RuntimeError::GasExhausted {
                gas_limit: self.limit,
                gas_used: self.used,
            })
        } else {
            None
        }
    }
}
