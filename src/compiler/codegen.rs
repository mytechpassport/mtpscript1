use crate::ir::IrProgram;

pub fn compile_ir_to_js(ir: &IrProgram) -> Result<String, String> {
    let mut js = String::new();
    for func in &ir.functions {
        js.push_str(&format!("function {}({}) {{\n", func.name, func.params.join(", ")));
        // Compile body
        js.push_str(&compile_body(&func.body));
        js.push_str("}\n");
    }
    Ok(js)
}

fn compile_body(body: &Vec<String>) -> String {
    let mut js = String::new();
    for stmt in body {
        if stmt.starts_with("const ") {
            js.push_str(&format!("  {};\n", stmt));
        } else {
            js.push_str(&format!("  return {};\n", stmt));
        }
    }
    js
}

// Use the real IrProgram from ir/mod.rs
    Ok(js)
}

fn compile_expr(expr: &str) -> String {
    // Basic: return as is, but handle some cases
    expr.to_string()
}

// Placeholder for proper IR
pub struct IrProgram {
    pub functions: Vec<IrFunction>,
}

pub struct IrFunction {
    pub name: String,
    pub params: Vec<String>,
    pub body: String, // simplified
}
