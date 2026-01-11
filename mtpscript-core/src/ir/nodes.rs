use crate::parser::ast::{BinOp, HttpMethod};
use crate::types::Type;

#[derive(Debug, Clone, PartialEq)]
pub enum IrExpr {
    // Literals
    String(String, Type),
    Number(i64, Type),
    Decimal(String, Type),
    Boolean(bool, Type),
    Array(Vec<IrExpr>, Type),
    Object(Vec<(String, IrExpr)>, Type),

    // Identifiers and access
    Var(String, Type),
    Dot(Box<IrExpr>, String, Type),
    Index(Box<IrExpr>, Box<IrExpr>, Type),

    // Function calls and operators
    Call {
        func: Box<IrExpr>,
        args: Vec<IrExpr>,
        result_type: Type,
    },
    TailCall {
        func: Box<IrExpr>,
        args: Vec<IrExpr>,
        result_type: Type,
    },
    Lambda {
        params: Vec<String>,
        body: Box<IrExpr>,
        result_type: Type,
    },
    Unary(BinOp, Box<IrExpr>, Type),
    Binary(BinOp, Box<IrExpr>, Box<IrExpr>, Type),

    // Control flow
    If {
        condition: Box<IrExpr>,
        then_branch: Box<IrExpr>,
        else_branch: Box<IrExpr>,
        result_type: Type,
    },
    Match {
        expr: Box<IrExpr>,
        cases: Vec<(IrPattern, IrExpr)>,
        result_type: Type,
    },

    // Declarations in expressions
    Let {
        name: String,
        value: Box<IrExpr>,
        body: Box<IrExpr>,
        result_type: Type,
    },

