use crate::ir::IrFunction;

// Placeholder for IrExpr
pub enum IrExpr {
    Call(String, Vec<IrExpr>),
    Var(String),
    Lit(i64),
}

impl IrFunction {
    pub fn detect_tail_calls(&mut self) {
        // Simple detection: if the last expression is a call to self
        if let Some(last_expr) = self.body.last() {
            if let IrExpr::Call(name, _) = last_expr {
                if name == &self.name {
                    self.is_tail_recursive = true;
                }
            }
        }
    }
}
