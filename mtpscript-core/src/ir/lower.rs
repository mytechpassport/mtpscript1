use super::nodes::{IrApi, IrDecl, IrExpr, IrFunction, IrPattern, IrProgram};
use crate::errors::compile::CompileError;
use crate::parser::ast::{
    ApiDecl, BinOp, Expr as AstExpr, FuncDecl, ModuleDecl, Pattern as AstPattern, Program, TypeExpr,
};
use crate::types::Type;

fn resolve_type_expr(type_expr: &TypeExpr) -> Type {
    match type_expr {
        TypeExpr::Ident(name) => match name.as_str() {
            "number" => Type::Number,
            "boolean" => Type::Boolean,
            "string" => Type::String,
            "Decimal" => Type::Decimal,
            "Json" => Type::Json,
            _ => Type::Var(name.clone()), // User-defined types
        },
        TypeExpr::Generic(base, args) => {
            if base == "Option" && args.len() == 1 {
                let inner = resolve_type_expr(&args[0]);
                Type::option(inner)
            } else if base == "Result" && args.len() == 2 {
                let ok = resolve_type_expr(&args[0]);
                let err = resolve_type_expr(&args[1]);
                Type::result(ok, err)
            } else {
                Type::Var(format!("{}<...>", base))
            }
        }
    }
}

/// Lower AST to IR. Assumes type checking has been performed.
/// In a real implementation, this would take typed AST.
pub fn lower_ast_to_ir(ast: &Program) -> Result<IrProgram, CompileError> {
    let mut decls = Vec::new();

    for ast_decl in &ast.decls {
        match ast_decl {
            ModuleDecl::Func(func) => {
                let ir_func = lower_func(func)?;
                decls.push(IrDecl::Function(ir_func));
            }
            ModuleDecl::Api(api) => {
                let ir_api = lower_api(api)?;
                decls.push(IrDecl::Api(ir_api));
            }
            ModuleDecl::Type(_) | ModuleDecl::Import(_) => {
                // Skip for now - types and imports don't generate IR
            }
        }
    }

    Ok(IrProgram { decls })
}

fn lower_func(func: &FuncDecl) -> Result<IrFunction, CompileError> {
    let params: Vec<(String, Type)> = func
        .params
        .iter()
        .map(|(name, type_expr)| (name.clone(), resolve_type_expr(type_expr)))
        .collect();
    // Infer return type from body, but since body not lowered, use placeholder
    let return_type = Type::Var("return".to_string());

    let body = lower_expr_with_tail(&func.body, &return_type, true)?;
    let is_tail_recursive = detect_tail_calls(&body, &func.name);

    Ok(IrFunction {
        name: func.name.clone(),
        params,
        return_type,
        effects: func.effects.clone(),
        body,
        is_tail_recursive,
    })
}

fn lower_api(api: &ApiDecl) -> Result<IrApi, CompileError> {
    let body = lower_expr_with_tail(&api.body, &Type::Var("unknown".to_string()), true)?;

    Ok(IrApi {
        method: api.method.clone(),
        path: api.path.clone(),
        effects: api.effects.clone(),
        body,
    })
}

fn lower_expr(ast_expr: &AstExpr, expected_type: &Type) -> Result<IrExpr, CompileError> {
    lower_expr_with_tail(ast_expr, expected_type, false)
}

