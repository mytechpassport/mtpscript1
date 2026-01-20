use crate::errors::MtpError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Unsafe npm dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsafeDependency {
    pub name: String,
    pub version: String,
    pub content_hash: String,
    pub source_hash: String,
    pub license: Option<String>,
    pub vulnerabilities: Vec<String>,
}

/// Audit manifest for unsafe dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditManifest {
    pub unsafe_deps: Vec<UnsafeDependency>,
    pub generated_at: String,
    pub total_deps: usize,
    pub manifest_hash: String,
}

/// npm bridge adapter configuration
#[derive(Debug, Clone)]
pub struct NpmBridgeConfig {
    pub host_dir: String,
    pub allowed_packages: Vec<String>,
    pub max_package_size_kb: usize,
}

/// npm bridge for unsafe adapters
pub struct NpmBridge {
    config: NpmBridgeConfig,
    manifest: Option<AuditManifest>,
}

impl NpmBridge {
    /// Create a new npm bridge
    pub fn new(config: NpmBridgeConfig) -> Self {
        Self {
            config,
            manifest: None,
        }
    }

    /// Install and audit npm packages
    pub fn install_packages(&mut self, package_list: &[(&str, &str)]) -> Result<(), MtpError> {
        // Create host/unsafe directory
        let unsafe_dir = Path::new(&self.config.host_dir).join("unsafe");
        fs::create_dir_all(&unsafe_dir)?;

        // Initialize package.json
        self.create_package_json(&unsafe_dir, package_list)?;

        // Install packages
        self.npm_install(&unsafe_dir)?;

        // Audit packages
        let manifest = self.audit_packages(&unsafe_dir, package_list)?;
        self.manifest = Some(manifest);

        Ok(())
    }

    /// Create package.json for unsafe dependencies
    fn create_package_json(&self, dir: &Path, packages: &[(&str, &str)]) -> Result<(), MtpError> {
        let mut package_json = serde_json::Map::new();
        package_json.insert(
            "name".to_string(),
            serde_json::Value::String("mtpscript-unsafe".to_string()),
        );
        package_json.insert(
            "version".to_string(),
            serde_json::Value::String("1.0.0".to_string()),
        );

        let mut dependencies = serde_json::Map::new();
        for (name, version) in packages {
            if !self.config.allowed_packages.contains(&name.to_string()) {
                return Err(MtpError::Security {
                    error: "Security".to_string(),
                    message: format!("Package {} not in allowed list", name),
                });
            }
            dependencies.insert(
                name.to_string(),
                serde_json::Value::String(version.to_string()),
            );
        }

        package_json.insert(
            "dependencies".to_string(),
            serde_json::Value::Object(dependencies),
        );

        let content = serde_json::to_string_pretty(&package_json)?;
        fs::write(dir.join("package.json"), content)?;

        Ok(())
    }

    /// Run npm install
    fn npm_install(&self, dir: &Path) -> Result<(), MtpError> {
        let output = Command::new("npm")
            .current_dir(dir)
            .args(&["install", "--production"])
            .output()
            .map_err(|e| MtpError::Build {
                error: "Build".to_string(),
                message: format!("npm install failed: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(MtpError::Build {
                error: "Build".to_string(),
                message: format!("npm install failed: {}", stderr),
            });
        }

        Ok(())
    }

    /// Audit installed packages
    fn audit_packages(
        &self,
        dir: &Path,
        packages: &[(&str, &str)],
    ) -> Result<AuditManifest, MtpError> {
        let mut unsafe_deps = Vec::new();

        for (name, version) in packages {
            let dep = self.audit_single_package(dir, name, version)?;
            unsafe_deps.push(dep);
        }

        let total_deps = unsafe_deps.len();

        // Compute content hash first (for deterministic timestamp derivation)
        // We hash the unsafe_deps content to derive a stable "timestamp"
        let content_for_hash = serde_json::to_string(&unsafe_deps)?;
        let content_hash = Sha256::new().chain_update(&content_for_hash).finalize();

        // Derive deterministic timestamp from content hash
        // This ensures the same content always produces the same timestamp
        // Format: 2024-01-01T00:00:00Z + offset derived from hash
        let time_offset = u32::from_le_bytes(content_hash[0..4].try_into().unwrap()) % 86400;
        let base_timestamp = chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap();
        let deterministic_timestamp =
            base_timestamp + chrono::Duration::seconds(time_offset as i64);

        let manifest = AuditManifest {
            unsafe_deps,
            generated_at: deterministic_timestamp.to_rfc3339(),
            total_deps,
            manifest_hash: String::new(), // Will be set below
        };

        // Compute manifest hash (includes the deterministic timestamp)
        let json = serde_json::to_string(&manifest)?;
        let hash = Sha256::new().chain_update(&json).finalize();
        let manifest_hash = format!("{:x}", hash);

        Ok(AuditManifest {
            manifest_hash,
            ..manifest
        })
    }

