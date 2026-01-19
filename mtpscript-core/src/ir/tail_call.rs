use crate::ir::nodes::{IrExpr, IrFunction};

/// Analyze IR functions for tail calls
pub fn analyze_tail_calls(functions: &mut Vec<IrFunction>) {
    for func in functions.iter_mut() {
        analyze_function_tail_calls(func);
    }
}

fn analyze_function_tail_calls(func: &mut IrFunction) {
    func.is_tail_recursive = is_tail_recursive_call(&func.body, &func.name);
}

fn is_tail_recursive_call(expr: &IrExpr, func_name: &str) -> bool {
    match expr {
        IrExpr::Call { func, .. } if matches!(**func, IrExpr::Var(ref name, _) if name == func_name) => {
            true
        }
        IrExpr::TailCall { func, .. } if matches!(**func, IrExpr::Var(ref name, _) if name == func_name) => {
            true
        }
        IrExpr::If {
            then_branch,
            else_branch,
            ..
        } => {
            // A function is tail-recursive if ANY branch contains a tail call to itself
            is_tail_recursive_call(then_branch, func_name)
                || is_tail_recursive_call(else_branch, func_name)
        }
        IrExpr::Match { cases, .. } => cases
            .iter()
            .any(|(_, body)| is_tail_recursive_call(body, func_name)),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::nodes::{IrExpr, IrFunction};
    use crate::parser::ast::BinOp;
    use crate::types::Type;
    use proptest::prelude::*;

    #[test]
    fn test_tail_recursive_factorial() {
        let mut func = IrFunction {
            name: "factorial".to_string(),
            params: vec![
                ("n".to_string(), Type::Number),
                ("acc".to_string(), Type::Number),
            ],
            return_type: Type::Number,
            effects: vec![],
            body: IrExpr::If {
                condition: Box::new(IrExpr::Var("n".to_string(), Type::Number)),
                then_branch: Box::new(IrExpr::Var("acc".to_string(), Type::Number)),
                else_branch: Box::new(IrExpr::Call {
                    func: Box::new(IrExpr::Var("factorial".to_string(), Type::Number)), // Simplified type
                    args: vec![
                        IrExpr::Binary(
                            BinOp::Sub,
                            Box::new(IrExpr::Var("n".to_string(), Type::Number)),
                            Box::new(IrExpr::Number(1, Type::Number)),
                            Type::Number,
                        ),
                        IrExpr::Binary(
                            BinOp::Mul,
                            Box::new(IrExpr::Var("n".to_string(), Type::Number)),
                            Box::new(IrExpr::Var("acc".to_string(), Type::Number)),
                            Type::Number,
                        ),
                    ],
                    result_type: Type::Number,
                }),
                result_type: Type::Number,
            },
            is_tail_recursive: false,
        };

        analyze_function_tail_calls(&mut func);
        // Now correctly identifies as tail-recursive because else branch has a tail call
        assert!(func.is_tail_recursive);
    }

    proptest! {
        #[test]
        fn tail_call_detection_complex_expressions(depth in 0..10usize) {
            // Generate a function with nested if/match expressions and check tail call detection
            let func_name = "test_func";
            let body = generate_nested_expr(depth, func_name);
            let mut func = IrFunction {
                name: func_name.to_string(),
                params: vec![],
                return_type: Type::Number,
                effects: vec![],
                body,
                is_tail_recursive: false,
            };

            analyze_function_tail_calls(&mut func);
            // The property is that it doesn't crash and produces a boolean result
            let _is_tail_recursive = func.is_tail_recursive;
        }
    }

    fn generate_nested_expr(depth: usize, func_name: &str) -> IrExpr {
        if depth == 0 {
            // Base case: direct tail call
            IrExpr::Call {
                func: Box::new(IrExpr::Var(func_name.to_string(), Type::Number)),
                args: vec![],
                result_type: Type::Number,
            }
        } else {
            // Recursive case: nested in if
            IrExpr::If {
                condition: Box::new(IrExpr::Number(1, Type::Number)),
                then_branch: Box::new(generate_nested_expr(depth - 1, func_name)),
                else_branch: Box::new(IrExpr::Number(0, Type::Number)),
                result_type: Type::Number,
            }
        }
    }

    #[test]
    fn test_non_tail_recursive_add_after_call() {
        // f(x) = f(x-1) + 1 is NOT tail recursive because there's computation after the call
        let mut func = IrFunction {
            name: "add_one".to_string(),
            params: vec![("x".to_string(), Type::Number)],
            return_type: Type::Number,
            effects: vec![],
            body: IrExpr::Binary(
                BinOp::Add,
                Box::new(IrExpr::Call {
                    func: Box::new(IrExpr::Var("add_one".to_string(), Type::Number)),
                    args: vec![IrExpr::Binary(
                        BinOp::Sub,
                        Box::new(IrExpr::Var("x".to_string(), Type::Number)),
                        Box::new(IrExpr::Number(1, Type::Number)),
                        Type::Number,
                    )],
                    result_type: Type::Number,
                }),
                Box::new(IrExpr::Number(1, Type::Number)),
                Type::Number,
            ),
            is_tail_recursive: false,
        };

        analyze_function_tail_calls(&mut func);
        // Not tail recursive because call is wrapped in Binary
        assert!(!func.is_tail_recursive);
    }

    #[test]
    fn test_non_tail_recursive_nested_call() {
        // g(f(x)) where f is the function being analyzed - not tail recursive
        let mut func = IrFunction {
            name: "inner".to_string(),
            params: vec![("x".to_string(), Type::Number)],
            return_type: Type::Number,
            effects: vec![],
            body: IrExpr::Call {
                func: Box::new(IrExpr::Var("outer".to_string(), Type::Number)),
                args: vec![IrExpr::Call {
                    func: Box::new(IrExpr::Var("inner".to_string(), Type::Number)),
                    args: vec![IrExpr::Var("x".to_string(), Type::Number)],
                    result_type: Type::Number,
                }],
                result_type: Type::Number,
            },
            is_tail_recursive: false,
        };

        analyze_function_tail_calls(&mut func);
        // Not tail recursive because the recursive call is an argument to another call
        assert!(!func.is_tail_recursive);
    }

    #[test]
    fn test_tail_recursive_match() {
        use crate::ir::nodes::IrPattern;
        // match x { 0 => base, n => f(n-1) } - IS tail recursive
        let mut func = IrFunction {
            name: "count".to_string(),
            params: vec![("x".to_string(), Type::Number)],
            return_type: Type::Number,
            effects: vec![],
            body: IrExpr::Match {
                expr: Box::new(IrExpr::Var("x".to_string(), Type::Number)),
                cases: vec![
                    (
                        IrPattern::Literal(IrExpr::Number(0, Type::Number)),
                        IrExpr::Number(0, Type::Number),
                    ),
                    (
                        IrPattern::Var("n".to_string()),
                        IrExpr::Call {
                            func: Box::new(IrExpr::Var("count".to_string(), Type::Number)),
                            args: vec![IrExpr::Binary(
                                BinOp::Sub,
                                Box::new(IrExpr::Var("n".to_string(), Type::Number)),
                                Box::new(IrExpr::Number(1, Type::Number)),
                                Type::Number,
                            )],
                            result_type: Type::Number,
                        },
                    ),
                ],
                result_type: Type::Number,
            },
            is_tail_recursive: false,
        };

        analyze_function_tail_calls(&mut func);
        assert!(func.is_tail_recursive);
    }

    #[test]
    fn test_non_recursive_function() {
        // A function that doesn't call itself at all
        let mut func = IrFunction {
            name: "identity".to_string(),
            params: vec![("x".to_string(), Type::Number)],
            return_type: Type::Number,
            effects: vec![],
            body: IrExpr::Var("x".to_string(), Type::Number),
            is_tail_recursive: false,
        };

        analyze_function_tail_calls(&mut func);
        assert!(!func.is_tail_recursive);
    }

    #[test]
    fn test_calls_different_function() {
        // A function that calls a different function in tail position
        let mut func = IrFunction {
            name: "wrapper".to_string(),
            params: vec![("x".to_string(), Type::Number)],
            return_type: Type::Number,
            effects: vec![],
            body: IrExpr::Call {
                func: Box::new(IrExpr::Var("other".to_string(), Type::Number)),
                args: vec![IrExpr::Var("x".to_string(), Type::Number)],
                result_type: Type::Number,
            },
            is_tail_recursive: false,
        };

        analyze_function_tail_calls(&mut func);
        // Not self-recursive
        assert!(!func.is_tail_recursive);
    }

    #[test]
    fn test_tail_recursive_both_branches() {
        // if cond then f(x) else f(y) - tail recursive in both branches
        let mut func = IrFunction {
            name: "branch_both".to_string(),
            params: vec![
                ("cond".to_string(), Type::Boolean),
                ("x".to_string(), Type::Number),
                ("y".to_string(), Type::Number),
            ],
            return_type: Type::Number,
            effects: vec![],
            body: IrExpr::If {
                condition: Box::new(IrExpr::Var("cond".to_string(), Type::Boolean)),
                then_branch: Box::new(IrExpr::Call {
                    func: Box::new(IrExpr::Var("branch_both".to_string(), Type::Number)),
                    args: vec![
                        IrExpr::Var("cond".to_string(), Type::Boolean),
                        IrExpr::Var("x".to_string(), Type::Number),
                        IrExpr::Var("y".to_string(), Type::Number),
                    ],
                    result_type: Type::Number,
                }),
                else_branch: Box::new(IrExpr::Call {
                    func: Box::new(IrExpr::Var("branch_both".to_string(), Type::Number)),
                    args: vec![
                        IrExpr::Var("cond".to_string(), Type::Boolean),
                        IrExpr::Var("x".to_string(), Type::Number),
                        IrExpr::Var("y".to_string(), Type::Number),
                    ],
                    result_type: Type::Number,
                }),
                result_type: Type::Number,
            },
            is_tail_recursive: false,
        };

        analyze_function_tail_calls(&mut func);
        assert!(func.is_tail_recursive);
    }

    #[test]
    fn test_tail_recursive_only_one_branch() {
        // if cond then f(x) else 0 - tail recursive in one branch is enough
        let mut func = IrFunction {
            name: "branch_one".to_string(),
            params: vec![
                ("cond".to_string(), Type::Boolean),
                ("x".to_string(), Type::Number),
            ],
            return_type: Type::Number,
            effects: vec![],
            body: IrExpr::If {
                condition: Box::new(IrExpr::Var("cond".to_string(), Type::Boolean)),
                then_branch: Box::new(IrExpr::Call {
                    func: Box::new(IrExpr::Var("branch_one".to_string(), Type::Number)),
                    args: vec![
                        IrExpr::Var("cond".to_string(), Type::Boolean),
                        IrExpr::Var("x".to_string(), Type::Number),
                    ],
                    result_type: Type::Number,
                }),
                else_branch: Box::new(IrExpr::Number(0, Type::Number)),
                result_type: Type::Number,
            },
            is_tail_recursive: false,
        };

        analyze_function_tail_calls(&mut func);
        assert!(func.is_tail_recursive);
    }

    #[test]
    fn test_deeply_nested_if_tail_call() {
        // Deeply nested: if c1 then (if c2 then (if c3 then f(x) else y) else z) else w
        let mut func = IrFunction {
            name: "deep".to_string(),
            params: vec![("x".to_string(), Type::Number)],
            return_type: Type::Number,
            effects: vec![],
            body: IrExpr::If {
                condition: Box::new(IrExpr::Boolean(true, Type::Boolean)),
                then_branch: Box::new(IrExpr::If {
                    condition: Box::new(IrExpr::Boolean(true, Type::Boolean)),
                    then_branch: Box::new(IrExpr::If {
                        condition: Box::new(IrExpr::Boolean(true, Type::Boolean)),
                        then_branch: Box::new(IrExpr::Call {
                            func: Box::new(IrExpr::Var("deep".to_string(), Type::Number)),
                            args: vec![IrExpr::Var("x".to_string(), Type::Number)],
                            result_type: Type::Number,
                        }),
                        else_branch: Box::new(IrExpr::Number(1, Type::Number)),
                        result_type: Type::Number,
                    }),
                    else_branch: Box::new(IrExpr::Number(2, Type::Number)),
                    result_type: Type::Number,
                }),
                else_branch: Box::new(IrExpr::Number(3, Type::Number)),
                result_type: Type::Number,
            },
            is_tail_recursive: false,
        };

        analyze_function_tail_calls(&mut func);
        assert!(func.is_tail_recursive);
    }

    #[test]
    fn test_explicit_tail_call_marker() {
        // Test that TailCall IR node is recognized
        let mut func = IrFunction {
            name: "explicit".to_string(),
            params: vec![("x".to_string(), Type::Number)],
            return_type: Type::Number,
            effects: vec![],
            body: IrExpr::TailCall {
                func: Box::new(IrExpr::Var("explicit".to_string(), Type::Number)),
                args: vec![IrExpr::Var("x".to_string(), Type::Number)],
                result_type: Type::Number,
            },
            is_tail_recursive: false,
        };

        analyze_function_tail_calls(&mut func);
        assert!(func.is_tail_recursive);
    }

    #[test]
    fn test_analyze_multiple_functions() {
        let mut functions = vec![
            IrFunction {
                name: "recursive".to_string(),
                params: vec![("x".to_string(), Type::Number)],
                return_type: Type::Number,
                effects: vec![],
                body: IrExpr::Call {
                    func: Box::new(IrExpr::Var("recursive".to_string(), Type::Number)),
                    args: vec![IrExpr::Var("x".to_string(), Type::Number)],
                    result_type: Type::Number,
                },
                is_tail_recursive: false,
            },
            IrFunction {
                name: "not_recursive".to_string(),
                params: vec![("x".to_string(), Type::Number)],
                return_type: Type::Number,
                effects: vec![],
                body: IrExpr::Var("x".to_string(), Type::Number),
                is_tail_recursive: false,
            },
        ];

        analyze_tail_calls(&mut functions);

        assert!(functions[0].is_tail_recursive);
        assert!(!functions[1].is_tail_recursive);
    }
}