fn lower_expr_with_tail(
    ast_expr: &AstExpr,
    expected_type: &Type,
    is_tail: bool,
) -> Result<IrExpr, CompileError> {
    match ast_expr {
        AstExpr::String(s) => Ok(IrExpr::String(s.clone(), Type::String)),
        AstExpr::Number(n) => Ok(IrExpr::Number(*n, Type::Number)),
        AstExpr::Decimal(d) => Ok(IrExpr::Decimal(d.clone(), Type::Decimal)),
        AstExpr::Boolean(b) => Ok(IrExpr::Boolean(*b, Type::Boolean)),
        AstExpr::Ident(name) => Ok(IrExpr::Var(name.clone(), expected_type.clone())),

        AstExpr::Array(elements) => {
            let ir_elements = elements
                .iter()
                .map(|e| lower_expr_with_tail(e, &Type::Var("elem".to_string()), false))
                .collect::<Result<_, _>>()?;
            Ok(IrExpr::Array(ir_elements, Type::Var("array".to_string())))
        }

        AstExpr::Object(fields) => {
            let ir_fields = fields
                .iter()
                .map(|(k, v)| {
                    Ok((
                        k.clone(),
                        lower_expr_with_tail(v, &Type::Var("field".to_string()), false)?,
                    ))
                })
                .collect::<Result<_, _>>()?;
            Ok(IrExpr::Object(ir_fields, Type::Var("object".to_string())))
        }

        AstExpr::Dot(expr, field) => {
            let ir_expr = lower_expr_with_tail(expr, &Type::Var("receiver".to_string()), false)?;
            Ok(IrExpr::Dot(
                Box::new(ir_expr),
                field.clone(),
                expected_type.clone(),
            ))
        }

        AstExpr::Index(array, index) => {
            let ir_array = lower_expr_with_tail(array, &Type::Var("array".to_string()), false)?;
            let ir_index = lower_expr_with_tail(index, &Type::Number, false)?;
            Ok(IrExpr::Index(
                Box::new(ir_array),
                Box::new(ir_index),
                expected_type.clone(),
            ))
        }

        AstExpr::Call { func, args } => {
            // Check if this is an effect call
            if let AstExpr::Ident(name) = func.as_ref() {
                if is_effect_function(name) {
                    let ir_args = args
                        .iter()
                        .map(|a| lower_expr_with_tail(a, &Type::Var("arg".to_string()), false))
                        .collect::<Result<_, _>>()?;
                    return Ok(IrExpr::EffectCall(
                        name.clone(),
                        ir_args,
                        expected_type.clone(),
                    ));
                }
            }

            let ir_func = lower_expr_with_tail(func, &Type::Var("func".to_string()), false)?;
            let ir_args = args
                .iter()
                .map(|a| lower_expr_with_tail(a, &Type::Var("arg".to_string()), false))
                .collect::<Result<_, _>>()?;
            if is_tail {
                Ok(IrExpr::TailCall {
                    func: Box::new(ir_func),
                    args: ir_args,
                    result_type: expected_type.clone(),
                })
            } else {
                Ok(IrExpr::Call {
                    func: Box::new(ir_func),
                    args: ir_args,
                    result_type: expected_type.clone(),
                })
            }
        }

        AstExpr::Unary(op, expr) => {
            let expr_expected = match op {
                BinOp::Add | BinOp::Sub => Type::Number, // + and - unary
                BinOp::Not => Type::Boolean,
                _ => Type::Var("unary".to_string()),
            };
            let result_type = match op {
                BinOp::Add | BinOp::Sub => Type::Number,
                BinOp::Not => Type::Boolean,
                _ => expected_type.clone(),
            };
            let ir_expr = lower_expr_with_tail(expr, &expr_expected, false)?;
            Ok(IrExpr::Unary(op.clone(), Box::new(ir_expr), result_type))
        }

        AstExpr::Binary(op, left, right) => {
            let (left_expected, right_expected, result_type) = match op {
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                    (Type::Number, Type::Number, Type::Number)
                }
                BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                    // Assume same type for comparison
                    let t = Type::Var("comparable".to_string());
                    (t.clone(), t.clone(), Type::Boolean)
                }
                BinOp::And | BinOp::Or => (Type::Boolean, Type::Boolean, Type::Boolean),
                BinOp::Not => unreachable!("Not is unary"),
            };
            let ir_left = lower_expr_with_tail(left, &left_expected, false)?;
            let ir_right = lower_expr_with_tail(right, &right_expected, false)?;
            Ok(IrExpr::Binary(
                op.clone(),
                Box::new(ir_left),
                Box::new(ir_right),
                result_type,
            ))
        }

        AstExpr::Pipeline(left, right) => {
            // Desugar: a |> f ≡ f(a)
            // So left becomes the argument to right
            let ir_right = lower_expr_with_tail(right, &Type::Var("func".to_string()), false)?;
            let ir_left = lower_expr_with_tail(left, &Type::Var("arg".to_string()), false)?;
            if is_tail {
                Ok(IrExpr::TailCall {
                    func: Box::new(ir_right),
                    args: vec![ir_left],
                    result_type: expected_type.clone(),
                })
            } else {
                Ok(IrExpr::Call {
                    func: Box::new(ir_right),
                    args: vec![ir_left],
                    result_type: expected_type.clone(),
                })
            }
        }

        AstExpr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let ir_condition = lower_expr_with_tail(condition, &Type::Boolean, false)?;
            let ir_then = lower_expr_with_tail(then_branch, expected_type, is_tail)?;
            let ir_else = lower_expr_with_tail(else_branch, expected_type, is_tail)?;
            Ok(IrExpr::If {
                condition: Box::new(ir_condition),
                then_branch: Box::new(ir_then),
                else_branch: Box::new(ir_else),
                result_type: expected_type.clone(),
            })
        }

        AstExpr::Match { expr, cases } => {
            let ir_expr = lower_expr_with_tail(expr, &Type::Var("match".to_string()), false)?;
            let ir_cases = cases
                .iter()
                .map(|(pat, expr)| {
                    Ok((
                        lower_pattern(pat)?,
                        lower_expr_with_tail(expr, expected_type, is_tail)?,
                    ))
                })
                .collect::<Result<_, _>>()?;
            Ok(IrExpr::Match {
                expr: Box::new(ir_expr),
                cases: ir_cases,
                result_type: expected_type.clone(),
            })
        }

        AstExpr::Const { name, value, body } => {
            let ir_value = lower_expr_with_tail(value, &Type::Var("const".to_string()), false)?;
            let ir_body = lower_expr_with_tail(body, expected_type, is_tail)?;
            Ok(IrExpr::Let {
                name: name.clone(),
                value: Box::new(ir_value),
                body: Box::new(ir_body),
                result_type: expected_type.clone(),
            })
        }

        AstExpr::Lambda { params, body } => {
            // Lower lambda to IR lambda
            let ir_body = lower_expr_with_tail(body, expected_type, false)?;
            Ok(IrExpr::Lambda {
                params: params.iter().map(|(name, _)| name.clone()).collect(),
                body: Box::new(ir_body),
                result_type: expected_type.clone(),
            })
        }

        AstExpr::RespondJson(expr) => {
            let ir_expr = lower_expr_with_tail(expr, &Type::Var("json".to_string()), false)?;
            Ok(IrExpr::RespondJson(
                Box::new(ir_expr),
                expected_type.clone(),
            ))
        }

        AstExpr::Group(expr) => lower_expr_with_tail(expr, expected_type, is_tail),

        AstExpr::Await(_) => {
            // Await should have been desugared by async effect pass
            Err(CompileError::ParserError(
                "Await found in IR lowering - should have been desugared".to_string(),
            ))
        }
    }
}

