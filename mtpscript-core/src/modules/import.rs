use crate::errors::MtpError;
use std::collections::HashMap;
use std::path::Path;

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
}

impl ImportResolver {
    pub fn new() -> Self {
        Self {
            resolved_modules: HashMap::new(),
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

        // Cache the resolved module
        self.resolved_modules
            .insert(import.module_name.clone(), module_path.clone());

        Ok(module_path)
    }

    /// Validate git reference (simplified)
    fn validate_git_reference(
        &self,
        git_url: &str,
        git_hash: &str,
        tag: Option<&str>,
    ) -> Result<(), MtpError> {
        // In a real implementation, this would clone/verify the git repo
        // For now, just check that the hash looks like a SHA-256

        if git_hash.len() != 64 {
            return Err(MtpError::Build(format!(
                "Invalid git hash length: {}",
                git_hash.len()
            )));
        }

        if let Some(tag_name) = tag {
            // Verify tag exists and points to the hash
            // This is simplified
            if tag_name.is_empty() {
                return Err(MtpError::Build("Empty tag name".to_string()));
            }
        }

        Ok(())
    }

    /// Download module (placeholder)
    fn download_module(&self, import: &ImportDecl) -> Result<String, MtpError> {
        // In a real implementation, this would:
        // 1. Clone/fetch the git repo
        // 2. Checkout the specific hash
        // 3. Verify signature if present
        // 4. Copy to vendored directory

        // For now, return a placeholder path
        Ok(format!("/vendor/{}", import.module_name))
    }

    /// Get all resolved modules
    pub fn resolved_modules(&self) -> &HashMap<String, String> {
        &self.resolved_modules
    }
}

/// Parse import declaration from source
pub fn parse_import_decl(source: &str) -> Result<ImportDecl, MtpError> {
    // Simplified parser for: import "github.com/user/repo@v1.0.0#abc123" as alias

    let import_keyword = "import \"";
    let as_keyword = "\" as ";

    if !source.starts_with(import_keyword) {
        return Err(MtpError::Build("Invalid import syntax".to_string()));
    }

    let after_import = &source[import_keyword.len()..];
    let as_pos = after_import
        .find(as_keyword)
        .ok_or_else(|| MtpError::Build("Missing 'as' keyword".to_string()))?;

    let url_part = &after_import[..as_pos];
    let alias_part = &after_import[as_pos + as_keyword.len()..];

    // Parse URL part: github.com/user/repo@v1.0.0#abc123
    let hash_sep = url_part
        .rfind('#')
        .ok_or_else(|| MtpError::Build("Missing git hash".to_string()))?;
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
}
