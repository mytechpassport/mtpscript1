use crate::errors::compile::CompileError;
use crate::ir::nodes::{IrExpr, IrPattern};

/// Advanced pattern matching compilation for complex patterns
/// This handles nested patterns, guards, and optimizations

pub struct PatternCompiler {
    temp_var_counter: usize,
}

impl PatternCompiler {
    pub fn new() -> Self {
        PatternCompiler {
            temp_var_counter: 0,
        }
    }

    /// Compile a match expression with advanced pattern matching
    pub fn compile_match(
        &mut self,
        expr: &IrExpr,
        cases: &[(IrPattern, IrExpr)],
        indent: usize,
    ) -> Result<String, CompileError> {
        let indent_str = "  ".repeat(indent);
        let expr_js = self.compile_expr(expr, 0)?;

        let mut output = format!("{}// Match expression with advanced patterns\n", indent_str);

        // Bind the expression to a variable to avoid re-evaluation
        let match_var = self.next_temp_var();
        output.push_str(&format!(
            "{}const {} = {};\n",
            indent_str, match_var, expr_js
        ));

        // Generate simple pattern matching as ternary expressions
        let match_expr = self.compile_simple_ternary_match(&match_var, cases)?;
        output.push_str(&format!("{}{}", indent_str, match_expr));

        Ok(output)
    }

    /// Compile simple pattern cases as ternary expressions with variable substitution
    fn compile_simple_ternary_match(
        &mut self,
        match_var: &str,
        cases: &[(IrPattern, IrExpr)],
    ) -> Result<String, CompileError> {
        if cases.is_empty() {
            return Err(CompileError::CodeGenError(
                "Match must have at least one case".to_string(),
            ));
        }

        let mut result = String::new();

        for (i, (pattern, body)) in cases.iter().enumerate() {
            let (condition, bindings) = self.compile_pattern_binding(pattern, match_var)?;

            // Create substitution map
            let mut subs = std::collections::HashMap::new();
            for (var_name, var_expr) in &bindings {
                subs.insert(var_name.clone(), var_expr.clone());
            }

            let body_js = self.compile_expr_with_subs(body, &subs)?;

            if i == 0 {
                result = format!("{} ? {} : ", condition, body_js);
            } else if i == cases.len() - 1 {
                result.push_str(&body_js);
            } else {
                result.push_str(&format!("{} ? {} : ", condition, body_js));
            }
        }

        Ok(format!("({})", result))
    }

    /// Compile a pattern and return (condition, variable_bindings as expressions)
    fn compile_pattern_binding(
        &mut self,
        pattern: &IrPattern,
        expr_var: &str,
    ) -> Result<(String, Vec<(String, String)>), CompileError> {
        match pattern {
            IrPattern::Wildcard => Ok(("true".to_string(), vec![])),
            IrPattern::Var(name) => {
                // Variable binding - always matches
                Ok((
                    "true".to_string(),
                    vec![(name.clone(), expr_var.to_string())],
                ))
            }
            IrPattern::Literal(lit_expr) => {
                let lit_js = self.compile_expr(lit_expr, 0)?;
                Ok((format!("{} === {}", expr_var, lit_js), vec![]))
            }
            IrPattern::Variant(name, sub_patterns) => {
                self.compile_variant_pattern(name, sub_patterns, expr_var)
            }
            IrPattern::Record(name, fields) => self.compile_record_pattern(name, fields, expr_var),
        }
    }

