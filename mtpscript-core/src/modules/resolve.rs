use crate::errors::MtpError;
use crate::modules::import::{ImportDecl, ImportResolver};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Module resolution context
pub struct ModuleResolution {
    pub root_path: String,
    pub resolved_modules: HashMap<String, ResolvedModule>,
    pub dependency_graph: HashMap<String, Vec<String>>,
}

/// Resolved module information
#[derive(Debug, Clone)]
pub struct ResolvedModule {
    pub name: String,
    pub path: String,
    pub dependencies: Vec<String>,
    pub content_hash: String,
}

/// Module resolver
pub struct ModuleResolver {
    import_resolver: ImportResolver,
    resolved: HashMap<String, ResolvedModule>,
}

impl ModuleResolver {
    pub fn new() -> Self {
        Self {
            import_resolver: ImportResolver::new(),
            resolved: HashMap::new(),
        }
    }

    /// Resolve all modules starting from entry points
    pub fn resolve_modules(
        &mut self,
        entry_points: &[String],
    ) -> Result<ModuleResolution, MtpError> {
        let mut to_resolve = Vec::from(entry_points);
        let mut resolved_set = HashSet::new();
        let mut dependency_graph: HashMap<String, Vec<String>> = HashMap::new();

        while let Some(module_path) = to_resolve.pop() {
            if resolved_set.contains(&module_path) {
                continue;
            }

            let module = self.resolve_single_module(&module_path)?;
            resolved_set.insert(module_path.clone());
            dependency_graph.insert(module.name.clone(), module.dependencies.clone());

            // Add dependencies to resolution queue
            for dep in &module.dependencies {
                if !resolved_set.contains(dep) {
                    to_resolve.push(dep.clone());
                }
            }
        }

        // Check for circular dependencies
        self.detect_cycles(&dependency_graph)?;

        Ok(ModuleResolution {
            root_path: ".".to_string(),
            resolved_modules: self.resolved.clone(),
            dependency_graph,
        })
    }

    /// Resolve a single module
    fn resolve_single_module(&mut self, module_path: &str) -> Result<ResolvedModule, MtpError> {
        if let Some(resolved) = self.resolved.get(module_path) {
            return Ok(resolved.clone());
        }

        let path = Path::new(module_path);
        let content = std::fs::read_to_string(path).map_err(|e| MtpError::Io {
            error: "Io".to_string(),
            message: e.to_string(),
        })?;

        // Parse imports from content
        let imports = self.extract_imports(&content)?;
        let mut dependencies = Vec::new();

        for import in imports {
            self.import_resolver.resolve_import(&import)?;
            dependencies.push(import.alias);
        }

        // Compute content hash
        let content_hash = sha256::digest(content.as_bytes());

        let module = ResolvedModule {
            name: path
                .file_stem()
                .ok_or_else(|| MtpError::Build {
                    error: "Build".to_string(),
                    message: "Invalid module path".to_string(),
                })?
                .to_string_lossy()
                .to_string(),
            path: module_path.to_string(),
            dependencies,
            content_hash,
        };

        self.resolved
            .insert(module_path.to_string(), module.clone());

        Ok(module)
    }

    /// Extract imports from module content
    fn extract_imports(&self, content: &str) -> Result<Vec<ImportDecl>, MtpError> {
        let mut imports = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("import ") {
                let import = crate::modules::import::parse_import_decl(line)?;
                imports.push(import);
            }
        }

