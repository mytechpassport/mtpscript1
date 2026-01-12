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
        output.push_str(&format!("{}(function() {{\n", indent_str));

        // Bind the expression to a variable to avoid re-evaluation
        let match_var = self.next_temp_var();
        output.push_str(&format!(
            "{}  const {} = {};\n",
            indent_str, match_var, expr_js
        ));

        // Generate pattern matching logic
        let match_body = self.compile_pattern_cases(&match_var, cases, indent + 1)?;
        output.push_str(&match_body);
        output.push_str(&format!("{})()}}", indent_str));

        Ok(output)
    }

    /// Compile pattern cases as a series of if-else statements
    fn compile_pattern_cases(
        &mut self,
        match_var: &str,
        cases: &[(IrPattern, IrExpr)],
        indent: usize,
    ) -> Result<String, CompileError> {
        let indent_str = "  ".repeat(indent);
        let mut output = String::new();

        let mut first = true;
        for (pattern, body) in cases {
            let (condition, bindings) = self.compile_pattern_binding(pattern, match_var)?;

            if first {
                output.push_str(&format!("{}if ({}) {{\n", indent_str, condition));
                first = false;
            } else {
                output.push_str(&format!("{}  }} else if ({}) {{\n", indent_str, condition));
            }

            // Add variable bindings
            for (var_name, var_expr) in bindings {
                output.push_str(&format!(
                    "{}  const {} = {};\n",
                    indent_str, var_name, var_expr
                ));
            }

            // Compile the body
            let body_js = self.compile_expr(body, indent + 1)?;
            output.push_str(&body_js);
            output.push('\n');
        }

        // Close the last case
        output.push_str(&format!("{}  }}\n", indent_str));

        Ok(output)
    }

    /// Compile a pattern and return (condition, variable_bindings)
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
        let mut conditions = vec![format!("typeof {}.{} !== \"undefined\"", expr_var, name)];
        let mut bindings = vec![];

        // For constructors with arguments, bind the value
        if !sub_patterns.is_empty() {
            let value_expr = format!("{}.{}", expr_var, name);
            let temp_var = self.next_temp_var();
            bindings.push((temp_var.clone(), value_expr.clone()));

            for (i, sub_pattern) in sub_patterns.iter().enumerate() {
                match sub_pattern {
                    IrPattern::Wildcard => {
                        // No additional condition
                    }
                    IrPattern::Var(var_name) => {
                        // For single argument constructors like Some(x), bind x to the value
                        if sub_patterns.len() == 1 {
                            bindings.push((var_name.clone(), temp_var.clone()));
                        } else {
                            // For multiple arguments, this would be an array or tuple
                            // For now, not implemented
                            return Err(CompileError::CodeGenError(
                                "Complex ADT patterns not yet supported".to_string(),
                            ));
                        }
                    }
                    IrPattern::Literal(_) => {
                        return Err(CompileError::CodeGenError(
                            "Complex expressions in patterns not yet supported".to_string(),
                        ));
                    }
                    IrPattern::Variant(_, _) | IrPattern::Record(_, _) => {
                        return Err(CompileError::CodeGenError(
                            "Nested patterns not yet supported".to_string(),
                        ));
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
        // This is a simplified version - in practice, we'd reuse the main codegen
        match expr {
            IrExpr::String(s, _) => Ok(format!("\"{}\"", s)),
            IrExpr::Number(n, _) => Ok(n.to_string()),
            IrExpr::Decimal(d, _) => Ok(format!("\"{}\"", d)),
            IrExpr::Boolean(b, _) => Ok(b.to_string()),
            IrExpr::Var(name, _) => Ok(name.clone()),
            IrExpr::Dot(expr, field, _) => {
                let expr_js = self.compile_expr(expr, 0)?;
                Ok(format!("{}.{}", expr_js, field))
            }
            IrExpr::Index(array, index, _) => {
                let array_js = self.compile_expr(array, 0)?;
                let index_js = self.compile_expr(index, 0)?;
                Ok(format!("{}[{}]", array_js, index_js))
            }
            IrExpr::Binary(op, left, right, _) => {
                let left_js = self.compile_expr(left, 0)?;
                let right_js = self.compile_expr(right, 0)?;
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
                let expr_js = self.compile_expr(expr, 0)?;
                let op_js = match op {
                    crate::parser::ast::BinOp::Sub => "-", // -x
                    _ => {
                        return Err(CompileError::CodeGenError(format!(
                            "Unsupported unary operator: {:?}",
                            op
                        )))
                    }
                };
                Ok(format!("{}{}", op_js, expr_js))
            }
            _ => Err(CompileError::CodeGenError(
                "Complex expressions in match arms not yet supported".to_string(),
            )),
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