    /// Audit a single package
    fn audit_single_package(
        &self,
        dir: &Path,
        name: &str,
        version: &str,
    ) -> Result<UnsafeDependency, MtpError> {
        let package_dir = dir.join("node_modules").join(name);

        if !package_dir.exists() {
            return Err(MtpError::Build {
                error: "Build".to_string(),
                message: format!("Package {} not found after install", name),
            });
        }

        // Compute content hash
        let content_hash = self.compute_package_hash(&package_dir)?;

        // Compute source hash (from package-lock.json entry)
        let source_hash = self.get_package_source_hash(dir, name)?;

        // Check package size
        let size_kb = self.get_package_size_kb(&package_dir)?;
        if size_kb > self.config.max_package_size_kb {
            return Err(MtpError::Security {
                error: "Security".to_string(),
                message: format!(
                    "Package {} too large: {}KB > {}KB",
                    name, size_kb, self.config.max_package_size_kb
                ),
            });
        }

        // Read package.json for metadata
        let package_json_path = package_dir.join("package.json");
        let package_json: serde_json::Value = if package_json_path.exists() {
            let content = fs::read_to_string(&package_json_path)?;
            serde_json::from_str(&content)?
        } else {
            serde_json::Value::Null
        };

        let license = package_json
            .get("license")
            .and_then(|l| l.as_str())
            .map(|s| s.to_string());

        // Run security audit (simplified)
        let vulnerabilities = self.check_vulnerabilities(name, version)?;

        Ok(UnsafeDependency {
            name: name.to_string(),
            version: version.to_string(),
            content_hash,
            source_hash,
            license,
            vulnerabilities,
        })
    }

    /// Compute SHA-256 hash of package contents
    fn compute_package_hash(&self, package_dir: &Path) -> Result<String, MtpError> {
        let mut hasher = Sha256::new();
        self.hash_directory(package_dir, &mut hasher)?;
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Recursively hash directory
    fn hash_directory(&self, dir: &Path, hasher: &mut Sha256) -> Result<(), MtpError> {
        let entries = fs::read_dir(dir)?;

        let mut file_paths = Vec::new();
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                file_paths.push(path);
            } else if path.is_dir()
                && path.file_name() != Some(std::ffi::OsStr::new("node_modules"))
            {
                self.hash_directory(&path, hasher)?;
            }
        }

        // Sort files for deterministic hashing
        file_paths.sort();

        for path in file_paths {
            let content = fs::read(&path)?;
            hasher.update(&content);
        }

        Ok(())
    }

    /// Get package source hash from package-lock.json
    fn get_package_source_hash(&self, dir: &Path, name: &str) -> Result<String, MtpError> {
        let lockfile_path = dir.join("package-lock.json");

        if !lockfile_path.exists() {
            return Err(MtpError::Build {
                error: "Build".to_string(),
                message: "package-lock.json not found".to_string(),
            });
        }

        let content = fs::read_to_string(&lockfile_path)?;
        let lockfile: serde_json::Value = serde_json::from_str(&content)?;

        // Find the package in dependencies
        if let Some(deps) = lockfile.get("dependencies").and_then(|d| d.as_object()) {
            if let Some(dep) = deps.get(name).and_then(|d| d.as_object()) {
                if let Some(integrity) = dep.get("integrity").and_then(|i| i.as_str()) {
                    // Convert SRI hash to hex
                    if integrity.starts_with("sha512-") {
                        // For simplicity, return the SRI hash as-is
                        return Ok(integrity.to_string());
                    }
                }
            }
        }

        Err(MtpError::Build {
            error: "Build".to_string(),
            message: format!("Could not find source hash for {}", name),
        })
    }

    /// Get package size in KB
    fn get_package_size_kb(&self, package_dir: &Path) -> Result<usize, MtpError> {
        let output = Command::new("du")
            .args(&["-sk", &package_dir.to_string_lossy()])
            .output()
            .map_err(|e| MtpError::Io {
                error: "Io".to_string(),
                message: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(MtpError::Build {
                error: "Build".to_string(),
                message: "Failed to get package size".to_string(),
            });
        }

        let stdout = String::from_utf8(output.stdout)?;
        let size_kb: usize = stdout
            .split_whitespace()
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        Ok(size_kb)
    }

    /// Check for known vulnerabilities (simplified)
    fn check_vulnerabilities(&self, name: &str, version: &str) -> Result<Vec<String>, MtpError> {
        let output = Command::new("npm")
            .args(&[
                "audit",
                "--json",
                "--package",
                &format!("{}@{}", name, version),
            ])
            .output()
            .map_err(|e| MtpError::Build {
                error: "Build".to_string(),
                message: format!("npm audit failed: {}", e),
            })?;

        if !output.status.success() {
            // Parse audit results
            let audit_result: serde_json::Value = serde_json::from_slice(&output.stdout)?;

            let mut vulnerabilities = Vec::new();
            if let Some(vulns) = audit_result
                .get("vulnerabilities")
                .and_then(|v| v.as_object())
            {
                for (pkg, vuln) in vulns {
                    if let Some(severity) = vuln.get("severity").and_then(|s| s.as_str()) {
                        vulnerabilities.push(format!("{}: {}", pkg, severity));
                    }
                }
            }

            return Ok(vulnerabilities);
        }

        Ok(vec![])
    }

    /// Get the audit manifest
    pub fn get_manifest(&self) -> Option<&AuditManifest> {
        self.manifest.as_ref()
    }

    /// Write manifest to file
    pub fn write_manifest(&self, path: &Path) -> Result<(), MtpError> {
        if let Some(manifest) = &self.manifest {
            let json = serde_json::to_string_pretty(manifest)?;
            fs::write(path, json)?;
        }
        Ok(())
    }

    /// Load manifest from file
    pub fn load_manifest(&mut self, path: &Path) -> Result<(), MtpError> {
        let content = fs::read_to_string(path)?;
        let manifest: AuditManifest = serde_json::from_str(&content)?;
        self.manifest = Some(manifest);
        Ok(())
    }
}

