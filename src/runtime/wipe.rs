use std::collections::HashMap;
use std::ptr;

/// Interpreter with secure memory handling for PCI compliance
pub struct Interpreter {
    /// Heap memory that may contain sensitive data
    pub heap: Vec<u8>,
    /// Flag indicating PCI-sensitive data was processed
    pub pci_touched: bool,
    /// Stack memory
    pub stack: Vec<u8>,
    /// Global variables
    pub globals: HashMap<String, Vec<u8>>,
    /// Execution context data
    pub context: Vec<u8>,
}

impl Interpreter {
    /// Create a new interpreter instance
    pub fn new() -> Self {
        Interpreter {
            heap: Vec::new(),
            pci_touched: false,
            stack: Vec::new(),
            globals: HashMap::new(),
            context: Vec::new(),
        }
    }

    /// Create interpreter with pre-allocated memory
    pub fn with_capacity(heap_size: usize, stack_size: usize) -> Self {
        Interpreter {
            heap: Vec::with_capacity(heap_size),
            pci_touched: false,
            stack: Vec::with_capacity(stack_size),
            globals: HashMap::new(),
            context: Vec::new(),
        }
    }

    /// Mark that PCI-sensitive data has been touched
    pub fn mark_pci_touched(&mut self) {
        self.pci_touched = true;
    }

    /// Securely zero all sensitive memory
    /// Uses volatile writes to prevent compiler optimization
    pub fn zero_sensitive(&mut self) {
        if self.pci_touched {
            // Zero heap memory with volatile writes
            secure_zero(&mut self.heap);

            // Zero stack memory
            secure_zero(&mut self.stack);

            // Zero globals
            for (_, data) in self.globals.iter_mut() {
                secure_zero(data);
            }

            // Zero context
            secure_zero(&mut self.context);
        }
    }

    /// Get current heap size
    pub fn heap_size(&self) -> usize {
        self.heap.len()
    }

    /// Allocate memory on the heap
    pub fn allocate(&mut self, size: usize) -> Option<usize> {
        let offset = self.heap.len();
        self.heap.resize(offset + size, 0);
        Some(offset)
    }

    /// Read from heap at offset
    pub fn read_heap(&self, offset: usize, size: usize) -> Option<&[u8]> {
        if offset + size <= self.heap.len() {
            Some(&self.heap[offset..offset + size])
        } else {
            None
        }
    }

    /// Write to heap at offset
    pub fn write_heap(&mut self, offset: usize, data: &[u8]) -> bool {
        if offset + data.len() <= self.heap.len() {
            self.heap[offset..offset + data.len()].copy_from_slice(data);
            true
        } else {
            false
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Interpreter {
    fn drop(&mut self) {
        // Always zero memory on drop if PCI data was touched
        self.zero_sensitive();
    }
}

/// Securely zero a byte vector using volatile writes
/// This prevents the compiler from optimizing away the zeroing
fn secure_zero(data: &mut Vec<u8>) {
    // Use volatile write to prevent optimization
    for byte in data.iter_mut() {
        // volatile_write is not stable, use ptr::write_volatile
        unsafe {
            ptr::write_volatile(byte, 0);
        }
    }

    // Memory barrier to ensure writes complete
    std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst);
}

/// Securely zero a byte slice
pub fn secure_zero_slice(data: &mut [u8]) {
    for byte in data.iter_mut() {
        unsafe {
            ptr::write_volatile(byte, 0);
        }
    }
    std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst);
}

/// Wipe interpreter and release resources
/// If pci is true, performs secure zeroing before deallocation
pub fn wipe_interpreter(mut interp: Interpreter, pci: bool) {
    if pci || interp.pci_touched {
        interp.zero_sensitive();
    }
    // Interpreter will be dropped here, triggering additional cleanup
    drop(interp);
}

/// Create a cloned interpreter for a new request
/// The clone gets a fresh heap but shares no state with the original
pub fn clone_for_request(template: &Interpreter) -> Interpreter {
    Interpreter {
        // Fresh heap, no data from template
        heap: Vec::with_capacity(template.heap.capacity()),
        // PCI flag starts false
        pci_touched: false,
        // Fresh stack
        stack: Vec::with_capacity(template.stack.capacity()),
        // No globals inherited (effects injected separately)
        globals: HashMap::new(),
        // Fresh context
        context: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_zero() {
        let mut data = vec![1, 2, 3, 4, 5];
        secure_zero(&mut data);
        assert!(data.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_interpreter_wipe() {
        let mut interp = Interpreter::with_capacity(1024, 256);
        interp.heap.extend_from_slice(b"sensitive data");
        interp.mark_pci_touched();

        interp.zero_sensitive();

        assert!(interp.heap.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_clone_isolation() {
        let mut template = Interpreter::with_capacity(1024, 256);
        template.heap.extend_from_slice(b"template data");
        template.mark_pci_touched();

        let clone = clone_for_request(&template);

        // Clone should have empty heap
        assert!(clone.heap.is_empty());
        // Clone should not inherit PCI flag
        assert!(!clone.pci_touched);
    }

    #[test]
    fn test_heap_operations() {
        let mut interp = Interpreter::new();

        let offset = interp.allocate(10).unwrap();
        assert_eq!(offset, 0);

        let data = b"test data!";
        assert!(interp.write_heap(offset, data));

        let read = interp.read_heap(offset, 10).unwrap();
        assert_eq!(read, data);
    }
}
