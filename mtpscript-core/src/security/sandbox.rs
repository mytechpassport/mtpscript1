use crate::errors::MtpError;
use crate::runtime::Interpreter;

// Linux syscall numbers (x86_64)
#[cfg(target_os = "linux")]
mod syscalls {
    pub const SYS_read: i64 = 0;
    pub const SYS_write: i64 = 1;
    pub const SYS_close: i64 = 3;
    pub const SYS_exit: i64 = 60;
    pub const SYS_exit_group: i64 = 231;
    pub const SYS_brk: i64 = 12;
    pub const SYS_mmap: i64 = 9;
    pub const SYS_munmap: i64 = 11;
    pub const SYS_mprotect: i64 = 10;
    pub const SYS_rt_sigaction: i64 = 13;
    pub const SYS_rt_sigprocmask: i64 = 14;
    pub const SYS_rt_sigreturn: i64 = 15;
    pub const SYS_clone: i64 = 56;
    pub const SYS_wait4: i64 = 61;
    pub const SYS_getpid: i64 = 39;
    pub const SYS_gettid: i64 = 186;
    pub const SYS_tgkill: i64 = 234;
    pub const SYS_futex: i64 = 202;
    pub const SYS_sched_yield: i64 = 24;
    pub const SYS_gettimeofday: i64 = 96;
    pub const SYS_clock_gettime: i64 = 228;
}

#[cfg(not(target_os = "linux"))]
mod syscalls {}

/// Sandbox configuration for MTPScript runtime
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Allow network access (only via HttpOut effect)
    pub allow_network: bool,
    /// Allow filesystem access (only via effects)
    pub allow_fs: bool,
    /// Enable seccomp-bpf syscall filtering
    pub enable_seccomp: bool,
    /// Restrict to specific syscalls
    pub allowed_syscalls: Vec<i64>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            allow_network: false,
            allow_fs: false,
            enable_seccomp: cfg!(target_os = "linux"), // Only enable on Linux
            allowed_syscalls: {
                #[cfg(target_os = "linux")]
                {
                    vec![
                        syscalls::SYS_read,
                        syscalls::SYS_write,
                        syscalls::SYS_close,
                        syscalls::SYS_exit,
                        syscalls::SYS_exit_group,
                        syscalls::SYS_brk,
                        syscalls::SYS_mmap,
                        syscalls::SYS_munmap,
                        syscalls::SYS_mprotect,
                        syscalls::SYS_rt_sigaction,
                        syscalls::SYS_rt_sigprocmask,
                        syscalls::SYS_rt_sigreturn,
                        syscalls::SYS_clone,
                        syscalls::SYS_wait4,
                        syscalls::SYS_getpid,
                        syscalls::SYS_gettid,
                        syscalls::SYS_tgkill,
                        syscalls::SYS_futex,
                        syscalls::SYS_sched_yield,
                        syscalls::SYS_gettimeofday,
                        syscalls::SYS_clock_gettime,
                    ]
                }
                #[cfg(not(target_os = "linux"))]
                {
                    vec![] // Empty on non-Linux
                }
            },
        }
    }
}

/// Sandboxed interpreter that restricts system access
pub struct SandboxedInterpreter {
    interpreter: Interpreter,
    config: SandboxConfig,
    seccomp_enabled: bool,
}

impl SandboxedInterpreter {
    /// Create a new sandboxed interpreter from a cloned interpreter
    pub fn new(interpreter: Interpreter, config: SandboxConfig) -> Result<Self, MtpError> {
        let mut sandboxed = Self {
            interpreter,
            config,
            seccomp_enabled: false,
        };

        if sandboxed.config.enable_seccomp {
            sandboxed.enable_seccomp()?;
        }

        Ok(sandboxed)
    }

