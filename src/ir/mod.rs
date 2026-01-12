pub mod lower;

use crate::effects::Effect;
use crate::errors::MtpError;
use crate::types::Type;
use std::collections::HashMap;

/// IR instruction types
#[derive(Debug, Clone, PartialEq)]
pub enum IrInstruction {
    /// Load a constant value
    LoadConst { value: IrValue, dest: String },
    /// Load a variable
    LoadVar { src: String, dest: String },
    /// Store to a variable
    StoreVar { src: String, dest: String },
    /// Binary operation
    BinOp {
        op: BinOpKind,
        left: String,
        right: String,
        dest: String,
    },
    /// Unary operation
    UnOp {
        op: UnOpKind,
        operand: String,
        dest: String,
    },
    /// Function call
    Call {
        func: String,
        args: Vec<String>,
        dest: Option<String>,
    },
    /// Return from function
    Return { value: Option<String> },
    /// Jump to label
    Jump { label: String },
    /// Conditional jump
    JumpIf {
        condition: String,
        true_label: String,
        false_label: String,
    },
    /// Label definition
    Label { name: String },
    /// Effect call
    EffectCall { effect: String, args: Vec<String> },
}

/// Binary operation kinds
#[derive(Debug, Clone, PartialEq)]
pub enum BinOpKind {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

/// Unary operation kinds
#[derive(Debug, Clone, PartialEq)]
pub enum UnOpKind {
    Neg,
    Not,
}

/// IR values
#[derive(Debug, Clone, PartialEq)]
pub enum IrValue {
    Number(i64),
    Decimal(String), // String representation for precision
    Boolean(bool),
    String(String),
    Null,
}

/// IR function
#[derive(Debug, Clone)]
pub struct IrFunction {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub effects: Vec<Effect>,
    pub locals: HashMap<String, Type>,
    pub instructions: Vec<IrInstruction>,
    pub is_tail_recursive: bool,
}

/// IR program
#[derive(Debug, Clone)]
pub struct IrProgram {
    pub functions: Vec<IrFunction>,
    pub types: HashMap<String, Type>,
}

/// IR Schema Validator
pub struct IrSchemaValidator;

impl IrSchemaValidator {
    pub fn new() -> Self {
        IrSchemaValidator
    }

    /// Validate an entire IR program
    pub fn validate_program(&self, program: &IrProgram) -> Result<(), MtpError> {
        // Check for main function
        let has_main = program.functions.iter().any(|f| f.name == "main");
        if !has_main {
            return Err(MtpError::ValidationError {
                error: "MissingMainFunction".to_string(),
                message: "IR program must have a 'main' function".to_string(),
            });
        }

        // Validate each function
        for func in &program.functions {
            self.validate_function(func)?;
        }

        // Check for undefined types
        for func in &program.functions {
            for (_, param_type) in &func.params {
                self.validate_type_usage(param_type, &program.types)?;
            }
            self.validate_type_usage(&func.return_type, &program.types)?;
        }

        Ok(())
    }

    /// Validate a single function
    pub fn validate_function(&self, func: &IrFunction) -> Result<(), MtpError> {
        // Check function name
        if func.name.is_empty() {
            return Err(MtpError::ValidationError {
                error: "InvalidFunctionName".to_string(),
                message: "Function name cannot be empty".to_string(),
            });
        }

        // Check parameters
        let mut param_names = std::collections::HashSet::new();
        for (param_name, _) in &func.params {
            if param_name.is_empty() {
                return Err(MtpError::ValidationError {
                    error: "InvalidParameterName".to_string(),
                    message: format!("Parameter name cannot be empty in function '{}'", func.name),
                });
            }
            if !param_names.insert(param_name) {
                return Err(MtpError::ValidationError {
                    error: "DuplicateParameter".to_string(),
                    message: format!(
                        "Duplicate parameter '{}' in function '{}'",
                        param_name, func.name
                    ),
                });
            }
        }

        // Validate instructions
        self.validate_instructions(&func.instructions, func)?;

        // Check that all locals are declared
        let mut used_vars = std::collections::HashSet::new();
        self.collect_used_variables(&func.instructions, &mut used_vars);

        for var in used_vars {
            if !func.locals.contains_key(&var) && !func.params.iter().any(|(name, _)| name == &var)
            {
                return Err(MtpError::ValidationError {
                    error: "UndefinedVariable".to_string(),
                    message: format!(
                        "Variable '{}' used but not declared in function '{}'",
                        var, func.name
                    ),
                });
            }
        }

        Ok(())
    }

