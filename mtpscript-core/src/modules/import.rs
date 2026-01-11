use crate::errors::MtpError;
use git2::{Oid, Repository};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Import declaration
#[derive(Debug, Clone)]
pub struct ImportDecl {
    pub module_name: String,
    pub alias: String,
    pub git_url: String,
    pub git_hash: String,
    pub tag: Option<String>,
}

/// Static import resolver
pub struct ImportResolver {
    resolved_modules: HashMap<String, String>,
    audit_manifest: Vec<ModuleAuditEntry>,
}

/// Audit entry for imported modules
#[derive(Debug, Clone, serde::Serialize)]
pub struct ModuleAuditEntry {
    pub name: String,
    pub version: Option<String>,
    pub git_url: String,
    pub git_hash: String,
    pub content_hash: String,
}

impl ImportResolver {
    pub fn new() -> Self {
        Self {
            resolved_modules: HashMap::new(),
            audit_manifest: Vec::new(),
        }
    }

    /// Resolve an import declaration
    pub fn resolve_import(&mut self, import: &ImportDecl) -> Result<String, MtpError> {
        // Check if already resolved
        if let Some(path) = self.resolved_modules.get(&import.module_name) {
            return Ok(path.clone());
        }

        // Validate git hash/tag
        self.validate_git_reference(&import.git_url, &import.git_hash, import.tag.as_deref())?;

        // Download and verify module
        let module_path = self.download_module(import)?;

        // Verify cryptographic signature if present
        self.verify_module_signature(import, &module_path)?;

        // Compute content hash for audit
        let repo_path = PathBuf::from(&module_path);
        let content_hash = self.compute_repo_content_hash(&repo_path)?;

        // Create audit entry
        let audit_entry = ModuleAuditEntry {
            name: import.module_name.clone(),
            version: import.tag.clone(),
            git_url: import.git_url.clone(),
            git_hash: import.git_hash.clone(),
            content_hash,
        };

        self.audit_manifest.push(audit_entry);

        // Cache the resolved module
        self.resolved_modules
            .insert(import.module_name.clone(), module_path.clone());

        Ok(module_path)
    }

    /// Verify module cryptographic signature
    fn verify_module_signature(
        &self,
        import: &ImportDecl,
        module_path: &str,
    ) -> Result<(), MtpError> {
        // Look for signature file
        let signature_path = format!("{}.sig", module_path);
        if !Path::new(&signature_path).exists() {
            return Err(MtpError::Security {
                error: "Security".to_string(),
                message: "Module signature not found".to_string(),
            });
        }

        // Load signature
        let signature_pem = fs::read_to_string(&signature_path)
            .map_err(|e| MtpError::Io {
                error: "Io".to_string(),
                message: format!("Failed to read signature: {}", e),
            })?;

        // Compute content hash
        let repo_path = PathBuf::from(module_path);
        let content = fs::read_to_string(&repo_path)
            .map_err(|e| MtpError::Io {
                error: "Io".to_string(),
                message: format!("Failed to read module content: {}", e),
            })?;
        let content_hash = Sha256::digest(content.as_bytes());

        // Verify signature (placeholder - would use actual public key)
        // In real implementation, would load trusted public keys
        // For now, just check signature format
        if !signature_pem.contains("-----BEGIN") || !signature_pem.contains("-----END") {
            return Err(MtpError::Security {
                error: "Security".to_string(),
                message: "Invalid signature format".to_string(),
            });
        }

        // Placeholder verification - in real impl:
        // crate::security::sign::verify_ecdsa_p256(&content_hash, signature_bytes, public_key)

        Ok(())
    }

