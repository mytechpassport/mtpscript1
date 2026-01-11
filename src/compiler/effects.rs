// Placeholder for IrExpr
pub enum IrExpr {
    Call(String, Vec<String>),
}

pub fn compile_effect_call(effect_name: &str, args: Vec<String>) -> String {
    format!(
        "{}(\"{}\", [{}])",
        effect_name,
        effect_name,
        args.join(", ")
    )
}
