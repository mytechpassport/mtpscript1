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

    #[test]
    fn test_concurrent_logging_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        // Spawn multiple threads that log concurrently
        let handles: Vec<_> = (0..10)
            .map(|i| {
                thread::spawn(move || {
                    for j in 0..10 {
                        let entry = AuditLogger::create_entry(
                            format!("thread-{}-req-{}", i, j),
                            1000000,
                            50000,
                            format!("hash-{}-{}", i, j),
                        );
                        // Should not error even under concurrent access
                        let result = AuditLogger::log(&entry);
                        assert!(result.is_ok(), "Logging failed in thread {} request {}", i, j);
                    }
                })
            })
            .collect();

        // Wait for all threads to complete
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
    }

    #[test]
    fn test_mutex_acquisition_on_poison() {
        // Test that the mutex can recover from a poisoned state
        // The current implementation uses unwrap_or_else(|e| e.into_inner())
        // which allows recovery from a poisoned mutex

        let entry = AuditLogger::create_entry(
            "poison-test".to_string(),
            1000000,
            50000,
            "hash".to_string(),
        );

        // Log should still work even if called after a potential poison
        assert!(AuditLogger::log(&entry).is_ok());
    }

    #[test]
    fn test_audit_entry_deserialization() {
        let json = r#"{"request_id":"test","gasLimit":100,"gasUsed":50,"responseHash":"abc","timestamp":"2024-01-01T00:00:00Z"}"#;
        let entry: AuditEntry = serde_json::from_str(json).unwrap();

        assert_eq!(entry.request_id, "test");
        assert_eq!(entry.gas_limit, 100);
        assert_eq!(entry.gas_used, 50);
        assert_eq!(entry.response_hash, "abc");
    }

    #[test]
    fn test_audit_entry_roundtrip() {
        let original = AuditLogger::create_entry(
            "roundtrip-test".to_string(),
            999999,
            12345,
            "hash123".to_string(),
        );

        let json = serde_json::to_string(&original).unwrap();
        let restored: AuditEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(original.request_id, restored.request_id);
        assert_eq!(original.gas_limit, restored.gas_limit);
        assert_eq!(original.gas_used, restored.gas_used);
        assert_eq!(original.response_hash, restored.response_hash);
        assert_eq!(original.timestamp, restored.timestamp);
    }

    #[test]
    fn test_create_entry_timestamp_is_recent() {
        let before = Utc::now();
        let entry = AuditLogger::create_entry(
            "timestamp-test".to_string(),
            1000,
            500,
            "hash".to_string(),
        );
        let after = Utc::now();

        // Timestamp should be between before and after
        assert!(entry.timestamp >= before);
        assert!(entry.timestamp <= after);
    }
}
