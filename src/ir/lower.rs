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
            let ir_func = lower_function(func_decl)?;
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

fn lower_function(func_decl: &FuncDecl) -> Result<IrFunction, MtpError> {
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
    )?;

    // Add return instruction
    instructions.push(IrInstruction::Return {
        value: Some(result_var),
    });

    Ok(IrFunction {
        name: func_decl.name.clone(),
        params: func_decl
            .params
            .iter()
            .map(|(name, ty)| (name.clone(), ty.clone()))
            .collect(),
        return_type: func_decl.return_type.clone().unwrap_or(Type::Number), // Default for now
        effects: func_decl.effects.clone(),
        locals,
        instructions,
        is_tail_recursive: false, // TODO: detect tail recursion
    })
}

fn lower_expr(
    expr: &Expr,
    instructions: &mut Vec<IrInstruction>,
    locals: &mut HashMap<String, Type>,
    temp_counter: &mut usize,
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
            // TODO: Implement array lowering
            let temp = format!("temp_{}", temp_counter);
            *temp_counter += 1;
            locals.insert(temp.clone(), Type::Array(Box::new(Type::Number))); // Default element type
            Ok(temp)
        }
        Expr::Object(fields) => {
            // TODO: Implement object lowering
            let temp = format!("temp_{}", temp_counter);
            *temp_counter += 1;
            locals.insert(temp.clone(), Type::Object(HashMap::new()));
            Ok(temp)
        }
        Expr::Call { func, args } => {
            let mut arg_vars = Vec::new();
            for arg in args {
                let arg_var = lower_expr(arg, instructions, locals, temp_counter)?;
                arg_vars.push(arg_var);
            }

            let dest = format!("temp_{}", temp_counter);
            *temp_counter += 1;
            locals.insert(dest.clone(), Type::Number); // Default return type

            // Special handling for built-in functions
            if func == "Json.parse" || func == "Json.stringify" {
                instructions.push(IrInstruction::Call {
                    func: func.clone(),
                    args: arg_vars,
                    dest: Some(dest.clone()),
                });
            } else {
                instructions.push(IrInstruction::Call {
                    func: func.clone(),
                    args: arg_vars,
                    dest: Some(dest.clone()),
                });
            }
            Ok(dest)
        }
        Expr::Binary { op, left, right } => {
            let left_var = lower_expr(left, instructions, locals, temp_counter)?;
            let right_var = lower_expr(right, instructions, locals, temp_counter)?;
            let dest = format!("temp_{}", temp_counter);
            *temp_counter += 1;
            locals.insert(dest.clone(), Type::Number); // Assume number for now

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
                BinOp::Pipe => {
                    return Err(MtpError::ValidationError {
                        error: "NotImplemented".to_string(),
                        message: "Pipe operator not implemented".to_string(),
                    })
                }
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
            let operand = lower_expr(expr, instructions, locals, temp_counter)?;
            let dest = format!("temp_{}", temp_counter);
            *temp_counter += 1;
            locals.insert(dest.clone(), Type::Number);

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
        Expr::If {
            cond,
            then_branch,
            else_branch,
        } => {
            let cond_var = lower_expr(cond, instructions, locals, temp_counter)?;
            let then_label = format!("then_{}", temp_counter);
            let else_label = format!("else_{}", temp_counter);
            let end_label = format!("end_{}", temp_counter);
            *temp_counter += 1;

            instructions.push(IrInstruction::JumpIf {
                condition: cond_var,
                true_label: then_label.clone(),
                false_label: else_label.clone(),
            });

            instructions.push(IrInstruction::Label { name: then_label });
            let then_var = lower_expr(then_branch, instructions, locals, temp_counter)?;
            instructions.push(IrInstruction::Jump {
                label: end_label.clone(),
            });

            instructions.push(IrInstruction::Label { name: else_label });
            let else_var = if let Some(else_expr) = else_branch {
                lower_expr(else_expr, instructions, locals, temp_counter)?
            } else {
                // Default to null/undefined
                let temp = format!("temp_{}", temp_counter);
                *temp_counter += 1;
                locals.insert(temp.clone(), Type::Number);
                instructions.push(IrInstruction::LoadConst {
                    value: IrValue::Null,
                    dest: temp.clone(),
                });
                temp
            };

            instructions.push(IrInstruction::Label { name: end_label });

            // TODO: Phi node or proper result merging
            Ok(then_var) // Simplified - return then branch result
        }
        Expr::Match { expr, cases } => lower_match(expr, cases, instructions, locals, temp_counter),
        Expr::Block(exprs) => {
            let mut result = None;
            for expr in exprs {
                result = Some(lower_expr(expr, instructions, locals, temp_counter)?);
            }
            result.ok_or_else(|| MtpError::ValidationError {
                error: "EmptyBlock".to_string(),
                message: "Block expression cannot be empty".to_string(),
            })
        }
        Expr::Lambda { params, body } => {
            // Create a lambda function
            let lambda_body = lower_expr(body, instructions, locals, temp_counter)?;
            let temp = format!("temp_{}", temp_counter);
            *temp_counter += 1;
            locals.insert(temp.clone(), Type::Number); // Placeholder

            // For now, just store the lambda body result
            instructions.push(IrInstruction::StoreVar {
                src: lambda_body,
                dest: temp.clone(),
            });
            Ok(temp)
        }
        Expr::Respond(expr) => {
            let value = lower_expr(expr, instructions, locals, temp_counter)?;
            instructions.push(IrInstruction::Return { value: Some(value) });
            let temp = format!("temp_{}", temp_counter);
            *temp_counter += 1;
            locals.insert(temp.clone(), Type::Number);
            Ok(temp)
        }
    }
}

