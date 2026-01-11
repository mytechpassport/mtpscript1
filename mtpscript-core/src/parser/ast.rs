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
pub enum Expr {
    // For now, just a placeholder for expressions
    // We'll expand this as needed
    Ident(String),
    String(String),
    Number(i64),
    Boolean(bool),
    Call { func: String, args: Vec<Expr> },
    RespondJson(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModuleDecl {
    ApiDecl(ApiDecl),
    // Other declarations will be added later
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
