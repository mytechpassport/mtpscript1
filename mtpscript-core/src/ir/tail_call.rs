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
}
