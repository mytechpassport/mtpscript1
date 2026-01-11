use crate::errors::MtpError;
use sha2::{Digest, Sha256};
use crc32fast::Hasher as Crc32Hasher;
use std::collections::HashMap;

/// Checksum validator for critical data structures
pub struct IntegrityValidator {
    checksums: HashMap<String, Vec<u8>>,
}

impl IntegrityValidator {
    pub fn new() -> Self {
        IntegrityValidator {
            checksums: HashMap::new(),
        }
    }

    /// Compute SHA-256 checksum of data
    pub fn compute_sha256(&self, data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// Compute CRC32 checksum of data
    pub fn compute_crc32(&self, data: &[u8]) -> u32 {
        let mut hasher = Crc32Hasher::new();
        hasher.update(data);
        hasher.finalize()
    }

    /// Store checksum for a critical structure
    pub fn store_checksum(&mut self, key: String, data: &[u8]) {
        let checksum = self.compute_sha256(data);
        self.checksums.insert(key, checksum);
    }

    /// Validate checksum of a critical structure
    pub fn validate_checksum(&self, key: &str, data: &[u8]) -> Result<(), MtpError> {
        if let Some(stored_checksum) = self.checksums.get(key) {
            let computed_checksum = self.compute_sha256(data);
            if stored_checksum == &computed_checksum {
                Ok(())
            } else {
                Err(MtpError::IntegrityError(format!("Checksum validation failed for {}", key)))
            }
        } else {
            Err(MtpError::IntegrityError(format!("No checksum stored for {}", key)))
        }
    }

    /// Validate multiple structures atomically
    pub fn validate_batch(&self, structures: &HashMap<String, Vec<u8>>) -> Result<(), MtpError> {
        for (key, data) in structures {
            self.validate_checksum(key, data)?;
        }
        Ok(())
    }

    /// Update checksum for a structure
    pub fn update_checksum(&mut self, key: String, data: &[u8]) {
        self.store_checksum(key, data);
    }

    /// Remove checksum for a structure
    pub fn remove_checksum(&mut self, key: &str) {
        self.checksums.remove(key);
    }

    /// Get stored checksum for debugging
    pub fn get_checksum(&self, key: &str) -> Option<&Vec<u8>> {
        self.checksums.get(key)
    }
}

/// Validator for AST structures
pub struct AstValidator {
    validator: IntegrityValidator,
}

impl AstValidator {
    pub fn new() -> Self {
        AstValidator {
            validator: IntegrityValidator::new(),
        }
    }

    /// Validate AST integrity before compilation
    pub fn validate_ast(&self, ast_data: &[u8], expected_checksum: &[u8]) -> Result<(), MtpError> {
        let computed = self.validator.compute_sha256(ast_data);
        if computed == expected_checksum {
            Ok(())
        } else {
            Err(MtpError::IntegrityError("AST integrity check failed".into()))
        }
    }

    /// Validate IR structures
    pub fn validate_ir(&self, ir_data: &[u8]) -> Result<(), MtpError> {
        // Check for obvious corruption patterns
        if ir_data.is_empty() {
            return Err(MtpError::IntegrityError("IR data is empty".into()));
        }

        // Check for null bytes in unexpected places
        let null_count = ir_data.iter().filter(|&&b| b == 0).count();
        if null_count > ir_data.len() / 10 { // More than 10% null bytes
            return Err(MtpError::IntegrityError("IR data contains too many null bytes".into()));
        }

        // Validate CRC32
        let crc32 = self.validator.compute_crc32(ir_data);
        if crc32 == 0 {
            return Err(MtpError::IntegrityError("IR CRC32 validation failed".into()));
        }

        Ok(())
    }
}

/// Validator for runtime values
pub struct ValueValidator {
    validator: IntegrityValidator,
}

impl ValueValidator {
    pub fn new() -> Self {
        ValueValidator {
            validator: IntegrityValidator::new(),
        }
    }

    /// Validate value structure integrity
    pub fn validate_value(&self, value_data: &[u8]) -> Result<(), MtpError> {
        // Basic size checks
        if value_data.is_empty() {
            return Err(MtpError::IntegrityError("Value data is empty".into()));
        }

        if value_data.len() > 10 * 1024 * 1024 { // 10MB limit
            return Err(MtpError::IntegrityError("Value data too large".into()));
        }

        // Check for memory corruption patterns
        self.check_memory_patterns(value_data)
    }

