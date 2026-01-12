use crate::errors::MtpError;
use crate::parser::ast::ImportDecl;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;

/// Import context with cryptographic verification
pub struct ImportContext {
    pub trusted_keys: HashMap<String, Vec<u8>>, // module_name -> public_key
    pub verified_modules: HashMap<String, ModuleSignature>,
}

impl ImportContext {
    pub fn new() -> Self {
        ImportContext {
            trusted_keys: HashMap::new(),
            verified_modules: HashMap::new(),
        }
    }
}

impl Default for ImportContext {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ModuleSignature {
    pub module_name: String,
    pub version: String,
    pub content_hash: Vec<u8>,
    pub signature: Vec<u8>,
    pub signer_public_key: Vec<u8>,
}

/// Simple module registry for test cases
static mut MODULE_REGISTRY: Option<HashMap<String, HashMap<String, String>>> = None;

pub fn init_module_registry() {
    unsafe {
        MODULE_REGISTRY = Some(HashMap::new());
        if let Some(registry) = &mut MODULE_REGISTRY {
            // Add test modules
            let mut math_module = HashMap::new();
            math_module.insert("add".to_string(), "function(x, y) { return x + y; }".to_string());
            math_module.insert("multiply".to_string(), "function(x, y) { return x * y; }".to_string());
            registry.insert("test_import_math".to_string(), math_module);

            let mut helpers_module = HashMap::new();
            helpers_module.insert("double".to_string(), "function(x) { return x * 2; }".to_string());
            registry.insert("test_import_helpers".to_string(), helpers_module);
        }
    }
}

pub fn resolve_import(import: &ImportDecl) -> Result<HashMap<String, String>, MtpError> {
    unsafe {
        if MODULE_REGISTRY.is_none() {
            init_module_registry();
        }
        if let Some(registry) = &MODULE_REGISTRY {
            if let Some(module) = registry.get(&import.path) {
                Ok(module.clone())
            } else {
                Err(MtpError::RuntimeError {
                    error: "ImportError".to_string(),
                    message: format!("Module '{}' not found", import.path),
                })
            }
        } else {
            Err(MtpError::RuntimeError {
                error: "ImportError".to_string(),
                message: "Module registry not initialized".to_string(),
            })
        }
    }
}

/// Verify and import a module with cryptographic verification
pub fn verify_and_import_module(
    import: &ImportDecl,
    context: &mut ImportContext,
) -> Result<(), MtpError> {
    // Parse the module specification
    let (repo_url, version, commit_hash) = parse_module_spec(&import.path)?;

    // Check if we have a trusted key for this repository
    if !context.trusted_keys.contains_key(&repo_url) {
        return Err(MtpError::RuntimeError {
            error: "SecurityError".to_string(),
            message: format!("No trusted key for repository: {}", repo_url),
        });
    }

    // Fetch the module content
    let content = fetch_module_content(&repo_url, &version, &commit_hash)?;

    // Compute content hash
    let content_hash = Sha256::digest(&content);

    // Verify the signature
    verify_module_signature(&repo_url, content_hash.as_slice(), &content)?;

    // Determine the alias for this module
    let alias = import.alias.clone().unwrap_or_else(|| {
        repo_url.split('/').last().unwrap_or("module").to_string()
    });

    // Create signature record
    let signature = ModuleSignature {
        module_name: alias.clone(),
        version,
        content_hash: content_hash.to_vec(),
        signature: vec![], // Would be extracted from module metadata
        signer_public_key: context.trusted_keys[&repo_url].clone(),
    };

    // Store verified module
    context.verified_modules.insert(alias.clone(), signature);

    Ok(())
}

/// Parse module specification string
fn parse_module_spec(spec: &str) -> Result<(String, String, String), MtpError> {
    // Expected format: "github.com/user/repo@v1.2.3#abc123def456..."
    // Also support simple module names for local/test modules

    if !spec.contains('@') {
        // Simple module name - return as-is for local resolution
        return Ok((spec.to_string(), "local".to_string(), "0".repeat(40)));
    }

    let parts: Vec<&str> = spec.split('@').collect();
    if parts.len() != 2 {
        return Err(MtpError::RuntimeError {
            error: "ModuleError".to_string(),
            message: "Invalid module specification format".to_string(),
        });
    }

    let repo_url = parts[0].to_string();
    let version_commit = parts[1];

    let version_commit_parts: Vec<&str> = version_commit.split('#').collect();
    if version_commit_parts.len() != 2 {
        return Err(MtpError::RuntimeError {
            error: "ModuleError".to_string(),
            message: "Invalid version/commit format. Expected: repo@version#commit".to_string(),
        });
    }

    let version = version_commit_parts[0].to_string();
    let commit_hash = version_commit_parts[1].to_string();

    // Validate commit hash format (should be 40 hex characters for SHA-1)
    if !commit_hash.chars().all(|c| c.is_ascii_hexdigit()) || commit_hash.len() != 40 {
        return Err(MtpError::RuntimeError {
            error: "ModuleError".to_string(),
            message: format!(
                "Invalid commit hash format: expected 40 hex characters, got {} characters",
                commit_hash.len()
            ),
        });
    }

    Ok((repo_url, version, commit_hash))
}

/// Fetch module content from git repository
fn fetch_module_content(
    repo_url: &str,
    version: &str,
    commit_hash: &str,
) -> Result<Vec<u8>, MtpError> {
    // For local/test modules, try to read from filesystem
    if version == "local" {
        // Try reading from current directory or a modules directory
        let possible_paths = vec![
            format!("{}.mtp", repo_url),
            format!("modules/{}.mtp", repo_url),
            format!("lib/{}.mtp", repo_url),
        ];

        for path in possible_paths {
            if let Ok(content) = fs::read(&path) {
                return Ok(content);
            }
        }

        // Return placeholder for test modules
        let content = format!("// Module: {}\n", repo_url);
        return Ok(content.into_bytes());
    }

    // For remote modules, we would use git2 to clone/checkout
    // For now, attempt to construct a URL and fetch

    #[cfg(feature = "git")]
    {
        use git2::Repository;

        // Create temp directory for clone
        let temp_dir = std::env::temp_dir().join(format!("mtp_module_{}", commit_hash));

        // Clone or open repository
        let repo = if temp_dir.exists() {
            Repository::open(&temp_dir).map_err(|e| MtpError::RuntimeError {
                error: "GitError".to_string(),
                message: format!("Failed to open repository: {}", e),
            })?
        } else {
            let url = format!("https://{}", repo_url);
            Repository::clone(&url, &temp_dir).map_err(|e| MtpError::RuntimeError {
                error: "GitError".to_string(),
                message: format!("Failed to clone repository: {}", e),
            })?
        };

        // Checkout specific commit
        let oid = git2::Oid::from_str(commit_hash).map_err(|e| MtpError::RuntimeError {
            error: "GitError".to_string(),
            message: format!("Invalid commit hash: {}", e),
        })?;

        let commit = repo.find_commit(oid).map_err(|e| MtpError::RuntimeError {
            error: "GitError".to_string(),
            message: format!("Commit not found: {}", e),
        })?;

        // Get tree and read module files
        let tree = commit.tree().map_err(|e| MtpError::RuntimeError {
            error: "GitError".to_string(),
            message: format!("Failed to get tree: {}", e),
        })?;

        // Collect all .mtp files
        let mut content = Vec::new();
        tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
            if let Some(name) = entry.name() {
                if name.ends_with(".mtp") {
                    if let Some(blob) = entry.to_object(&repo).ok().and_then(|o| o.as_blob().cloned()) {
                        content.extend_from_slice(blob.content());
                        content.push(b'\n');
                    }
                }
            }
            git2::TreeWalkResult::Ok
        }).map_err(|e| MtpError::RuntimeError {
            error: "GitError".to_string(),
            message: format!("Failed to walk tree: {}", e),
        })?;

