use crate::parser::ast::{BinOp, HttpMethod};
use crate::types::Type;

#[derive(Debug, Clone, PartialEq)]
pub enum IrExpr {
    // Literals
    String(String, Type),
    Number(i64, Type),
    Decimal(String, Type),
    Boolean(bool, Type),
    Array(Vec<IrExpr>, Type),
    Object(Vec<(String, IrExpr)>, Type),

    // Identifiers and access
    Var(String, Type),
    Dot(Box<IrExpr>, String, Type),
    Index(Box<IrExpr>, Box<IrExpr>, Type),

    // Function calls and operators
    Call {
        func: Box<IrExpr>,
        args: Vec<IrExpr>,
        result_type: Type,
    },
    TailCall {
        func: Box<IrExpr>,
        args: Vec<IrExpr>,
        result_type: Type,
    },
    Unary(BinOp, Box<IrExpr>, Type),
    Binary(BinOp, Box<IrExpr>, Box<IrExpr>, Type),

    // Control flow
    If {
        condition: Box<IrExpr>,
        then_branch: Box<IrExpr>,
        else_branch: Box<IrExpr>,
        result_type: Type,
    },
    Match {
        expr: Box<IrExpr>,
        cases: Vec<(IrPattern, IrExpr)>,
        result_type: Type,
    },

    // Declarations in expressions
    Let {
        name: String,
        value: Box<IrExpr>,
        body: Box<IrExpr>,
        result_type: Type,
    },

    // Special constructs
    EffectCall(String, Vec<IrExpr>, Type), // For built-in effects
    RespondJson(Box<IrExpr>, Type),
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrPattern {
    Wildcard,
    Var(String),
    Literal(IrExpr),
    Variant(String, Vec<IrPattern>),
    Record(String, Vec<(String, IrPattern)>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrFunction {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub effects: Vec<String>,
    pub body: IrExpr,
    pub is_tail_recursive: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrApi {
    pub method: HttpMethod,
    pub path: String,
    pub effects: Vec<String>,
    pub body: IrExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrDecl {
    Function(IrFunction),
    Api(IrApi),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrProgram {
    pub decls: Vec<IrDecl>,
}

impl IrProgram {
    pub fn validate(&self) -> Result<(), String> {
        // Basic validation
        for decl in &self.decls {
            match decl {
                IrDecl::Function(func) => {
                    if func.body.result_type() != func.return_type {
                        return Err(format!(
                            "Function {} return type mismatch: expected {:?}, got {:?}",
                            func.name,
                            func.return_type,
                            func.body.result_type()
                        ));
                    }
                }
                IrDecl::Api(_) => {} // API bodies can be any type
            }
        }
        Ok(())
    }
}

impl IrExpr {
    pub fn result_type(&self) -> Type {
        match self {
            IrExpr::String(_, t)
            | IrExpr::Number(_, t)
            | IrExpr::Decimal(_, t)
            | IrExpr::Boolean(_, t)
            | IrExpr::Array(_, t)
            | IrExpr::Object(_, t)
            | IrExpr::Var(_, t)
            | IrExpr::Dot(_, _, t)
            | IrExpr::Index(_, _, t)
            | IrExpr::Call { result_type: t, .. }
            | IrExpr::TailCall { result_type: t, .. }
            | IrExpr::Unary(_, _, t)
            | IrExpr::Binary(_, _, _, t)
            | IrExpr::If { result_type: t, .. }
            | IrExpr::Match { result_type: t, .. }
            | IrExpr::Let { result_type: t, .. }
            | IrExpr::EffectCall(_, _, t)
            | IrExpr::RespondJson(_, t) => t.clone(),
        }
    }
}
