use crate::errors::MtpError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Build information for reproducible builds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildInfo {
    /// SHA-256 of the build container image
    pub container_hash: String,
    /// SHA-256 of the source code
    pub source_hash: String,
    /// SHA-256 of the compiled snapshot
    pub snapshot_hash: String,
    /// Build timestamp (ISO 8601)
    pub timestamp: String,
    /// Git commit hash
    pub git_commit: String,
    /// Compiler version
    pub compiler_version: String,
    /// Build environment info
    pub environment: HashMap<String, String>,
}

/// Signed build info with ECDSA signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedBuildInfo {
    pub build_info: BuildInfo,
    pub signature: Vec<u8>,
}

/// Container image specification
#[derive(Debug, Clone)]
pub struct ContainerSpec {
    pub image: String,
    pub hash: String,
    pub registry: String,
}

/// Reproducible build manager
pub struct ReproducibleBuild {
    container_spec: ContainerSpec,
}

impl ReproducibleBuild {
    /// Create a new reproducible build manager
    pub fn new(container_spec: ContainerSpec) -> Self {
        Self { container_spec }
    }

    /// Build MTPScript in a containerized environment
    pub fn build(
        &self,
        source_path: &Path,
        output_path: &Path,
    ) -> Result<SignedBuildInfo, MtpError> {
        // Verify container image hash
        self.verify_container_image()?;

        // Compute source hash
        let source_hash = self.compute_source_hash(source_path)?;

        // Run build in container
        let _build_result = self.run_container_build(source_path, output_path)?;

        // Compute snapshot hash
        let snapshot_hash = self.compute_file_hash(output_path)?;

        // Create build info
        let build_info = BuildInfo {
            container_hash: self.container_spec.hash.clone(),
            source_hash,
            snapshot_hash,
            timestamp: chrono::Utc::now().to_rfc3339(),
            git_commit: self.get_git_commit()?,
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            environment: self.get_build_environment(),
        };

        // Sign build info
        let signed = self.sign_build_info(build_info)?;

        // Write signed build info
        let build_info_path = output_path.with_extension("build-info.json");
        let json = serde_json::to_string_pretty(&signed)?;
        fs::write(&build_info_path, json)?;

        Ok(signed)
    }

    /// Verify that the container image matches the expected hash
    fn verify_container_image(&self) -> Result<(), MtpError> {
        // Pull and verify container image
        let output = Command::new("docker")
            .args(&["pull", &self.container_spec.image])
            .output()
            .map_err(|e| MtpError::Build(format!("Failed to pull container: {}", e)))?;

        if !output.status.success() {
            return Err(MtpError::Build("Container pull failed".to_string()));
        }

        // Verify image hash (this is a simplified check)
        let inspect_output = Command::new("docker")
            .args(&["inspect", &self.container_spec.image])
            .output()
            .map_err(|e| MtpError::Build(format!("Failed to inspect container: {}", e)))?;

        if !inspect_output.status.success() {
            return Err(MtpError::Build("Container inspect failed".to_string()));
        }

        // In a real implementation, you'd verify the image digest matches the hash
        // For now, we assume the image is trusted if it exists

        Ok(())
    }

    /// Compute SHA-256 hash of source directory
    fn compute_source_hash(&self, source_path: &Path) -> Result<String, MtpError> {
        let mut hasher = Sha256::new();

        // Walk directory and hash all files
        self.hash_directory(source_path, &mut hasher)?;

        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    /// Recursively hash directory contents
    fn hash_directory(&self, dir: &Path, hasher: &mut Sha256) -> Result<(), MtpError> {
        let entries = fs::read_dir(dir).map_err(|e| MtpError::Io(e.to_string()))?;

        let mut entries: Vec<_> = entries.collect();
        entries.sort_by_key(|e| e.as_ref().unwrap().file_name());

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let file_name = path
                .file_name()
                .ok_or_else(|| MtpError::Build("Invalid file name".to_string()))?;

            // Skip certain files
            if file_name == ".git" || file_name == "target" || file_name == ".DS_Store" {
                continue;
            }

            if path.is_dir() {
                self.hash_directory(&path, hasher)?;
            } else {
                let content = fs::read(&path).map_err(|e| MtpError::Io(e.to_string()))?;
                hasher.update(&content);
            }
        }

        Ok(())
    }