    fn compile_variant_pattern(
        &mut self,
        name: &str,
        sub_patterns: &[IrPattern],
        expr_var: &str,
    ) -> Result<(String, Vec<(String, String)>), CompileError> {
        // Check if the ADT has this constructor
        let mut conditions = vec![format!("{}[\"{}\"] !== undefined", expr_var, name)];
        let mut bindings = vec![];

        // For constructors with arguments, bind the value
        if !sub_patterns.is_empty() {
            let value_expr = format!("{}.{}", expr_var, name);
            let temp_var = self.next_temp_var();
            bindings.push((temp_var.clone(), value_expr.clone()));

            for (i, sub_pattern) in sub_patterns.iter().enumerate() {
                // For multi-argument constructors, access by index; for single arg, use directly
                let sub_expr = if sub_patterns.len() == 1 {
                    temp_var.clone()
                } else {
                    format!("{}[{}]", temp_var, i)
                };

                match sub_pattern {
                    IrPattern::Wildcard => {
                        // No additional condition or binding needed
                    }
                    IrPattern::Var(var_name) => {
                        // Bind the variable to the appropriate expression
                        bindings.push((var_name.clone(), sub_expr));
                    }
                    IrPattern::Literal(lit_expr) => {
                        // Compile literal and add equality check
                        let lit_js = self.compile_expr(lit_expr, 0)?;
                        conditions.push(format!("{} === {}", sub_expr, lit_js));
                    }
                    IrPattern::Variant(nested_name, nested_subs) => {
                        // Recursively compile nested variant pattern
                        let nested_temp = self.next_temp_var();
                        bindings.push((nested_temp.clone(), sub_expr));
                        let (nested_cond, nested_bindings) =
                            self.compile_variant_pattern(nested_name, nested_subs, &nested_temp)?;
                        if nested_cond != "true" {
                            conditions.push(nested_cond);
                        }
                        bindings.extend(nested_bindings);
                    }
                    IrPattern::Record(rec_name, rec_fields) => {
                        // Recursively compile nested record pattern
                        let nested_temp = self.next_temp_var();
                        bindings.push((nested_temp.clone(), sub_expr));
                        let (rec_cond, rec_bindings) =
                            self.compile_record_pattern(rec_name, rec_fields, &nested_temp)?;
                        if rec_cond != "true" {
                            conditions.push(rec_cond);
                        }
                        bindings.extend(rec_bindings);
                    }
                }
            }
        }

        let condition = if conditions.len() == 1 {
            conditions[0].clone()
        } else {
            format!("({})", conditions.join(" && "))
        };

        Ok((condition, bindings))
    }

    fn compile_record_pattern(
        &mut self,
        name: &str,
        fields: &[(String, IrPattern)],
        expr_var: &str,
    ) -> Result<(String, Vec<(String, String)>), CompileError> {
        let mut conditions = vec![format!("{}.type === \"{}\"", expr_var, name)];
        let mut bindings = vec![];

        for (field_name, field_pattern) in fields {
            let field_expr = format!("{}.{}", expr_var, field_name);
            let temp_var = self.next_temp_var();
            bindings.push((temp_var.clone(), field_expr.clone()));

            match field_pattern {
                IrPattern::Wildcard => {
                    // No additional condition
                }
                IrPattern::Var(var_name) => {
                    bindings.push((var_name.clone(), temp_var));
                }
                IrPattern::Literal(lit_expr) => {
                    let lit_js = self.compile_expr(lit_expr, 0)?;
                    conditions.push(format!("{} === {}", temp_var, lit_js));
                }
                IrPattern::Variant(sub_name, sub_subs) => {
                    let (sub_cond, sub_bindings) =
                        self.compile_variant_pattern(sub_name, sub_subs, &temp_var)?;
                    if sub_cond != "true" {
                        conditions.push(sub_cond);
                    }
                    bindings.extend(sub_bindings);
                }
                IrPattern::Record(rec_name, rec_fields) => {
                    let (rec_cond, rec_bindings) =
                        self.compile_record_pattern(rec_name, rec_fields, &temp_var)?;
                    if rec_cond != "true" {
                        conditions.push(rec_cond);
                    }
                    bindings.extend(rec_bindings);
                }
            }
        }

        Ok((conditions.join(" && "), bindings))
    }

