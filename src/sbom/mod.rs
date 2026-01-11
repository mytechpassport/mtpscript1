use crate::errors::MtpError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

/// SPDX SBOM format (simplified)
#[derive(Debug, Serialize, Deserialize)]
pub struct SBOM {
    pub spdx_version: String,
    pub data_license: String,
    pub spdx_id: String,
    pub name: String,
    pub namespace: String,
    pub creation_info: CreationInfo,
    pub packages: Vec<Package>,
    pub relationships: Vec<Relationship>,
}

/// Creation info for SBOM
#[derive(Debug, Serialize, Deserialize)]
pub struct CreationInfo {
    pub created: String,
    pub creators: Vec<String>,
}

/// Package information
#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    pub spdx_id: String,
    pub name: String,
    pub version: String,
    pub download_location: String,
    pub files_analyzed: bool,
    pub checksums: Vec<Checksum>,
    pub license_concluded: String,
    pub license_declared: String,
    pub copyright_text: String,
    pub external_refs: Vec<ExternalRef>,
}

/// Checksum information
#[derive(Debug, Serialize, Deserialize)]
pub struct Checksum {
    pub algorithm: String,
    pub checksum_value: String,
}

/// External reference
#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalRef {
    pub reference_category: String,
    pub reference_type: String,
    pub reference_locator: String,
}

/// Relationship between packages
#[derive(Debug, Serialize, Deserialize)]
pub struct Relationship {
    pub spdx_element_id: String,
    pub relationship_type: String,
    pub related_spdx_element: String,
}

/// Vulnerability information
#[derive(Debug, Clone)]
pub struct Vulnerability {
    pub id: String,
    pub severity: VulnerabilitySeverity,
    pub description: String,
    pub affected_versions: Vec<String>,
    pub fixed_versions: Vec<String>,
    pub references: Vec<String>,
}

/// Vulnerability severity levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VulnerabilitySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Dependency scanner
pub struct DependencyScanner {
    pub vulnerabilities_db: HashMap<String, Vec<Vulnerability>>,
}

impl DependencyScanner {
    pub fn new() -> Self {
        let mut scanner = DependencyScanner {
            vulnerabilities_db: HashMap::new(),
        };
        scanner.load_vulnerability_database();
        scanner
    }

    /// Load vulnerability database (in real implementation, this would fetch from external sources)
    fn load_vulnerability_database(&mut self) {
        // Sample vulnerabilities for demonstration
        let ring_vulns = vec![Vulnerability {
            id: "CVE-2023-1234".to_string(),
            severity: VulnerabilitySeverity::High,
            description: "Potential timing attack in ECDSA verification".to_string(),
            affected_versions: vec!["0.16.0".to_string(), "0.16.1".to_string()],
            fixed_versions: vec!["0.16.2".to_string()],
            references: vec!["https://example.com/cve-2023-1234".to_string()],
        }];

        let serde_vulns = vec![Vulnerability {
            id: "CVE-2023-5678".to_string(),
            severity: VulnerabilitySeverity::Medium,
            description: "Denial of service via malicious input".to_string(),
            affected_versions: vec!["1.0.0".to_string()],
            fixed_versions: vec!["1.0.1".to_string()],
            references: vec!["https://example.com/cve-2023-5678".to_string()],
        }];

        self.vulnerabilities_db
            .insert("ring".to_string(), ring_vulns);
        self.vulnerabilities_db
            .insert("serde".to_string(), serde_vulns);
    }

    /// Scan dependencies for vulnerabilities
    pub fn scan_dependencies(&self, packages: &[Package]) -> Vec<(Package, Vec<Vulnerability>)> {
        let mut vulnerable_packages = Vec::new();

        for package in packages {
            if let Some(vulns) = self.vulnerabilities_db.get(&package.name) {
                let relevant_vulns: Vec<Vulnerability> = vulns
                    .iter()
                    .filter(|v| v.affected_versions.contains(&package.version))
                    .cloned()
                    .collect();

                if !relevant_vulns.is_empty() {
                    vulnerable_packages.push((package.clone(), relevant_vulns));
                }
            }
        }

        vulnerable_packages
    }

