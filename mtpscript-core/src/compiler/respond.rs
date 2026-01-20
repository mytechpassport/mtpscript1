use crate::parser::ast::{BinOp, Expr};

pub fn compile_respond_json(expr: &Expr) -> String {
    // Generate JS that calls the canonical JSON serializer per TECHSPECV5.md §23
    format!(
        "return JSON.stringifyCanonical({});",
        compile_expr_to_js(expr)
    )
}

fn compile_expr_to_js(expr: &Expr) -> String {
    match expr {
        // Literals
        Expr::Ident(name) => name.clone(),
        Expr::String(s) => format!("\"{}\"", escape_string(s)),
        Expr::Number(n) => n.to_string(),
        Expr::Decimal(d) => format!("\"{}\"", d), // Decimals as strings per spec §4-a
        Expr::Boolean(b) => b.to_string(),

        // Arrays and Objects
        Expr::Array(items) => {
            let items_js: Vec<String> = items.iter().map(compile_expr_to_js).collect();
            format!("[{}]", items_js.join(", "))
        }
        Expr::Object(fields) => {
            let fields_js: Vec<String> = fields
                .iter()
                .map(|(k, v)| format!("\"{}\": {}", escape_string(k), compile_expr_to_js(v)))
                .collect();
            format!("{{{}}}", fields_js.join(", "))
        }

        // Property and index access
        Expr::Dot(expr, field) => {
            format!("{}.{}", compile_expr_to_js(expr), field)
        }
        Expr::Index(expr, index) => {
            format!("{}[{}]", compile_expr_to_js(expr), compile_expr_to_js(index))
        }

        // Function calls
        Expr::Call { func, args } => {
            let func_js = compile_expr_to_js(func);
            let args_js: Vec<String> = args.iter().map(compile_expr_to_js).collect();
            format!("{}({})", func_js, args_js.join(", "))
        }

        // Operators
        Expr::Binary(op, left, right) => {
            let op_str = match op {
                BinOp::Add => "+",
                BinOp::Sub => "-",
                BinOp::Mul => "*",
                BinOp::Div => "/",
                BinOp::Eq => "===",
                BinOp::Ne => "!==",
                BinOp::Lt => "<",
                BinOp::Gt => ">",
                BinOp::Le => "<=",
                BinOp::Ge => ">=",
                BinOp::And => "&&",
                BinOp::Or => "||",
                BinOp::Not => "!", // Should not appear in binary, but handle gracefully
            };
            format!(
                "({} {} {})",
                compile_expr_to_js(left),
                op_str,
                compile_expr_to_js(right)
            )
        }
        Expr::Unary(op, expr) => {
            let op_str = match op {
                BinOp::Sub => "-",
                BinOp::Not => "!",
                BinOp::Add => "+", // Unary plus (identity)
                // Other operators shouldn't appear as unary - treat as identity
                _ => "",
            };
            format!("{}{}", op_str, compile_expr_to_js(expr))
        }

        // Pipeline: a |> f => f(a)
        Expr::Pipeline(left, right) => {
            format!("{}({})", compile_expr_to_js(right), compile_expr_to_js(left))
        }

        // Control flow
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            format!(
                "({} ? {} : {})",
                compile_expr_to_js(condition),
                compile_expr_to_js(then_branch),
                compile_expr_to_js(else_branch)
            )
        }

        Expr::Match { expr, cases } => {
            // Compile match as nested ternary
            compile_match_to_js(expr, cases)
        }

        // Const binding
        Expr::Const { name, value, body } => {
            format!(
                "(function() {{ const {} = {}; return {}; }})()",
                name,
                compile_expr_to_js(value),
                compile_expr_to_js(body)
            )
        }

        // Lambda
        Expr::Lambda { params, body } => {
            let params_str: Vec<String> = params.iter().map(|(n, _)| n.clone()).collect();
            format!(
                "function({}) {{ return {}; }}",
                params_str.join(", "),
                compile_expr_to_js(body)
            )
        }

        // Await (desugars to Async.await call)
        Expr::Await(inner) => {
            format!("Async.await({})", compile_expr_to_js(inner))
        }

        // RespondJson
        Expr::RespondJson(inner) => compile_expr_to_js(inner),

        // Grouping
        Expr::Group(inner) => format!("({})", compile_expr_to_js(inner)),
    }
}

/// Compile match expression to nested ternary
fn compile_match_to_js(expr: &Expr, cases: &[(crate::parser::ast::Pattern, Expr)]) -> String {
    let expr_js = compile_expr_to_js(expr);
    let match_var = "_match_val";

    let mut result = format!("(function() {{ const {} = {}; return ", match_var, expr_js);

    for (i, (pattern, body)) in cases.iter().enumerate() {
        let (condition, bindings) = compile_pattern(pattern, match_var);
        let body_js = compile_expr_to_js(body);

        // Wrap body with variable bindings if needed
        let body_with_bindings = if bindings.is_empty() {
            body_js
        } else {
            let binding_strs: Vec<String> = bindings
                .iter()
                .map(|(name, expr)| format!("const {} = {};", name, expr))
                .collect();
            format!(
                "(function() {{ {} return {}; }})()",
                binding_strs.join(" "),
                body_js
            )
        };

        if i == cases.len() - 1 {
            // Last case
            result.push_str(&body_with_bindings);
        } else {
            result.push_str(&format!("{} ? {} : ", condition, body_with_bindings));
        }
    }

    result.push_str("; })()");
    result
}

/// Compile a pattern and return (condition, variable bindings)
fn compile_pattern(pattern: &crate::parser::ast::Pattern, expr_var: &str) -> (String, Vec<(String, String)>) {
    use crate::parser::ast::Pattern;

    match pattern {
        Pattern::Wildcard => ("true".to_string(), vec![]),
        Pattern::Ident(name) => ("true".to_string(), vec![(name.clone(), expr_var.to_string())]),
        Pattern::Literal(lit_expr) => {
            let lit_js = compile_expr_to_js(lit_expr);
            (format!("{} === {}", expr_var, lit_js), vec![])
        }
        Pattern::Variant(name, sub_patterns) => {
            let mut conditions = vec![format!("{}[\"{}\"] !== undefined", expr_var, name)];
            let mut bindings = vec![];

            if !sub_patterns.is_empty() {
                let value_expr = format!("{}.{}", expr_var, name);

                for (i, sub_pattern) in sub_patterns.iter().enumerate() {
                    let sub_expr = if sub_patterns.len() == 1 {
                        value_expr.clone()
                    } else {
                        format!("{}[{}]", value_expr, i)
                    };

                    let (sub_cond, sub_bindings) = compile_pattern(sub_pattern, &sub_expr);
                    if sub_cond != "true" {
                        conditions.push(sub_cond);
                    }
                    bindings.extend(sub_bindings);
                }
            }

            (conditions.join(" && "), bindings)
        }
        Pattern::Record(name, fields) => {
            let mut conditions = vec![format!("{}.type === \"{}\"", expr_var, name)];
            let mut bindings = vec![];

            for (field_name, field_pattern) in fields {
                let field_expr = format!("{}.{}", expr_var, field_name);
                let (field_cond, field_bindings) = compile_pattern(field_pattern, &field_expr);
                if field_cond != "true" {
                    conditions.push(field_cond);
                }
                bindings.extend(field_bindings);
            }

            (conditions.join(" && "), bindings)
        }
    }
}

/// Escape special characters in strings for JS output
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
