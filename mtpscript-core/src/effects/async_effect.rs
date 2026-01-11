use crate::errors::compile::CompileError;
use crate::parser::ast::{Expr, ModuleDecl, Program};
use sha2::{Digest, Sha256};

/// Desugars await expressions to Async.await calls
pub fn desugar_async_effects(program: &mut Program) -> Result<(), CompileError> {
    let mut cont_id_counter = 0u32;
    for decl in &mut program.decls {
        match decl {
            ModuleDecl::Func(func) => {
                desugar_expr_async(&mut func.body, &mut cont_id_counter)?;
            }
            ModuleDecl::Api(api) => {
                desugar_expr_async(&mut api.body, &mut cont_id_counter)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn desugar_expr_async(expr: &mut Expr, cont_id_counter: &mut u32) -> Result<(), CompileError> {
    match expr {
        Expr::Await(awaited_expr) => {
            // Compute promiseHash: SHA-256 of CBOR-encoded expression
            let cbor_data = cbor_encode_expr(awaited_expr)?;
            let promise_hash = Sha256::digest(&cbor_data);
            let promise_hash_hex = hex::encode(promise_hash);

            // Generate unique contId
            let cont_id = *cont_id_counter;
            *cont_id_counter += 1;

            // Desugar to: Async.await(promiseHash, contId, awaited_expr)
            *expr = Expr::Call {
                func: Box::new(Expr::Ident("Async.await".to_string())),
                args: vec![
                    Expr::String(promise_hash_hex),
                    Expr::Number(cont_id as i64),
                    (**awaited_expr).clone(), // Clone the inner expression
                ],
            };
        }
        Expr::Call { func, args } => {
            desugar_expr_async(func, cont_id_counter)?;
            for arg in args {
                desugar_expr_async(arg, cont_id_counter)?;
            }
        }
        Expr::Binary(_, left, right) | Expr::Pipeline(left, right) => {
            desugar_expr_async(left, cont_id_counter)?;
            desugar_expr_async(right, cont_id_counter)?;
        }
        Expr::Unary(_, inner) | Expr::Dot(inner, _) | Expr::Group(inner) => {
            desugar_expr_async(inner, cont_id_counter)?;
        }
        Expr::Index(array, index) => {
            desugar_expr_async(array, cont_id_counter)?;
            desugar_expr_async(index, cont_id_counter)?;
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            desugar_expr_async(condition, cont_id_counter)?;
            desugar_expr_async(then_branch, cont_id_counter)?;
            desugar_expr_async(else_branch, cont_id_counter)?;
        }
        Expr::Match {
            expr: match_expr,
            cases,
        } => {
            desugar_expr_async(match_expr, cont_id_counter)?;
            for (_, case_expr) in cases {
                desugar_expr_async(case_expr, cont_id_counter)?;
            }
        }
        Expr::Const { value, body, .. } => {
            desugar_expr_async(value, cont_id_counter)?;
            desugar_expr_async(body, cont_id_counter)?;
        }
        Expr::Lambda { body, .. } => {
            desugar_expr_async(body, cont_id_counter)?;
        }
        Expr::Array(elements) => {
            for elem in elements {
                desugar_expr_async(elem, cont_id_counter)?;
            }
        }
        Expr::Object(fields) => {
            for (_, value) in fields {
                desugar_expr_async(value, cont_id_counter)?;
            }
        }
        Expr::RespondJson(inner) => {
            desugar_expr_async(inner, cont_id_counter)?;
        }
        // Literals don't need desugaring
        Expr::String(_)
        | Expr::Number(_)
        | Expr::Decimal(_)
        | Expr::Boolean(_)
        | Expr::Ident(_) => {}
    }
    Ok(())
}

/// Simple CBOR encoding of expressions for promise hash computation
/// This is a simplified implementation - real CBOR would be more complex
fn cbor_encode_expr(expr: &Expr) -> Result<Vec<u8>, CompileError> {
    let mut data = Vec::new();
    match expr {
        Expr::String(s) => {
            data.push(0x60 | (s.len() as u8)); // text string
            data.extend_from_slice(s.as_bytes());
        }
        Expr::Number(n) => {
            data.push(0x00); // positive integer
            data.extend_from_slice(&n.to_be_bytes());
        }
        Expr::Boolean(true) => data.push(0xF5),  // true
        Expr::Boolean(false) => data.push(0xF4), // false
        Expr::Ident(name) => {
            data.push(0x60 | (name.len() as u8)); // text string
            data.extend_from_slice(name.as_bytes());
        }
        Expr::Call { func, args } => {
            data.push(0x80 | (args.len() as u8 + 1) as u8); // array
            let func_data = cbor_encode_expr(func)?;
            data.extend(func_data);
            for arg in args {
                let arg_data = cbor_encode_expr(arg)?;
                data.extend(arg_data);
            }
        }
        _ => {
            // For other expressions, use a simple representation
            // Real implementation would need full CBOR encoding
            data.extend(format!("{:?}", expr).as_bytes());
        }
    }
    Ok(data)
}
