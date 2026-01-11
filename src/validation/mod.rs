use crate::errors::MtpError;
use std::collections::HashMap;

/// Schema validation result
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        ValidationResult {
            is_valid: true,
            errors: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.is_valid = false;
        self.errors.push(error);
    }

    pub fn merge(&mut self, other: ValidationResult) {
        if !other.is_valid {
            self.is_valid = false;
            self.errors.extend(other.errors);
        }
    }
}

/// Schema validator trait
pub trait SchemaValidator {
    fn validate(&self, data: &serde_json::Value) -> ValidationResult;
}

/// JSON Schema validator
pub struct JsonSchemaValidator {
    schema: serde_json::Value,
}

impl JsonSchemaValidator {
    pub fn new(schema: serde_json::Value) -> Self {
        JsonSchemaValidator { schema }
    }

    pub fn from_str(schema_str: &str) -> Result<Self, MtpError> {
        let schema: serde_json::Value =
            serde_json::from_str(schema_str).map_err(|e| MtpError::ValidationError {
                error: "SchemaParseError".to_string(),
                message: format!("Failed to parse JSON schema: {}", e),
            })?;
        Ok(JsonSchemaValidator::new(schema))
    }
}

impl SchemaValidator for JsonSchemaValidator {
    fn validate(&self, data: &serde_json::Value) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Basic validation - check required fields and types
        if let Some(required) = self.schema.get("required") {
            if let Some(required_fields) = required.as_array() {
                for field in required_fields {
                    if let Some(field_name) = field.as_str() {
                        if !data.get(field_name).is_some() {
                            result.add_error(format!("Missing required field: {}", field_name));
                        }
                    }
                }
            }
        }

