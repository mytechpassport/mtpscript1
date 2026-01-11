use crate::errors::MtpError;
use crate::parser::ast::*;
use std::collections::HashMap;

pub struct TypeCheckerConfig {
    pub max_recursion_depth: usize,
}

impl Default for TypeCheckerConfig {
    fn default() -> TypeCheckerConfig {
        TypeCheckerConfig {
            max_recursion_depth: 100,
        }
    }
}

pub struct TypeContext {
    pub types: HashMap<String, Type>,
    pub recursion_depth: usize,
    pub config: TypeCheckerConfig,
}

impl TypeContext {
    pub fn new(config: TypeCheckerConfig) -> Self {
        TypeContext {
            types: HashMap::new(),
            recursion_depth: 0,
            config,
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), MtpError> {
        for decl in &program.decls {
            self.check_module_decl(decl)?;
        }
        Ok(())
    }

    fn check_module_decl(&mut self, decl: &ModuleDecl) -> Result<(), MtpError> {
        self.check_recursion_limit()?;

        match decl {
            ModuleDecl::Type(type_decl) => self.check_type_decl(type_decl),
            ModuleDecl::Func(func_decl) => self.check_func_decl(func_decl),
            ModuleDecl::Api(api_decl) => self.check_api_decl(api_decl),
            ModuleDecl::Import(_) => Ok(()), // Imports are checked elsewhere
        }
    }

    fn check_type_decl(&mut self, type_decl: &TypeDecl) -> Result<(), MtpError> {
        // Basic validation - ensure no cycles
        self.recursion_depth += 1;
        if self.recursion_depth > self.config.max_recursion_depth {
            return Err(MtpError::TypeError(
                "Type declaration recursion depth exceeded".into(),
            ));
        }

        // Check for duplicate type names
        if self.types.contains_key(&type_decl.name) {
            return Err(MtpError::TypeError(format!(
                "Duplicate type declaration: {}",
                type_decl.name
            )));
        }

        // Validate variants
        for variant in &type_decl.variants {
            if let Some(payload) = &variant.payload {
                self.check_type(payload)?;
            }
        }

        self.types.insert(
            type_decl.name.clone(),
            Type::Adt(type_decl.name.clone(), Vec::new()),
        );
        self.recursion_depth -= 1;
        Ok(())
    }

    fn check_func_decl(&mut self, func_decl: &FuncDecl) -> Result<(), MtpError> {
        self.check_recursion_limit()?;

        // Check parameter types
        for (_, param_type) in &func_decl.params {
            self.check_type(param_type)?;
        }

        // Check return type
        if let Some(return_type) = &func_decl.return_type {
            self.check_type(return_type)?;
        }

        // Check body
        self.recursion_depth += 1;
        let body_type = self.infer_expr(&func_decl.body)?;
        self.recursion_depth -= 1;

        // Check return type compatibility
        if let Some(expected) = &func_decl.return_type {
            if !self.types_compatible(expected, &body_type) {
                return Err(MtpError::TypeError(
                    "Function body type doesn't match return type".into(),
                ));
            }
        }

        Ok(())
    }

    fn check_api_decl(&mut self, api_decl: &ApiDecl) -> Result<(), MtpError> {
        self.check_recursion_limit()?;

        // API body should return a response
        self.recursion_depth += 1;
        let body_type = self.infer_expr(&api_decl.body)?;
        self.recursion_depth -= 1;

        // For now, just ensure it's not an error
        match body_type {
            Type::Generic(_) => Ok(()), // Allow generic response types
            _ => Ok(()),
        }
    }

    fn infer_expr(&mut self, expr: &Expr) -> Result<Type, MtpError> {
        self.check_recursion_limit()?;

        match expr {
            Expr::Ident(name) => {
                // Look up in context
                if let Some(ty) = self.types.get(name) {
                    Ok(ty.clone())
                } else {
                    Err(MtpError::TypeError(format!(
                        "Undefined identifier: {}",
                        name
                    )))
                }
            }
            Expr::StringLit(_) => Ok(Type::String),
            Expr::NumberLit(_) => Ok(Type::Number),
            Expr::BoolLit(_) => Ok(Type::Boolean),
            Expr::Array(elements) => {
                if elements.is_empty() {
                    Ok(Type::Array(Box::new(Type::Generic("T".to_string()))))
                } else {
                    let elem_type = self.infer_expr(&elements[0])?;
                    // Check all elements have same type
                    for elem in &elements[1..] {
                        let elem_ty = self.infer_expr(elem)?;
                        if !self.types_compatible(&elem_type, &elem_ty) {
                            return Err(MtpError::TypeError(
                                "Array elements have inconsistent types".into(),
                            ));
                        }
                    }
                    Ok(Type::Array(Box::new(elem_type)))
                }
            }
            Expr::Object(fields) => {
                let mut field_types = HashMap::new();
                for (key, value) in fields {
                    field_types.insert(key.clone(), self.infer_expr(value)?);
                }
                Ok(Type::Object(field_types))
            }
            Expr::Call { func, args } => {
                // For now, assume all calls return generic type
                // In a full implementation, this would look up function signatures
                Ok(Type::Generic("Return".to_string()))
            }
            Expr::Binary { op, left, right } => {
                let left_type = self.infer_expr(left)?;
                let right_type = self.infer_expr(right)?;

                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                        if matches!(left_type, Type::Number) && matches!(right_type, Type::Number) {
                            Ok(Type::Number)
                        } else {
                            Err(MtpError::TypeError(
                                "Arithmetic operators require number operands".into(),
                            ))
                        }
                    }
                    BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                        if self.types_compatible(&left_type, &right_type) {
                            Ok(Type::Boolean)
                        } else {
                            Err(MtpError::TypeError(
                                "Comparison operands must have compatible types".into(),
                            ))
                        }
                    }
                    BinOp::And | BinOp::Or => {
                        if matches!(left_type, Type::Boolean) && matches!(right_type, Type::Boolean)
                        {
                            Ok(Type::Boolean)
                        } else {
                            Err(MtpError::TypeError(
                                "Logical operators require boolean operands".into(),
                            ))
                        }
                    }
                    BinOp::Pipe => {
                        // Pipeline - right operand should be a function
                        Ok(right_type)
                    }
                }
            }
            Expr::Unary { op, expr } => {
                let expr_type = self.infer_expr(expr)?;
                match op {
                    UnOp::Not => {
                        if matches!(expr_type, Type::Boolean) {
                            Ok(Type::Boolean)
                        } else {
                            Err(MtpError::TypeError(
                                "Not operator requires boolean operand".into(),
                            ))
                        }
                    }
                    UnOp::Neg => {
                        if matches!(expr_type, Type::Number) {
                            Ok(Type::Number)
                        } else {
                            Err(MtpError::TypeError(
                                "Negation operator requires number operand".into(),
                            ))
                        }
                    }
                }
            }
            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond_type = self.infer_expr(cond)?;
                if !matches!(cond_type, Type::Boolean) {
                    return Err(MtpError::TypeError("If condition must be boolean".into()));
                }