    /// Validate a sequence of instructions
    pub fn validate_instructions(
        &self,
        instructions: &[IrInstruction],
        func: &IrFunction,
    ) -> Result<(), MtpError> {
        let mut labels = std::collections::HashSet::new();
        let mut label_positions = HashMap::new();

        // First pass: collect labels
        for (i, inst) in instructions.iter().enumerate() {
            if let IrInstruction::Label { name } = inst {
                if !labels.insert(name.clone()) {
                    return Err(MtpError::ValidationError {
                        error: "DuplicateLabel".to_string(),
                        message: format!("Duplicate label '{}' in function '{}'", name, func.name),
                    });
                }
                label_positions.insert(name.clone(), i);
            }
        }

        // Second pass: validate instructions
        for inst in instructions {
            self.validate_instruction(inst, func, &labels)?;
        }

        Ok(())
    }

    /// Validate a single instruction
    pub fn validate_instruction(
        &self,
        inst: &IrInstruction,
        func: &IrFunction,
        labels: &std::collections::HashSet<String>,
    ) -> Result<(), MtpError> {
        match inst {
            IrInstruction::LoadConst { dest, .. } => {
                self.validate_dest(dest, func)?;
            }
            IrInstruction::LoadVar { src, dest } => {
                self.validate_src(src, func)?;
                self.validate_dest(dest, func)?;
            }
            IrInstruction::StoreVar { src, dest } => {
                self.validate_src(src, func)?;
                self.validate_dest(dest, func)?;
            }
            IrInstruction::BinOp {
                left, right, dest, ..
            } => {
                self.validate_src(left, func)?;
                self.validate_src(right, func)?;
                self.validate_dest(dest, func)?;
            }
            IrInstruction::UnOp { operand, dest, .. } => {
                self.validate_src(operand, func)?;
                self.validate_dest(dest, func)?;
            }
            IrInstruction::Call {
                func: func_name,
                args,
                dest,
            } => {
                // Check if function exists (simplified - would need function registry)
                for arg in args {
                    self.validate_src(arg, func)?;
                }
                if let Some(dest) = dest {
                    self.validate_dest(dest, func)?;
                }
            }
            IrInstruction::Return { value } => {
                if let Some(val) = value {
                    self.validate_src(val, func)?;
                }
            }
            IrInstruction::Jump { label } => {
                if !labels.contains(label) {
                    return Err(MtpError::ValidationError {
                        error: "UndefinedLabel".to_string(),
                        message: format!("Undefined label '{}' in function '{}'", label, func.name),
                    });
                }
            }
            IrInstruction::JumpIf {
                condition,
                true_label,
                false_label,
            } => {
                self.validate_src(condition, func)?;
                if !labels.contains(true_label) {
                    return Err(MtpError::ValidationError {
                        error: "UndefinedLabel".to_string(),
                        message: format!(
                            "Undefined true label '{}' in function '{}'",
                            true_label, func.name
                        ),
                    });
                }
                if !labels.contains(false_label) {
                    return Err(MtpError::ValidationError {
                        error: "UndefinedLabel".to_string(),
                        message: format!(
                            "Undefined false label '{}' in function '{}'",
                            false_label, func.name
                        ),
                    });
                }
            }
            IrInstruction::Label { .. } => {} // Already validated
            IrInstruction::EffectCall { effect, args } => {
                // Check if effect is declared
                if !func.effects.iter().any(|e| e.name == *effect) {
                    return Err(MtpError::ValidationError {
                        error: "UndeclaredEffect".to_string(),
                        message: format!(
                            "Effect '{}' used but not declared in function '{}'",
                            effect, func.name
                        ),
                    });
                }
                for arg in args {
                    self.validate_src(arg, func)?;
                }
            }
        }
        Ok(())
    }

    /// Validate that a source variable exists
    fn validate_src(&self, var: &str, func: &IrFunction) -> Result<(), MtpError> {
        if !func.locals.contains_key(var) && !func.params.iter().any(|(name, _)| name == var) {
            return Err(MtpError::ValidationError {
                error: "UndefinedVariable".to_string(),
                message: format!(
                    "Source variable '{}' not declared in function '{}'",
                    var, func.name
                ),
            });
        }
        Ok(())
    }

    /// Validate that a destination variable can be assigned
    fn validate_dest(&self, var: &str, func: &IrFunction) -> Result<(), MtpError> {
        if func.params.iter().any(|(name, _)| name == var) {
            return Err(MtpError::ValidationError {
                error: "CannotAssignToParameter".to_string(),
                message: format!(
                    "Cannot assign to parameter '{}' in function '{}'",
                    var, func.name
                ),
            });
        }
        // For now, allow assignment to any local
        Ok(())
    }

    /// Validate type usage
    fn validate_type_usage(
        &self,
        ty: &Type,
        defined_types: &HashMap<String, Type>,
    ) -> Result<(), MtpError> {
        match ty {
            Type::Custom(name) => {
                if !defined_types.contains_key(name) {
                    return Err(MtpError::ValidationError {
                        error: "UndefinedType".to_string(),
                        message: format!("Type '{}' used but not defined", name),
                    });
                }
            }
            Type::Record(fields) => {
                for (_, field_type) in fields {
                    self.validate_type_usage(field_type, defined_types)?;
                }
            }
            Type::Adt(variants) => {
                for variant in variants {
                    if let Some(payload_type) = &variant.payload {
                        self.validate_type_usage(payload_type, defined_types)?;
                    }
                }
            }
            _ => {} // Primitive types are always valid
        }
        Ok(())
    }