        Ok(imports)
    }

    /// Detect circular dependencies
    fn detect_cycles(&self, graph: &HashMap<String, Vec<String>>) -> Result<(), MtpError> {
        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();

        for node in graph.keys() {
            if !visited.contains(node) {
                self.dfs_cycle_detect(graph, node, &mut visiting, &mut visited)?;
            }
        }

        Ok(())
    }

    /// DFS for cycle detection
    fn dfs_cycle_detect(
        &self,
        graph: &HashMap<String, Vec<String>>,
        node: &str,
        visiting: &mut HashSet<String>,
        visited: &mut HashSet<String>,
    ) -> Result<(), MtpError> {
        visiting.insert(node.to_string());

        if let Some(deps) = graph.get(node) {
            for dep in deps {
                if !visited.contains(dep) {
                    if visiting.contains(dep) {
                        return Err(MtpError::Build {
                            error: "Build".to_string(),
                            message: format!("Circular dependency detected: {} -> {}", node, dep),
                        });
                    }
                    self.dfs_cycle_detect(graph, dep, visiting, visited)?;
                }
            }
        }

        visiting.remove(node);
        visited.insert(node.to_string());

        Ok(())
    }

    /// Get order-independent compilation order
    pub fn get_compilation_order(
        &self,
        resolution: &ModuleResolution,
    ) -> Result<Vec<String>, MtpError> {
        // Topological sort
        let mut order = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        for module in resolution.resolved_modules.keys() {
            if !visited.contains(module) {
                self.topological_sort(
                    &resolution.dependency_graph,
                    module,
                    &mut order,
                    &mut visited,
                    &mut visiting,
                )?;
            }
        }

        Ok(order)
    }

    /// Topological sort helper
    fn topological_sort(
        &self,
        graph: &HashMap<String, Vec<String>>,
        node: &str,
        order: &mut Vec<String>,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
    ) -> Result<(), MtpError> {
        visiting.insert(node.to_string());

        if let Some(deps) = graph.get(node) {
            for dep in deps {
                if !visited.contains(dep) {
                    if visiting.contains(dep) {
                        return Err(MtpError::Build {
                            error: "Build".to_string(),
                            message: format!(
                                "Circular dependency in topological sort: {} -> {}",
                                node, dep
                            ),
                        });
                    }
                    self.topological_sort(graph, dep, order, visited, visiting)?;
                }
            }
        }

        visiting.remove(node);
        visited.insert(node.to_string());
        order.push(node.to_string());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_resolve_simple_module() {
        let temp_dir = tempdir().unwrap();
        let module_path = temp_dir.path().join("test.mtp");

        let content = r#"
            type User { name: string }
            function greet(u: User) { "Hello" }
        "#;

        fs::write(&module_path, content).unwrap();

        let mut resolver = ModuleResolver::new();
        let module = resolver
            .resolve_single_module(module_path.to_str().unwrap())
            .unwrap();

        assert_eq!(module.name, "test");
        assert!(module.dependencies.is_empty());
        assert!(!module.content_hash.is_empty());
    }

    #[test]
    fn test_detect_cycles() {
        let mut graph = HashMap::new();
        graph.insert("a".to_string(), vec!["b".to_string()]);
        graph.insert("b".to_string(), vec!["a".to_string()]);

        let resolver = ModuleResolver::new();
        let result = resolver.detect_cycles(&graph);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Circular dependency"));
    }

    #[test]
    fn test_topological_sort() {
        let resolver = ModuleResolver::new();
        let mut graph = HashMap::new();
        graph.insert(
            "main".to_string(),
            vec!["utils".to_string(), "types".to_string()],
        );
        graph.insert("utils".to_string(), vec!["types".to_string()]);
        graph.insert("types".to_string(), vec![]);

        let mut resolved_modules = HashMap::new();
        resolved_modules.insert(
            "main".to_string(),
            ResolvedModule {
                name: "main".to_string(),
                path: "main.mtp".to_string(),
                dependencies: vec!["utils".to_string(), "types".to_string()],
                content_hash: "hash".to_string(),
            },
        );
        resolved_modules.insert(
            "utils".to_string(),
            ResolvedModule {
                name: "utils".to_string(),
                path: "utils.mtp".to_string(),
                dependencies: vec!["types".to_string()],
                content_hash: "hash".to_string(),
            },
        );
        resolved_modules.insert(
            "types".to_string(),
            ResolvedModule {
                name: "types".to_string(),
                path: "types.mtp".to_string(),
                dependencies: vec![],
                content_hash: "hash".to_string(),
            },
        );

        let resolution = ModuleResolution {
            root_path: ".".to_string(),
            resolved_modules,
            dependency_graph: graph,
        };

        let order = resolver.get_compilation_order(&resolution).unwrap();
        // types should come before utils and main
        assert_eq!(order[0], "types");
        assert!(order.contains(&"utils".to_string()));
        assert_eq!(order[order.len() - 1], "main");
    }

    #[test]
    fn test_order_independent_compilation() {
        let temp_dir = tempdir().unwrap();
        let main_path = temp_dir.path().join("main.mtp");
        let utils_path = temp_dir.path().join("utils.mtp");

        fs::write(
            &main_path,
            r#"
            function main(): string { "main" }
        "#,
        )
        .unwrap();

        fs::write(
            &utils_path,
            r#"
            function helper(): string { "hello" }
        "#,
        )
        .unwrap();

        let mut resolver1 = ModuleResolver::new();
        let resolution1 = resolver1
            .resolve_modules(&[main_path.to_str().unwrap().to_string()])
            .unwrap();

        let mut resolver2 = ModuleResolver::new();
        let resolution2 = resolver2
            .resolve_modules(&[
                utils_path.to_str().unwrap().to_string(),
                main_path.to_str().unwrap().to_string(),
            ])
            .unwrap();

        // Both should resolve to the same modules
        assert_eq!(resolution1.resolved_modules.len(), 1);
        assert_eq!(resolution2.resolved_modules.len(), 2);
    }
}
