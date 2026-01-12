use crate::errors::MtpError;
use crate::ir::{BinOpKind, IrFunction, IrInstruction, IrProgram, IrValue, UnOpKind};
use crate::parser::ast::{BinOp, Expr, FuncDecl, ModuleDecl, Pattern, Program, UnOp};
use crate::types::Type;
use std::collections::HashMap;

pub fn lower_program(program: &Program) -> Result<IrProgram, MtpError> {
    let mut functions = Vec::new();
    let mut types = HashMap::new();

    // Process type declarations first
    for decl in &program.decls {
        if let ModuleDecl::Type(type_decl) = decl {
            // Store ADT types
            let adt_type = Type::Adt(
                type_decl.name.clone(),
                type_decl
                    .type_params
                    .iter()
                    .map(|p| Type::Generic(p.clone()))
                    .collect(),
            );
            types.insert(type_decl.name.clone(), adt_type);
        }
    }

    // Process function declarations
    for decl in &program.decls {
        if let ModuleDecl::Func(func_decl) = decl {
            let ir_func = lower_function(func_decl, &functions)?;
            functions.push(ir_func);
        }
    }

    // Ensure we have a main function or create one from APIs
    if !functions.iter().any(|f| f.name == "main") {
        // Create main from API declarations
        for decl in &program.decls {
            if let ModuleDecl::Api(api_decl) = decl {
                let main_func = create_main_from_api(api_decl)?;
                functions.push(main_func);
                break; // Only handle first API for now
            }
        }
    }

    Ok(IrProgram { functions, types })
}

fn lower_function(func_decl: &FuncDecl, all_funcs: &[IrFunction]) -> Result<IrFunction, MtpError> {
    let mut locals = HashMap::new();
    let mut instructions = Vec::new();
    let mut temp_counter = 0;

    // Add parameters to locals
    for (param_name, param_type) in &func_decl.params {
        locals.insert(param_name.clone(), param_type.clone());
    }

    // Lower the function body
    let result_var = lower_expr(
        &func_decl.body,
        &mut instructions,
        &mut locals,
        &mut temp_counter,
        &func_decl.name,
    )?;

    // Add return instruction
    instructions.push(IrInstruction::Return {
        value: Some(result_var),
    });

    // Detect tail recursion
    let is_tail_recursive = detect_tail_recursion(&func_decl.name, &func_decl.body);

    Ok(IrFunction {
        name: func_decl.name.clone(),
        params: func_decl
            .params
            .iter()
            .map(|(name, ty)| (name.clone(), ty.clone()))
            .collect(),
        return_type: func_decl.return_type.clone().unwrap_or(Type::Number),
        effects: func_decl.effects.clone(),
        locals,
        instructions,
        is_tail_recursive,
    })
}

/// Detect if a function is tail-recursive
fn detect_tail_recursion(func_name: &str, body: &Expr) -> bool {
    match body {
        // Direct recursive call in tail position
        Expr::Call { func, .. } if func == func_name => true,

        // If expression - check both branches
        Expr::If { then_branch, else_branch, .. } => {
            let then_tail = detect_tail_recursion(func_name, then_branch);
            let else_tail = else_branch.as_ref()
                .map(|e| detect_tail_recursion(func_name, e))
                .unwrap_or(false);
            then_tail || else_tail
        }

        // Match expression - check all case bodies
        Expr::Match { cases, .. } => {
            cases.iter().any(|(_, case_body)| detect_tail_recursion(func_name, case_body))
        }

        // Block - check last expression
        Expr::Block(exprs) => {
            exprs.last()
                .map(|last| detect_tail_recursion(func_name, last))
                .unwrap_or(false)
        }

        // Respond wraps another expression
        Expr::Respond(inner) => detect_tail_recursion(func_name, inner),

        _ => false,
    }
}