    /// Validate git reference (simplified)
    fn validate_git_reference(
        &self,
        _git_url: &str,
        git_hash: &str,
        tag: Option<&str>,
    ) -> Result<(), MtpError> {
        // In a real implementation, this would clone/verify the git repo
        // For now, just check that the hash looks like a SHA-256

        if git_hash.len() != 64 {
            return Err(MtpError::Build {
                error: "Build".to_string(),
                message: format!("Invalid git hash length: {}", git_hash.len()),
            });
        }

        if let Some(tag_name) = tag {
            // Verify tag exists and points to the hash
            // This is simplified
            if tag_name.is_empty() {
                return Err(MtpError::Build {
                    error: "Build".to_string(),
                    message: "Empty tag name".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Download and verify module from git
    fn download_module(&self, import: &ImportDecl) -> Result<String, MtpError> {
        // Create vendor directory if it doesn't exist
        let vendor_dir = PathBuf::from("vendor");
        fs::create_dir_all(&vendor_dir).map_err(|e| MtpError::Io {
            error: "Io".to_string(),
            message: e.to_string(),
        })?;

        // Clone or fetch the repository
        let repo_path = vendor_dir.join(&import.module_name);
        let repo = if repo_path.exists() {
            // Repository already exists, fetch updates
            Repository::open(&repo_path)
                .map_err(|e| MtpError::Build {
                    error: "Build".to_string(),
                    message: format!("Failed to open repo: {}", e),
                })?
        } else {
            // Clone the repository
            let url = format!("https://{}", import.git_url);
            Repository::clone(&url, &repo_path)
                .map_err(|e| MtpError::Build {
                    error: "Build".to_string(),
                    message: format!("Failed to clone repo: {}", e),
                })?
        };

        // Verify the commit hash exists
        let oid = Oid::from_str(&import.git_hash)
            .map_err(|_| MtpError::Build {
                error: "Build".to_string(),
                message: "Invalid git hash".to_string(),
            })?;

        let commit = repo
            .find_commit(oid)
            .map_err(|_| MtpError::Build {
                error: "Build".to_string(),
                message: "Commit hash not found in repository".to_string(),
            })?;

        // Verify tag if specified
        if let Some(tag_name) = &import.tag {
            self.verify_git_tag(&repo, tag_name, oid)?;
        }

        // Checkout the specific commit
        repo.checkout_tree(commit.as_object(), None)
            .map_err(|e| MtpError::Build {
                error: "Build".to_string(),
                message: format!("Failed to checkout commit: {}", e),
            })?;

        repo.set_head_detached(oid)
            .map_err(|e| MtpError::Build {
                error: "Build".to_string(),
                message: format!("Failed to set HEAD: {}", e),
            })?;

        // Verify repository content hash
        let content_hash = self.compute_repo_content_hash(&repo_path)?;
        if content_hash != import.git_hash {
            return Err(MtpError::Build {
                error: "Build".to_string(),
                message: "Content hash mismatch".to_string(),
            });
        }

        Ok(repo_path.to_string_lossy().to_string())
    }

    /// Verify git tag and that it points to the expected commit
    fn verify_git_tag(
        &self,
        repo: &Repository,
        tag_name: &str,
        expected_oid: Oid,
    ) -> Result<(), MtpError> {
        // Find the tag
        let tag_obj = repo
            .find_reference(&format!("refs/tags/{}", tag_name))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", tag_name)))
            .map_err(|_| MtpError::Build {
                error: "Build".to_string(),
                message: format!("Tag '{}' not found", tag_name),
            })?;

        // Get the tag target
        let tag_oid = tag_obj
            .target()
            .ok_or_else(|| MtpError::Build {
                error: "Build".to_string(),
                message: "Tag has no target".to_string(),
            })?;

        if tag_oid != expected_oid {
            return Err(MtpError::Build {
                error: "Build".to_string(),
                message: format!("Tag '{}' does not point to expected commit", tag_name),
            });
        }

        // Verify tag signature if present
        // This is a simplified implementation. Full GPG verification would require:
        // 1. Access to the tag's signature data
        // 2. A keyring with trusted public keys
        // 3. Verification of the signature against the tag content

        // For now, we check if the tag is annotated and has a signature
        if let Ok(_tag) = repo.find_tag(tag_oid) {
            // This is an annotated tag
            // In a real implementation, extract and verify GPG signature
            // For this implementation, we assume the tag is trusted if it exists
            // and points to the right commit
        }

        Ok(())
    }

    /// Compute content hash of repository (simplified - hashes all .mtp files)
    fn compute_repo_content_hash(&self, repo_path: &Path) -> Result<String, MtpError> {
        let mut hasher = Sha256::new();
        let mut files = Vec::new();

        // Collect all .mtp files
        self.collect_mtp_files(repo_path, &mut files)?;

        // Sort files for deterministic hashing
        files.sort();

        // Hash file contents
        for file_path in files {
            let content = fs::read(&file_path).map_err(|e| MtpError::Io {
                error: "Io".to_string(),
                message: e.to_string(),
            })?;
            hasher.update(&content);
        }

        let hash_bytes = hasher.finalize();
        Ok(hex::encode(hash_bytes))
    }

    /// Recursively collect .mtp files
    fn collect_mtp_files(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), MtpError> {
        let entries = fs::read_dir(dir).map_err(|e| MtpError::Io {
            error: "Io".to_string(),
            message: e.to_string(),
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| MtpError::Io {
                error: "Io".to_string(),
                message: e.to_string(),
            })?;
            let path = entry.path();

            if path.is_dir() {
                // Skip .git directory
                if !path.ends_with(".git") {
                    self.collect_mtp_files(&path, files)?;
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("mtp") {
                files.push(path);
            }
        }

        Ok(())
    }

    /// Get all resolved modules
    pub fn resolved_modules(&self) -> &HashMap<String, String> {
        &self.resolved_modules
    }

    /// Get audit manifest
    pub fn audit_manifest(&self) -> &[ModuleAuditEntry] {
        &self.audit_manifest
    }

    /// Save audit manifest to file
    pub fn save_audit_manifest(&self, path: &str) -> Result<(), MtpError> {
        let json = serde_json::to_string_pretty(&self.audit_manifest)
            .map_err(|e| MtpError::Io {
                error: "Io".to_string(),
                message: e.to_string(),
            })?;
        fs::write(path, json).map_err(|e| MtpError::Io {
            error: "Io".to_string(),
            message: e.to_string(),
        })
    }
}

/// Parse import declaration from source
pub fn parse_import_decl(source: &str) -> Result<ImportDecl, MtpError> {
    // Simplified parser for: import "github.com/user/repo@v1.0.0#abc123" as alias

    let import_keyword = "import \"";
    let as_keyword = "\" as ";

    if !source.starts_with(import_keyword) {
        return Err(MtpError::Build {
            error: "Build".to_string(),
            message: "Invalid import syntax".to_string(),
        });
    }

    let after_import = &source[import_keyword.len()..];
    let as_pos = after_import
        .find(as_keyword)
        .ok_or_else(|| MtpError::Build {
            error: "Build".to_string(),
            message: "Missing 'as' keyword".to_string(),
        })?;

    let url_part = &after_import[..as_pos];
    let alias_part = &after_import[as_pos + as_keyword.len()..];

    // Parse URL part: github.com/user/repo@v1.0.0#abc123
    let hash_sep = url_part
        .rfind('#')
        .ok_or_else(|| MtpError::Build {
            error: "Build".to_string(),
            message: "Missing git hash".to_string(),
        })?;
    let (url_and_tag, git_hash) = url_part.split_at(hash_sep);

    let git_hash = &git_hash[1..]; // Remove '#'

    let tag = if let Some(at_pos) = url_and_tag.rfind('@') {
        Some(url_and_tag[at_pos + 1..].to_string())
    } else {
        None
    };

    let git_url = if tag.is_some() {
        url_and_tag.split('@').next().unwrap()
    } else {
        url_and_tag
    };

    Ok(ImportDecl {
        module_name: alias_part.to_string(),
        alias: alias_part.to_string(),
        git_url: git_url.to_string(),
        git_hash: git_hash.to_string(),
        tag,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_import_with_tag() {
        let source = r#"import "github.com/example/lib@v1.2.3#abc123def456" as lib"#;
        let import = parse_import_decl(source).unwrap();

        assert_eq!(import.module_name, "lib");
        assert_eq!(import.alias, "lib");
        assert_eq!(import.git_url, "github.com/example/lib");
        assert_eq!(import.git_hash, "abc123def456");
        assert_eq!(import.tag, Some("v1.2.3".to_string()));
    }

    #[test]
    fn test_parse_import_without_tag() {
        let source = r#"import "github.com/example/lib#abc123def456" as lib"#;
        let import = parse_import_decl(source).unwrap();

        assert_eq!(import.module_name, "lib");
        assert_eq!(import.git_url, "github.com/example/lib");
        assert_eq!(import.git_hash, "abc123def456");
        assert_eq!(import.tag, None);
    }

    #[test]
    fn test_import_resolver_initialization() {
        let resolver = ImportResolver::new();
        assert!(resolver.resolved_modules().is_empty());
        assert!(resolver.audit_manifest().is_empty());
    }

    #[test]
    fn test_git_hash_validation() {
        let resolver = ImportResolver::new();

        // Valid 64-character hash should pass
        let valid_hash = "a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3";
        assert!(resolver
            .validate_git_reference("github.com/test/repo", valid_hash, None)
            .is_ok());

        // Invalid length should fail
        let short_hash = "abc123";
        assert!(resolver
            .validate_git_reference("github.com/test/repo", short_hash, None)
            .is_err());

        // Valid tag should pass
        assert!(resolver
            .validate_git_reference("github.com/test/repo", valid_hash, Some("v1.0.0"))
            .is_ok());

        // Empty tag should fail
        assert!(resolver
            .validate_git_reference("github.com/test/repo", valid_hash, Some(""))
            .is_err());
    }

    #[test]
    fn test_content_hash_computation() {
        let resolver = ImportResolver::new();

        // Create a temporary directory with test files
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.mtp");
        std::fs::write(&test_file, "function test() { return 42; }").unwrap();

        let hash = resolver.compute_repo_content_hash(temp_dir.path());
        assert!(hash.is_ok());
        let hash_str = hash.unwrap();
        assert_eq!(hash_str.len(), 64); // SHA-256 hex length

        // Same content should produce same hash
        let hash2 = resolver.compute_repo_content_hash(temp_dir.path()).unwrap();
        assert_eq!(hash_str, hash2);
    }

    #[test]
    fn test_audit_manifest_generation() {
        let mut resolver = ImportResolver::new();

        // Simulate resolving an import (without actual git operations)
        let import = ImportDecl {
            module_name: "test_lib".to_string(),
            alias: "test_lib".to_string(),
            git_url: "github.com/example/lib".to_string(),
            git_hash: "a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3"
                .to_string(),
            tag: Some("v1.0.0".to_string()),
        };

        // Manually add to audit manifest (simulating successful resolution)
        resolver.audit_manifest.push(ModuleAuditEntry {
            name: import.module_name.clone(),
            version: import.tag.clone(),
            git_url: import.git_url.clone(),
            git_hash: import.git_hash.clone(),
            content_hash: "mock_content_hash".to_string(),
        });

        let manifest = resolver.audit_manifest();
        assert_eq!(manifest.len(), 1);
        assert_eq!(manifest[0].name, "test_lib");
        assert_eq!(manifest[0].version, Some("v1.0.0".to_string()));
        assert_eq!(manifest[0].git_url, "github.com/example/lib");
    }
}
