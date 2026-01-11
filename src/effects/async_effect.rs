use crate::errors::MtpError;
use crate::parser::ast::Expr;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Represents an async effect call
#[derive(Debug, Clone)]
pub struct AsyncCall {
    pub promise_hash: Vec<u8>,
    pub cont_id: String,
    pub effect_args: Vec<String>,
}

impl AsyncCall {
    /// Create a new async call with deterministic promise hash
    pub fn new(expr: &Expr, cont_id: String, effect_args: Vec<String>) -> Result<Self, MtpError> {
        let promise_hash = compute_promise_hash(expr)?;

        Ok(AsyncCall {
            promise_hash,
            cont_id,
            effect_args,
        })
    }
}

/// Compute deterministic promise hash from expression
pub fn compute_promise_hash(expr: &Expr) -> Result<Vec<u8>, MtpError> {
    let serialized = serialize_expr_deterministically(expr)?;
    let hash = Sha256::digest(&serialized);
    Ok(hash.to_vec())
}

/// Serialize expression to a deterministic string representation
fn serialize_expr_deterministically(expr: &Expr) -> Result<String, MtpError> {
    match expr {
        Expr::Ident(name) => Ok(format!("ident:{}", name)),
        Expr::StringLit(s) => Ok(format!("string:{}", s)),
        Expr::NumberLit(n) => Ok(format!("number:{}", n)),
        Expr::BoolLit(b) => Ok(format!("bool:{}", b)),
        Expr::Call { func, args } => {
            let mut parts = vec![format!("call:{}", func)];
            for arg in args {
                parts.push(serialize_expr_deterministically(arg)?);
            }
            Ok(parts.join("|"))
        }
        Expr::BinOp { op, left, right } => Ok(format!(
            "binop:{}|{}|{}",
            op,
            serialize_expr_deterministically(left)?,
            serialize_expr_deterministically(right)?
        )),
        // Add more cases as needed for other expression types
        _ => Err(MtpError::CompileError(
            "Unsupported expression type in async hash".into(),
        )),
    }
}

/// Cache for async responses by (seed, cont_id)
pub struct AsyncCache {
    cache: HashMap<(Vec<u8>, String), String>,
}

impl AsyncCache {
    pub fn new() -> Self {
        AsyncCache {
            cache: HashMap::new(),
        }
    }

    pub fn get(&self, seed: &[u8], cont_id: &str) -> Option<&String> {
        self.cache.get(&(seed.to_vec(), cont_id.to_string()))
    }

    pub fn insert(&mut self, seed: &[u8], cont_id: String, response: String) {
        self.cache.insert((seed.to_vec(), cont_id), response);
    }
}
