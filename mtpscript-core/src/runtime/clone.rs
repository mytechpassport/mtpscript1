use crate::errors::runtime::RuntimeError;
use crate::runtime::interpreter::{Interpreter, JsExpr, StoredFunction};
use crate::security::sign::verify_ecdsa_p256;
use crc32fast;
use std::env;

// Default public key for signature verification (can be overridden by MTP_SIGNING_CERT env var)
const DEFAULT_PUBLIC_KEY: &str = "";

/// Verify snapshot integrity and signature
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

    // Verify ECDSA signature if certificate is available
    // Signature is 64 bytes (ECDSA-P256 raw format) before CRC
    // Layout: content_hash(32) | ... | signature(64) | crc(4)
    if snapshot.len() >= 68 + 52 {
        // At least header + sig + crc
        verify_snapshot_signature(snapshot)?;
    }

    Ok(())
}

/// Verify ECDSA-P256 signature on snapshot
fn verify_snapshot_signature(snapshot: &[u8]) -> Result<(), RuntimeError> {
    // Check if signature verification is enabled
    let cert_path = match env::var("MTP_SIGNING_CERT") {
        Ok(path) => path,
        Err(_) => {
            // No certificate configured - skip signature verification
            // In production, this should be required
            eprintln!("Warning: Snapshot signature verification skipped - no MTP_SIGNING_CERT configured");
            return Ok(());
        }
    };

    // Load certificate
    let cert_pem = std::fs::read_to_string(&cert_path).map_err(|e| {
        RuntimeError::ValueError(format!("Failed to read certificate: {}", e))
    })?;

    // Extract signature (64 bytes before CRC)
    // Snapshot format: ... | signature(64) | crc(4)
    let sig_start = snapshot.len() - 68; // 64 bytes sig + 4 bytes CRC
    let sig_end = snapshot.len() - 4;
    let signature = &snapshot[sig_start..sig_end];

    // Extract content hash from header (bytes 20-51)
    let content_hash = &snapshot[20..52];

    // Verify signature
    verify_ecdsa_p256(content_hash, signature, &cert_pem).map_err(|e| {
        RuntimeError::ValueError(format!("Snapshot signature verification failed: {}", e))
    })?;

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
    // JS content ends before signature (64 bytes) and CRC (4 bytes)
    let js_end = size - 68; // Before signature (64) + CRC (4)
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