fn lower_match(
    expr: &Expr,
    cases: &[(Pattern, Expr)],
    instructions: &mut Vec<IrInstruction>,
    locals: &mut HashMap<String, Type>,
    temp_counter: &mut usize,
) -> Result<String, MtpError> {
    let value_var = lower_expr(expr, instructions, locals, temp_counter)?;
    let result_var = format!("match_result_{}", temp_counter);
    *temp_counter += 1;
    locals.insert(result_var.clone(), Type::Number); // Default type

    // For now, simplified match - just take first case
    if let Some((_, case_expr)) = cases.first() {
        let case_result = lower_expr(case_expr, instructions, locals, temp_counter)?;
        instructions.push(IrInstruction::StoreVar {
            src: case_result,
            dest: result_var.clone(),
        });
    }

    Ok(result_var)
}
    }

    instructions.push(IrInstruction::Label { name: end_label });
    Ok(result_var)
}

fn matches_pattern(
    pattern: &Pattern,
    value_var: &str,
    instructions: &mut Vec<IrInstruction>,
    locals: &mut HashMap<String, Type>,
    temp_counter: &mut usize,
) -> Result<bool, MtpError> {
    match pattern {
        Pattern::Wildcard => Ok(true),
        Pattern::Ident(name) => {
            // Bind variable
            locals.insert(name.clone(), Type::Number); // Default type
            instructions.push(IrInstruction::StoreVar {
                src: value_var.to_string(),
                dest: name.clone(),
            });
            Ok(true)
        }
        Pattern::Lit(expr) => {
            let lit_var = lower_expr(expr, instructions, locals, temp_counter)?;
            let temp = format!("temp_{}", temp_counter);
            *temp_counter += 1;
            locals.insert(temp.clone(), Type::Boolean);
            instructions.push(IrInstruction::BinOp {
                op: BinOpKind::Eq,
                left: value_var.to_string(),
                right: lit_var,
                dest: temp.clone(),
            });
            // TODO: Return the comparison result properly
            Ok(true)
        }
        Pattern::Variant(name, sub_patterns) => {
            // TODO: Implement ADT constructor pattern matching
            // For now, assume it matches
            Ok(true)
        }
        Pattern::Record(fields) => {
            // TODO: Implement record pattern matching
            Ok(true)
        }
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
