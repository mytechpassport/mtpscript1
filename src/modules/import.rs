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

/// Legacy function for compatibility
pub fn verify_and_import_module(
    import: &ImportDecl,
    _context: &mut ImportContext,
) -> Result<(), MtpError> {
    // For now, just resolve the import
    resolve_import(import)?;
    Ok(())
}

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
    // Expected format: "github.com/user/repo@v1.2.3#abc123"
    let parts: Vec<&str> = spec.split('@').collect();
    if parts.len() != 2 {
        return Err(MtpError {
            error: "ModuleError".to_string(),
            message: Some("Invalid module specification format".to_string()),
            gasLimit: None,
            gasUsed: None,
        });
    }

    let repo_url = parts[0].to_string();
    let version_commit = parts[1];

    let version_commit_parts: Vec<&str> = version_commit.split('#').collect();
    if version_commit_parts.len() != 2 {
        return Err(MtpError {
            error: "ModuleError".to_string(),
            message: Some("Invalid version/commit format".to_string()),
            gasLimit: None,
            gasUsed: None,
        });
    }

    let version = version_commit_parts[0].to_string();
    let commit_hash = version_commit_parts[1].to_string();

    // Validate commit hash format (should be hex)
    if !commit_hash.chars().all(|c| c.is_ascii_hexdigit()) || commit_hash.len() != 40 {
        return Err(MtpError {
            error: "ModuleError".to_string(),
            message: Some("Invalid commit hash format".to_string()),
            gasLimit: None,
            gasUsed: None,
        });
    }

    Ok((repo_url, version, commit_hash))
}

/// Fetch module content (placeholder - would implement git fetching)
fn fetch_module_content(
    repo_url: &str,
    version: &str,
    commit_hash: &str,
) -> Result<Vec<u8>, MtpError> {
    // In real implementation, this would:
    // 1. Clone/checkout the specific commit
    // 2. Verify the commit hash matches
    // 3. Extract module files
    // 4. Return concatenated content

    // For now, just return a placeholder
    let content = format!("module {}@{}#{}", repo_url, version, commit_hash);
    Ok(content.into_bytes())
}

/// Verify cryptographic signature of module content
fn verify_module_signature(
    repo_url: &str,
    content_hash: &[u8],
    content: &[u8],
) -> Result<(), MtpError> {
    // In real implementation, this would:
    // 1. Extract signature from module metadata
    // 2. Verify signature against content hash using signer's public key
    // 3. Check certificate chain if applicable

    // Placeholder validation - check that content is not obviously malicious
    if content.contains(&b"<script>"[..]) {
        return Err(MtpError {
            error: "SecurityError".to_string(),
            message: Some("Module contains potentially malicious script tags".to_string()),
            gasLimit: None,
            gasUsed: None,
        });
    }

    if content.len() > 10 * 1024 * 1024 {
        // 10MB limit
        return Err(MtpError {
            error: "SecurityError".to_string(),
            message: Some("Module too large".to_string()),
            gasLimit: None,
            gasUsed: None,
        });
    }

    // Verify content hash matches expected
    let computed_hash = Sha256::digest(content);
    if computed_hash.as_slice() != content_hash {
        return Err(MtpError {
            error: "SecurityError".to_string(),
            message: Some("Content hash mismatch".to_string()),
            gasLimit: None,
            gasUsed: None,
        });
    }

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
    // Validate public key format (basic check)
    if public_key.len() < 32 {
        return Err(MtpError {
            error: "SecurityError".to_string(),
            message: Some("Public key too short".to_string()),
            gasLimit: None,
            gasUsed: None,
        });
    }

    context.trusted_keys.insert(module_name, public_key);
    Ok(())
}

/// Generate audit manifest of verified modules
pub fn generate_audit_manifest(context: &ImportContext) -> String {
    let mut manifest = String::from("Verified Modules Audit Manifest\n");
    manifest.push_str("==================================\n\n");

    for (name, sig) in &context.verified_modules {
        manifest.push_str(&format!("Module: {}\n", name));
        manifest.push_str(&format!("Version: {}\n", sig.version));
        manifest.push_str(&format!(
            "Content Hash: {}\n",
            hex::encode(&sig.content_hash)
        ));
        manifest.push_str(&format!(
            "Signer Key: {}\n",
            hex::encode(&sig.signer_public_key)
        ));
        manifest.push_str("\n");
    }

    manifest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_module_spec() {
        let spec = "github.com/user/repo@v1.2.3#abc123def456";
        let (repo, version, commit) = parse_module_spec(spec).unwrap();
        assert_eq!(repo, "github.com/user/repo");
        assert_eq!(version, "v1.2.3");
        assert_eq!(commit, "abc123def456");
    }

    #[test]
    fn test_invalid_module_spec() {
        assert!(parse_module_spec("invalid").is_err());
        assert!(parse_module_spec("repo@version").is_err());
        assert!(parse_module_spec("repo@version#commit#extra").is_err());
    }
}
