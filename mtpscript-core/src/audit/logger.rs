use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::sync::{Mutex, OnceLock};

// Global mutex for thread-safe audit logging
fn audit_mutex() -> &'static Mutex<()> {
    static AUDIT_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
    AUDIT_MUTEX.get_or_init(|| Mutex::new(()))
}

/// Audit entry for logging request execution
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuditEntry {
    /// Unique request identifier
    pub request_id: String,
    /// Gas limit for this request
    #[serde(rename = "gasLimit")]
    pub gas_limit: u64,
    /// Gas actually used
    #[serde(rename = "gasUsed")]
    pub gas_used: u64,
    /// SHA-256 hash of the canonical JSON response body
    #[serde(rename = "responseHash")]
    pub response_hash: String,
    /// ISO 8601 timestamp
    pub timestamp: DateTime<Utc>,
}

/// Logger for audit events
pub struct AuditLogger;

impl AuditLogger {
    /// Log an audit entry to stderr as JSON line
    ///
    /// This function uses a mutex to ensure atomic line writes,
    /// preventing interleaved output from multiple threads.
    pub fn log(entry: &AuditEntry) -> Result<(), io::Error> {
        // Pre-serialize the JSON before acquiring the lock to minimize lock time
        let json = serde_json::to_string(entry)?;

        // Acquire lock for atomic write
        let _guard = audit_mutex().lock().unwrap_or_else(|e| e.into_inner());

        // Write the complete line atomically
        let stderr = io::stderr();
        let mut handle = stderr.lock();
        writeln!(handle, "{}", json)?;
        handle.flush()?;

        Ok(())
    }

    /// Create a new audit entry
    pub fn create_entry(
        request_id: String,
        gas_limit: u64,
        gas_used: u64,
        response_hash: String,
    ) -> AuditEntry {
        AuditEntry {
            request_id,
            gas_limit,
            gas_used,
            response_hash,
            timestamp: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_entry_serialization() {
        let entry = AuditEntry {
            request_id: "test-123".to_string(),
            gas_limit: 1000000,
            gas_used: 50000,
            response_hash: "abcd1234".to_string(),
            timestamp: Utc::now(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("gasLimit"));
        assert!(json.contains("gasUsed"));
        assert!(json.contains("responseHash"));
    }

    #[test]
    fn test_audit_logging() {
        let entry = AuditLogger::create_entry(
            "test-123".to_string(),
            1000000,
            50000,
            "abcd1234".to_string(),
        );

        // This would write to stderr, but in test we just check no error
        assert!(AuditLogger::log(&entry).is_ok());
    }

    #[test]
    fn test_audit_entry_fields() {
        let entry = AuditEntry {
            request_id: "req-456".to_string(),
            gas_limit: 2000000,
            gas_used: 150000,
            response_hash: "hash789".to_string(),
            timestamp: Utc::now(),
        };

        assert_eq!(entry.request_id, "req-456");
        assert_eq!(entry.gas_limit, 2000000);
        assert_eq!(entry.gas_used, 150000);
        assert_eq!(entry.response_hash, "hash789");
        assert!(entry.timestamp <= Utc::now());
    }

    #[test]
    fn test_audit_json_format() {
        let entry = AuditLogger::create_entry(
            "test-json".to_string(),
            1000000,
            50000,
            "abcd1234".to_string(),
        );

        let json = serde_json::to_string(&entry).unwrap();

        // Verify required fields are present with correct names
        assert!(json.contains(r#""request_id":"test-json""#));
        assert!(json.contains(r#""gasLimit":1000000"#));
        assert!(json.contains(r#""gasUsed":50000"#));
        assert!(json.contains(r#""responseHash":"abcd1234""#));
        assert!(json.contains(r#""timestamp""#));
    }
}