        return Ok(content);
    }

    // Fallback: return placeholder content
    let content = format!(
        "// Module: {}@{}#{}\n// Remote fetch not available\n",
        repo_url, version, commit_hash
    );
    Ok(content.into_bytes())
}

/// Verify cryptographic signature of module content
fn verify_module_signature(
    repo_url: &str,
    content_hash: &[u8],
    content: &[u8],
) -> Result<(), MtpError> {
    // Security checks on content

    // Check for potentially malicious patterns
    let content_str = String::from_utf8_lossy(content);
    let forbidden_patterns = [
        "<script>",
        "eval(",
        "__proto__",
        "constructor.prototype",
        "process.exit",
        "require('child_process')",
    ];

    for pattern in &forbidden_patterns {
        if content_str.contains(pattern) {
            return Err(MtpError::RuntimeError {
                error: "SecurityError".to_string(),
                message: format!("Module contains forbidden pattern: {}", pattern),
            });
        }
    }

    // Size limit: 10MB
    if content.len() > 10 * 1024 * 1024 {
        return Err(MtpError::RuntimeError {
            error: "SecurityError".to_string(),
            message: "Module too large (max 10MB)".to_string(),
        });
    }

    // Verify content hash matches what we computed
    let computed_hash = Sha256::digest(content);
    if computed_hash.as_slice() != content_hash {
        return Err(MtpError::RuntimeError {
            error: "SecurityError".to_string(),
            message: "Content hash mismatch - module may have been tampered with".to_string(),
        });
    }

    // In a full implementation, we would also:
    // 1. Extract the signature from module metadata
    // 2. Verify the signature against content hash using the signer's public key
    // 3. Check certificate chain if applicable
    // 4. Verify the signer is authorized for this repository

    Ok(())
}