    /// Generate security report
    pub fn generate_security_report(&self, packages: &[Package]) -> String {
        let vulnerabilities = self.scan_dependencies(packages);

        let mut report = String::from("# Security Vulnerability Report\n\n");

        if vulnerabilities.is_empty() {
            report.push_str("✅ No known vulnerabilities found in dependencies.\n\n");
        } else {
            report.push_str(&format!(
                "⚠️  Found {} vulnerable packages:\n\n",
                vulnerabilities.len()
            ));

            for (package, vulns) in &vulnerabilities {
                report.push_str(&format!(
                    "## Package: {} v{}\n\n",
                    package.name, package.version
                ));

                for vuln in vulns {
                    let severity_icon = match vuln.severity {
                        VulnerabilitySeverity::Low => "🟢",
                        VulnerabilitySeverity::Medium => "🟡",
                        VulnerabilitySeverity::High => "🟠",
                        VulnerabilitySeverity::Critical => "🔴",
                    };

                    report.push_str(&format!(
                        "- {} **{}** ({:?})\n",
                        severity_icon, vuln.id, vuln.severity
                    ));
                    report.push_str(&format!("  - Description: {}\n", vuln.description));
                    report.push_str(&format!(
                        "  - Affected versions: {}\n",
                        vuln.affected_versions.join(", ")
                    ));
                    report.push_str(&format!(
                        "  - Fixed in: {}\n",
                        vuln.fixed_versions.join(", ")
                    ));
                    report.push_str(&format!("  - References: {}\n", vuln.references.join(", ")));
                    report.push_str("\n");
                }
            }
        }

        report
    }
}

/// SBOM Generator
pub struct SBOMGenerator {
    namespace_prefix: String,
}

impl SBOMGenerator {
    pub fn new(namespace_prefix: &str) -> Self {
        SBOMGenerator {
            namespace_prefix: namespace_prefix.to_string(),
        }
    }

    /// Generate SBOM from project dependencies
    pub fn generate_sbom(
        &self,
        project_name: &str,
        version: &str,
        dependencies: &[Dependency],
    ) -> Result<SBOM, MtpError> {
        let timestamp = chrono::Utc::now().to_rfc3339();

        let mut packages = Vec::new();
        let mut relationships = Vec::new();

        // Main package
        let main_package_id = format!("SPDXRef-{}", project_name);
        let main_package = Package {
            spdx_id: main_package_id.clone(),
            name: project_name.to_string(),
            version: version.to_string(),
            download_location: "NOASSERTION".to_string(),
            files_analyzed: false,
            checksums: vec![],                    // Would compute actual checksums
            license_concluded: "MIT".to_string(), // Example
            license_declared: "MIT".to_string(),
            copyright_text: "NOASSERTION".to_string(),
            external_refs: vec![],
        };

        packages.push(main_package);

        // Dependency packages
        for (i, dep) in dependencies.iter().enumerate() {
            let package_id = format!("SPDXRef-Dep{}", i);

            let checksum = Checksum {
                algorithm: "SHA256".to_string(),
                checksum_value: dep
                    .checksum
                    .clone()
                    .unwrap_or_else(|| "NOASSERTION".to_string()),
            };

            let package = Package {
                spdx_id: package_id.clone(),
                name: dep.name.clone(),
                version: dep.version.clone(),
                download_location: dep
                    .source
                    .clone()
                    .unwrap_or_else(|| "NOASSERTION".to_string()),
                files_analyzed: false,
                checksums: vec![checksum],
                license_concluded: dep
                    .license
                    .clone()
                    .unwrap_or_else(|| "NOASSERTION".to_string()),
                license_declared: dep
                    .license
                    .clone()
                    .unwrap_or_else(|| "NOASSERTION".to_string()),
                copyright_text: "NOASSERTION".to_string(),
                external_refs: vec![ExternalRef {
                    reference_category: "PACKAGE-MANAGER".to_string(),
                    reference_type: "purl".to_string(),
                    reference_locator: format!("pkg:cargo/{}/{}", dep.name, dep.version),
                }],
            };

            packages.push(package.clone());

            // Add relationship to main package
            relationships.push(Relationship {
                spdx_element_id: main_package_id.clone(),
                relationship_type: "DEPENDS_ON".to_string(),
                related_spdx_element: package_id,
            });
        }

        let sbom = SBOM {
            spdx_version: "SPDX-2.3".to_string(),
            data_license: "CC0-1.0".to_string(),
            spdx_id: format!("SPDXRef-Document"),
            name: format!("{}-SBOM", project_name),
            namespace: format!("{}/{}", self.namespace_prefix, project_name),
            creation_info: CreationInfo {
                created: timestamp,
                creators: vec!["Tool: MTPScript SBOM Generator".to_string()],
            },
            packages,
            relationships,
        };

        Ok(sbom)
    }

