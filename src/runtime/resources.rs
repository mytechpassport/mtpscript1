use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Resource tracker using RAII pattern
pub struct ResourceTracker {
    resources: HashMap<String, Resource>,
    total_memory: AtomicUsize,
}

#[derive(Debug)]
struct Resource {
    id: String,
    size: usize,
    kind: ResourceKind,
}

#[derive(Debug, Clone)]
enum ResourceKind {
    Interpreter,
    Heap,
    Stack,
    Network,
}

impl ResourceTracker {
    pub fn new() -> Self {
        ResourceTracker {
            resources: HashMap::new(),
            total_memory: AtomicUsize::new(0),
        }
    }

    /// Track a new resource
    pub fn track(&mut self, id: String, size: usize, kind: ResourceKind) {
        let resource = Resource {
            id: id.clone(),
            size,
            kind,
        };
        self.resources.insert(id, resource);
        self.total_memory.fetch_add(size, Ordering::Relaxed);
    }

    /// Release a tracked resource
    pub fn release(&mut self, id: &str) -> bool {
        if let Some(resource) = self.resources.remove(id) {
            self.total_memory
                .fetch_sub(resource.size, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    /// Get total tracked memory
    pub fn total_memory(&self) -> usize {
        self.total_memory.load(Ordering::Relaxed)
    }

    /// Check if we're within memory limits
    pub fn check_limits(&self, max_memory: usize) -> Result<(), &'static str> {
        if self.total_memory() > max_memory {
            Err("Memory limit exceeded")
        } else {
            Ok(())
        }
    }
}

impl Drop for ResourceTracker {
    fn drop(&mut self) {
        // Ensure all resources are released
        if !self.resources.is_empty() {
            eprintln!(
                "Warning: {} resources not properly released",
                self.resources.len()
            );
        }
    }
}

/// RAII wrapper for interpreter instances
pub struct InterpreterGuard<'a> {
    tracker: &'a mut ResourceTracker,
    id: String,
}

impl<'a> InterpreterGuard<'a> {
    pub fn new(tracker: &'a mut ResourceTracker, id: String, size: usize) -> Self {
        tracker.track(id.clone(), size, ResourceKind::Interpreter);
        InterpreterGuard { tracker, id }
    }
}

impl<'a> Drop for InterpreterGuard<'a> {
    fn drop(&mut self) {
        self.tracker.release(&self.id);
    }
}

/// RAII wrapper for heap allocations
pub struct HeapGuard<'a> {
    tracker: &'a mut ResourceTracker,
    id: String,
}

impl<'a> HeapGuard<'a> {
    pub fn new(tracker: &'a mut ResourceTracker, id: String, size: usize) -> Self {
        tracker.track(id.clone(), size, ResourceKind::Heap);
        HeapGuard { tracker, id }
    }

    /// Resize the heap allocation
    pub fn resize(&mut self, new_size: usize) -> Result<(), &'static str> {
        // Update tracking
        let old_resource = self.tracker.resources.get(&self.id).unwrap().clone();
        let size_diff = new_size as isize - old_resource.size as isize;

        if size_diff > 0 {
            self.tracker
                .total_memory
                .fetch_add(size_diff as usize, Ordering::Relaxed);
        } else {
            self.tracker
                .total_memory
                .fetch_sub((-size_diff) as usize, Ordering::Relaxed);
        }

        // Update resource record
        if let Some(resource) = self.tracker.resources.get_mut(&self.id) {
            resource.size = new_size;
        }

        Ok(())
    }
}

impl<'a> Drop for HeapGuard<'a> {
    fn drop(&mut self) {
        self.tracker.release(&self.id);
    }
}

/// RAII wrapper for network connections
pub struct NetworkGuard<'a> {
    tracker: &'a mut ResourceTracker,
    id: String,
}

impl<'a> NetworkGuard<'a> {
    pub fn new(tracker: &'a mut ResourceTracker, id: String) -> Self {
        // Network resources have a fixed size for tracking
        tracker.track(id.clone(), 1024, ResourceKind::Network);
        NetworkGuard { tracker, id }
    }
}

impl<'a> Drop for NetworkGuard<'a> {
    fn drop(&mut self) {
        self.tracker.release(&self.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_tracking() {
        let mut tracker = ResourceTracker::new();

        {
            let _guard = InterpreterGuard::new(&mut tracker, "interp1".to_string(), 1000);
            assert_eq!(tracker.total_memory(), 1000);
        }

        assert_eq!(tracker.total_memory(), 0);
    }

    #[test]
    fn test_heap_resize() {
        let mut tracker = ResourceTracker::new();

        {
            let mut guard = HeapGuard::new(&mut tracker, "heap1".to_string(), 1000);
            assert_eq!(tracker.total_memory(), 1000);

            guard.resize(2000).unwrap();
            assert_eq!(tracker.total_memory(), 2000);
        }

        assert_eq!(tracker.total_memory(), 0);
    }

    #[test]
    fn test_memory_limits() {
        let mut tracker = ResourceTracker::new();

        let _guard = InterpreterGuard::new(&mut tracker, "interp1".to_string(), 1000);

        assert!(tracker.check_limits(2000).is_ok());
        assert!(tracker.check_limits(500).is_err());
    }
}