fn lower_expr(
    expr: &Expr,
    instructions: &mut Vec<IrInstruction>,
    locals: &mut HashMap<String, Type>,
    temp_counter: &mut usize,
    current_func: &str,
) -> Result<String, MtpError> {
    match expr {
        Expr::Ident(name) => Ok(name.clone()),

        Expr::StringLit(s) => {
            let temp = format!("temp_{}", temp_counter);
            *temp_counter += 1;
            locals.insert(temp.clone(), Type::String);
            instructions.push(IrInstruction::LoadConst {
                value: IrValue::String(s.clone()),
                dest: temp.clone(),
            });
            Ok(temp)
        }

        Expr::NumberLit(n) => {
            let temp = format!("temp_{}", temp_counter);
            *temp_counter += 1;
            locals.insert(temp.clone(), Type::Number);
            let num = n.parse::<i64>().unwrap_or(0);
            instructions.push(IrInstruction::LoadConst {
                value: IrValue::Number(num),
                dest: temp.clone(),
            });
            Ok(temp)
        }

        Expr::BoolLit(b) => {
            let temp = format!("temp_{}", temp_counter);
            *temp_counter += 1;
            locals.insert(temp.clone(), Type::Boolean);
            instructions.push(IrInstruction::LoadConst {
                value: IrValue::Boolean(*b),
                dest: temp.clone(),
            });
            Ok(temp)
        }

        Expr::Array(elements) => {
            // Lower each element and create array
            let temp = format!("temp_{}", temp_counter);
            *temp_counter += 1;

            // Determine element type from first element or default to Number
            let elem_type = if elements.is_empty() {
                Type::Number
            } else {
                infer_expr_type(&elements[0])
            };
            locals.insert(temp.clone(), Type::Array(Box::new(elem_type)));

            // Lower each element
            let mut elem_vars = Vec::new();
            for elem in elements {
                let elem_var = lower_expr(elem, instructions, locals, temp_counter, current_func)?;
                elem_vars.push(elem_var);
            }

            // Create array instruction (using a call to internal array constructor)
            instructions.push(IrInstruction::Call {
                func: "__array_create".to_string(),
                args: elem_vars,
                dest: Some(temp.clone()),
            });

            Ok(temp)
        }

        Expr::Object(fields) => {
            // Lower each field value and create object
            let temp = format!("temp_{}", temp_counter);
            *temp_counter += 1;

            let mut field_types = HashMap::new();
            let mut field_vars = Vec::new();

            for (key, value) in fields {
                let value_var = lower_expr(value, instructions, locals, temp_counter, current_func)?;
                field_types.insert(key.clone(), infer_expr_type(value));

                // Pass key as string constant followed by value
                let key_temp = format!("temp_{}", temp_counter);
                *temp_counter += 1;
                locals.insert(key_temp.clone(), Type::String);
                instructions.push(IrInstruction::LoadConst {
                    value: IrValue::String(key.clone()),
                    dest: key_temp.clone(),
                });
                field_vars.push(key_temp);
                field_vars.push(value_var);
            }

            locals.insert(temp.clone(), Type::Object(field_types));

            // Create object instruction
            instructions.push(IrInstruction::Call {
                func: "__object_create".to_string(),
                args: field_vars,
                dest: Some(temp.clone()),
            });

            Ok(temp)
        }

        Expr::Call { func, args } => {
            let mut arg_vars = Vec::new();
            for arg in args {
                let arg_var = lower_expr(arg, instructions, locals, temp_counter, current_func)?;
                arg_vars.push(arg_var);
            }

            let dest = format!("temp_{}", temp_counter);
            *temp_counter += 1;
            locals.insert(dest.clone(), Type::Number); // Default return type

            instructions.push(IrInstruction::Call {
                func: func.clone(),
                args: arg_vars,
                dest: Some(dest.clone()),
            });
            Ok(dest)
        }

        Expr::Binary { op, left, right } => {
            // Handle pipe operator specially
            if matches!(op, BinOp::Pipe) {
                return lower_pipe_operator(left, right, instructions, locals, temp_counter, current_func);
            }

            let left_var = lower_expr(left, instructions, locals, temp_counter, current_func)?;
            let right_var = lower_expr(right, instructions, locals, temp_counter, current_func)?;
            let dest = format!("temp_{}", temp_counter);
            *temp_counter += 1;

            // Determine result type based on operator
            let result_type = match op {
                BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge |
                BinOp::And | BinOp::Or => Type::Boolean,
                _ => Type::Number,
            };
            locals.insert(dest.clone(), result_type);

            let bin_op = match op {
                BinOp::Add => BinOpKind::Add,
                BinOp::Sub => BinOpKind::Sub,
                BinOp::Mul => BinOpKind::Mul,
                BinOp::Div => BinOpKind::Div,
                BinOp::Eq => BinOpKind::Eq,
                BinOp::Ne => BinOpKind::Ne,
                BinOp::Lt => BinOpKind::Lt,
                BinOp::Le => BinOpKind::Le,
                BinOp::Gt => BinOpKind::Gt,
                BinOp::Ge => BinOpKind::Ge,
                BinOp::And => BinOpKind::And,
                BinOp::Or => BinOpKind::Or,
                BinOp::Pipe => unreachable!(), // Handled above
            };

            instructions.push(IrInstruction::BinOp {
                op: bin_op,
                left: left_var,
                right: right_var,
                dest: dest.clone(),
            });
            Ok(dest)
        }

        Expr::Unary { op, expr } => {
            let operand = lower_expr(expr, instructions, locals, temp_counter, current_func)?;
            let dest = format!("temp_{}", temp_counter);
            *temp_counter += 1;

            let result_type = match op {
                UnOp::Not => Type::Boolean,
                UnOp::Neg => Type::Number,
            };
            locals.insert(dest.clone(), result_type);

            let un_op = match op {
                UnOp::Not => UnOpKind::Not,
                UnOp::Neg => UnOpKind::Neg,
            };

            instructions.push(IrInstruction::UnOp {
                op: un_op,
                operand,
                dest: dest.clone(),
            });
            Ok(dest)
        }

        Expr::If { cond, then_branch, else_branch } => {
            let cond_var = lower_expr(cond, instructions, locals, temp_counter, current_func)?;
            let then_label = format!("then_{}", temp_counter);
            let else_label = format!("else_{}", temp_counter);
            let end_label = format!("end_{}", temp_counter);
            let result_var = format!("if_result_{}", temp_counter);
            *temp_counter += 1;

            locals.insert(result_var.clone(), Type::Number); // Will be unified later

            instructions.push(IrInstruction::JumpIf {
                condition: cond_var,
                true_label: then_label.clone(),
                false_label: else_label.clone(),
            });

            // Then branch
            instructions.push(IrInstruction::Label { name: then_label });
            let then_var = lower_expr(then_branch, instructions, locals, temp_counter, current_func)?;
            instructions.push(IrInstruction::StoreVar {
                src: then_var,
                dest: result_var.clone(),
            });
            instructions.push(IrInstruction::Jump { label: end_label.clone() });

            // Else branch
            instructions.push(IrInstruction::Label { name: else_label });
            if let Some(else_expr) = else_branch {
                let else_var = lower_expr(else_expr, instructions, locals, temp_counter, current_func)?;
                instructions.push(IrInstruction::StoreVar {
                    src: else_var,
                    dest: result_var.clone(),
                });
            } else {
                // Default to null
                let null_temp = format!("temp_{}", temp_counter);
                *temp_counter += 1;
                locals.insert(null_temp.clone(), Type::Number);
                instructions.push(IrInstruction::LoadConst {
                    value: IrValue::Null,
                    dest: null_temp.clone(),
                });
                instructions.push(IrInstruction::StoreVar {
                    src: null_temp,
                    dest: result_var.clone(),
                });
            }

            instructions.push(IrInstruction::Label { name: end_label });
            Ok(result_var)
        }

        Expr::Match { expr, cases } => {
            lower_match(expr, cases, instructions, locals, temp_counter, current_func)
        }

        Expr::Block(exprs) => {
            let mut result = None;
            for expr in exprs {
                result = Some(lower_expr(expr, instructions, locals, temp_counter, current_func)?);
            }
            result.ok_or_else(|| MtpError::ValidationError {
                error: "EmptyBlock".to_string(),
                message: "Block expression cannot be empty".to_string(),
            })
        }

        Expr::Lambda { params, body } => {
            // Create a closure/lambda
            let temp = format!("lambda_{}", temp_counter);
            *temp_counter += 1;

            // For now, create a nested function
            let param_types: Vec<Type> = params.iter().map(|_| Type::Number).collect();
            let func_type = Type::Function(param_types, Box::new(Type::Number));
            locals.insert(temp.clone(), func_type);

            // Lower lambda body in a new scope
            let mut lambda_locals = locals.clone();
            for param in params {
                lambda_locals.insert(param.clone(), Type::Number);
            }

            let mut lambda_instructions = Vec::new();
            let lambda_result = lower_expr(body, &mut lambda_instructions, &mut lambda_locals, temp_counter, &temp)?;
            lambda_instructions.push(IrInstruction::Return { value: Some(lambda_result) });

            // Store lambda as a function reference
            instructions.push(IrInstruction::LoadConst {
                value: IrValue::String(temp.clone()), // Reference to lambda
                dest: temp.clone(),
            });

            Ok(temp)
        }

        Expr::Respond(expr) => {
            let value = lower_expr(expr, instructions, locals, temp_counter, current_func)?;
            instructions.push(IrInstruction::Return { value: Some(value.clone()) });
            Ok(value)
        }
    }
}