                let then_type = self.infer_expr(then_branch)?;
                if let Some(else_branch) = else_branch {
                    let else_type = self.infer_expr(else_branch)?;
                    if self.types_compatible(&then_type, &else_type) {
                        Ok(then_type)
                    } else {
                        Err(MtpError::TypeError(
                            "If branches have incompatible types".into(),
                        ))
                    }
                } else {
                    Ok(then_type)
                }
            }
            Expr::Match { expr, cases } => {
                let expr_type = self.infer_expr(expr)?;

                if cases.is_empty() {
                    return Err(MtpError::TypeError(
                        "Match expression must have at least one case".into(),
                    ));
                }

                let first_case_type = self.infer_expr(&cases[0].1)?;
                for (_, case_expr) in &cases[1..] {
                    let case_type = self.infer_expr(case_expr)?;
                    if !self.types_compatible(&first_case_type, &case_type) {
                        return Err(MtpError::TypeError(
                            "Match case expressions have incompatible types".into(),
                        ));
                    }
                }

                Ok(first_case_type)
            }
            Expr::Block(exprs) => {
                if exprs.is_empty() {
                    Ok(Type::Generic("Unit".to_string()))
                } else {
                    let last_type = self.infer_expr(exprs.last().unwrap())?;
                    Ok(last_type)
                }
            }
            Expr::Lambda { params: _, body } => {
                // For now, return function type
                Ok(Type::Generic("Function".to_string()))
            }
            Expr::Respond(expr) => {
                // Respond expressions can have any type
                self.infer_expr(expr)
            }
        }
    }

    fn check_type(&mut self, ty: &Type) -> Result<(), MtpError> {
        self.check_recursion_limit()?;

        match ty {
            Type::Array(elem_type) => self.check_type(elem_type),
            Type::Object(fields) => {
                for field_type in fields.values() {
                    self.check_type(field_type)?;
                }
                Ok(())
            }
            Type::Adt(name, type_args) => {
                if !self.types.contains_key(name) {
                    return Err(MtpError::TypeError(format!("Undefined type: {}", name)));
                }
                for arg in type_args {
                    self.check_type(arg)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn types_compatible(&self, t1: &Type, t2: &Type) -> bool {
        match (t1, t2) {
            (Type::Number, Type::Number) => true,
            (Type::Boolean, Type::Boolean) => true,
            (Type::String, Type::String) => true,
            (Type::Decimal, Type::Decimal) => true,
            (Type::Array(e1), Type::Array(e2)) => self.types_compatible(e1, e2),
            (Type::Object(f1), Type::Object(f2)) => {
                if f1.len() != f2.len() {
                    return false;
                }
                for (k, v1) in f1 {
                    if let Some(v2) = f2.get(k) {
                        if !self.types_compatible(v1, v2) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                true
            }
            (Type::Generic(_), _) | (_, Type::Generic(_)) => true, // Generics are compatible with anything
            _ => false,
        }
    }

    fn check_recursion_limit(&self) -> Result<(), MtpError> {
        if self.recursion_depth >= self.config.max_recursion_depth {
            return Err(MtpError::TypeError(
                "Maximum type checking recursion depth exceeded".into(),
            ));
        }
        Ok(())
    }
}
