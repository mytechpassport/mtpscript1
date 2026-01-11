use crate::errors::compile::CompileError;
use crate::parser::ast::{Program, ModuleDecl, FuncDecl, ApiDecl, Expr};
use std::collections::HashSet;

const BUILT_IN_EFFECTS: &[&str] = &["DbRead", "DbWrite", "HttpOut", "Log", "Async"];

pub fn check_program_effects(program: &Program) -> Result<(), CompileError> {
    for decl in &program.decls {
        match decl {
            ModuleDecl::Func(func) => check_func_effects(func)?,
            ModuleDecl::Api(api) => check_api_effects(api)?,
            _ => {} // Other declarations don't have effects
        }
    }
    Ok(())
}

fn check_func_effects(func: &FuncDecl) -> Result<(), CompileError> {
    let declared_effects: HashSet<_> = func.effects.iter().cloned().collect();
    check_expr_effects(&func.body, &declared_effects, false)?;
    Ok(())
}

fn check_api_effects(api: &ApiDecl) -> Result<(), CompileError> {
    let declared_effects: HashSet<_> = api.effects.iter().cloned().collect();
    check_expr_effects(&api.body, &declared_effects, true)?;
    Ok(())
}

fn check_expr_effects(
    expr: &Expr,
    declared_effects: &HashSet<String>,
    allow_respond: bool,
) -> Result<(), CompileError> {
    match expr {
        Expr::Call { func, args } => {
            // Check if this is an effect call
            if let Expr::Ident(name) = func.as_ref() {
                if BUILT_IN_EFFECTS.contains(&name.as_str()) {
                    if !declared_effects.contains(name) {
                        return Err(CompileError::EffectNotDeclared {
                            effect: name.clone(),
                        });
                    }
                }
            }
            // Recursively check function and args
            check_expr_effects(func, declared_effects, allow_respond)?;
            for arg in args {
                check_expr_effects(arg, declared_effects, allow_respond)?;
            }
        }
        Expr::Await(inner) => {
            // Await requires Async effect
            if !declared_effects.contains("Async") {
                return Err(CompileError::AwaitWithoutAsync);
            }
            check_expr_effects(inner, declared_effects, allow_respond)?;
        }
        Expr::RespondJson(inner) => {
            if !allow_respond {
                return Err(CompileError::RespondOutsideApi);
            }
            check_expr_effects(inner, declared_effects, allow_respond)?;
        }
        Expr::Binary(_, left, right)
        | Expr::Pipeline(left, right) => {
            check_expr_effects(left, declared_effects, allow_respond)?;
            check_expr_effects(right, declared_effects, allow_respond)?;
        }
        Expr::Unary(_, inner)
        | Expr::Dot(inner, _)
        | Expr::Group(inner) => {
            check_expr_effects(inner, declared_effects, allow_respond)?;
        }
        Expr::Index(array, index) => {
            check_expr_effects(array, declared_effects, allow_respond)?;
            check_expr_effects(index, declared_effects, allow_respond)?;
        }
        Expr::If { condition, then_branch, else_branch } => {
            check_expr_effects(condition, declared_effects, allow_respond)?;
            check_expr_effects(then_branch, declared_effects, allow_respond)?;
            check_expr_effects(else_branch, declared_effects, allow_respond)?;
        }
        Expr::Match { expr: match_expr, cases } => {
            check_expr_effects(match_expr, declared_effects, allow_respond)?;
            for (_, case_expr) in cases {
                check_expr_effects(case_expr, declared_effects, allow_respond)?;
            }
        }
        Expr::Const { value, body, .. } => {
            check_expr_effects(value, declared_effects, allow_respond)?;
            check_expr_effects(body, declared_effects, allow_respond)?;
        }
        Expr::Lambda { body, .. } => {
            // Lambdas cannot use effects
            check_expr_effects(body, &HashSet::new(), false)?;
        }
        Expr::Array(elements) => {
            for elem in elements {
                check_expr_effects(elem, declared_effects, allow_respond)?;
            }
        }
        Expr::Object(fields) => {
            for (_, value) in fields {
                check_expr_effects(value, declared_effects, allow_respond)?;
            }
        }
        // Literals and identifiers don't use effects
        Expr::String(_) | Expr::Number(_) | Expr::Decimal(_) | Expr::Boolean(_) | Expr::Ident(_) => {}
    }
    Ok(())
}