/// Lower pipe operator: `x |> f` becomes `f(x)`
fn lower_pipe_operator(
    left: &Expr,
    right: &Expr,
    instructions: &mut Vec<IrInstruction>,
    locals: &mut HashMap<String, Type>,
    temp_counter: &mut usize,
    current_func: &str,
) -> Result<String, MtpError> {
    // Evaluate the left side (the value to pipe)
    let left_var = lower_expr(left, instructions, locals, temp_counter, current_func)?;

    // Right side should be a function or call
    match right {
        // Pipe to function identifier: `x |> f` becomes `f(x)`
        Expr::Ident(func_name) => {
            let dest = format!("temp_{}", temp_counter);
            *temp_counter += 1;
            locals.insert(dest.clone(), Type::Number);

            instructions.push(IrInstruction::Call {
                func: func_name.clone(),
                args: vec![left_var],
                dest: Some(dest.clone()),
            });
            Ok(dest)
        }

        // Pipe to partial call: `x |> f(y)` becomes `f(x, y)`
        Expr::Call { func, args } => {
            let mut all_args = vec![left_var];
            for arg in args {
                let arg_var = lower_expr(arg, instructions, locals, temp_counter, current_func)?;
                all_args.push(arg_var);
            }

            let dest = format!("temp_{}", temp_counter);
            *temp_counter += 1;
            locals.insert(dest.clone(), Type::Number);

            instructions.push(IrInstruction::Call {
                func: func.clone(),
                args: all_args,
                dest: Some(dest.clone()),
            });
            Ok(dest)
        }

        // Pipe to lambda: `x |> (a) => a + 1`
        Expr::Lambda { params, body } => {
            if params.is_empty() {
                return Err(MtpError::ValidationError {
                    error: "PipeError".to_string(),
                    message: "Lambda in pipe must have at least one parameter".to_string(),
                });
            }

            // Bind the piped value to the first parameter
            let param_name = &params[0];
            instructions.push(IrInstruction::StoreVar {
                src: left_var,
                dest: param_name.clone(),
            });
            locals.insert(param_name.clone(), Type::Number);

            // Evaluate the lambda body
            lower_expr(body, instructions, locals, temp_counter, current_func)
        }

        _ => Err(MtpError::ValidationError {
            error: "PipeError".to_string(),
            message: "Right side of pipe must be a function, call, or lambda".to_string(),
        }),
    }
}

