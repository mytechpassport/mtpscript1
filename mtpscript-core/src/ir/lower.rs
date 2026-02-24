use super::nodes::{IrAdtType, IrAdtVariant, IrApi, IrDecl, IrExpr, IrFunction, IrPattern, IrProgram};
use crate::errors::compile::CompileError;
use crate::parser::ast::{
    ApiDecl, BinOp, Expr as AstExpr, FuncDecl, ModuleDecl, Pattern as AstPattern, Program, TypeDecl, TypeExpr,
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
    let mut adt_types = Vec::new();

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
            ModuleDecl::Type(type_decl) => {
                // Extract ADT types for constructor generation
                if let TypeDecl::Adt { name, variants, .. } = type_decl {
                    let ir_variants: Vec<IrAdtVariant> = variants
                        .iter()
                        .map(|v| IrAdtVariant {
                            name: v.name.clone(),
                            arity: v.payload.len(),
                        })
                        .collect();
                    adt_types.push(IrAdtType {
                        name: name.clone(),
                        variants: ir_variants,
                    });
                }
            }
            ModuleDecl::Import(_) => {
                // Imports don't generate IR
            }
        }
    }

    Ok(IrProgram { decls, adt_types })
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
            // Lower lambda to IR lambda, preserving type annotations
            let ir_body = lower_expr_with_tail(body, expected_type, false)?;
            let ir_params: Vec<(String, Type)> = params
                .iter()
                .map(|(name, type_expr)| (name.clone(), resolve_type_expr(type_expr)))
                .collect();
            Ok(IrExpr::Lambda {
                params: ir_params,
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

        AstExpr::Block(exprs) => {
            // Lower block as a sequence of lets with dummy names, final expr is the result
            if exprs.is_empty() {
                return Ok(IrExpr::Boolean(true, Type::Boolean));
            }
            if exprs.len() == 1 {
                return lower_expr_with_tail(&exprs[0], expected_type, is_tail);
            }

            // Convert block to nested lets
            // For each non-final expression, wrap in a let with a dummy name
            let mut result = lower_expr_with_tail(exprs.last().unwrap(), expected_type, is_tail)?;
            for (i, expr) in exprs.iter().rev().skip(1).enumerate() {
                let ir_expr = lower_expr_with_tail(expr, &Type::Var("stmt".to_string()), false)?;
                result = IrExpr::Let {
                    name: format!("__block_stmt_{}", i),
                    value: Box::new(ir_expr),
                    body: Box::new(result),
                    result_type: expected_type.clone(),
                };
            }
            Ok(result)
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
        AstPattern::Ident(name) => {
            // Convention: uppercase identifiers in patterns are variant constructors
            // This matches ML-family language conventions where constructors start with uppercase
            if name.chars().next().map(|c| c.is_ascii_uppercase()).unwrap_or(false) {
                // Treat as unit variant pattern (e.g., Red, Green, None)
                Ok(IrPattern::Variant(name.clone(), vec![]))
            } else {
                // Lowercase identifiers are variable bindings
                Ok(IrPattern::Var(name.clone()))
            }
        }
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

    #[test]
    fn test_lower_literals() {
        let string_ir = lower_expr(&Expr::String("hello".to_string()), &Type::String).unwrap();
        assert!(matches!(string_ir, IrExpr::String(ref s, _) if s == "hello"));

        let number_ir = lower_expr(&Expr::Number(42), &Type::Number).unwrap();
        assert!(matches!(number_ir, IrExpr::Number(42, _)));

        let decimal_ir = lower_expr(&Expr::Decimal("3.14".to_string()), &Type::Decimal).unwrap();
        assert!(matches!(decimal_ir, IrExpr::Decimal(ref d, _) if d == "3.14"));

        let bool_ir = lower_expr(&Expr::Boolean(true), &Type::Boolean).unwrap();
        assert!(matches!(bool_ir, IrExpr::Boolean(true, _)));
    }

    #[test]
    fn test_lower_ident() {
        let ir = lower_expr(&Expr::Ident("x".to_string()), &Type::Number).unwrap();
        assert!(
            matches!(ir, IrExpr::Var(ref name, ref typ) if name == "x" && *typ == Type::Number)
        );
    }

    #[test]
    fn test_lower_array() {
        let ast = Expr::Array(vec![Expr::Number(1), Expr::Number(2)]);
        let ir = lower_expr(&ast, &Type::Var("array".to_string())).unwrap();
        match ir {
            IrExpr::Array(elements, _) => {
                assert_eq!(elements.len(), 2);
                assert!(matches!(elements[0], IrExpr::Number(1, _)));
                assert!(matches!(elements[1], IrExpr::Number(2, _)));
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_lower_object() {
        let ast = Expr::Object(vec![
            ("key1".to_string(), Expr::String("value1".to_string())),
            ("key2".to_string(), Expr::Number(42)),
        ]);
        let ir = lower_expr(&ast, &Type::Var("object".to_string())).unwrap();
        match ir {
            IrExpr::Object(fields, _) => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "key1");
                assert!(matches!(fields[0].1, IrExpr::String(ref s, _) if s == "value1"));
                assert_eq!(fields[1].0, "key2");
                assert!(matches!(fields[1].1, IrExpr::Number(42, _)));
            }
            _ => panic!("Expected Object"),
        }
    }

    #[test]
    fn test_lower_dot() {
        let ast = Expr::Dot(
            Box::new(Expr::Ident("obj".to_string())),
            "field".to_string(),
        );
        let ir = lower_expr(&ast, &Type::String).unwrap();
        match ir {
            IrExpr::Dot(expr, field, _) => {
                assert!(matches!(*expr, IrExpr::Var(ref name, _) if name == "obj"));
                assert_eq!(field, "field");
            }
            _ => panic!("Expected Dot"),
        }
    }

    #[test]
    fn test_lower_index() {
        let ast = Expr::Index(
            Box::new(Expr::Ident("arr".to_string())),
            Box::new(Expr::Number(0)),
        );
        let ir = lower_expr(&ast, &Type::Number).unwrap();
        match ir {
            IrExpr::Index(array, index, _) => {
                assert!(matches!(*array, IrExpr::Var(ref name, _) if name == "arr"));
                assert!(matches!(*index, IrExpr::Number(0, _)));
            }
            _ => panic!("Expected Index"),
        }
    }

    #[test]
    fn test_lower_call() {
        let ast = Expr::Call {
            func: Box::new(Expr::Ident("add".to_string())),
            args: vec![Expr::Number(1), Expr::Number(2)],
        };
        let ir = lower_expr(&ast, &Type::Number).unwrap();
        match ir {
            IrExpr::Call { func, args, .. } => {
                assert!(matches!(*func, IrExpr::Var(ref name, _) if name == "add"));
                assert_eq!(args.len(), 2);
                assert!(matches!(args[0], IrExpr::Number(1, _)));
                assert!(matches!(args[1], IrExpr::Number(2, _)));
            }
            _ => panic!("Expected Call"),
        }
    }

    #[test]
    fn test_lower_effect_call() {
        let ast = Expr::Call {
            func: Box::new(Expr::Ident("DbRead".to_string())),
            args: vec![Expr::String("key".to_string())],
        };
        let ir = lower_expr(&ast, &Type::Var("result".to_string())).unwrap();
        match ir {
            IrExpr::EffectCall(name, args, _) => {
                assert_eq!(name, "DbRead");
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0], IrExpr::String(ref s, _) if s == "key"));
            }
            _ => panic!("Expected EffectCall"),
        }
    }

    #[test]
    fn test_lower_unary() {
        let ast = Expr::Unary(BinOp::Not, Box::new(Expr::Boolean(false)));
        let ir = lower_expr(&ast, &Type::Boolean).unwrap();
        match ir {
            IrExpr::Unary(op, expr, _) => {
                assert_eq!(op, BinOp::Not);
                assert!(matches!(*expr, IrExpr::Boolean(false, _)));
            }
            _ => panic!("Expected Unary"),
        }
    }

    #[test]
    fn test_lower_if() {
        let ast = Expr::If {
            condition: Box::new(Expr::Boolean(true)),
            then_branch: Box::new(Expr::Number(1)),
            else_branch: Box::new(Expr::Number(2)),
        };
        let ir = lower_expr(&ast, &Type::Number).unwrap();
        match ir {
            IrExpr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                assert!(matches!(*condition, IrExpr::Boolean(true, _)));
                assert!(matches!(*then_branch, IrExpr::Number(1, _)));
                assert!(matches!(*else_branch, IrExpr::Number(2, _)));
            }
            _ => panic!("Expected If"),
        }
    }

    #[test]
    fn test_lower_match() {
        let ast = Expr::Match {
            expr: Box::new(Expr::Number(42)),
            cases: vec![
                (
                    AstPattern::Literal(Expr::Number(42)),
                    Expr::String("matched".to_string()),
                ),
                (AstPattern::Wildcard, Expr::String("default".to_string())),
            ],
        };
        let ir = lower_expr(&ast, &Type::String).unwrap();
        match ir {
            IrExpr::Match { expr, cases, .. } => {
                assert!(matches!(*expr, IrExpr::Number(42, _)));
                assert_eq!(cases.len(), 2);
                // Further checks can be added for patterns
            }
            _ => panic!("Expected Match"),
        }
    }

    #[test]
    fn test_lower_const() {
        let ast = Expr::Const {
            name: "x".to_string(),
            value: Box::new(Expr::Number(10)),
            body: Box::new(Expr::Ident("x".to_string())),
        };
        let ir = lower_expr(&ast, &Type::Number).unwrap();
        match ir {
            IrExpr::Let {
                name, value, body, ..
            } => {
                assert_eq!(name, "x");
                assert!(matches!(*value, IrExpr::Number(10, _)));
                assert!(matches!(*body, IrExpr::Var(ref n, _) if n == "x"));
            }
            _ => panic!("Expected Let"),
        }
    }

    #[test]
    fn test_lower_lambda() {
        let ast = Expr::Lambda {
            params: vec![("x".to_string(), TypeExpr::Ident("number".to_string()))],
            body: Box::new(Expr::Ident("x".to_string())),
        };
        let ir = lower_expr(&ast, &Type::Var("func".to_string())).unwrap();
        match ir {
            IrExpr::Lambda { params, body, .. } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].0, "x");
                assert_eq!(params[0].1, Type::Number); // Type annotation preserved
                assert!(matches!(*body, IrExpr::Var(ref n, _) if n == "x"));
            }
            _ => panic!("Expected Lambda"),
        }
    }

    #[test]
    fn test_lower_respond_json() {
        let ast = Expr::RespondJson(Box::new(Expr::String("data".to_string())));
        let ir = lower_expr(&ast, &Type::Var("response".to_string())).unwrap();
        match ir {
            IrExpr::RespondJson(expr, _) => {
                assert!(matches!(*expr, IrExpr::String(ref s, _) if s == "data"));
            }
            _ => panic!("Expected RespondJson"),
        }
    }

    #[test]
    fn test_resolve_type_expr() {
        assert_eq!(
            resolve_type_expr(&TypeExpr::Ident("number".to_string())),
            Type::Number
        );
        assert_eq!(
            resolve_type_expr(&TypeExpr::Ident("boolean".to_string())),
            Type::Boolean
        );
        assert_eq!(
            resolve_type_expr(&TypeExpr::Ident("string".to_string())),
            Type::String
        );
        assert_eq!(
            resolve_type_expr(&TypeExpr::Ident("Decimal".to_string())),
            Type::Decimal
        );
        assert_eq!(
            resolve_type_expr(&TypeExpr::Ident("Json".to_string())),
            Type::Json
        );
        assert_eq!(
            resolve_type_expr(&TypeExpr::Ident("Custom".to_string())),
            Type::Var("Custom".to_string())
        );
    }

    #[test]
    fn test_detect_tail_calls() {
        let tail_call = IrExpr::TailCall {
            func: Box::new(IrExpr::Var(
                "factorial".to_string(),
                Type::Var("func".to_string()),
            )),
            args: vec![IrExpr::Number(5, Type::Number)],
            result_type: Type::Number,
        };
        assert!(detect_tail_calls(&tail_call, "factorial"));

        let non_tail = IrExpr::Call {
            func: Box::new(IrExpr::Var(
                "factorial".to_string(),
                Type::Var("func".to_string()),
            )),
            args: vec![IrExpr::Number(5, Type::Number)],
            result_type: Type::Number,
        };
        assert!(!detect_tail_calls(&non_tail, "factorial"));
    }

    #[test]
    fn test_nested_pipeline() {
        // Test chained pipeline: x |> f |> g ≡ g(f(x))
        let ast = Expr::Pipeline(
            Box::new(Expr::Pipeline(
                Box::new(Expr::Number(5)),
                Box::new(Expr::Ident("double".to_string())),
            )),
            Box::new(Expr::Ident("inc".to_string())),
        );
        let ir = lower_expr(&ast, &Type::Number).unwrap();

        // Should be inc(double(5))
        match ir {
            IrExpr::Call { func, args, .. } => {
                assert!(matches!(*func, IrExpr::Var(ref name, _) if name == "inc"));
                assert_eq!(args.len(), 1);
                // First arg should be double(5)
                match &args[0] {
                    IrExpr::Call {
                        func: inner_func,
                        args: inner_args,
                        ..
                    } => {
                        assert!(matches!(**inner_func, IrExpr::Var(ref name, _) if name == "double"));
                        assert_eq!(inner_args.len(), 1);
                        assert!(matches!(inner_args[0], IrExpr::Number(5, _)));
                    }
                    _ => panic!("Expected inner Call"),
                }
            }
            _ => panic!("Expected outer Call"),
        }
    }

    #[test]
    fn test_deeply_nested_arithmetic() {
        // Test (1 + 2) * (3 + 4)
        let ast = Expr::Binary(
            BinOp::Mul,
            Box::new(Expr::Binary(
                BinOp::Add,
                Box::new(Expr::Number(1)),
                Box::new(Expr::Number(2)),
            )),
            Box::new(Expr::Binary(
                BinOp::Add,
                Box::new(Expr::Number(3)),
                Box::new(Expr::Number(4)),
            )),
        );
        let ir = lower_expr(&ast, &Type::Number).unwrap();

        match ir {
            IrExpr::Binary(op, left, right, _) => {
                assert_eq!(op, BinOp::Mul);
                // Left should be 1 + 2
                match *left {
                    IrExpr::Binary(inner_op, inner_left, inner_right, _) => {
                        assert_eq!(inner_op, BinOp::Add);
                        assert!(matches!(*inner_left, IrExpr::Number(1, _)));
                        assert!(matches!(*inner_right, IrExpr::Number(2, _)));
                    }
                    _ => panic!("Expected left Binary"),
                }
                // Right should be 3 + 4
                match *right {
                    IrExpr::Binary(inner_op, inner_left, inner_right, _) => {
                        assert_eq!(inner_op, BinOp::Add);
                        assert!(matches!(*inner_left, IrExpr::Number(3, _)));
                        assert!(matches!(*inner_right, IrExpr::Number(4, _)));
                    }
                    _ => panic!("Expected right Binary"),
                }
            }
            _ => panic!("Expected Binary"),
        }
    }

    #[test]
    fn test_lower_nested_const() {
        // Test nested let bindings: const x = 1; const y = 2; x + y
        let ast = Expr::Const {
            name: "x".to_string(),
            value: Box::new(Expr::Number(1)),
            body: Box::new(Expr::Const {
                name: "y".to_string(),
                value: Box::new(Expr::Number(2)),
                body: Box::new(Expr::Binary(
                    BinOp::Add,
                    Box::new(Expr::Ident("x".to_string())),
                    Box::new(Expr::Ident("y".to_string())),
                )),
            }),
        };
        let ir = lower_expr(&ast, &Type::Number).unwrap();

        match ir {
            IrExpr::Let {
                name, value, body, ..
            } => {
                assert_eq!(name, "x");
                assert!(matches!(*value, IrExpr::Number(1, _)));
                // Body should be another let
                match *body {
                    IrExpr::Let {
                        name: inner_name,
                        value: inner_value,
                        body: inner_body,
                        ..
                    } => {
                        assert_eq!(inner_name, "y");
                        assert!(matches!(*inner_value, IrExpr::Number(2, _)));
                        // Inner body should be x + y
                        match *inner_body {
                            IrExpr::Binary(op, left, right, _) => {
                                assert_eq!(op, BinOp::Add);
                                assert!(matches!(*left, IrExpr::Var(ref n, _) if n == "x"));
                                assert!(matches!(*right, IrExpr::Var(ref n, _) if n == "y"));
                            }
                            _ => panic!("Expected Binary in inner body"),
                        }
                    }
                    _ => panic!("Expected inner Let"),
                }
            }
            _ => panic!("Expected Let"),
        }
    }

    #[test]
    fn test_lambda_multiple_params_with_types() {
        // Test lambda with multiple typed parameters
        let ast = Expr::Lambda {
            params: vec![
                ("a".to_string(), TypeExpr::Ident("number".to_string())),
                ("b".to_string(), TypeExpr::Ident("string".to_string())),
                ("c".to_string(), TypeExpr::Ident("boolean".to_string())),
            ],
            body: Box::new(Expr::Ident("a".to_string())),
        };
        let ir = lower_expr(&ast, &Type::Var("func".to_string())).unwrap();

        match ir {
            IrExpr::Lambda { params, body, .. } => {
                assert_eq!(params.len(), 3);
                assert_eq!(params[0], ("a".to_string(), Type::Number));
                assert_eq!(params[1], ("b".to_string(), Type::String));
                assert_eq!(params[2], ("c".to_string(), Type::Boolean));
                assert!(matches!(*body, IrExpr::Var(ref n, _) if n == "a"));
            }
            _ => panic!("Expected Lambda"),
        }
    }

    #[test]
    fn test_call_with_complex_args() {
        // Test function call with expressions as arguments
        let ast = Expr::Call {
            func: Box::new(Expr::Ident("f".to_string())),
            args: vec![
                Expr::Binary(BinOp::Add, Box::new(Expr::Number(1)), Box::new(Expr::Number(2))),
                Expr::String("hello".to_string()),
                Expr::Boolean(true),
            ],
        };
        let ir = lower_expr(&ast, &Type::Number).unwrap();

        match ir {
            IrExpr::Call { func, args, .. } => {
                assert!(matches!(*func, IrExpr::Var(ref n, _) if n == "f"));
                assert_eq!(args.len(), 3);
                // First arg should be 1 + 2
                assert!(matches!(&args[0], IrExpr::Binary(BinOp::Add, _, _, _)));
                assert!(matches!(&args[1], IrExpr::String(s, _) if s == "hello"));
                assert!(matches!(&args[2], IrExpr::Boolean(true, _)));
            }
            _ => panic!("Expected Call"),
        }
    }

    #[test]
    fn test_pattern_variable_binding() {
        // Test that pattern matching with variable captures works
        let ast = Expr::Match {
            expr: Box::new(Expr::Number(42)),
            cases: vec![
                (AstPattern::Ident("n".to_string()), Expr::Ident("n".to_string())),
            ],
        };
        let ir = lower_expr(&ast, &Type::Number).unwrap();

        match ir {
            IrExpr::Match { expr, cases, .. } => {
                assert!(matches!(*expr, IrExpr::Number(42, _)));
                assert_eq!(cases.len(), 1);
                // The variable pattern should create a binding
                match &cases[0].0 {
                    IrPattern::Var(name) => assert_eq!(name, "n"),
                    _ => panic!("Expected variable pattern"),
                }
            }
            _ => panic!("Expected Match"),
        }
    }

    #[test]
    fn test_if_with_complex_branches() {
        // Test if with complex expressions in branches
        let ast = Expr::If {
            condition: Box::new(Expr::Binary(
                BinOp::Gt,
                Box::new(Expr::Ident("x".to_string())),
                Box::new(Expr::Number(0)),
            )),
            then_branch: Box::new(Expr::Binary(
                BinOp::Mul,
                Box::new(Expr::Ident("x".to_string())),
                Box::new(Expr::Number(2)),
            )),
            else_branch: Box::new(Expr::Unary(BinOp::Sub, Box::new(Expr::Ident("x".to_string())))),
        };
        let ir = lower_expr(&ast, &Type::Number).unwrap();

        match ir {
            IrExpr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                // Condition should be x > 0
                assert!(matches!(*condition, IrExpr::Binary(BinOp::Gt, _, _, _)));
                // Then should be x * 2
                assert!(matches!(*then_branch, IrExpr::Binary(BinOp::Mul, _, _, _)));
                // Else should be unary minus x
                assert!(matches!(*else_branch, IrExpr::Unary(BinOp::Sub, _, _)));
            }
            _ => panic!("Expected If"),
        }
    }
}
