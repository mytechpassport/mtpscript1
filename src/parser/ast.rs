use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub decls: Vec<ModuleDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModuleDecl {
    Import(ImportDecl),
    Type(TypeDecl),
    Func(FuncDecl),
    Api(ApiDecl),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportDecl {
    pub path: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDecl {
    pub name: String,
    pub type_params: Vec<String>,
    pub variants: Vec<Variant>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub name: String,
    pub payload: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuncDecl {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub effects: Vec<String>,
    pub return_type: Option<Type>,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApiDecl {
    pub method: HttpMethod,
    pub path: String,
    pub effects: Vec<String>,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Number,
    Boolean,
    String,
    Decimal,
    Array(Box<Type>),
    Object(HashMap<String, Type>),
    Adt(String, Vec<Type>), // name and type args
    Generic(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Ident(String),
    StringLit(String),
    NumberLit(String),
    BoolLit(bool),
    Array(Vec<Expr>),
    Object(Vec<(String, Expr)>),
    Call {
        func: String,
        args: Vec<Expr>,
    },
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        op: UnOp,
        expr: Box<Expr>,
    },
    If {
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },
    Match {
        expr: Box<Expr>,
        cases: Vec<(Pattern, Expr)>,
    },
    Block(Vec<Expr>),
    Lambda {
        params: Vec<String>,
        body: Box<Expr>,
    },
    Respond(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Pipe, // |>
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnOp {
    Not,
    Neg,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Wildcard,
    Ident(String),
    Lit(Expr),
    Variant(String, Vec<Pattern>),
    Record(Vec<(String, Pattern)>),
}