fn lower_match(
    expr: &Expr,
    cases: &[(Pattern, Expr)],
    instructions: &mut Vec<IrInstruction>,
    locals: &mut HashMap<String, Type>,
    temp_counter: &mut usize,
    current_func: &str,
) -> Result<String, MtpError> {
    let value_var = lower_expr(expr, instructions, locals, temp_counter, current_func)?;
    let result_var = format!("match_result_{}", temp_counter);
    let end_label = format!("match_end_{}", temp_counter);
    *temp_counter += 1;
    locals.insert(result_var.clone(), Type::Number);

    // Generate code for each case
    for (i, (pattern, case_expr)) in cases.iter().enumerate() {
        let case_label = format!("case_{}_{}", temp_counter, i);
        let next_label = if i + 1 < cases.len() {
            format!("case_{}_{}", temp_counter, i + 1)
        } else {
            end_label.clone()
        };

        instructions.push(IrInstruction::Label { name: case_label.clone() });

        // Generate pattern matching code
        let (matches, bindings) = lower_pattern(pattern, &value_var, instructions, locals, temp_counter)?;

        // Apply bindings
        for (name, src) in bindings {
            locals.insert(name.clone(), Type::Number);
            instructions.push(IrInstruction::StoreVar {
                src,
                dest: name,
            });
        }

        if !matches {
            // Pattern might not match - generate conditional jump
            let cond_temp = format!("temp_{}", temp_counter);
            *temp_counter += 1;
            locals.insert(cond_temp.clone(), Type::Boolean);

            // For non-wildcard patterns, we'd generate actual comparison
            // For now, assume pattern always matches for simplicity
        }

        // Evaluate case body
        let case_result = lower_expr(case_expr, instructions, locals, temp_counter, current_func)?;
        instructions.push(IrInstruction::StoreVar {
            src: case_result,
            dest: result_var.clone(),
        });
        instructions.push(IrInstruction::Jump { label: end_label.clone() });
    }

    instructions.push(IrInstruction::Label { name: end_label });
    Ok(result_var)
}

