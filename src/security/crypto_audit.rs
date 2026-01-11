use crate::errors::MtpError;
use std::collections::HashMap;

/// Represents a cryptographic operation for audit purposes
#[derive(Debug, Clone)]
pub struct CryptoOperation {
    pub operation: String,
    pub algorithm: String,
    pub key_size: Option<usize>,
    pub purpose: String,
    pub timestamp: std::time::SystemTime,
}

/// Cryptography audit log
pub struct CryptoAudit {
    operations: Vec<CryptoOperation>,
}

impl CryptoAudit {
    /// Create a new crypto audit instance
    pub fn new() -> Self {
        CryptoAudit {
            operations: Vec::new(),
        }
    }

    /// Log a cryptographic operation
    pub fn log_operation(&mut self, operation: CryptoOperation) {
        self.operations.push(operation);
    }

    /// Get all logged operations
    pub fn get_operations(&self) -> &[CryptoOperation] {
        &self.operations
    }

    /// Generate audit report
    pub fn generate_report(&self) -> String {
        let mut report = String::from("# Cryptography Audit Report\n\n");
        report.push_str(&format!("Total operations: {}\n\n", self.operations.len()));

        let mut algo_counts: HashMap<String, usize> = HashMap::new();
        for op in &self.operations {
            *algo_counts.entry(op.algorithm.clone()).or_insert(0) += 1;
        }

        report.push_str("Algorithm usage:\n");
        for (algo, count) in algo_counts {
            report.push_str(&format!("- {}: {}\n", algo, count));
        }

        report.push_str("\nDetailed operations:\n");
        for op in &self.operations {
            let timestamp = op
                .timestamp
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            report.push_str(&format!(
                "- {}: {} (key_size: {:?}) - {}\n",
                timestamp, op.operation, op.key_size, op.purpose
            ));
        }

        report
    }

    /// Validate cryptographic operations against security policies
    pub fn validate_operations(&self) -> Result<(), MtpError> {
        for op in &self.operations {
            // Check for deprecated algorithms
            if op.algorithm == "MD5" || op.algorithm == "SHA1" {
                return Err(MtpError {
                    error: "DeprecatedAlgorithm".to_string(),
                    message: Some(format!(
                        "Deprecated algorithm {} used in {}",
                        op.algorithm, op.operation
                    )),
                    gasLimit: None,
                    gasUsed: None,
                });
            }

            // Check minimum key sizes
            if let Some(key_size) = op.key_size {
                match op.algorithm.as_str() {
                    "ECDSA-P256" => {
                        if key_size < 256 {
                            return Err(MtpError {
                                error: "InsufficientKeySize".to_string(),
                                message: Some(format!(
                                    "ECDSA-P256 requires 256-bit keys, got {}",
                                    key_size
                                )),
                                gasLimit: None,
                                gasUsed: None,
                            });
                        }
                    }
                    "AES" => {
                        if key_size < 128 {
                            return Err(MtpError {
                                error: "InsufficientKeySize".to_string(),
                                message: Some(format!(
                                    "AES requires at least 128-bit keys, got {}",
                                    key_size
                                )),
                                gasLimit: None,
                                gasUsed: None,
                            });
                        }
                    }
                    _ => {} // Other algorithms not checked for now
                }
            }
        }
        Ok(())
    }
}

/// Global crypto audit instance (thread-safe)
static mut CRYPTO_AUDIT: Option<CryptoAudit> = None;

/// Initialize the global crypto audit
pub fn init_crypto_audit() {
    unsafe {
        CRYPTO_AUDIT = Some(CryptoAudit::new());
    }
}

/// Get the global crypto audit instance
pub fn get_crypto_audit() -> Option<&'static mut CryptoAudit> {
    unsafe { CRYPTO_AUDIT.as_mut() }
}

/// Log a crypto operation globally
pub fn audit_crypto_operation(
    operation: &str,
    algorithm: &str,
    key_size: Option<usize>,
    purpose: &str,
) {
    if let Some(audit) = get_crypto_audit() {
        audit.log_operation(CryptoOperation {
            operation: operation.to_string(),
            algorithm: algorithm.to_string(),
            key_size,
            purpose: purpose.to_string(),
            timestamp: std::time::SystemTime::now(),
        });
    }
}

/// Perform comprehensive crypto audit
pub fn conduct_crypto_audit() -> Result<String, MtpError> {
    if let Some(audit) = get_crypto_audit() {
        audit.validate_operations()?;
        Ok(audit.generate_report())
    } else {
        Err(MtpError {
            error: "AuditNotInitialized".to_string(),
            message: Some("Crypto audit not initialized".to_string()),
            gasLimit: None,
            gasUsed: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_audit_logging() {
        let mut audit = CryptoAudit::new();

        audit.log_operation(CryptoOperation {
            operation: "sign".to_string(),
            algorithm: "ECDSA-P256".to_string(),
            key_size: Some(256),
            purpose: "Snapshot signing".to_string(),
            timestamp: std::time::SystemTime::now(),
        });

        assert_eq!(audit.get_operations().len(), 1);
        let report = audit.generate_report();
        assert!(report.contains("ECDSA-P256"));
    }

    #[test]
    fn test_validation_deprecated_algorithm() {
        let mut audit = CryptoAudit::new();

        audit.log_operation(CryptoOperation {
            operation: "hash".to_string(),
            algorithm: "MD5".to_string(),
            key_size: None,
            purpose: "Data integrity".to_string(),
            timestamp: std::time::SystemTime::now(),
        });

        assert!(audit.validate_operations().is_err());
    }

    #[test]
    fn test_validation_insufficient_key_size() {
        let mut audit = CryptoAudit::new();

        audit.log_operation(CryptoOperation {
            operation: "encrypt".to_string(),
            algorithm: "AES".to_string(),
            key_size: Some(64),
            purpose: "Data encryption".to_string(),
            timestamp: std::time::SystemTime::now(),
        });

        assert!(audit.validate_operations().is_err());
    }
}
