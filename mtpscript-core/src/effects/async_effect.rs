use crate::errors::compile::CompileError;
use crate::parser::ast::{Expr, ModuleDecl, Pattern, Program};
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

/// Deterministic serialization of expressions for promise hash
fn deterministic_expr_serialize(expr: &Expr) -> String {
    match expr {
        Expr::String(s) => format!("String({})", s),
        Expr::Number(n) => format!("Number({})", n),
        Expr::Decimal(d) => format!("Decimal({})", d),
        Expr::Boolean(b) => format!("Boolean({})", b),
        Expr::Ident(name) => format!("Ident({})", name),
        Expr::Array(elements) => {
            let mut s = "Array(".to_string();
            for (i, e) in elements.iter().enumerate() {
                if i > 0 {
                    s.push(',');
                }
                s.push_str(&deterministic_expr_serialize(e));
            }
            s.push(')');
            s
        }
        Expr::Object(fields) => {
            let mut s = "Object(".to_string();
            // Sort fields by key for determinism
            let mut sorted = fields.clone();
            sorted.sort_by(|a, b| a.0.cmp(&b.0));
            for (i, (k, v)) in sorted.iter().enumerate() {
                if i > 0 {
                    s.push(',');
                }
                s.push_str(&format!("{}:{}", k, deterministic_expr_serialize(v)));
            }
            s.push(')');
            s
        }
        Expr::Binary(op, left, right) => format!(
            "Binary({:?},{},{})",
            op,
            deterministic_expr_serialize(left),
            deterministic_expr_serialize(right)
        ),
        Expr::Unary(op, expr) => format!("Unary({:?},{})", op, deterministic_expr_serialize(expr)),
        Expr::Call { func, args } => {
            let mut s = format!("Call({}", deterministic_expr_serialize(func));
            for arg in args {
                s.push(',');
                s.push_str(&deterministic_expr_serialize(arg));
            }
            s.push(')');
            s
        }
        Expr::Dot(expr, field) => format!("Dot({},{}),", deterministic_expr_serialize(expr), field),
        Expr::Index(array, index) => format!(
            "Index({},{})",
            deterministic_expr_serialize(array),
            deterministic_expr_serialize(index)
        ),
        Expr::Pipeline(left, right) => format!(
            "Pipeline({},{})",
            deterministic_expr_serialize(left),
            deterministic_expr_serialize(right)
        ),
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => format!(
            "If({},{},{})",
            deterministic_expr_serialize(condition),
            deterministic_expr_serialize(then_branch),
            deterministic_expr_serialize(else_branch)
        ),
        Expr::Match {
            expr: match_expr,
            cases,
        } => {
            let mut s = format!("Match({}", deterministic_expr_serialize(match_expr));
            for (pat, body) in cases {
                s.push(',');
                s.push_str(&deterministic_pattern_serialize(pat));
                s.push(':');
                s.push_str(&deterministic_expr_serialize(body));
            }
            s.push(')');
            s
        }
        Expr::Const { name, value, body } => format!(
            "Const({},{},{})",
            name,
            deterministic_expr_serialize(value),
            deterministic_expr_serialize(body)
        ),
        Expr::Lambda { params, body } => {
            let param_str = params
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "Lambda([{}],{})",
                param_str,
                deterministic_expr_serialize(body)
            )
        }
        Expr::RespondJson(expr) => format!("RespondJson({})", deterministic_expr_serialize(expr)),
        Expr::Await(expr) => format!("Await({})", deterministic_expr_serialize(expr)),
        Expr::Group(expr) => format!("Group({})", deterministic_expr_serialize(expr)),
    }
}

fn deterministic_pattern_serialize(pat: &Pattern) -> String {
    match pat {
        Pattern::Wildcard => "Wildcard".to_string(),
        Pattern::Ident(name) => format!("Ident({})", name),
        Pattern::Literal(expr) => format!("Literal({})", deterministic_expr_serialize(expr)),
        Pattern::Variant(name, sub) => {
            let sub_str = sub
                .iter()
                .map(deterministic_pattern_serialize)
                .collect::<Vec<_>>()
                .join(",");
            format!("Variant({},[{}])", name, sub_str)
        }
        Pattern::Record(name, fields) => {
            let mut s = format!("Record({}", name);
            for (k, p) in fields {
                s.push(',');
                s.push_str(&format!("{}:{}", k, deterministic_pattern_serialize(p)));
            }
            s.push(')');
            s
        }
    }
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
            // For other expressions, use a deterministic serialization
            // to ensure promiseHash is deterministic
            data.extend(deterministic_expr_serialize(expr).as_bytes());
        }
    }
    Ok(data)
}