fn lower_pattern(ast_pattern: &AstPattern) -> Result<IrPattern, CompileError> {
    match ast_pattern {
        AstPattern::Wildcard => Ok(IrPattern::Wildcard),
        AstPattern::Ident(name) => Ok(IrPattern::Var(name.clone())),
        AstPattern::Literal(expr) => {
            let ir_expr = lower_expr(expr, &Type::Var("literal".to_string()))?;
            Ok(IrPattern::Literal(ir_expr))
        }
        AstPattern::Variant(name, sub_patterns) => {
            let ir_sub = sub_patterns
                .iter()
                .map(lower_pattern)
                .collect::<Result<_, _>>()?;
            Ok(IrPattern::Variant(name.clone(), ir_sub))
        }
        AstPattern::Record(name, fields) => {
            let ir_fields = fields
                .iter()
                .map(|(k, p)| Ok((k.clone(), lower_pattern(p)?)))
                .collect::<Result<_, _>>()?;
            Ok(IrPattern::Record(name.clone(), ir_fields))
        }
    }
}

fn is_effect_function(name: &str) -> bool {
    matches!(
        name,
        "DbRead" | "DbWrite" | "HttpOut" | "Log" | "Async" | "Async.await"
    )
}

fn detect_tail_calls(expr: &IrExpr, func_name: &str) -> bool {
    match expr {
        IrExpr::TailCall { func, .. } => {
            if let IrExpr::Var(name, _) = func.as_ref() {
                name == func_name
            } else {
                false
            }
        }
        IrExpr::If {
            then_branch,
            else_branch,
            ..
        } => detect_tail_calls(then_branch, func_name) || detect_tail_calls(else_branch, func_name),
        IrExpr::Match { cases, .. } => cases
            .iter()
            .any(|(_, expr)| detect_tail_calls(expr, func_name)),
        IrExpr::Let { body, .. } => detect_tail_calls(body, func_name),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::{BinOp, Expr, FuncDecl, Program, TypeExpr};

    #[test]
    fn test_lower_pipeline() {
        // Test pipeline desugaring: a |> f ≡ f(a)
        let ast = Expr::Pipeline(
            Box::new(Expr::Number(42)),
            Box::new(Expr::Ident("double".to_string())),
        );
        let ir = lower_expr(&ast, &Type::Number).unwrap();

        match ir {
            IrExpr::Call { func, args, .. } => {
                assert_eq!(args.len(), 1);
                assert!(matches!(*func, IrExpr::Var(ref name, _) if name == "double"));
                assert!(matches!(args[0], IrExpr::Number(42, _)));
            }
            _ => panic!("Expected Call"),
        }
    }

    #[test]
    fn test_lower_binary_op() {
        let ast = Expr::Binary(
            BinOp::Add,
            Box::new(Expr::Number(1)),
            Box::new(Expr::Number(2)),
        );
        let ir = lower_expr(&ast, &Type::Number).unwrap();

        match ir {
            IrExpr::Binary(op, left, right, _) => {
                assert_eq!(op, BinOp::Add);
                assert!(matches!(*left, IrExpr::Number(1, _)));
                assert!(matches!(*right, IrExpr::Number(2, _)));
            }
            _ => panic!("Expected Binary"),
        }
    }

    #[test]
    fn test_lower_function() {
        let func = FuncDecl {
            name: "add".to_string(),
            params: vec![
                ("a".to_string(), TypeExpr::Ident("number".to_string())),
                ("b".to_string(), TypeExpr::Ident("number".to_string())),
            ],
            effects: vec![],
            body: Expr::Binary(
                BinOp::Add,
                Box::new(Expr::Ident("a".to_string())),
                Box::new(Expr::Ident("b".to_string())),
            ),
        };

        let ir_func = lower_func(&func).unwrap();
        assert_eq!(ir_func.name, "add");
        assert_eq!(ir_func.params.len(), 2);
        assert_eq!(ir_func.effects.len(), 0);

        match ir_func.body {
            IrExpr::Binary(op, left, right, _) => {
                assert_eq!(op, BinOp::Add);
                assert!(matches!(*left, IrExpr::Var(ref name, _) if name == "a"));
                assert!(matches!(*right, IrExpr::Var(ref name, _) if name == "b"));
            }
            _ => panic!("Expected Binary in body"),
        }
    }

    #[test]
    fn test_equivalence_pipeline_desugaring() {
        // Ensure a |> f is equivalent to f(a)
        let pipeline_ast = Expr::Pipeline(
            Box::new(Expr::Number(10)),
            Box::new(Expr::Ident("inc".to_string())),
        );

        let direct_call_ast = Expr::Call {
            func: Box::new(Expr::Ident("inc".to_string())),
            args: vec![Expr::Number(10)],
        };

        let pipeline_ir = lower_expr(&pipeline_ast, &Type::Number).unwrap();
        let direct_ir = lower_expr(&direct_call_ast, &Type::Number).unwrap();

        // Both should produce the same IR structure
        match (&pipeline_ir, &direct_ir) {
            (
                IrExpr::Call {
                    func: pf, args: pa, ..
                },
                IrExpr::Call {
                    func: df, args: da, ..
                },
            ) => {
                assert_eq!(pf, df);
                assert_eq!(pa, da);
            }
            _ => panic!("Both should be Call expressions"),
        }
    }
}