/// Lower a pattern, returning whether it definitely matches and variable bindings
fn lower_pattern(
    pattern: &Pattern,
    value_var: &str,
    instructions: &mut Vec<IrInstruction>,
    locals: &mut HashMap<String, Type>,
    temp_counter: &mut usize,
) -> Result<(bool, Vec<(String, String)>), MtpError> {
    match pattern {
        Pattern::Wildcard => {
            // Wildcard always matches, no bindings
            Ok((true, vec![]))
        }

        Pattern::Ident(name) => {
            // Identifier pattern - binds the value to the name
            Ok((true, vec![(name.clone(), value_var.to_string())]))
        }

        Pattern::Lit(expr) => {
            // Literal pattern - compare value to literal
            let lit_temp = format!("temp_{}", temp_counter);
            *temp_counter += 1;

            match expr.as_ref() {
                Expr::NumberLit(n) => {
                    locals.insert(lit_temp.clone(), Type::Number);
                    let num = n.parse::<i64>().unwrap_or(0);
                    instructions.push(IrInstruction::LoadConst {
                        value: IrValue::Number(num),
                        dest: lit_temp.clone(),
                    });
                }
                Expr::StringLit(s) => {
                    locals.insert(lit_temp.clone(), Type::String);
                    instructions.push(IrInstruction::LoadConst {
                        value: IrValue::String(s.clone()),
                        dest: lit_temp.clone(),
                    });
                }
                Expr::BoolLit(b) => {
                    locals.insert(lit_temp.clone(), Type::Boolean);
                    instructions.push(IrInstruction::LoadConst {
                        value: IrValue::Boolean(*b),
                        dest: lit_temp.clone(),
                    });
                }
                _ => {
                    return Err(MtpError::ValidationError {
                        error: "PatternError".to_string(),
                        message: "Only literal values allowed in patterns".to_string(),
                    });
                }
            }

            // Generate comparison
            let cmp_temp = format!("temp_{}", temp_counter);
            *temp_counter += 1;
            locals.insert(cmp_temp.clone(), Type::Boolean);

            instructions.push(IrInstruction::BinOp {
                op: BinOpKind::Eq,
                left: value_var.to_string(),
                right: lit_temp,
                dest: cmp_temp,
            });

            Ok((false, vec![])) // Not definitely matching
        }

        Pattern::Variant(name, sub_patterns) => {
            // ADT variant pattern - check constructor and destructure
            let mut bindings = Vec::new();

            // Extract constructor tag check
            let tag_temp = format!("temp_{}", temp_counter);
            *temp_counter += 1;
            locals.insert(tag_temp.clone(), Type::String);

            instructions.push(IrInstruction::Call {
                func: "__get_variant_tag".to_string(),
                args: vec![value_var.to_string()],
                dest: Some(tag_temp.clone()),
            });

            // Compare tag
            let expected_tag = format!("temp_{}", temp_counter);
            *temp_counter += 1;
            locals.insert(expected_tag.clone(), Type::String);
            instructions.push(IrInstruction::LoadConst {
                value: IrValue::String(name.clone()),
                dest: expected_tag.clone(),
            });

            // For each sub-pattern, extract field and recursively match
            for (i, sub_pat) in sub_patterns.iter().enumerate() {
                let field_temp = format!("temp_{}", temp_counter);
                *temp_counter += 1;
                locals.insert(field_temp.clone(), Type::Number);

                // Extract field from variant
                let idx_temp = format!("temp_{}", temp_counter);
                *temp_counter += 1;
                locals.insert(idx_temp.clone(), Type::Number);
                instructions.push(IrInstruction::LoadConst {
                    value: IrValue::Number(i as i64),
                    dest: idx_temp.clone(),
                });

                instructions.push(IrInstruction::Call {
                    func: "__get_variant_field".to_string(),
                    args: vec![value_var.to_string(), idx_temp],
                    dest: Some(field_temp.clone()),
                });

                // Recursively process sub-pattern
                let (_, sub_bindings) = lower_pattern(sub_pat, &field_temp, instructions, locals, temp_counter)?;
                bindings.extend(sub_bindings);
            }

            Ok((false, bindings))
        }

        Pattern::Record(fields) => {
            // Record pattern - destructure fields
            let mut bindings = Vec::new();

            for (field_name, sub_pat) in fields {
                let field_temp = format!("temp_{}", temp_counter);
                *temp_counter += 1;
                locals.insert(field_temp.clone(), Type::Number);

                // Extract field from record
                let key_temp = format!("temp_{}", temp_counter);
                *temp_counter += 1;
                locals.insert(key_temp.clone(), Type::String);
                instructions.push(IrInstruction::LoadConst {
                    value: IrValue::String(field_name.clone()),
                    dest: key_temp.clone(),
                });

                instructions.push(IrInstruction::Call {
                    func: "__get_field".to_string(),
                    args: vec![value_var.to_string(), key_temp],
                    dest: Some(field_temp.clone()),
                });

                // Recursively process sub-pattern
                let (_, sub_bindings) = lower_pattern(sub_pat, &field_temp, instructions, locals, temp_counter)?;
                bindings.extend(sub_bindings);
            }

            Ok((false, bindings))
        }
    }
}

