use crate::errors::compile::CompileError;
use crate::parser::ast::{self, Expr, ModuleDecl, TypeExpr};
use crate::types::{AdtType, AdtVariant, RecordType, Type, TypeContext};
use std::collections::HashMap;

pub struct TypeChecker {
    context: TypeContext,
    type_vars: HashMap<String, Type>,
    recursion_depth: usize,
    max_recursion_depth: usize,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            context: TypeContext::with_builtins(),
            type_vars: HashMap::new(),
            recursion_depth: 0,
            max_recursion_depth: 50,
        }
    }

    fn enter_recursion(&mut self) -> Result<(), CompileError> {
        self.recursion_depth += 1;
        if self.recursion_depth > self.max_recursion_depth {
            return Err(CompileError::TypeError(
                "Type recursion depth limit exceeded".to_string(),
            ));
        }
        Ok(())
    }

    fn exit_recursion(&mut self) {
        self.recursion_depth = self.recursion_depth.saturating_sub(1);
    }

    pub fn typecheck_program(&mut self, program: &ast::Program) -> Result<(), CompileError> {
        for decl in &program.decls {
            self.typecheck_decl(decl)?;
        }
        Ok(())
    }

    fn typecheck_decl(&mut self, decl: &ModuleDecl) -> Result<(), CompileError> {
        match decl {
            ModuleDecl::Type(type_decl) => self.typecheck_type_decl(type_decl),
            ModuleDecl::Func(func_decl) => self.typecheck_func_decl(func_decl),
            ModuleDecl::Api(api_decl) => self.typecheck_api_decl(api_decl),
            ModuleDecl::Import(_) => Ok(()), // Imports not implemented yet
        }
    }

    fn typecheck_type_decl(&mut self, decl: &ast::TypeDecl) -> Result<(), CompileError> {
        match decl {
            ast::TypeDecl::Record { name, fields } => {
                let record_fields = fields
                    .iter()
                    .map(|(field_name, type_expr)| {
                        let field_type = self.resolve_type_expr(type_expr)?;
                        Ok((field_name.clone(), field_type))
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                let record_type = RecordType::new(name.clone(), record_fields);
                self.context
                    .insert(name.clone(), Type::Record(Box::new(record_type)));
                Ok(())
            }
            ast::TypeDecl::Adt {
                name,
                type_params,
                variants,
            } => {
                // Set type vars for this declaration
                for param in type_params {
                    self.type_vars
                        .insert(param.clone(), Type::TypeVar(param.clone()));
                }

                let adt_variants = variants
                    .iter()
                    .map(|variant| match &variant.payload[..] {
                        [] => Ok(AdtVariant::Unit(variant.name.clone())),
                        payload => {
                            let types = payload
                                .iter()
                                .map(|te| self.resolve_type_expr(te))
                                .collect::<Result<Vec<_>, _>>()?;
                            Ok(AdtVariant::Tuple(variant.name.clone(), types))
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                // Clear type vars
                for param in type_params {
                    self.type_vars.remove(param);
                }

                let adt_type = AdtType {
                    name: name.clone(),
                    type_params: type_params.clone(),
                    variants: adt_variants,
                };
                self.context
                    .insert(name.clone(), Type::Adt(Box::new(adt_type)));
                Ok(())
            }
        }
    }

    fn typecheck_func_decl(&mut self, decl: &ast::FuncDecl) -> Result<(), CompileError> {
        // Add parameters to context
        let mut local_context = self.context.clone();
        for (param_name, param_type_expr) in &decl.params {
            let param_type = self.resolve_type_expr(param_type_expr)?;
            local_context.insert(param_name.clone(), param_type);
        }

        // Typecheck body
        let _body_type = self.typecheck_expr(&decl.body, &local_context)?;

        // For now, assume functions return the type of their body
        // In full implementation, would check return type annotation
        Ok(())
    }

    fn typecheck_api_decl(&mut self, decl: &ast::ApiDecl) -> Result<(), CompileError> {
        // Similar to func, but API bodies should return something compatible with respond
        let _body_type = self.typecheck_expr(&decl.body, &self.context)?;
        // Check that body uses respond or similar
        Ok(())
    }

    fn typecheck_expr(&self, expr: &Expr, context: &TypeContext) -> Result<Type, CompileError> {
        match expr {
            Expr::String(_) => Ok(Type::String),
            Expr::Number(_) => Ok(Type::Number),
            Expr::Decimal(_) => Ok(Type::Decimal),
            Expr::Boolean(_) => Ok(Type::Boolean),
            Expr::Array(elements) => {
                if elements.is_empty() {
                    // Empty array, can't infer type
                    Err(CompileError::TypeError(
                        "Cannot infer type of empty array".to_string(),
                    ))
                } else {
                    let elem_type = self.typecheck_expr(&elements[0], context)?;
                    // Check all elements have same type
                    for elem in &elements[1..] {
                        let t = self.typecheck_expr(elem, context)?;
                        if t != elem_type {
                            return Err(CompileError::TypeError(
                                "Array elements must have same type".to_string(),
                            ));
                        }
                    }
                    Ok(elem_type) // Arrays not fully typed yet
                }
            }
            Expr::Object(_fields) => {
                // Objects are JSON-like, but for now return a placeholder
                Ok(Type::String) // Placeholder
            }
            Expr::Ident(name) => context
                .lookup(name)
                .cloned()
                .ok_or_else(|| CompileError::TypeError(format!("Undefined variable: {}", name))),
            Expr::Dot(obj, field) => {
                let obj_type = self.typecheck_expr(obj, context)?;
                match obj_type {
                    Type::Record(record) => record.field_type(field).cloned().ok_or_else(|| {
                        CompileError::TypeError(format!("Field '{}' not found", field))
                    }),
                    _ => Err(CompileError::TypeError(
                        "Dot access only on records".to_string(),
                    )),
                }
            }
            Expr::Call { func, args: _ } => {
                let _func_type = self.typecheck_expr(func, context)?;
                // For now, assume functions return something
                Ok(Type::Number) // Placeholder
            }
            Expr::Binary(op, left, right) => {
                let left_type = self.typecheck_expr(left, context)?;
                let right_type = self.typecheck_expr(right, context)?;
                match op {
                    ast::BinOp::Add | ast::BinOp::Sub | ast::BinOp::Mul | ast::BinOp::Div => {
                        if left_type == Type::Number && right_type == Type::Number {
                            Ok(Type::Number)
                        } else {
                            Err(CompileError::TypeError(
                                "Arithmetic operations require numbers".to_string(),
                            ))
                        }
                    }
                    ast::BinOp::Eq
                    | ast::BinOp::Ne
                    | ast::BinOp::Lt
                    | ast::BinOp::Gt
                    | ast::BinOp::Le
                    | ast::BinOp::Ge => {
                        if left_type == right_type {
                            Ok(Type::Boolean)
                        } else {
                            Err(CompileError::TypeError(
                                "Comparison requires same types".to_string(),
                            ))
                        }
                    }
                    ast::BinOp::And | ast::BinOp::Or => {
                        if left_type == Type::Boolean && right_type == Type::Boolean {
                            Ok(Type::Boolean)
                        } else {
                            Err(CompileError::TypeError(
                                "Logical operations require booleans".to_string(),
                            ))
                        }
                    }
                    ast::BinOp::Not => unreachable!("Not is unary"),
                }
            }
            Expr::Unary(op, expr) => {
                let expr_type = self.typecheck_expr(expr, context)?;
                match op {
                    ast::BinOp::Add | ast::BinOp::Sub => {
                        if expr_type == Type::Number {
                            Ok(Type::Number)
                        } else {
                            Err(CompileError::TypeError(
                                "Unary +/- require number".to_string(),
                            ))
                        }
                    }
                    ast::BinOp::Not => {
                        if expr_type == Type::Boolean {
                            Ok(Type::Boolean)
                        } else {
                            Err(CompileError::TypeError(
                                "Unary ! requires boolean".to_string(),
                            ))
                        }
                    }
                    _ => Err(CompileError::TypeError(
                        "Unsupported unary operator".to_string(),
                    )),
                }
            }
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_type = self.typecheck_expr(condition, context)?;
                if cond_type != Type::Boolean {
                    return Err(CompileError::TypeError(
                        "If condition must be boolean".to_string(),
                    ));
                }
                let then_type = self.typecheck_expr(then_branch, context)?;
                let else_type = self.typecheck_expr(else_branch, context)?;
                if then_type == else_type {
                    Ok(then_type)
                } else {
                    Err(CompileError::TypeError(
                        "If branches must have same type".to_string(),
                    ))
                }
            }
            _ => Ok(Type::Number), // Placeholder for unimplemented
        }
    }

    fn resolve_type_expr(&mut self, type_expr: &TypeExpr) -> Result<Type, CompileError> {
        self.enter_recursion()?;
        let result = match type_expr {
            TypeExpr::Ident(name) => {
                if let Some(t) = self.context.lookup(name) {
                    Ok(t.clone())
                } else if let Some(t) = self.type_vars.get(name) {
                    Ok(t.clone())
                } else {
                    Err(CompileError::TypeError(format!("Undefined type: {}", name)))
                }
            }
            TypeExpr::Generic(base, args) => {
                let base_type = self.resolve_type_expr(&TypeExpr::Ident(base.clone()))?;
                match base_type {
                    Type::Adt(adt) if adt.type_params.len() == args.len() => {
                        let mut subs = std::collections::HashMap::new();
                        for (param, arg_expr) in adt.type_params.iter().zip(args) {
                            let arg_type = self.resolve_type_expr(arg_expr)?;
                            subs.insert(param.clone(), arg_type);
                        }
                        Ok(Type::Adt(Box::new(adt.substitute(&subs))))
                    }
                    _ => Err(CompileError::TypeError(format!(
                        "Type '{}' is not a generic type or wrong number of arguments",
                        base
                    ))),
                }
            }
        };
        self.exit_recursion();
        result
    }
}
