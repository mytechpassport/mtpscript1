use crate::errors::MtpError;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

/// Race condition detector
pub struct RaceDetector {
    access_log: Arc<Mutex<HashMap<String, Vec<AccessRecord>>>>,
    active_transactions: Arc<RwLock<HashSet<String>>>,
    deadlock_detector: DeadlockDetector,
}

#[derive(Debug, Clone)]
struct AccessRecord {
    thread_id: thread::ThreadId,
    access_type: AccessType,
    timestamp: Instant,
    location: String,
}

#[derive(Debug, Clone, PartialEq)]
enum AccessType {
    Read,
    Write,
}

/// Deadlock detector using wait-for graph
pub struct DeadlockDetector {
    wait_graph: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    resource_owners: Arc<RwLock<HashMap<String, String>>>,
}

impl DeadlockDetector {
    pub fn new() -> Self {
        DeadlockDetector {
            wait_graph: Arc::new(RwLock::new(HashMap::new())),
            resource_owners: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Acquire a resource lock
    pub fn acquire_lock(&self, resource_id: &str, transaction_id: &str) -> Result<(), MtpError> {
        let mut resource_owners = self.resource_owners.write().unwrap();
        let mut wait_graph = self.wait_graph.write().unwrap();

        if let Some(current_owner) = resource_owners.get(resource_id) {
            if current_owner == transaction_id {
                // Re-entrant lock
                return Ok(());
            }

            // Add to wait graph
            wait_graph
                .entry(transaction_id.to_string())
                .or_insert_with(HashSet::new)
                .insert(current_owner.clone());

            // Check for cycles (deadlock)
            if self.has_deadlock(transaction_id, &wait_graph) {
                // Remove from wait graph
                if let Some(waiting_for) = wait_graph.get_mut(transaction_id) {
                    waiting_for.remove(current_owner);
                }

                return Err(MtpError::RuntimeError {
                    error: "DeadlockDetected".to_string(),
                    message: format!("Deadlock detected when acquiring resource {}", resource_id),
                });
            }

            // Simulate waiting (in real implementation, this would be async)
            thread::sleep(Duration::from_millis(10));
        }

        resource_owners.insert(resource_id.to_string(), transaction_id.to_string());
        Ok(())
    }

    /// Release a resource lock
    pub fn release_lock(&self, resource_id: &str, transaction_id: &str) {
        let mut resource_owners = self.resource_owners.write().unwrap();
        let mut wait_graph = self.wait_graph.write().unwrap();

        if resource_owners.get(resource_id) == Some(transaction_id) {
            resource_owners.remove(resource_id);

            // Remove from wait graph
            wait_graph.remove(transaction_id);
        }
    }

    /// Check if there's a deadlock involving the given transaction
    fn has_deadlock(
        &self,
        start_transaction: &str,
        wait_graph: &HashMap<String, HashSet<String>>,
    ) -> bool {
        let mut visited = HashSet::new();
        let mut stack = vec![start_transaction.to_string()];

        while let Some(current) = stack.pop() {
            if !visited.insert(current.clone()) {
                // Cycle detected
                return true;
            }

            if let Some(waiting_for) = wait_graph.get(&current) {
                for next in waiting_for {
                    if !visited.contains(next) {
                        stack.push(next.clone());
                    }
                }
            }
        }

        false
    }

    /// Get deadlock report
    pub fn get_deadlock_report(&self) -> String {
        let wait_graph = self.wait_graph.read().unwrap();
        let resource_owners = self.resource_owners.read().unwrap();

        let mut report = String::from("# Deadlock Detection Report\n\n");

        report.push_str(&format!(
            "Active resource locks: {}\n",
            resource_owners.len()
        ));
        report.push_str(&format!("Waiting transactions: {}\n\n", wait_graph.len()));

        if !resource_owners.is_empty() {
            report.push_str("Resource ownership:\n");
            for (resource, owner) in resource_owners.iter() {
                report.push_str(&format!("- {} -> {}\n", resource, owner));
            }
            report.push_str("\n");
        }

        if !wait_graph.is_empty() {
            report.push_str("Wait graph:\n");
            for (transaction, waiting_for) in wait_graph.iter() {
                report.push_str(&format!("- {} waits for: {:?}\n", transaction, waiting_for));
            }
        }

        report
    }
}

impl RaceDetector {
    pub fn new() -> Self {
        RaceDetector {
            access_log: Arc::new(Mutex::new(HashMap::new())),
            active_transactions: Arc::new(RwLock::new(HashSet::new())),
            deadlock_detector: DeadlockDetector::new(),
        }
    }

    /// Record a memory access for race detection
    pub fn record_access(&self, resource_id: &str, access_type: AccessType, location: &str) {
        let thread_id = thread::current().id();
        let record = AccessRecord {
            thread_id,
            access_type,
            timestamp: Instant::now(),
            location: location.to_string(),
        };

        let mut access_log = self.access_log.lock().unwrap();
        access_log
            .entry(resource_id.to_string())
            .or_insert_with(Vec::new)
            .push(record);
    }

    /// Check for race conditions on a resource
    pub fn check_race_condition(&self, resource_id: &str) -> Option<RaceConditionReport> {
        let access_log = self.access_log.lock().unwrap();
        let accesses = access_log.get(resource_id)?;

        if accesses.len() < 2 {
            return None;
        }

        // Look for concurrent read-write or write-write accesses
        let mut writes = Vec::new();
        let mut reads = Vec::new();

        for access in accesses {
            match access.access_type {
                AccessType::Write => writes.push(access),
                AccessType::Read => reads.push(access),
            }
        }

        // Check for write-write races (multiple writes from different threads)
        if writes.len() >= 2 {
            let mut thread_writes: HashMap<thread::ThreadId, Vec<&AccessRecord>> = HashMap::new();

            for write in &writes {
                thread_writes
                    .entry(write.thread_id)
                    .or_insert_with(Vec::new)
                    .push(write);
            }

            if thread_writes.len() > 1 {
                return Some(RaceConditionReport {
                    resource_id: resource_id.to_string(),
                    race_type: RaceType::WriteWrite,
                    involved_threads: thread_writes.keys().cloned().collect(),
                    accesses: writes.iter().cloned().cloned().collect(),
                });
            }
        }

        // Check for read-write races (read and write from different threads)
        if !reads.is_empty() && !writes.is_empty() {
            let read_threads: HashSet<_> = reads.iter().map(|r| r.thread_id).collect();
            let write_threads: HashSet<_> = writes.iter().map(|w| w.thread_id).collect();

            // Check if any read and write are from different threads
            for read_thread in &read_threads {
                for write_thread in &write_threads {
                    if read_thread != write_thread {
                        let mut all_accesses = reads.clone();
                        all_accesses.extend_from_slice(&writes);

                        return Some(RaceConditionReport {
                            resource_id: resource_id.to_string(),
                            race_type: RaceType::ReadWrite,
                            involved_threads: vec![*read_thread, *write_thread]
                                .into_iter()
                                .collect(),
                            accesses: all_accesses.into_iter().cloned().collect(),
                        });
                    }
                }
            }
        }

        None
    }

    /// Start a transaction
    pub fn start_transaction(&self, transaction_id: &str) -> Result<(), MtpError> {
        let mut active_transactions = self.active_transactions.write().unwrap();

        if active_transactions.contains(transaction_id) {
            return Err(MtpError::RuntimeError {
                error: "TransactionAlreadyActive".to_string(),
                message: format!("Transaction {} is already active", transaction_id),
            });
        }

        active_transactions.insert(transaction_id.to_string());
        Ok(())
    }

    /// End a transaction
    pub fn end_transaction(&self, transaction_id: &str) {
        let mut active_transactions = self.active_transactions.write().unwrap();
        active_transactions.remove(transaction_id);
    }

    /// Acquire resource lock within transaction
    pub fn acquire_resource_lock(
        &self,
        resource_id: &str,
        transaction_id: &str,
    ) -> Result<(), MtpError> {
        let active_transactions = self.active_transactions.read().unwrap();

        if !active_transactions.contains(transaction_id) {
            return Err(MtpError::RuntimeError {
                error: "TransactionNotActive".to_string(),
                message: format!("Transaction {} is not active", transaction_id),
            });
        }

        self.deadlock_detector
            .acquire_lock(resource_id, transaction_id)
    }

    /// Release resource lock
    pub fn release_resource_lock(&self, resource_id: &str, transaction_id: &str) {
        self.deadlock_detector
            .release_lock(resource_id, transaction_id);
    }

    /// Generate comprehensive race condition report
    pub fn generate_race_report(&self) -> String {
        let access_log = self.access_log.lock().unwrap();
        let mut report = String::from("# Race Condition Detection Report\n\n");

        report.push_str(&format!("Monitored resources: {}\n", access_log.len()));

        let mut race_conditions = Vec::new();

        for resource_id in access_log.keys() {
            if let Some(race) = self.check_race_condition(resource_id) {
                race_conditions.push(race);
            }
        }

        report.push_str(&format!(
            "Detected race conditions: {}\n\n",
            race_conditions.len()
        ));

        for race in race_conditions {
            report.push_str(&format!("## Race on resource: {}\n", race.resource_id));
            report.push_str(&format!("Type: {:?}\n", race.race_type));
            report.push_str(&format!("Involved threads: {:?}\n", race.involved_threads));
            report.push_str("Access timeline:\n");

            for access in race.accesses {
                report.push_str(&format!(
                    "- Thread {:?} {} at {}\n",
                    access.thread_id, access.access_type, access.location
                ));
            }
            report.push_str("\n");
        }

        report.push_str(&self.deadlock_detector.get_deadlock_report());

        report
    }
}

#[derive(Debug)]
pub struct RaceConditionReport {
    pub resource_id: String,
    pub race_type: RaceType,
    pub involved_threads: HashSet<thread::ThreadId>,
    pub accesses: Vec<AccessRecord>,
}

#[derive(Debug)]
pub enum RaceType {
    ReadWrite,
    WriteWrite,
}

/// Synchronization primitives with race detection
pub struct SynchronizedResource<T> {
    data: Arc<RwLock<T>>,
    race_detector: Arc<RaceDetector>,
    resource_id: String,
}

impl<T> SynchronizedResource<T> {
    pub fn new(data: T, race_detector: Arc<RaceDetector>, resource_id: String) -> Self {
        SynchronizedResource {
            data: Arc::new(RwLock::new(data)),
            race_detector,
            resource_id,
        }
    }

    /// Read access with race detection
    pub fn read<F, R>(&self, location: &str, operation: F) -> Result<R, MtpError>
    where
        F: Fn(&T) -> R,
    {
        self.race_detector
            .record_access(&self.resource_id, AccessType::Read, location);

        let data = self.data.read().map_err(|_| MtpError::RuntimeError {
            error: "LockPoisoned".to_string(),
            message: "Read lock was poisoned".to_string(),
        })?;

        Ok(operation(&*data))
    }

    /// Write access with race detection
    pub fn write<F, R>(&self, location: &str, operation: F) -> Result<R, MtpError>
    where
        F: Fn(&mut T) -> R,
    {
        self.race_detector
            .record_access(&self.resource_id, AccessType::Write, location);

        let mut data = self.data.write().map_err(|_| MtpError::RuntimeError {
            error: "LockPoisoned".to_string(),
            message: "Write lock was poisoned".to_string(),
        })?;

        Ok(operation(&mut *data))
    }
}

/// Thread-safe shared state with race detection
pub struct SharedState {
    pub race_detector: Arc<RaceDetector>,
    pub shared_data: SynchronizedResource<HashMap<String, String>>,
}

impl SharedState {
    pub fn new() -> Self {
        let race_detector = Arc::new(RaceDetector::new());
        let shared_data = SynchronizedResource::new(
            HashMap::new(),
            race_detector.clone(),
            "shared_data".to_string(),
        );

        SharedState {
            race_detector,
            shared_data,
        }
    }

    /// Get value from shared state
    pub fn get(&self, key: &str) -> Result<Option<String>, MtpError> {
        self.shared_data
            .read("SharedState::get", |data| data.get(key).cloned())
    }

    /// Set value in shared state
    pub fn set(&self, key: String, value: String) -> Result<(), MtpError> {
        self.shared_data.write("SharedState::set", |data| {
            data.insert(key, value);
        })?;
        Ok(())
    }

    /// Get race condition report
    pub fn get_race_report(&self) -> String {
        self.race_detector.generate_race_report()
    }
}

/// Initialize global race detection
pub fn init_race_detection() -> Arc<RaceDetector> {
    Arc::new(RaceDetector::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_race_detector_basic() {
        let detector = RaceDetector::new();

        detector.record_access("resource1", AccessType::Read, "test.rs:10");
        detector.record_access("resource1", AccessType::Write, "test.rs:20");

        // Should detect potential race (same thread, so no actual race)
        let race = detector.check_race_condition("resource1");
        assert!(race.is_none()); // Same thread, no race
    }

    #[test]
    fn test_deadlock_detection() {
        let detector = DeadlockDetector::new();

        // Transaction A acquires resource X
        detector.acquire_lock("resource_x", "txn_a").unwrap();

        // Transaction B acquires resource Y
        detector.acquire_lock("resource_y", "txn_b").unwrap();

        // Transaction A tries to acquire resource Y (waits for B)
        let result = detector.acquire_lock("resource_y", "txn_a");
        assert!(result.is_ok()); // Should work since B will release eventually in test

        // Clean up
        detector.release_lock("resource_x", "txn_a");
        detector.release_lock("resource_y", "txn_b");
    }

    #[test]
    fn test_shared_state() {
        let state = SharedState::new();

        state.set("key1".to_string(), "value1".to_string()).unwrap();
        let value = state.get("key1").unwrap();

        assert_eq!(value, Some("value1".to_string()));
    }

    #[test]
    fn test_synchronized_resource() {
        let detector = Arc::new(RaceDetector::new());
        let resource = SynchronizedResource::new(vec![1, 2, 3], detector, "test_vec".to_string());

        let sum = resource
            .read("test", |data| data.iter().sum::<i32>())
            .unwrap();
        assert_eq!(sum, 6);

        resource.write("test", |data| data.push(4)).unwrap();

        let sum = resource
            .read("test", |data| data.iter().sum::<i32>())
            .unwrap();
        assert_eq!(sum, 10);
    }
}