    // Special constructs
    EffectCall(String, Vec<IrExpr>, Type), // For built-in effects
    RespondJson(Box<IrExpr>, Type),
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrPattern {
    Wildcard,
    Var(String),
    Literal(IrExpr),
    Variant(String, Vec<IrPattern>),
    Record(String, Vec<(String, IrPattern)>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrFunction {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub effects: Vec<String>,
    pub body: IrExpr,
    pub is_tail_recursive: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrApi {
    pub method: HttpMethod,
    pub path: String,
    pub effects: Vec<String>,
    pub body: IrExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrDecl {
    Function(IrFunction),
    Api(IrApi),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrProgram {
    pub decls: Vec<IrDecl>,
}

impl IrProgram {
    pub fn validate(&self) -> Result<(), String> {
        // Schema validation with depth and size limits
        if self.decls.len() > 1000 {
            return Err("Too many declarations".to_string());
        }

        let mut names = std::collections::HashSet::new();

        for decl in &self.decls {
            match decl {
                IrDecl::Function(func) => {
                    // Check function name uniqueness
                    if !names.insert(func.name.clone()) {
                        return Err(format!("Duplicate function name: {}", func.name));
                    }

                    // Validate function name length
                    if func.name.is_empty() || func.name.len() > 100 {
                        return Err(format!("Invalid function name length: {}", func.name));
                    }

                    // Check parameter count
                    if func.params.len() > 20 {
                        return Err(format!("Too many parameters in function {}", func.name));
                    }

                    // Check return type match
                    if func.body.result_type() != func.return_type {
                        return Err(format!(
                            "Function {} return type mismatch: expected {:?}, got {:?}",
                            func.name,
                            func.return_type,
                            func.body.result_type()
                        ));
                    }

                    // Validate function body depth and size
                    validate_ir_expr(&func.body, 0, 50)?;
                }
                IrDecl::Api(api) => {
                    // Validate API path
                    if api.path.is_empty() || api.path.len() > 200 {
                        return Err("Invalid API path length".to_string());
                    }

                    // Validate API body
                    validate_ir_expr(&api.body, 0, 50)?;
                }
            }

            fn validate_ir_expr(
                expr: &IrExpr,
                current_depth: usize,
                max_depth: usize,
            ) -> Result<(), String> {
                if current_depth > max_depth {
                    return Err("IR expression exceeds maximum depth".to_string());
                }

                match expr {
                    IrExpr::Array(elements, _) => {
                        if elements.len() > 1000 {
                            return Err("Array too large".to_string());
                        }
                        for elem in elements {
                            validate_ir_expr(elem, current_depth + 1, max_depth)?;
                        }
                    }
                    IrExpr::Object(fields, _) => {
                        if fields.len() > 100 {
                            return Err("Object too large".to_string());
                        }
                        for (key, value) in fields {
                            if key.is_empty() || key.len() > 100 {
                                return Err("Invalid object key".to_string());
                            }
                            validate_ir_expr(value, current_depth + 1, max_depth)?;
                        }
                    }
                    IrExpr::Call { func, args, .. } => {
                        if args.len() > 20 {
                            return Err("Too many call arguments".to_string());
                        }
                        validate_ir_expr(func, current_depth + 1, max_depth)?;
                        for arg in args {
                            validate_ir_expr(arg, current_depth + 1, max_depth)?;
                        }
                    }
                    IrExpr::TailCall { func, args, .. } => {
                        if args.len() > 20 {
                            return Err("Too many tail call arguments".to_string());
                        }
                        validate_ir_expr(func, current_depth + 1, max_depth)?;
                        for arg in args {
                            validate_ir_expr(arg, current_depth + 1, max_depth)?;
                        }
                    }
                    IrExpr::Lambda { body, .. } => {
                        validate_ir_expr(body, current_depth + 1, max_depth)?;
                    }
                    IrExpr::Unary(_, expr, _) | IrExpr::Dot(expr, _, _) => {
                        validate_ir_expr(expr, current_depth + 1, max_depth)?;
                    }
                    IrExpr::Binary(_, left, right, _) | IrExpr::Index(left, right, _) => {
                        validate_ir_expr(left, current_depth + 1, max_depth)?;
                        validate_ir_expr(right, current_depth + 1, max_depth)?;
                    }
                    IrExpr::If {
                        condition,
                        then_branch,
                        else_branch,
                        ..
                    } => {
                        validate_ir_expr(condition, current_depth + 1, max_depth)?;
                        validate_ir_expr(then_branch, current_depth + 1, max_depth)?;
                        validate_ir_expr(else_branch, current_depth + 1, max_depth)?;
                    }
                    IrExpr::Match {
                        expr: match_expr,
                        cases,
                        ..
                    } => {
                        if cases.len() > 20 {
                            return Err("Too many match cases".to_string());
                        }
                        validate_ir_expr(match_expr, current_depth + 1, max_depth)?;
                        for (_, case_expr) in cases {
                            validate_ir_expr(case_expr, current_depth + 1, max_depth)?;
                        }
                    }
                    IrExpr::Let { value, body, .. } => {
                        validate_ir_expr(value, current_depth + 1, max_depth)?;
                        validate_ir_expr(body, current_depth + 1, max_depth)?;
                    }
                    IrExpr::EffectCall(_, args, _) => {
                        if args.len() > 10 {
                            return Err("Too many effect call arguments".to_string());
                        }
                        for arg in args {
                            validate_ir_expr(arg, current_depth + 1, max_depth)?;
                        }
                    }
                    IrExpr::RespondJson(expr, _) => {
                        validate_ir_expr(expr, current_depth + 1, max_depth)?;
                    }
                    // Literals and simple expressions don't need recursive validation
                    IrExpr::String(_, _)
                    | IrExpr::Number(_, _)
                    | IrExpr::Decimal(_, _)
                    | IrExpr::Boolean(_, _)
                    | IrExpr::Var(_, _) => {}
                }

                Ok(())
            }
        }
        Ok(())
    }
}

impl IrExpr {
    pub fn result_type(&self) -> Type {
        match self {
            IrExpr::String(_, t)
            | IrExpr::Number(_, t)
            | IrExpr::Decimal(_, t)
            | IrExpr::Boolean(_, t)
            | IrExpr::Array(_, t)
            | IrExpr::Object(_, t)
            | IrExpr::Var(_, t)
            | IrExpr::Dot(_, _, t)
            | IrExpr::Index(_, _, t)
            | IrExpr::Call { result_type: t, .. }
            | IrExpr::TailCall { result_type: t, .. }
            | IrExpr::Lambda { result_type: t, .. }
            | IrExpr::Unary(_, _, t)
            | IrExpr::Binary(_, _, _, t)
            | IrExpr::If { result_type: t, .. }
            | IrExpr::Match { result_type: t, .. }
            | IrExpr::Let { result_type: t, .. }
            | IrExpr::EffectCall(_, _, t)
            | IrExpr::RespondJson(_, t) => t.clone(),
        }
    }
}
