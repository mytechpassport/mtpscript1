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
            ModuleDecl::Import(import_decl) => self.typecheck_import(import_decl),
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

                // Register a placeholder type first to allow recursive type references
                // This allows types like `type Tree = Leaf(number) | Node(Tree, Tree)`
                self.context.insert(name.clone(), Type::Var(name.clone()));

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

                // Create the result type for this ADT
                let result_type = if type_params.is_empty() {
                    Type::Var(name.clone())
                } else {
                    Type::TypeVar(format!("{}${}", name, type_params.join("$")))
                };

                // Register each variant as a constructor in the context
                for variant in variants {
                    match &variant.payload[..] {
                        [] => {
                            // Unit variant is a value of the ADT type
                            self.context.insert(variant.name.clone(), result_type.clone());
                        }
                        payload => {
                            // Variant with payload is a function: (payload types) -> ADT type
                            let payload_types = payload
                                .iter()
                                .map(|te| self.resolve_type_expr(te))
                                .collect::<Result<Vec<_>, _>>()?;
                            let constructor_type = Type::Function(payload_types, Box::new(result_type.clone()));
                            self.context.insert(variant.name.clone(), constructor_type);
                        }
                    }
                }

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
        // Get parameter types
        let param_types: Vec<Type> = decl
            .params
            .iter()
            .map(|(_, param_type_expr)| self.resolve_type_expr(param_type_expr))
            .collect::<Result<Vec<_>, _>>()?;

        // Register the function in context first with placeholder return type
        // This enables recursive call type checking
        let placeholder_return = Type::TypeVar(format!("{}$return", decl.name));
        self.context.insert(
            decl.name.clone(),
            Type::Function(param_types.clone(), Box::new(placeholder_return)),
        );

        // Add parameters to local context
        let mut local_context = self.context.clone();
        for (param_name, param_type_expr) in &decl.params {
            let param_type = self.resolve_type_expr(param_type_expr)?;
            local_context.insert(param_name.clone(), param_type);
        }

        // Typecheck body and infer return type
        let body_type = self.typecheck_expr(&decl.body, &local_context)?;

        // Update the function's type with the inferred return type
        self.context.insert(
            decl.name.clone(),
            Type::Function(param_types, Box::new(body_type)),
        );

        Ok(())
    }

    fn typecheck_api_decl(&mut self, decl: &ast::ApiDecl) -> Result<(), CompileError> {
        // Similar to func, but API bodies should return something compatible with respond
        let _body_type = self.typecheck_expr(&decl.body, &self.context)?;
        // Check that body uses respond or similar
        Ok(())
    }

    fn typecheck_import(&mut self, import: &ast::ImportDecl) -> Result<(), CompileError> {
        // Validate import path format
        if import.path.is_empty() {
            return Err(CompileError::TypeError(
                "Import path cannot be empty".to_string(),
            ));
        }

        // Validate alias is a valid identifier
        if import.alias.is_empty() {
            return Err(CompileError::TypeError(
                "Import alias cannot be empty".to_string(),
            ));
        }

        // Check for reserved names
        let reserved = ["number", "string", "boolean", "void", "any", "null"];
        if reserved.contains(&import.alias.as_str()) {
            return Err(CompileError::TypeError(format!(
                "Cannot use reserved type name '{}' as import alias",
                import.alias
            )));
        }

        // Register the import alias in context as a module type
        // Module resolution would happen at a later compilation stage
        // For now, register as an opaque type to enable basic name resolution
        self.context.insert(
            import.alias.clone(),
            Type::TypeVar(format!("module:{}", import.path)),
        );

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
            Expr::Call { func, args } => {
                let func_type = self.typecheck_expr(func, context)?;
                // Typecheck arguments
                for arg in args {
                    let _ = self.typecheck_expr(arg, context)?;
                }
                // Extract return type from function type
                match func_type {
                    Type::Function(_, return_type) => Ok(*return_type),
                    Type::TypeVar(_) => {
                        // Unknown function type (e.g., from recursive call before inference)
                        // Return a type variable placeholder
                        Ok(Type::TypeVar("call$return".to_string()))
                    }
                    _ => Err(CompileError::TypeError(format!(
                        "Cannot call non-function type: {:?}",
                        func_type
                    ))),
                }
            }
            Expr::Binary(op, left, right) => {
                let left_type = self.typecheck_expr(left, context)?;
                let right_type = self.typecheck_expr(right, context)?;
                match op {
                    ast::BinOp::Add => {
                        // Allow both number and string addition (homogeneous types only)
                        if left_type == Type::Number && right_type == Type::Number {
                            Ok(Type::Number)
                        } else if left_type == Type::String && right_type == Type::String {
                            Ok(Type::String)
                        } else {
                            Err(CompileError::TypeError(format!(
                                "Cannot add {:?} and {:?} - addition requires matching types (number + number or string + string)",
                                left_type, right_type
                            )))
                        }
                    }
                    ast::BinOp::Sub | ast::BinOp::Mul | ast::BinOp::Div => {
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
            Expr::Index(arr, _index) => {
                // Array/object indexing - return element type or unknown
                let arr_type = self.typecheck_expr(arr, context)?;
                // For now, return the array's element type if known, otherwise Number
                match arr_type {
                    // Ideally would track Array<T> and return T
                    _ => Ok(Type::Number), // Placeholder - proper array types not yet implemented
                }
            }
            Expr::Pipeline(left, right) => {
                // Pipeline: left |> right means right(left)
                // Type of pipeline is the return type of right applied to left
                let _left_type = self.typecheck_expr(left, context)?;
                let right_type = self.typecheck_expr(right, context)?;
                match right_type {
                    Type::Function(_, return_type) => Ok(*return_type),
                    _ => Err(CompileError::TypeError(
                        "Pipeline right side must be a function".to_string(),
                    )),
                }
            }
            Expr::Match { expr, cases } => {
                // Match expression - all case bodies must return same type
                let scrutinee_type = self.typecheck_expr(expr, context)?;
                if cases.is_empty() {
                    return Err(CompileError::TypeError(
                        "Match expression must have at least one case".to_string(),
                    ));
                }

                // Type check each arm with pattern bindings
                let mut result_type: Option<Type> = None;
                for (pattern, case_body) in cases {
                    // Create context with pattern bindings
                    let mut local_context = context.clone();
                    self.bind_pattern_vars(pattern, &scrutinee_type, &mut local_context)?;

                    let case_type = self.typecheck_expr(case_body, &local_context)?;
                    if let Some(ref expected) = result_type {
                        if case_type != *expected {
                            return Err(CompileError::TypeError(
                                "All match arms must return same type".to_string(),
                            ));
                        }
                    } else {
                        result_type = Some(case_type);
                    }
                }
                Ok(result_type.unwrap())
            }
            Expr::Const { name, value, body } => {
                // Const binding: const x = value in body
                let value_type = self.typecheck_expr(value, context)?;
                let mut local_context = context.clone();
                local_context.insert(name.clone(), value_type);
                self.typecheck_expr(body, &local_context)
            }
            Expr::Lambda { params, body } => {
                // Lambda expression
                let mut local_context = context.clone();
                let mut param_types = Vec::new();
                for (param_name, param_type_expr) in params {
                    let param_type = self.resolve_type_expr_const(param_type_expr)?;
                    local_context.insert(param_name.clone(), param_type.clone());
                    param_types.push(param_type);
                }
                let body_type = self.typecheck_expr(body, &local_context)?;
                Ok(Type::Function(param_types, Box::new(body_type)))
            }
            Expr::Await(inner) => {
                // Await unwraps async values - for now, just check inner type
                self.typecheck_expr(inner, context)
            }
            Expr::RespondJson(inner) => {
                // respond_json should accept any JSON-serializable type
                let _inner_type = self.typecheck_expr(inner, context)?;
                // Response type is typically void/unit, but use String for JSON output
                Ok(Type::String)
            }
            Expr::Group(inner) => {
                // Grouping doesn't change type
                self.typecheck_expr(inner, context)
            }
        }
    }

    // Non-mutable version of resolve_type_expr for use in typecheck_expr
    fn resolve_type_expr_const(&self, type_expr: &TypeExpr) -> Result<Type, CompileError> {
        match type_expr {
            TypeExpr::Ident(name) => {
                if let Some(t) = self.context.lookup(name) {
                    Ok(t.clone())
                } else if let Some(t) = self.type_vars.get(name) {
                    Ok(t.clone())
                } else {
                    Err(CompileError::TypeError(format!("Undefined type: {}", name)))
                }
            }
            TypeExpr::Generic(base, _args) => {
                // Simplified: just return the base type for now
                self.resolve_type_expr_const(&TypeExpr::Ident(base.clone()))
            }
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

    /// Bind pattern variables to the context for type checking match arm bodies
    fn bind_pattern_vars(
        &self,
        pattern: &ast::Pattern,
        scrutinee_type: &Type,
        context: &mut TypeContext,
    ) -> Result<(), CompileError> {
        match pattern {
            ast::Pattern::Wildcard => Ok(()),
            ast::Pattern::Ident(name) => {
                // Simple variable binding - takes on the scrutinee type
                context.insert(name.clone(), scrutinee_type.clone());
                Ok(())
            }
            ast::Pattern::Literal(_) => Ok(()), // No bindings in literals
            ast::Pattern::Variant(variant_name, sub_patterns) => {
                // Look up the variant in the ADT type to get payload types
                let payload_types = self.get_variant_payload_types(scrutinee_type, variant_name)?;

                // Bind sub-patterns
                for (sub_pattern, payload_type) in sub_patterns.iter().zip(payload_types.iter()) {
                    self.bind_pattern_vars(sub_pattern, payload_type, context)?;
                }
                Ok(())
            }
            ast::Pattern::Record(_, fields) => {
                // For record patterns, bind each field variable
                for (field_name, field_pattern) in fields {
                    // Get field type from scrutinee if it's a record
                    let field_type = match scrutinee_type {
                        Type::Record(record) => record.field_type(field_name).cloned(),
                        _ => None,
                    };
                    let field_type = field_type.unwrap_or_else(|| Type::TypeVar("unknown".to_string()));
                    self.bind_pattern_vars(field_pattern, &field_type, context)?;
                }
                Ok(())
            }
        }
    }

    /// Get the payload types for a variant of an ADT
    fn get_variant_payload_types(&self, scrutinee_type: &Type, variant_name: &str) -> Result<Vec<Type>, CompileError> {
        // First check if this is a direct ADT type
        if let Type::Adt(adt) = scrutinee_type {
            for variant in &adt.variants {
                match variant {
                    AdtVariant::Unit(name) if name == variant_name => return Ok(vec![]),
                    AdtVariant::Tuple(name, types) if name == variant_name => return Ok(types.clone()),
                    _ => {}
                }
            }
        }

        // Check if it's a named ADT type (Type::Var)
        if let Type::Var(type_name) = scrutinee_type {
            if let Some(Type::Adt(adt)) = self.context.lookup(type_name) {
                for variant in &adt.variants {
                    match variant {
                        AdtVariant::Unit(name) if name == variant_name => return Ok(vec![]),
                        AdtVariant::Tuple(name, types) if name == variant_name => return Ok(types.clone()),
                        _ => {}
                    }
                }
            }
        }

        // For built-in types like Option/Result, check the context
        // For generic type variables or unknown types, return a type variable for each sub-pattern
        // This is a fallback that allows flexible pattern matching
        Ok(vec![Type::TypeVar(format!("{}$payload", variant_name))])
    }
}