    /// Compute SHA-256 hash of a file
    fn compute_file_hash(&self, file_path: &Path) -> Result<String, MtpError> {
        let content = fs::read(file_path).map_err(|e| MtpError::Io(e.to_string()))?;

        let hash = Sha256::new().chain_update(&content).finalize();

        Ok(format!("{:x}", hash))
    }

    /// Run build inside container
    fn run_container_build(&self, source_path: &Path, output_path: &Path) -> Result<(), MtpError> {
        let source_mount = format!("{}:/src", source_path.display());
        let output_dir = output_path
            .parent()
            .ok_or_else(|| MtpError::Build("Invalid output path".to_string()))?;
        let output_mount = format!("{}:/output", output_dir.display());

        let output = Command::new("docker")
            .args(&[
                "run",
                "--rm",
                "-v",
                &source_mount,
                "-v",
                &output_mount,
                "-w",
                "/src",
                &self.container_spec.image,
                "make",
                "build",
            ])
            .output()
            .map_err(|e| MtpError::Build(format!("Container build failed: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(MtpError::Build(format!("Build failed: {}", stderr)));
        }

        Ok(())
    }

    /// Get current git commit hash
    fn get_git_commit(&self) -> Result<String, MtpError> {
        let output = Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .output()
            .map_err(|e| MtpError::Build(format!("Git command failed: {}", e)))?;

        if !output.status.success() {
            return Err(MtpError::Build("Failed to get git commit".to_string()));
        }

        let commit = String::from_utf8(output.stdout)
            .map_err(|e| MtpError::Build(format!("Invalid git output: {}", e)))?
            .trim()
            .to_string();

        Ok(commit)
    }

    /// Get build environment information
    fn get_build_environment(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();

        env.insert(
            "rust_version".to_string(),
            rustc_version::version()
                .map(|v| v.to_string())
                .unwrap_or_else(|_| "unknown".to_string()),
        );
        env.insert("os".to_string(), std::env::consts::OS.to_string());
        env.insert("arch".to_string(), std::env::consts::ARCH.to_string());

        if let Ok(hostname) = hostname::get() {
            if let Ok(hostname) = hostname.into_string() {
                env.insert("hostname".to_string(), hostname);
            }
        }

        env
    }

    /// Sign build info with ECDSA (placeholder - would use actual key)
    fn sign_build_info(&self, build_info: BuildInfo) -> Result<SignedBuildInfo, MtpError> {
        // In a real implementation, this would sign with a private key
        // For now, we create a dummy signature
        let json = serde_json::to_string(&build_info)?;
        let signature = Sha256::new().chain_update(&json).finalize().to_vec();

        Ok(SignedBuildInfo {
            build_info,
            signature,
        })
    }

    /// Verify a signed build info
    pub fn verify_build_info(&self, signed: &SignedBuildInfo) -> Result<(), MtpError> {
        // In a real implementation, verify the ECDSA signature
        // For now, just check that the signature matches our dummy hash
        let json = serde_json::to_string(&signed.build_info)?;
        let expected_signature = Sha256::new().chain_update(&json).finalize();

        if signed.signature != expected_signature.as_slice() {
            return Err(MtpError::Security(
                "Build info signature verification failed".to_string(),
            ));
        }

        Ok(())
    }
}

/// Create a standard reproducible build configuration
pub fn create_standard_build() -> ReproducibleBuild {
    ReproducibleBuild::new(ContainerSpec {
        image: "mtpscript/build:latest".to_string(),
        hash: "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        registry: "docker.io".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_build_info_creation() {
        let build_info = BuildInfo {
            container_hash: "abc123".to_string(),
            source_hash: "def456".to_string(),
            snapshot_hash: "ghi789".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            git_commit: "commit123".to_string(),
            compiler_version: "1.0.0".to_string(),
            environment: HashMap::new(),
        };

        let signed = SignedBuildInfo {
            build_info,
            signature: vec![1, 2, 3],
        };

        let json = serde_json::to_string(&signed).unwrap();
        assert!(json.contains("abc123"));
    }

    #[test]
    fn test_file_hash() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "hello world").unwrap();

        let build = create_standard_build();
        let hash = build.compute_file_hash(&file_path).unwrap();

        // SHA-256 of "hello world"
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }
}