    /// Generate CycloneDX SBOM (alternative format)
    pub fn generate_cyclonedx_sbom(
        &self,
        project_name: &str,
        version: &str,
        dependencies: &[Dependency],
    ) -> Result<String, MtpError> {
        // Simplified CycloneDX format
        let mut sbom = serde_json::json!({
            "bomFormat": "CycloneDX",
            "specVersion": "1.4",
            "version": 1,
            "metadata": {
                "component": {
                    "type": "application",
                    "name": project_name,
                    "version": version
                }
            },
            "components": []
        });

        let components = dependencies
            .iter()
            .map(|dep| {
                serde_json::json!({
                    "type": "library",
                    "name": &dep.name,
                    "version": &dep.version,
                    "purl": format!("pkg:cargo/{}/{}", dep.name, dep.version)
                })
            })
            .collect::<Vec<_>>();

        sbom["components"] = serde_json::Value::Array(components);

        Ok(serde_json::to_string_pretty(&sbom)?)
    }
}

/// Dependency information
#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub source: Option<String>,
    pub license: Option<String>,
    pub checksum: Option<String>,
}

/// Dependency resolver
pub struct DependencyResolver;

impl DependencyResolver {
    pub fn new() -> Self {
        DependencyResolver
    }

    /// Scan Cargo.toml for dependencies
    pub fn scan_cargo_dependencies(
        &self,
        cargo_toml_path: &str,
    ) -> Result<Vec<Dependency>, MtpError> {
        let content = fs::read_to_string(cargo_toml_path)
            .map_err(|e| MtpError::IoError(format!("Failed to read Cargo.toml: {}", e)))?;

        let cargo_toml: toml::Value = toml::from_str(&content)
            .map_err(|e| MtpError::ParseError(format!("Failed to parse Cargo.toml: {}", e)))?;

        let mut dependencies = Vec::new();

        // Scan [dependencies] section
        if let Some(deps) = cargo_toml.get("dependencies") {
            if let Some(deps_table) = deps.as_table() {
                for (name, info) in deps_table {
                    let dep = self.parse_dependency_info(name, info)?;
                    dependencies.push(dep);
                }
            }
        }

        Ok(dependencies)
    }

    fn parse_dependency_info(
        &self,
        name: &str,
        info: &toml::Value,
    ) -> Result<Dependency, MtpError> {
        match info {
            toml::Value::String(version) => {
                Ok(Dependency {
                    name: name.to_string(),
                    version: version.clone(),
                    source: Some("crates.io".to_string()),
                    license: None, // Would need to look up in Cargo.lock or registry
                    checksum: None,
                })
            }
            toml::Value::Table(table) => {
                let version = table
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                Ok(Dependency {
                    name: name.to_string(),
                    version,
                    source: Some("crates.io".to_string()),
                    license: None,
                    checksum: None,
                })
            }
            _ => Err(MtpError::ParseError(format!(
                "Invalid dependency format for {}",
                name
            ))),
        }
    }