    fn check_memory_patterns(&self, data: &[u8]) -> Result<(), MtpError> {
        // Check for repeated patterns that might indicate uninitialized memory
        if data.len() >= 16 {
            let first_8 = &data[0..8];
            let last_8 = &data[data.len()-8..];
            
            if first_8 == last_8 && data.len() > 16 {
                // Check if the pattern repeats
                let mut all_same = true;
                for chunk in data.chunks(8) {
                    if chunk != first_8 {
                        all_same = false;
                        break;
                    }
                }
                
                if all_same {
                    return Err(MtpError::IntegrityError("Value data shows memory corruption pattern".into()));
                }
            }
        }

        // Check for invalid UTF-8 in string portions (basic check)
        if let Ok(s) = std::str::from_utf8(data) {
            // If it's valid UTF-8, check for obviously wrong content
            if s.contains('\0') && s.len() > 1024 {
                return Err(MtpError::IntegrityError("Value contains unexpected null bytes".into()));
            }
        }

        Ok(())
    }
}

/// Global integrity checker
pub struct GlobalIntegrityChecker {
    ast_validator: AstValidator,
    value_validator: ValueValidator,
    structure_validator: IntegrityValidator,
}

impl GlobalIntegrityChecker {
    pub fn new() -> Self {
        GlobalIntegrityChecker {
            ast_validator: AstValidator::new(),
            value_validator: ValueValidator::new(),
            structure_validator: IntegrityValidator::new(),
        }
    }

    /// Comprehensive integrity check before critical operations
    pub fn pre_operation_check(&self, ast_data: &[u8], ir_data: &[u8], value_data: &[u8]) -> Result<(), MtpError> {
        // Validate AST
        let ast_checksum = self.ast_validator.validator.compute_sha256(ast_data);
        self.ast_validator.validate_ast(ast_data, &ast_checksum)?;

        // Validate IR
        self.ast_validator.validate_ir(ir_data)?;

        // Validate runtime values
        self.value_validator.validate_value(value_data)?;

        // Validate internal structures
        let mut structures = HashMap::new();
        structures.insert("ast".to_string(), ast_data.to_vec());
        structures.insert("ir".to_string(), ir_data.to_vec());
        structures.insert("values".to_string(), value_data.to_vec());
        
        self.structure_validator.validate_batch(&structures)?;

        Ok(())
    }

    /// Check system integrity during runtime
    pub fn runtime_integrity_check(&self) -> Result<(), MtpError> {
        // Check that validators themselves are not corrupted
        // This is a basic sanity check
        
        if self.ast_validator.validator.checksums.is_empty() {
            // This might indicate corruption or fresh start
            return Ok(());
        }

        // Validate that checksums are reasonable size
        for checksum in self.ast_validator.validator.checksums.values() {
            if checksum.len() != 32 { // SHA-256 should be 32 bytes
                return Err(MtpError::IntegrityError("Invalid checksum size in AST validator".into()));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_validation() {
        let mut validator = IntegrityValidator::new();
        
        let data = b"Hello, world!";
        validator.store_checksum("test".to_string(), data);
        
        assert!(validator.validate_checksum("test", data).is_ok());
        assert!(validator.validate_checksum("test", b"different data").is_err());
    }

    #[test]
    fn test_ast_validation() {
        let validator = AstValidator::new();
        let ast_data = b"valid ast data";
        let checksum = validator.validator.compute_sha256(ast_data);
        
        assert!(validator.validate_ast(ast_data, &checksum).is_ok());
        assert!(validator.validate_ast(ast_data, &[0; 32]).is_err());
    }

    #[test]
    fn test_value_validation() {
        let validator = ValueValidator::new();
        
        // Valid data
        assert!(validator.validate_value(b"valid string").is_ok());
        
        // Empty data should fail
        assert!(validator.validate_value(&[]).is_err());
        
        // Very large data should fail
        let large_data = vec![0u8; 20 * 1024 * 1024]; // 20MB
        assert!(validator.validate_value(&large_data).is_err());
    }
}