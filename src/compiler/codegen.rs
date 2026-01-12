use crate::ir::{BinOpKind, IrInstruction, IrProgram, IrValue, UnOpKind};
use std::collections::HashMap;

pub fn compile_ir_to_js(ir: &IrProgram) -> Result<String, String> {
    let mut js = String::new();

    // Add runtime helpers
    js.push_str(include_str!("runtime_helpers.js"));
    js.push_str("\n");

    // For now, just execute the main function inline
    if let Some(main_func) = ir.functions.iter().find(|f| f.name == "main") {
        js.push_str("// Main function body\n");
        for inst in &main_func.instructions {
            js.push_str(&compile_instruction(inst)?);
            js.push_str("\n");
        }
        js.push_str("return result;\n");
    }

    Ok(js)
}

fn compile_function(func: &crate::ir::IrFunction) -> Result<String, String> {
    let mut js = String::new();
    let params = func.params.iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    js.push_str(&format!("function {}({}) {{\n", func.name, params));

    // Compile instructions
    for inst in &func.instructions {
        js.push_str(&compile_instruction(inst)?);
        js.push_str("\n");
    }

    js.push_str("}\n");
    Ok(js)
}
    }

    if !var_decls.is_empty() {
        js.push_str(&var_decls.join("\n"));
        js.push_str("\n");
    }

    // Compile instructions
    for inst in &func.instructions {
        js.push_str(&compile_instruction(inst)?);
        js.push_str("\n");
    }

    js.push_str("}\n");
    Ok(js)
}

fn compile_instruction(inst: &IrInstruction) -> Result<String, String> {
    match inst {
        IrInstruction::LoadConst { value, dest } => {
            let js_value = compile_value(value);
            Ok(format!("{} = {};", dest, js_value))
        }
        IrInstruction::LoadVar { src, dest } => {
            Ok(format!("{} = {};", dest, src))
        }
        IrInstruction::StoreVar { src, dest } => {
            Ok(format!("{} = {};", dest, src))
        }
        IrInstruction::BinOp { op, left, right, dest } => {
            let js_op = compile_binop(*op);
            Ok(format!("{} = {} {} {};", dest, left, js_op, right))
        }
        IrInstruction::UnOp { op, operand, dest } => {
            let js_op = compile_unop(*op);
            Ok(format!("{} = {}{};", dest, js_op, operand))
        }
        IrInstruction::Call { func, args, dest } => {
            let args_str = args.join(", ");
            if let Some(dest) = dest {
                Ok(format!("{} = {}({});", dest, func, args_str))
            } else {
                Ok(format!("{}({});", func, args_str))
            }
        }
        IrInstruction::Return { value } => {
            if let Some(val) = value {
                Ok(format!("return {};", val))
            } else {
                Ok("return;".to_string())
            }
        }
        IrInstruction::Jump { label } => {
            Ok(format!("// goto {}", label))
        }
        IrInstruction::JumpIf { condition, true_label, false_label } => {
            Ok(format!("if ({}) {{ /* goto {} */ }} else {{ /* goto {} */ }}", condition, true_label, false_label))
        }
        IrInstruction::Label { name } => {
            Ok(format!("// {}:", name))
        }
        IrInstruction::EffectCall { effect, args } => {
            let args_str = args.join(", ");
            Ok(format!("// effect {}({})", effect, args_str))
        }
    }
}
        IrInstruction::LoadVar { src, dest } => Ok(format!("  {} = {};", dest, src)),
        IrInstruction::StoreVar { src, dest } => Ok(format!("  {} = {};", dest, src)),
        IrInstruction::BinOp {
            op,
            left,
            right,
            dest,
        } => {
            let js_op = compile_binop(*op);
            Ok(format!("  {} = {} {} {};", dest, left, js_op, right))
        }
        IrInstruction::UnOp { op, operand, dest } => {
            let js_op = compile_unop(*op);
            Ok(format!("  {} = {}{};", dest, js_op, operand))
        }
        IrInstruction::Call { func, args, dest } => {
            let args_str = args.join(", ");
            if let Some(dest) = dest {
                Ok(format!("  {} = {}({});", dest, func, args_str))
            } else {
                Ok(format!("  {}({});", func, args_str))
            }
        }
        IrInstruction::Return { value } => {
            if let Some(val) = value {
                Ok(format!("  return {};", val))
            } else {
                Ok("  return;".to_string())
            }
        }
        IrInstruction::Jump { label } => Ok(format!("  // goto {}", label)),
        IrInstruction::JumpIf {
            condition,
            true_label,
            false_label,
        } => Ok(format!(
            "  if ({}) {{ /* goto {} */ }} else {{ /* goto {} */ }}",
            condition, true_label, false_label
        )),
        IrInstruction::Label { name } => Ok(format!("  // {}:", name)),
        IrInstruction::EffectCall { effect, args } => {
            let args_str = args.join(", ");
            Ok(format!("  // effect {}({})", effect, args_str))
        }
    }
}

fn compile_value(value: &IrValue) -> String {
    match value {
        IrValue::Number(n) => n.to_string(),
        IrValue::Decimal(s) => s.clone(),
        IrValue::Boolean(b) => b.to_string(),
        IrValue::String(s) => format!("\"{}\"", s),
        IrValue::Null => "null".to_string(),
    }
}

fn compile_binop(op: BinOpKind) -> &'static str {
    match op {
        BinOpKind::Add => "+",
        BinOpKind::Sub => "-",
        BinOpKind::Mul => "*",
        BinOpKind::Div => "/",
        BinOpKind::Eq => "===",
        BinOpKind::Ne => "!==",
        BinOpKind::Lt => "<",
        BinOpKind::Le => "<=",
        BinOpKind::Gt => ">",
        BinOpKind::Ge => ">=",
        BinOpKind::And => "&&",
        BinOpKind::Or => "||",
    }
}

fn compile_unop(op: UnOpKind) -> &'static str {
    match op {
        UnOpKind::Neg => "-",
        UnOpKind::Not => "!",
    }
}
