use crate::errors::compile::CompileError;

#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
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
    Gt,
    Le,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Literals
    String(String),
    Number(i64),
    Decimal(String),
    Boolean(bool),
    Array(Vec<Expr>),
    Object(Vec<(String, Expr)>),

    // Identifiers and access
    Ident(String),
    Dot(Box<Expr>, String),
    Index(Box<Expr>, Box<Expr>),

    // Function calls and operators
    Call { func: Box<Expr>, args: Vec<Expr> },
    Unary(BinOp, Box<Expr>), // reusing BinOp for simplicity
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Pipeline(Box<Expr>, Box<Expr>),

    // Control flow
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
    Match {
        expr: Box<Expr>,
        cases: Vec<(Pattern, Expr)>,
    },

    // Declarations in expressions
    Const {
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
    },
    Lambda {
        params: Vec<(String, TypeExpr)>,
        body: Box<Expr>,
    },

    // Special constructs
    Await(Box<Expr>),
    RespondJson(Box<Expr>),

    // Grouping
    Group(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Wildcard,
    Ident(String),
    Literal(Expr), // for numbers, strings, booleans
    Variant(String, Vec<Pattern>),
    Record(String, Vec<(String, Pattern)>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    Ident(String),
    Generic(String, Vec<TypeExpr>),
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
    pub alias: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeDecl {
    Record {
        name: String,
        fields: Vec<(String, TypeExpr)>,
    },
    Adt {
        name: String,
        type_params: Vec<String>,
        variants: Vec<VariantDecl>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariantDecl {
    pub name: String,
    pub payload: Vec<TypeExpr>, // empty for unit variants
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuncDecl {
    pub name: String,
    pub params: Vec<(String, TypeExpr)>,
    pub effects: Vec<String>,
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
pub struct Program {
    pub decls: Vec<ModuleDecl>,
}

impl Program {
    pub fn validate(&self) -> Result<(), CompileError> {
        // Basic validation - ensure no duplicate paths or something
        // For now, just return Ok
        Ok(())
    }
}