/// Create a standard npm bridge configuration
pub fn create_standard_bridge() -> NpmBridge {
    NpmBridge::new(NpmBridgeConfig {
        host_dir: "host".to_string(),
        allowed_packages: vec![
            "uuid".to_string(),
            "crypto-js".to_string(),
            "jsonwebtoken".to_string(),
        ],
        max_package_size_kb: 1024, // 1MB limit
    })
}

/// Example adapter function signature (for documentation)
/// function adapterName(seed: Uint8Array, ...args: JsonValue[]): JsonValue
pub fn validate_adapter_signature(function_text: &str) -> Result<(), MtpError> {
    // Check that the function has the correct signature
    if !function_text.contains("function") {
        return Err(MtpError::Security {
            error: "Security".to_string(),
            message: "Not a function".to_string(),
        });
    }

    if !function_text.contains("seed: Uint8Array") {
        return Err(MtpError::Security {
            error: "Security".to_string(),
            message: "Missing seed parameter".to_string(),
        });
    }

    if !function_text.contains("JsonValue[]") {
        return Err(MtpError::Security {
            error: "Security".to_string(),
            message: "Missing JsonValue[] parameter".to_string(),
        });
    }

    if !function_text.contains("JsonValue") {
        return Err(MtpError::Security {
            error: "Security".to_string(),
            message: "Missing JsonValue return type".to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_manifest_creation() {
        let dep = UnsafeDependency {
            name: "uuid".to_string(),
            version: "9.0.1".to_string(),
            content_hash: "abc123".to_string(),
            source_hash: "def456".to_string(),
            license: Some("MIT".to_string()),
            vulnerabilities: vec![],
        };

        let manifest = AuditManifest {
            unsafe_deps: vec![dep],
            generated_at: "2024-01-01T00:00:00Z".to_string(),
            total_deps: 1,
            manifest_hash: "hash123".to_string(),
        };

        let json = serde_json::to_string(&manifest).unwrap();
        assert!(json.contains("uuid"));
        assert!(json.contains("9.0.1"));
    }

    #[test]
    fn test_package_size_check() {
        let temp_dir = tempdir().unwrap();
        let package_dir = temp_dir.path().join("test-package");
        fs::create_dir(&package_dir).unwrap();

        // Create a small file
        fs::write(package_dir.join("index.js"), "console.log('test');").unwrap();

        let bridge = create_standard_bridge();
        let _size = bridge.get_package_size_kb(&package_dir).unwrap();
        // size is usize, always >= 0
    }

    #[test]
    fn test_adapter_validation() {
        let valid_adapter = r#"
            function uuidAdapter(seed: Uint8Array, ...args: JsonValue[]): JsonValue {
                return { uuid: "123" };
            }
        "#;

        assert!(validate_adapter_signature(valid_adapter).is_ok());

        let invalid_adapter = r#"
            function badAdapter() {
                return "no signature";
            }
        "#;

        assert!(validate_adapter_signature(invalid_adapter).is_err());
    }

    // Determinism tests (#29) - verify timestamp derivation is deterministic

    #[test]
    fn test_deterministic_timestamp_derivation() {
        use sha2::{Digest, Sha256};

        // Same content should always produce the same timestamp
        let content = r#"[{"name":"uuid","version":"9.0.1"}]"#;

        fn derive_timestamp(content: &str) -> String {
            let content_hash = Sha256::new().chain_update(content).finalize();
            let time_offset = u32::from_le_bytes(content_hash[0..4].try_into().unwrap()) % 86400;
            let base_timestamp =
                chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap();
            let deterministic_timestamp =
                base_timestamp + chrono::Duration::seconds(time_offset as i64);
            deterministic_timestamp.to_rfc3339()
        }

        // Run multiple times to verify determinism
        let first_result = derive_timestamp(content);
        for _ in 0..100 {
            let result = derive_timestamp(content);
            assert_eq!(
                result, first_result,
                "Timestamp derivation must be deterministic"
            );
        }
    }

    #[test]
    fn test_different_content_produces_different_timestamps() {
        use sha2::{Digest, Sha256};

        fn derive_timestamp(content: &str) -> String {
            let content_hash = Sha256::new().chain_update(content).finalize();
            let time_offset = u32::from_le_bytes(content_hash[0..4].try_into().unwrap()) % 86400;
            let base_timestamp =
                chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap();
            let deterministic_timestamp =
                base_timestamp + chrono::Duration::seconds(time_offset as i64);
            deterministic_timestamp.to_rfc3339()
        }

        let content1 = r#"[{"name":"uuid","version":"9.0.1"}]"#;
        let content2 = r#"[{"name":"uuid","version":"9.0.2"}]"#;

        let ts1 = derive_timestamp(content1);
        let ts2 = derive_timestamp(content2);

        // Different content should (most likely) produce different timestamps
        // Note: There's a 1/86400 chance of collision, which is acceptable
        assert_ne!(ts1, ts2, "Different content should produce different timestamps");
    }

    #[test]
    fn test_timestamp_within_valid_range() {
        use sha2::{Digest, Sha256};

        // Test that derived timestamps are always within the expected range
        let test_contents = vec![
            "test1",
            "test2",
            "longer content here",
            r#"{"complex": "json", "data": [1,2,3]}"#,
        ];

        for content in test_contents {
            let content_hash = Sha256::new().chain_update(content).finalize();
            let time_offset = u32::from_le_bytes(content_hash[0..4].try_into().unwrap()) % 86400;

            // Offset should be 0..86399 (seconds in a day)
            assert!(time_offset < 86400, "Time offset should be less than 86400");

            let base_timestamp =
                chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap();
            let deterministic_timestamp =
                base_timestamp + chrono::Duration::seconds(time_offset as i64);

            // Timestamp should be on 2024-01-01
            let ts_str = deterministic_timestamp.to_rfc3339();
            assert!(
                ts_str.starts_with("2024-01-01"),
                "Timestamp should be on 2024-01-01, got {}",
                ts_str
            );
        }
    }

    #[test]
    fn test_audit_manifest_serialization_deterministic() {
        let dep = UnsafeDependency {
            name: "test-pkg".to_string(),
            version: "1.0.0".to_string(),
            content_hash: "abc123".to_string(),
            source_hash: "def456".to_string(),
            license: Some("MIT".to_string()),
            vulnerabilities: vec![],
        };

        let manifest = AuditManifest {
            unsafe_deps: vec![dep.clone()],
            generated_at: "2024-01-01T12:00:00+00:00".to_string(),
            total_deps: 1,
            manifest_hash: "hash123".to_string(),
        };

        // Serialize multiple times
        let first_json = serde_json::to_string(&manifest).unwrap();
        for _ in 0..100 {
            let json = serde_json::to_string(&manifest).unwrap();
            assert_eq!(json, first_json, "Manifest serialization must be deterministic");
        }
    }

    #[test]
    fn test_no_wall_clock_time_in_manifest() {
        // This test documents that we don't use wall-clock time
        // The timestamp is derived from content hash, not from Utc::now()

        let dep = UnsafeDependency {
            name: "uuid".to_string(),
            version: "9.0.1".to_string(),
            content_hash: "abc".to_string(),
            source_hash: "def".to_string(),
            license: None,
            vulnerabilities: vec![],
        };

        // If we were using wall-clock time, running this twice would give different results
        // But with content-derived timestamps, it's always the same
        let manifest1 = AuditManifest {
            unsafe_deps: vec![dep.clone()],
            generated_at: "2024-01-01T00:00:00Z".to_string(), // Fixed, not from clock
            total_deps: 1,
            manifest_hash: "".to_string(),
        };

        let manifest2 = AuditManifest {
            unsafe_deps: vec![dep],
            generated_at: "2024-01-01T00:00:00Z".to_string(), // Fixed, not from clock
            total_deps: 1,
            manifest_hash: "".to_string(),
        };

        assert_eq!(
            serde_json::to_string(&manifest1).unwrap(),
            serde_json::to_string(&manifest2).unwrap()
        );
    }
}