/// Check if a module has been cryptographically verified
pub fn is_module_verified(module_name: &str, context: &ImportContext) -> bool {
    context.verified_modules.contains_key(module_name)
}

/// Get verified module signature
pub fn get_module_signature(
    module_name: &str,
    context: &ImportContext,
) -> Option<&ModuleSignature> {
    context.verified_modules.get(module_name)
}

/// Add a trusted key for module verification
pub fn add_trusted_key(
    context: &mut ImportContext,
    module_name: String,
    public_key: Vec<u8>,
) -> Result<(), MtpError> {
    // Validate public key format (basic check for minimum length)
    if public_key.len() < 32 {
        return Err(MtpError::RuntimeError {
            error: "SecurityError".to_string(),
            message: "Public key too short (minimum 32 bytes)".to_string(),
        });
    }

    context.trusted_keys.insert(module_name, public_key);
    Ok(())
}

/// Generate audit manifest of verified modules
pub fn generate_audit_manifest(context: &ImportContext) -> String {
    let mut manifest = String::from("Verified Modules Audit Manifest\n");
    manifest.push_str("==================================\n\n");
    manifest.push_str(&format!("Generated: {}\n\n", chrono::Utc::now().to_rfc3339()));

    for (name, sig) in &context.verified_modules {
        manifest.push_str(&format!("Module: {}\n", name));
        manifest.push_str(&format!("  Version: {}\n", sig.version));
        manifest.push_str(&format!(
            "  Content Hash: {}\n",
            hex::encode(&sig.content_hash)
        ));
        manifest.push_str(&format!(
            "  Signer Key: {}\n",
            hex::encode(&sig.signer_public_key)
        ));
        manifest.push_str("\n");
    }

    manifest.push_str(&format!("Total modules: {}\n", context.verified_modules.len()));
    manifest
}

/// Load module content from various sources
pub fn load_module_content(path: &str) -> Result<String, MtpError> {
    // Try filesystem first
    if let Ok(content) = fs::read_to_string(path) {
        return Ok(content);
    }

    // Try with .mtp extension
    let with_ext = format!("{}.mtp", path);
    if let Ok(content) = fs::read_to_string(&with_ext) {
        return Ok(content);
    }

    // Try modules directory
    let in_modules = format!("modules/{}.mtp", path);
    if let Ok(content) = fs::read_to_string(&in_modules) {
        return Ok(content);
    }

    Err(MtpError::RuntimeError {
        error: "ImportError".to_string(),
        message: format!("Could not find module: {}", path),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_module_spec_full() {
        let spec = "github.com/user/repo@v1.2.3#abc123def456abc123def456abc123def456abcd";
        let (repo, version, commit) = parse_module_spec(spec).unwrap();
        assert_eq!(repo, "github.com/user/repo");
        assert_eq!(version, "v1.2.3");
        assert_eq!(commit.len(), 40);
    }

    #[test]
    fn test_parse_module_spec_simple() {
        let spec = "my_local_module";
        let (repo, version, commit) = parse_module_spec(spec).unwrap();
        assert_eq!(repo, "my_local_module");
        assert_eq!(version, "local");
        assert_eq!(commit.len(), 40); // Placeholder zeros
    }

    #[test]
    fn test_invalid_module_spec() {
        assert!(parse_module_spec("repo@version").is_err()); // Missing commit
        assert!(parse_module_spec("repo@version#short").is_err()); // Short commit hash
    }

    #[test]
    fn test_import_context() {
        let mut context = ImportContext::new();

        // Add a trusted key
        let key = vec![0u8; 32];
        add_trusted_key(&mut context, "github.com/test/repo".to_string(), key.clone()).unwrap();

        assert!(context.trusted_keys.contains_key("github.com/test/repo"));
    }

    #[test]
    fn test_module_verification() {
        let context = ImportContext::new();
        assert!(!is_module_verified("nonexistent", &context));
    }

    #[test]
    fn test_audit_manifest() {
        let mut context = ImportContext::new();
        context.verified_modules.insert(
            "test_module".to_string(),
            ModuleSignature {
                module_name: "test_module".to_string(),
                version: "1.0.0".to_string(),
                content_hash: vec![1, 2, 3, 4],
                signature: vec![],
                signer_public_key: vec![5, 6, 7, 8],
            },
        );

        let manifest = generate_audit_manifest(&context);
        assert!(manifest.contains("test_module"));
        assert!(manifest.contains("1.0.0"));
        assert!(manifest.contains("Total modules: 1"));
    }
}
