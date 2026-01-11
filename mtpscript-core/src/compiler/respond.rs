use crate::parser::ast::Expr;

pub fn compile_respond_json(expr: &Expr) -> String {
    // For now, generate JS that calls a hypothetical canonical JSON serializer
    // In the future, this would integrate with the json module
    format!(
        "return JSON.stringifyCanonical({});",
        compile_expr_to_js(expr)
    )
}

fn compile_expr_to_js(expr: &Expr) -> String {
    match expr {
        Expr::Ident(name) => name.clone(),
        Expr::String(s) => format!("\"{}\"", s),
        Expr::Number(n) => n.to_string(),
        Expr::Boolean(b) => b.to_string(),
        Expr::Call { func, args } => {
            let func_js = compile_expr_to_js(func);
            let args_js: Vec<String> = args.iter().map(compile_expr_to_js).collect();
            format!("{}({})", func_js, args_js.join(", "))
        }
        Expr::RespondJson(inner) => compile_expr_to_js(inner),
        _ => unimplemented!("Expression type not yet supported in compiler"),
    }
}
