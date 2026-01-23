use mtpscript_core::ir::nodes::{IrDecl, IrExpr, IrFunction, IrProgram};
use mtpscript_core::types::Type;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ir_data_structures() {
        // Test creating IR nodes as per acceptance criteria
        let func = IrFunction {
            name: "add".to_string(),
            params: vec![
                ("a".to_string(), Type::Number),
                ("b".to_string(), Type::Number),
            ],
            return_type: Type::Number,
            effects: vec![],
            body: IrExpr::Binary(
                mtpscript_core::parser::ast::BinOp::Add,
                Box::new(IrExpr::Var("a".to_string(), Type::Number)),
                Box::new(IrExpr::Var("b".to_string(), Type::Number)),
                Type::Number,
            ),
            is_tail_recursive: false,
        };

        let ir = IrProgram {
            decls: vec![IrDecl::Function(func)],
            adt_types: vec![],
        };

        // Test that it validates
        assert!(ir.decls.len() == 1);
        if let IrDecl::Function(f) = &ir.decls[0] {
            assert_eq!(f.name, "add");
            assert_eq!(f.params.len(), 2);
        }
    }

    #[test]
    fn test_ir_expr_construction() {
        // Test that IR expressions can be constructed with types
        let num_expr = IrExpr::Number(42, Type::Number);
        let str_expr = IrExpr::String("hello".to_string(), Type::String);

        // Just test construction succeeds
        assert!(matches!(num_expr, IrExpr::Number(42, Type::Number)));
        assert!(matches!(str_expr, IrExpr::String(_, Type::String)));
    }
}