/// Infer the type of an expression (simplified)
fn infer_expr_type(expr: &Expr) -> Type {
    match expr {
        Expr::NumberLit(_) => Type::Number,
        Expr::StringLit(_) => Type::String,
        Expr::BoolLit(_) => Type::Boolean,
        Expr::Array(elements) => {
            let elem_type = elements.first()
                .map(|e| infer_expr_type(e))
                .unwrap_or(Type::Number);
            Type::Array(Box::new(elem_type))
        }
        Expr::Object(fields) => {
            let field_types: HashMap<String, Type> = fields.iter()
                .map(|(k, v)| (k.clone(), infer_expr_type(v)))
                .collect();
            Type::Object(field_types)
        }
        _ => Type::Number, // Default
    }
}

fn create_main_from_api(api_decl: &crate::parser::ast::ApiDecl) -> Result<IrFunction, MtpError> {
    let mut locals = HashMap::new();
    let mut instructions = Vec::new();
    let mut temp_counter = 0;

    // Lower the API body
    let result_var = lower_expr(
        &api_decl.body,
        &mut instructions,
        &mut locals,
        &mut temp_counter,
        "main",
    )?;

    instructions.push(IrInstruction::Return {
        value: Some(result_var),
    });

    Ok(IrFunction {
        name: "main".to_string(),
        params: vec![],
        return_type: Type::Number, // Default
        effects: api_decl.effects.clone(),
        locals,
        instructions,
        is_tail_recursive: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tail_recursion_detection() {
        // Direct recursive call
        let body = Expr::Call {
            func: "factorial".to_string(),
            args: vec![],
        };
        assert!(detect_tail_recursion("factorial", &body));

        // Non-recursive call
        let body = Expr::Call {
            func: "other_func".to_string(),
            args: vec![],
        };
        assert!(!detect_tail_recursion("factorial", &body));

        // Tail call in if branch
        let body = Expr::If {
            cond: Box::new(Expr::BoolLit(true)),
            then_branch: Box::new(Expr::Call {
                func: "factorial".to_string(),
                args: vec![],
            }),
            else_branch: Some(Box::new(Expr::NumberLit("1".to_string()))),
        };
        assert!(detect_tail_recursion("factorial", &body));
    }

    #[test]
    fn test_infer_expr_type() {
        assert_eq!(infer_expr_type(&Expr::NumberLit("42".to_string())), Type::Number);
        assert_eq!(infer_expr_type(&Expr::StringLit("hello".to_string())), Type::String);
        assert_eq!(infer_expr_type(&Expr::BoolLit(true)), Type::Boolean);
    }
}