    /// Compile expressions (simplified version for pattern compilation)
    fn compile_expr(&mut self, expr: &IrExpr, _indent: usize) -> Result<String, CompileError> {
        self.compile_expr_with_subs(expr, &std::collections::HashMap::new())
    }

    fn compile_expr_with_subs(
        &mut self,
        expr: &IrExpr,
        subs: &std::collections::HashMap<String, String>,
    ) -> Result<String, CompileError> {
        // This is a simplified version - in practice, we'd reuse the main codegen
        match expr {
            IrExpr::String(s, _) => Ok(format!("\"{}\"", s)),
            IrExpr::Number(n, _) => Ok(n.to_string()),
            IrExpr::Decimal(d, _) => Ok(format!("\"{}\"", d)),
            IrExpr::Boolean(b, _) => Ok(b.to_string()),
            IrExpr::Var(name, _) => {
                // Apply substitution if available
                if let Some(sub) = subs.get(name) {
                    Ok(sub.clone())
                } else {
                    Ok(name.clone())
                }
            }
            IrExpr::Dot(expr, field, _) => {
                let expr_js = self.compile_expr_with_subs(expr, subs)?;
                Ok(format!("{}.{}", expr_js, field))
            }
            IrExpr::Index(array, index, _) => {
                let array_js = self.compile_expr_with_subs(array, subs)?;
                let index_js = self.compile_expr_with_subs(index, subs)?;
                Ok(format!("{}[{}]", array_js, index_js))
            }
            IrExpr::Binary(op, left, right, _) => {
                let left_js = self.compile_expr_with_subs(left, subs)?;
                let right_js = self.compile_expr_with_subs(right, subs)?;
                let op_js = match op {
                    crate::parser::ast::BinOp::Add => "+",
                    crate::parser::ast::BinOp::Sub => "-",
                    crate::parser::ast::BinOp::Mul => "*",
                    crate::parser::ast::BinOp::Div => "/",
                    crate::parser::ast::BinOp::Eq => "===",
                    crate::parser::ast::BinOp::Ne => "!==",
                    crate::parser::ast::BinOp::Lt => "<",
                    crate::parser::ast::BinOp::Le => "<=",
                    crate::parser::ast::BinOp::Gt => ">",
                    crate::parser::ast::BinOp::Ge => ">=",
                    _ => {
                        return Err(CompileError::CodeGenError(format!(
                            "Unsupported binary operator: {:?}",
                            op
                        )))
                    }
                };
                Ok(format!("({} {} {})", left_js, op_js, right_js))
            }
            IrExpr::Unary(op, expr, _) => {
                let expr_js = self.compile_expr_with_subs(expr, subs)?;
                let op_js = match op {
                    crate::parser::ast::BinOp::Sub => "-", // -x
                    crate::parser::ast::BinOp::Not => "!", // !x
                    _ => {
                        return Err(CompileError::CodeGenError(format!(
                            "Unsupported unary operator: {:?}",
                            op
                        )))
                    }
                };
                Ok(format!("{}{}", op_js, expr_js))
            }
            IrExpr::Call { func, args, .. } => {
                let func_js = self.compile_expr_with_subs(func, subs)?;
                let args_js: Result<Vec<String>, _> = args
                    .iter()
                    .map(|a| self.compile_expr_with_subs(a, subs))
                    .collect();
                Ok(format!("{}({})", func_js, args_js?.join(", ")))
            }
            IrExpr::TailCall { func, args, .. } => {
                // TailCall compiles the same as Call at the JS level
                let func_js = self.compile_expr_with_subs(func, subs)?;
                let args_js: Result<Vec<String>, _> = args
                    .iter()
                    .map(|a| self.compile_expr_with_subs(a, subs))
                    .collect();
                Ok(format!("{}({})", func_js, args_js?.join(", ")))
            }
            IrExpr::If { condition, then_branch, else_branch, .. } => {
                let cond_js = self.compile_expr_with_subs(condition, subs)?;
                let then_js = self.compile_expr_with_subs(then_branch, subs)?;
                let else_js = self.compile_expr_with_subs(else_branch, subs)?;
                Ok(format!("({} ? {} : {})", cond_js, then_js, else_js))
            }
            IrExpr::Array(items, _) => {
                let items_js: Result<Vec<String>, _> = items
                    .iter()
                    .map(|i| self.compile_expr_with_subs(i, subs))
                    .collect();
                Ok(format!("[{}]", items_js?.join(", ")))
            }
            IrExpr::Object(fields, _) => {
                let fields_js: Result<Vec<String>, _> = fields
                    .iter()
                    .map(|(k, v)| {
                        let v_js = self.compile_expr_with_subs(v, subs)?;
                        Ok(format!("\"{}\": {}", k, v_js))
                    })
                    .collect();
                Ok(format!("{{{}}}", fields_js?.join(", ")))
            }
            IrExpr::Let { name, value, body, .. } => {
                let value_js = self.compile_expr_with_subs(value, subs)?;
                // Create new subs map without the bound variable to avoid shadowing issues
                let mut new_subs = subs.clone();
                new_subs.remove(name);
                let body_js = self.compile_expr_with_subs(body, &new_subs)?;
                Ok(format!("(function() {{ const {} = {}; return {}; }})()", name, value_js, body_js))
            }
            IrExpr::Lambda { params, body, .. } => {
                let params_str: Vec<String> = params.iter().map(|(n, _)| n.clone()).collect();
                // Remove lambda params from substitutions
                let mut new_subs = subs.clone();
                for (param_name, _) in params {
                    new_subs.remove(param_name);
                }
                let body_js = self.compile_expr_with_subs(body, &new_subs)?;
                Ok(format!("function({}) {{ return {}; }}", params_str.join(", "), body_js))
            }
            IrExpr::Match { expr, cases, .. } => {
                // Compile match as nested ternaries
                let expr_js = self.compile_expr_with_subs(expr, subs)?;
                let match_var = self.next_temp_var();
                let mut result = format!("(function() {{ const {} = {}; return ", match_var, expr_js);

                for (i, (pattern, case_body)) in cases.iter().enumerate() {
                    let (condition, bindings) = self.compile_pattern_binding(pattern, &match_var)?;

                    // Merge pattern bindings with existing subs
                    let mut case_subs = subs.clone();
                    for (var_name, var_expr) in &bindings {
                        case_subs.insert(var_name.clone(), var_expr.clone());
                    }

                    let body_js = self.compile_expr_with_subs(case_body, &case_subs)?;

                    if i == cases.len() - 1 {
                        // Last case (should be wildcard or catch-all)
                        result.push_str(&body_js);
                    } else {
                        result.push_str(&format!("{} ? {} : ", condition, body_js));
                    }
                }

                result.push_str("; })()");
                Ok(result)
            }
            IrExpr::EffectCall(effect_name, args, _) => {
                let args_js: Result<Vec<String>, _> = args
                    .iter()
                    .map(|a| self.compile_expr_with_subs(a, subs))
                    .collect();
                Ok(format!("{}({})", effect_name, args_js?.join(", ")))
            }
            IrExpr::RespondJson(inner, _) => {
                let inner_js = self.compile_expr_with_subs(inner, subs)?;
                Ok(format!("JSON.stringifyCanonical({})", inner_js))
            }
        }
    }

    fn next_temp_var(&mut self) -> String {
        let var = format!("_pat_{}", self.temp_var_counter);
        self.temp_var_counter += 1;
        var
    }
}

/// Public interface for compiling patterns
pub fn compile_match_with_patterns(
    expr: &IrExpr,
    cases: &[(IrPattern, IrExpr)],
    indent: usize,
) -> Result<String, CompileError> {
    let mut compiler = PatternCompiler::new();
    compiler.compile_match(expr, cases, indent)
}
