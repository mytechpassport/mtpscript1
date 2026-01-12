use crate::errors::MtpError;

/// Gas counter for execution metering
#[derive(Debug, Clone)]
pub struct GasCounter {
    limit: u64,
    used: u64,
}

impl GasCounter {
    /// Create a new gas counter with the given limit
    pub fn new(limit: u64) -> Self {
        GasCounter { limit, used: 0 }
    }

    /// Create from environment variable MTP_GAS_LIMIT
    pub fn from_env() -> Result<Self, MtpError> {
        let limit_str = std::env::var("MTP_GAS_LIMIT").unwrap_or_else(|_| "10000000".to_string());
        let limit: u64 = limit_str
            .parse()
            .map_err(|_| MtpError::GasError("Invalid MTP_GAS_LIMIT value".into()))?;

        if limit < 1 || limit > 2_000_000_000 {
            return Err(MtpError::GasError("Gas limit out of range (1-2B)".into()));
        }

        Ok(GasCounter::new(limit))
    }

    /// Consume gas, returning error if limit exceeded
    pub fn consume(&mut self, amount: u64) -> Result<(), MtpError> {
        self.used = self.used.saturating_add(amount);
        if self.used > self.limit {
            return Err(MtpError {
                error: "GasExhausted".to_string(),
                message: None,
                gasLimit: Some(self.limit),
                gasUsed: Some(self.used),
            });
        }
        Ok(())
    }

    /// Get remaining gas
    pub fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.used)
    }

    /// Get gas used
    pub fn used(&self) -> u64 {
        self.used
    }

    /// Check if gas is exhausted
    pub fn is_exhausted(&self) -> bool {
        self.used >= self.limit
    }

    /// Get error for gas exhaustion
    pub fn error(&self) -> MtpError {
        MtpError {
            error: "GasExhausted".to_string(),
            message: None,
            gasLimit: Some(self.limit),
            gasUsed: Some(self.used),
        }
    }
}

/// Gas cost table as per Annex A
pub struct GasCosts;

impl GasCosts {
    pub fn literal() -> u64 {
        1
    }
    pub fn binary_op() -> u64 {
        2
    }
    pub fn comparison() -> u64 {
        1
    }
    pub fn function_call() -> u64 {
        5
    }
    pub fn tail_call() -> u64 {
        0
    }
    pub fn non_tail_recursion() -> u64 {
        2
    }
    pub fn object_access() -> u64 {
        1
    }
    pub fn array_access() -> u64 {
        1
    }
    pub fn if_statement() -> u64 {
        1
    }
    pub fn pattern_match_case() -> u64 {
        3
    }
    pub fn json_parse(base: u64, length: usize) -> u64 {
        base + (length as u64 / 10)
    }
    pub fn effect_call_db_read() -> u64 {
        50
    }
    pub fn effect_call_db_write() -> u64 {
        100
    }
    pub fn effect_call_http_out() -> u64 {
        100
    }
    pub fn effect_call_log() -> u64 {
        20
    }
    pub fn effect_call_async() -> u64 {
        50
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_counter() {
        let mut counter = GasCounter::new(1000);

        assert!(counter.consume(500).is_ok());
        assert_eq!(counter.used(), 500);
        assert_eq!(counter.remaining(), 500);

        assert!(counter.consume(600).is_err()); // Exceeds limit
        assert!(counter.is_exhausted());
    }

    #[test]
    fn test_gas_limits() {
        let counter = GasCounter::new(100);
        assert!(counter.consume(50).is_ok());
        assert!(counter.consume(60).is_err());
    }
}