    /// Collect all variables used in instructions
    fn collect_used_variables(
        &self,
        instructions: &[IrInstruction],
        used: &mut std::collections::HashSet<String>,
    ) {
        for inst in instructions {
            match inst {
                IrInstruction::LoadConst { dest, .. } => {
                    used.insert(dest.clone());
                }
                IrInstruction::LoadVar { src, dest } => {
                    used.insert(src.clone());
                    used.insert(dest.clone());
                }
                IrInstruction::StoreVar { src, dest } => {
                    used.insert(src.clone());
                    used.insert(dest.clone());
                }
                IrInstruction::BinOp {
                    left, right, dest, ..
                } => {
                    used.insert(left.clone());
                    used.insert(right.clone());
                    used.insert(dest.clone());
                }
                IrInstruction::UnOp { operand, dest, .. } => {
                    used.insert(operand.clone());
                    used.insert(dest.clone());
                }
                IrInstruction::Call { args, dest, .. } => {
                    for arg in args {
                        used.insert(arg.clone());
                    }
                    if let Some(dest) = dest {
                        used.insert(dest.clone());
                    }
                }
                IrInstruction::Return { value } => {
                    if let Some(val) = value {
                        used.insert(val.clone());
                    }
                }
                IrInstruction::JumpIf { condition, .. } => {
                    used.insert(condition.clone());
                }
                IrInstruction::EffectCall { args, .. } => {
                    for arg in args {
                        used.insert(arg.clone());
                    }
                }
                _ => {}
            }
        }
    }
}

/// Lower AST to IR
pub fn lower_ast_to_ir(ast: &crate::parser::ast::Program) -> Result<IrProgram, MtpError> {
    lower::lower_program(ast)
}

/// Validate IR program
pub fn validate_ir_program(program: &IrProgram) -> Result<(), MtpError> {
    let validator = IrSchemaValidator::new();
    validator.validate_program(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::Effect;

    #[test]
    fn test_ir_function_validation() {
        let validator = IrSchemaValidator::new();

        // Valid function
        let mut func = IrFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), Type::Number)],
            return_type: Type::Number,
            effects: vec![],
            locals: [("result".to_string(), Type::Number)].into(),
            instructions: vec![
                IrInstruction::LoadVar {
                    src: "x".to_string(),
                    dest: "temp".to_string(),
                },
                IrInstruction::LoadConst {
                    value: IrValue::Number(1),
                    dest: "one".to_string(),
                },
                IrInstruction::BinOp {
                    op: BinOpKind::Add,
                    left: "temp".to_string(),
                    right: "one".to_string(),
                    dest: "result".to_string(),
                },
                IrInstruction::Return {
                    value: Some("result".to_string()),
                },
            ],
            is_tail_recursive: false,
        };

        // Should fail because 'temp' is not declared
        assert!(validator.validate_function(&func).is_err());

        // Add temp to locals
        func.locals.insert("temp".to_string(), Type::Number);
        func.locals.insert("one".to_string(), Type::Number);

        assert!(validator.validate_function(&func).is_ok());
    }

    #[test]
    fn test_ir_program_validation() {
        let validator = IrSchemaValidator::new();

        let program = IrProgram {
            functions: vec![],
            types: HashMap::new(),
        };

        // Should fail - no main function
        assert!(validator.validate_program(&program).is_err());
    }

    #[test]
    fn test_instruction_validation() {
        let validator = IrSchemaValidator::new();

        let func = IrFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), Type::Number)],
            return_type: Type::Number,
            effects: vec![Effect {
                name: "TestEffect".to_string(),
                params: vec![],
            }],
            locals: HashMap::new(),
            instructions: vec![],
            is_tail_recursive: false,
        };

        let mut labels = std::collections::HashSet::new();
        labels.insert("label1".to_string());

        // Valid instructions
        assert!(validator
            .validate_instruction(
                &IrInstruction::LoadConst {
                    value: IrValue::Number(42),
                    dest: "x".to_string()
                },
                &func,
                &labels
            )
            .is_ok());

        // Invalid - undefined label
        assert!(validator
            .validate_instruction(
                &IrInstruction::Jump {
                    label: "undefined".to_string()
                },
                &func,
                &labels
            )
            .is_err());

        // Invalid - undeclared effect
        assert!(validator
            .validate_instruction(
                &IrInstruction::EffectCall {
                    effect: "BadEffect".to_string(),
                    args: vec![]
                },
                &func,
                &labels
            )
            .is_err());
    }
}
