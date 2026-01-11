//! Secure memory wipe functionality per §27.3
//!
//! Provides secure disposal of sensitive data to prevent information leakage.

use std::ptr;

/// Securely wipe a vector by overwriting its contents
pub fn wipe_vec<T: Copy + Default>(data: &mut Vec<T>) {
    // Overwrite with default values
    for elem in data.iter_mut() {
        *elem = T::default();
    }
    // Shrink to zero capacity to free memory
    data.clear();
    data.shrink_to_fit();
}

/// Securely wipe a byte slice by overwriting with zeros
pub fn wipe_bytes(data: &mut [u8]) {
    // Use volatile writes to prevent compiler optimization
    for byte in data.iter_mut() {
        unsafe {
            ptr::write_volatile(byte, 0);
        }
    }
    // Memory barrier to ensure writes are committed
    std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst);
}

/// Securely wipe a string by overwriting and clearing
pub fn wipe_string(s: &mut String) {
    wipe_bytes(unsafe { s.as_bytes_mut() });
    s.clear();
}

/// Secure wipe for boxed data
pub fn wipe_box<T: Copy + Default>(data: Box<T>) -> Box<T> {
    let mut boxed = data;
    unsafe {
        ptr::write_volatile(&mut *boxed, T::default());
    }
    std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst);
    boxed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wipe_vec() {
        let mut v = vec![1i32, 2, 3, 4];
        wipe_vec(&mut v);
        assert!(v.is_empty());
        assert_eq!(v.capacity(), 0);
    }

    #[test]
    fn test_wipe_bytes() {
        let mut data = [1u8, 2, 3, 4];
        wipe_bytes(&mut data);
        assert_eq!(data, [0u8; 4]);
    }

    #[test]
    fn test_wipe_string() {
        let mut s = String::from("secret");
        wipe_string(&mut s);
        assert!(s.is_empty());
    }

    #[test]
    fn test_wipe_box() {
        let data = Box::new(42i32);
        let wiped = wipe_box(data);
        // Can't check contents easily, but ensure it's a valid Box
        assert_eq!(*wiped, 0); // Should be default value
    }
}