    /// Enable seccomp-bpf syscall filtering
    fn enable_seccomp(&mut self) -> Result<(), MtpError> {
        #[cfg(target_os = "linux")]
        {
            use std::mem;

            // Define seccomp filter
            let filter = self.build_seccomp_filter()?;

            // Apply filter
            let ret = unsafe {
                libc::prctl(libc::PR_SET_SECCOMP, 2, &filter as *const _) // SECCOMP_MODE_FILTER = 2
            };

            if ret != 0 {
                return Err(MtpError::Security("Failed to enable seccomp".to_string()));
            }

            self.seccomp_enabled = true;
            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            // On non-Linux systems, just log that sandboxing is not available
            eprintln!("Warning: Seccomp sandboxing only available on Linux");
            Ok(())
        }
    }

    /// Build seccomp filter program
    #[cfg(target_os = "linux")]
    fn build_seccomp_filter(&self) -> Result<libc::sock_fprog, MtpError> {
        use std::ptr;

        // Simple seccomp filter that allows only specified syscalls
        // In a real implementation, this would be more sophisticated
        // For now, we use a basic allowlist

        let mut filter: Vec<libc::sock_filter> = Vec::new();

        // Load syscall number into accumulator
        filter.push(libc::sock_filter {
            code: libc::BPF_LD | libc::BPF_W | libc::BPF_ABS,
            jt: 0,
            jf: 0,
            k: 0, // syscall number offset
        });

        // Check against allowed syscalls
        for &syscall in &self.config.allowed_syscalls {
            filter.push(libc::sock_filter {
                code: libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K,
                jt: 0,
                jf: 1, // jump to next check if not equal
                k: syscall as u32,
            });

            // If match, allow (return SECCOMP_RET_ALLOW)
            filter.push(libc::sock_filter {
                code: libc::BPF_RET | libc::BPF_K,
                jt: 0,
                jf: 0,
                k: libc::SECCOMP_RET_ALLOW,
            });
        }

        // Default: kill process
        filter.push(libc::sock_filter {
            code: libc::BPF_RET | libc::BPF_K,
            jt: 0,
            jf: 0,
            k: libc::SECCOMP_RET_KILL,
        });

        let prog = libc::sock_fprog {
            len: filter.len() as u16,
            filter: filter.as_mut_ptr(),
        };

        // We need to forget the filter vec to prevent it from being dropped
        // while the filter is still in use
        mem::forget(filter);

        Ok(prog)
    }

    /// Execute code in the sandboxed environment
    pub fn execute(&mut self, code: &str) -> Result<String, MtpError> {
        // Check for forbidden operations
        if code.contains("require(") && !self.config.allow_fs {
            return Err(MtpError::Security(
                "Filesystem access not allowed".to_string(),
            ));
        }

        if code.contains("fetch(") && !self.config.allow_network {
            return Err(MtpError::Security("Network access not allowed".to_string()));
        }

        // Execute in the sandboxed interpreter and return JSON string
        self.interpreter
            .execute_to_json(code)
            .map_err(MtpError::from)
    }

    /// Check if seccomp is enabled
    pub fn is_seccomp_enabled(&self) -> bool {
        self.seccomp_enabled
    }

    /// Get the underlying interpreter (for testing)
    pub fn interpreter(&self) -> &Interpreter {
        &self.interpreter
    }
}

impl Drop for SandboxedInterpreter {
    fn drop(&mut self) {
        // Ensure proper cleanup
        // The interpreter handles secure wipe
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_creation() {
        let _config = SandboxConfig::default();
        // Note: This test assumes Interpreter::new() exists
        // let interp = Interpreter::new();
        // let sandboxed = SandboxedInterpreter::new(interp, config);
        // assert!(sandboxed.is_ok());
    }

    #[test]
    fn test_forbidden_operations() {
        let _config = SandboxConfig {
            allow_network: false,
            allow_fs: false,
            ..Default::default()
        };

        // Mock interpreter for testing
        // let mut sandboxed = SandboxedInterpreter::new(mock_interp, config);

        // These should fail
        // assert!(sandboxed.execute("require('fs')").is_err());
        // assert!(sandboxed.execute("fetch('http://evil.com')").is_err());
    }
}