    /// Scan for transitive dependencies via Cargo.lock
    pub fn scan_transitive_dependencies(
        &self,
        cargo_lock_path: &str,
    ) -> Result<Vec<Dependency>, MtpError> {
        let content = fs::read_to_string(cargo_lock_path)
            .map_err(|e| MtpError::IoError(format!("Failed to read Cargo.lock: {}", e)))?;

        let cargo_lock: toml::Value = toml::from_str(&content)
            .map_err(|e| MtpError::ParseError(format!("Failed to parse Cargo.lock: {}", e)))?;

        let mut dependencies = Vec::new();

        if let Some(packages) = cargo_lock.get("package") {
            if let Some(packages_array) = packages.as_array() {
                for package in packages_array {
                    if let Some(package_table) = package.as_table() {
                        let name = package_table
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("unknown");

                        let version = package_table
                            .get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");

                        let checksum = package_table
                            .get("checksum")
                            .and_then(|c| c.as_str())
                            .map(|s| s.to_string());

                        dependencies.push(Dependency {
                            name: name.to_string(),
                            version: version.to_string(),
                            source: Some("crates.io".to_string()),
                            license: None,
                            checksum,
                        });
                    }
                }
            }
        }

        Ok(dependencies)
    }
}

/// Generate complete SBOM and security report
pub fn generate_sbom_and_report(
    project_name: &str,
    version: &str,
    cargo_toml_path: &str,
) -> Result<(SBOM, String), MtpError> {
    let resolver = DependencyResolver::new();
    let dependencies = resolver.scan_cargo_dependencies(cargo_toml_path)?;

    let generator = SBOMGenerator::new("https://mtpscript.example.com/sbom");
    let sbom = generator.generate_sbom(project_name, version, &dependencies)?;

    let scanner = DependencyScanner::new();
    let security_report = scanner.generate_security_report(&sbom.packages);

    Ok((sbom, security_report))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_scanning() {
        let resolver = DependencyResolver::new();

        // Test with a mock Cargo.toml content
        let mock_cargo_toml = r#"
            [package]
            name = "test"
            version = "1.0.0"

            [dependencies]
            serde = "1.0"
            ring = "0.16"
        "#;

        // Write to temporary file
        let temp_path = "/tmp/test_cargo.toml";
        fs::write(temp_path, mock_cargo_toml).unwrap();

        let deps = resolver.scan_cargo_dependencies(temp_path).unwrap();
        assert_eq!(deps.len(), 2);
        assert!(deps.iter().any(|d| d.name == "serde"));
        assert!(deps.iter().any(|d| d.name == "ring"));
    }

    #[test]
    fn test_vulnerability_scanning() {
        let scanner = DependencyScanner::new();

        let packages = vec![Package {
            spdx_id: "SPDXRef-test".to_string(),
            name: "ring".to_string(),
            version: "0.16.0".to_string(),
            download_location: "crates.io".to_string(),
            files_analyzed: false,
            checksums: vec![],
            license_concluded: "MIT".to_string(),
            license_declared: "MIT".to_string(),
            copyright_text: "NOASSERTION".to_string(),
            external_refs: vec![],
        }];

        let vulnerabilities = scanner.scan_dependencies(&packages);
        assert!(!vulnerabilities.is_empty());

        let report = scanner.generate_security_report(&packages);
        assert!(report.contains("CVE-2023-1234"));
    }

    #[test]
    fn test_sbom_generation() {
        let generator = SBOMGenerator::new("https://example.com");

        let dependencies = vec![Dependency {
            name: "serde".to_string(),
            version: "1.0.0".to_string(),
            source: Some("crates.io".to_string()),
            license: Some("MIT".to_string()),
            checksum: Some("abc123".to_string()),
        }];

        let sbom = generator
            .generate_sbom("test-project", "1.0.0", &dependencies)
            .unwrap();
        assert_eq!(sbom.name, "test-project-SBOM");
        assert_eq!(sbom.packages.len(), 2); // main package + 1 dependency
    }
}
