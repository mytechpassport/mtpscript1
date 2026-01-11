use crate::errors::runtime::RuntimeError;
use crate::runtime::interpreter::{Interpreter, JsExpr, StoredFunction};
use crc32fast;

// Placeholder for snapshot verification - would verify signature, etc.
fn verify_snapshot(snapshot: &[u8]) -> Result<(), RuntimeError> {
    if snapshot.len() < 52 {
        return Err(RuntimeError::ValueError("Snapshot too small".to_string()));
    }

    // Check magic bytes
    if &snapshot[0..8] != b"MTPJS\x00\x00\x00" {
        return Err(RuntimeError::ValueError("Invalid magic bytes".to_string()));
    }

    // Check version (51 for v5.1)
    let version = u32::from_le_bytes(snapshot[8..12].try_into().unwrap());
    if version != 51 {
        return Err(RuntimeError::ValueError("Unsupported version".to_string()));
    }

    // Verify CRC32
    if snapshot.len() < 4 {
        return Err(RuntimeError::ValueError(
            "Snapshot too small for CRC".to_string(),
        ));
    }
    let crc_start = snapshot.len() - 4;
    let expected_crc = u32::from_le_bytes(snapshot[crc_start..].try_into().unwrap());
    let content = &snapshot[..crc_start];
    let computed_crc = crc32fast::hash(content);
    if computed_crc != expected_crc {
        return Err(RuntimeError::ValueError(
            "CRC32 verification failed".to_string(),
        ));
    }

    Ok(())
}

// Extract JS code from snapshot
fn extract_js_code(snapshot: &[u8]) -> Result<String, RuntimeError> {
    let size = u64::from_le_bytes(snapshot[12..20].try_into().unwrap()) as usize;
    if snapshot.len() != size {
        return Err(RuntimeError::ValueError(
            "Snapshot size mismatch".to_string(),
        ));
    }

    let js_start = 52;
    let js_end = size - 132; // Before signature
    let js_bytes = &snapshot[js_start..js_end];

    String::from_utf8(js_bytes.to_vec())
        .map_err(|_| RuntimeError::ValueError("Invalid UTF-8 in JS code".to_string()))
}

pub fn clone_interpreter(snapshot: &[u8]) -> Result<Interpreter, RuntimeError> {
    // Verify snapshot
    verify_snapshot(snapshot)?;

    // Extract JS code
    let js_code = extract_js_code(snapshot)?;

    // Parse JS code into AST
    let ast = parse_js_to_ast(&js_code)?;

    // Create fresh interpreter
    let mut interp = Interpreter::new();

    // Initialize with parsed AST
    populate_interpreter_from_ast(&mut interp, &ast)?;

    // Mark as containing potentially PCI data
    interp.pci_touched = true;

    Ok(interp)
}

// Parse JS to AST
fn parse_js_to_ast(js: &str) -> Result<crate::runtime::interpreter::JsExpr, RuntimeError> {
    // Forbidden constructs
    let forbidden = [
        "class",
        "this",
        "eval",
        "try",
        "catch",
        "new",
        "prototype",
        "arguments",
    ];
    for word in forbidden {
        if js.contains(word) {
            return Err(RuntimeError::ValueError(format!(
                "Forbidden JS construct: {}",
                word
            )));
        }
    }

    let ast = crate::runtime::js_parser::parse_js_program(js)?;

    // Must contain at least one function
    if !js.contains("function") {
        return Err(RuntimeError::ValueError(
            "JS code must contain function declarations".to_string(),
        ));
    }

    // Basic structure check: ensure balanced braces and parens
    let mut brace_count = 0;
    let mut paren_count = 0;
    for c in js.chars() {
        match c {
            '{' => brace_count += 1,
            '}' => brace_count -= 1,
            '(' => paren_count += 1,
            ')' => paren_count -= 1,
            _ => {}
        }
        if brace_count < 0 || paren_count < 0 {
            return Err(RuntimeError::ValueError(
                "Unbalanced braces or parentheses".to_string(),
            ));
        }
    }
    if brace_count != 0 || paren_count != 0 {
        return Err(RuntimeError::ValueError(
            "Unbalanced braces or parentheses".to_string(),
        ));
    }

    Ok(ast)
}

/// Populate interpreter's function_bodies from the parsed AST
fn populate_interpreter_from_ast(
    interp: &mut Interpreter,
    ast: &JsExpr,
) -> Result<(), RuntimeError> {
    match ast {
        JsExpr::Program(statements) => {
            for stmt in statements {
                populate_from_stmt(interp, stmt)?;
            }
        }
        _ => {
            return Err(RuntimeError::ValueError(
                "AST root must be a Program".to_string(),
            ))
        }
    }
    Ok(())
}

fn populate_from_stmt(interp: &mut Interpreter, stmt: &JsExpr) -> Result<(), RuntimeError> {
    match stmt {
        JsExpr::FunctionDecl { name, params, body } => {
            interp.function_bodies.insert(
                name.clone(),
                StoredFunction {
                    params: params.clone(),
                    body: body.clone(),
                },
            );
        }
        JsExpr::Const { name, value } => {
            // For const declarations, we could pre-evaluate them, but for now just store as global
            // This is simplified; in full impl, might need to eval
            let _ = name; // placeholder
            let _ = value;
        }
        JsExpr::ExprStmt(expr) => {
            // For expression statements, could eval, but skip for now
            let _ = expr;
        }
        _ => {} // Other statements ignored for now
    }
    Ok(())
}