        // Type validation
        if let Some(properties) = self.schema.get("properties") {
            if let Some(props) = properties.as_object() {
                for (field_name, field_schema) in props {
                    if let Some(field_value) = data.get(field_name) {
                        if let Some(expected_type) = field_schema.get("type") {
                            if let Some(type_str) = expected_type.as_str() {
                                let actual_type = match field_value {
                                    serde_json::Value::Null => "null",
                                    serde_json::Value::Bool(_) => "boolean",
                                    serde_json::Value::Number(_) => "number",
                                    serde_json::Value::String(_) => "string",
                                    serde_json::Value::Array(_) => "array",
                                    serde_json::Value::Object(_) => "object",
                                };

                                if actual_type != type_str {
                                    result.add_error(format!(
                                        "Field '{}' has wrong type: expected {}, got {}",
                                        field_name, type_str, actual_type
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        result
    }
}

/// MTPScript AST schema validator
pub struct AstSchemaValidator;

impl AstSchemaValidator {
    pub fn new() -> Self {
        AstSchemaValidator
    }

    pub fn validate_program(&self, program: &crate::parser::ast::Program) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Validate program structure
        if program.decls.is_empty() {
            result.add_error("Program must have at least one declaration".to_string());
        }

        // Validate each declaration
        for decl in &program.decls {
            match decl {
                crate::parser::ast::ModuleDecl::Func(func) => {
                    let func_result = self.validate_function(func);
                    result.merge(func_result);
                }
                crate::parser::ast::ModuleDecl::Type(type_decl) => {
                    let type_result = self.validate_type_declaration(type_decl);
                    result.merge(type_result);
                }
                crate::parser::ast::ModuleDecl::Api(api) => {
                    let api_result = self.validate_api_declaration(api);
                    result.merge(api_result);
                }
                crate::parser::ast::ModuleDecl::Import(import) => {
                    let import_result = self.validate_import(import);
                    result.merge(import_result);
                }
            }
        }

        result
    }

    fn validate_function(&self, func: &crate::parser::ast::FuncDecl) -> ValidationResult {
        let mut result = ValidationResult::new();

        if func.name.is_empty() {
            result.add_error("Function name cannot be empty".to_string());
        }

        if func.body.is_none() {
            result.add_error(format!("Function '{}' must have a body", func.name));
        }

        result
    }

    fn validate_type_declaration(
        &self,
        type_decl: &crate::parser::ast::TypeDecl,
    ) -> ValidationResult {
        let mut result = ValidationResult::new();

        if type_decl.name.is_empty() {
            result.add_error("Type name cannot be empty".to_string());
        }

        match &type_decl.kind {
            crate::parser::ast::TypeDeclKind::Record { fields } => {
                if fields.is_empty() {
                    result.add_error(format!(
                        "Record type '{}' must have at least one field",
                        type_decl.name
                    ));
                }
                // Check for duplicate field names
                let mut field_names = std::collections::HashSet::new();
                for field in fields {
                    if !field_names.insert(&field.name) {
                        result.add_error(format!(
                            "Duplicate field name '{}' in record '{}'",
                            field.name, type_decl.name
                        ));
                    }
                }
            }
            crate::parser::ast::TypeDeclKind::Adt { variants } => {
                if variants.is_empty() {
                    result.add_error(format!(
                        "ADT '{}' must have at least one variant",
                        type_decl.name
                    ));
                }
            }
        }

        result
    }

    fn validate_api_declaration(&self, api: &crate::parser::ast::ApiDecl) -> ValidationResult {
        let mut result = ValidationResult::new();

        if api.path.is_empty() {
            result.add_error("API path cannot be empty".to_string());
        }

        if !api.path.starts_with('/') {
            result.add_error(format!("API path '{}' must start with '/'", api.path));
        }

        result
    }

    fn validate_import(&self, import: &crate::parser::ast::ImportDecl) -> ValidationResult {
        let mut result = ValidationResult::new();

        if import.path.is_empty() {
            result.add_error("Import path cannot be empty".to_string());
        }

        if !import.path.contains('@') {
            result.add_error(format!(
                "Import path '{}' must include version pin",
                import.path
            ));
        }

        result
    }
}

/// Configuration schema validator
pub struct ConfigSchemaValidator;

impl ConfigSchemaValidator {
    pub fn new() -> Self {
        ConfigSchemaValidator
    }

    pub fn validate_runtime_config(
        &self,
        config: &crate::runtime::InterpreterConfig,
    ) -> ValidationResult {
        let mut result = ValidationResult::new();

        if config.gas_limit == 0 {
            result.add_error("Gas limit must be greater than 0".to_string());
        }

        if config.max_memory_mb == 0 {
            result.add_error("Max memory must be greater than 0 MB".to_string());
        }

        if config.max_execution_time.as_secs() == 0 && config.max_execution_time.subsec_nanos() == 0
        {
            result.add_error("Max execution time must be greater than 0".to_string());
        }

        result
    }

    pub fn validate_compiler_config(
        &self,
        config: &crate::compiler::CompilerConfig,
    ) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Add compiler-specific validations
        if config.output_path.is_empty() {
            result.add_error("Output path cannot be empty".to_string());
        }

        result
    }
}

/// Global schema registry
pub struct SchemaRegistry {
    validators: HashMap<String, Box<dyn SchemaValidator>>,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        SchemaRegistry {
            validators: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, validator: Box<dyn SchemaValidator>) {
        self.validators.insert(name.to_string(), validator);
    }

    pub fn validate(&self, schema_name: &str, data: &serde_json::Value) -> Result<(), MtpError> {
        if let Some(validator) = self.validators.get(schema_name) {
            let result = validator.validate(data);
            if result.is_valid {
                Ok(())
            } else {
                Err(MtpError::ValidationError {
                    error: "SchemaValidationFailed".to_string(),
                    message: format!(
                        "Validation failed for schema '{}': {:?}",
                        schema_name, result.errors
                    ),
                })
            }
        } else {
            Err(MtpError::ValidationError {
                error: "UnknownSchema".to_string(),
                message: format!("Unknown schema: {}", schema_name),
            })
        }
    }

    pub fn list_schemas(&self) -> Vec<String> {
        self.validators.keys().cloned().collect()
    }
}

/// Global schema registry instance
static mut SCHEMA_REGISTRY: Option<SchemaRegistry> = None;

/// Initialize global schema registry
pub fn init_schema_registry() {
    unsafe {
        SCHEMA_REGISTRY = Some(SchemaRegistry::new());
    }
}

/// Get global schema registry
pub fn get_schema_registry() -> Option<&'static mut SchemaRegistry> {
    unsafe { SCHEMA_REGISTRY.as_mut() }
}

/// Validate data against a registered schema
pub fn validate_against_schema(
    schema_name: &str,
    data: &serde_json::Value,
) -> Result<(), MtpError> {
    if let Some(registry) = get_schema_registry() {
        registry.validate(schema_name, data)
    } else {
        Err(MtpError::ValidationError {
            error: "RegistryNotInitialized".to_string(),
            message: "Schema registry not initialized".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::{Field, FuncDecl, ModuleDecl, Program, TypeDecl, TypeDeclKind};

    #[test]
    fn test_json_schema_validation() {
        let schema_str = r#"{
            "type": "object",
            "required": ["name", "age"],
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number"}
            }
        }"#;

        let validator = JsonSchemaValidator::from_str(schema_str).unwrap();

        let valid_data = serde_json::json!({
            "name": "Alice",
            "age": 30
        });

        let result = validator.validate(&valid_data);
        assert!(result.is_valid);

        let invalid_data = serde_json::json!({
            "name": "Bob"
            // missing age
        });

        let result = validator.validate(&invalid_data);
        assert!(!result.is_valid);
        assert!(result.errors.len() > 0);
    }

    #[test]
    fn test_ast_validation() {
        let validator = AstSchemaValidator::new();

        let mut program = Program { decls: vec![] };
        let result = validator.validate_program(&program);
        assert!(!result.is_valid); // Empty program should fail

        let func_decl = FuncDecl {
            name: "test".to_string(),
            params: vec![],
            return_type: crate::types::Type::Number,
            effects: vec![],
            body: Some(crate::parser::ast::Expr::Literal(
                crate::parser::ast::Literal::Number(42),
            )),
        };

        program.decls.push(ModuleDecl::Func(func_decl));
        let result = validator.validate_program(&program);
        assert!(result.is_valid);
    }

    #[test]
    fn test_config_validation() {
        let validator = ConfigSchemaValidator::new();

        let invalid_config = crate::runtime::InterpreterConfig {
            gas_limit: 0, // Invalid
            max_memory_mb: 100,
            max_execution_time: std::time::Duration::from_secs(30),
        };

        let result = validator.validate_runtime_config(&invalid_config);
        assert!(!result.is_valid);
    }
}
