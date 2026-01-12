pub mod ast;

use crate::errors::MtpError;
use crate::lexer::token::{Token, TokenKind};
use ast::*;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: &[Token]) -> Self {
        Parser {
            tokens: tokens.to_vec(),
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Program, MtpError> {
        let mut decls = Vec::new();

        while !self.is_at_end() {
            match self.current_token().kind {
                TokenKind::Api => {
                    decls.push(ModuleDecl::Api(self.parse_api()?));
                }
                TokenKind::Function => {
                    decls.push(ModuleDecl::Func(self.parse_function()?));
                }
                _ => {
                    // Skip unknown tokens for now
                    self.advance();
                }
            }
        }

        Ok(Program { decls })
    }

    fn parse_import(&mut self) -> Result<ImportDecl, MtpError> {
        self.consume(TokenKind::Import)?;
        let path = self.consume_string()?;
        let alias = if self.match_token(TokenKind::Identifier("as".to_string()))? {
            Some(self.consume_identifier()?)
        } else {
            None
        };
        Ok(ImportDecl { path, alias })
    }

    fn parse_type(&mut self) -> Result<TypeDecl, MtpError> {
        self.consume(TokenKind::Type)?;
        let name = self.consume_identifier()?;
        let type_params = self.parse_type_params()?;
        self.consume(TokenKind::Equal)?;
        let variants = self.parse_variants()?;
        Ok(TypeDecl {
            name,
            type_params,
            variants,
        })
    }

    fn parse_variants(&mut self) -> Result<Vec<Variant>, MtpError> {
        let mut variants = Vec::new();
        self.consume(TokenKind::LeftBrace)?;
        while !self.check(TokenKind::RightBrace) {
            variants.push(self.parse_variant()?);
            if !self.match_token(TokenKind::Comma)? {
                break;
            }
        }
        self.consume(TokenKind::RightBrace)?;
        Ok(variants)
    }

    fn parse_variant(&mut self) -> Result<Variant, MtpError> {
        let name = self.consume_identifier()?;
        let payload = if self.match_token(TokenKind::LeftParen)? {
            let ty = self.parse_type()?;
            self.consume(TokenKind::RightParen)?;
            Some(ty)
        } else {
            None
        };
        Ok(Variant { name, payload })
    }

    fn parse_function(&mut self) -> Result<FuncDecl, MtpError> {
        self.consume(TokenKind::Function)?;
        let name = self.consume_identifier()?;
        self.consume(TokenKind::LeftParen)?;
        let params = self.parse_params()?;
        self.consume(TokenKind::RightParen)?;
        let effects = if self.match_token(TokenKind::Uses)? {
            self.parse_effects()?
        } else {
            Vec::new()
        };
        let return_type = if self.match_token(TokenKind::Colon)? {
            Some(self.parse_type()?)
        } else {
            None
        };
        self.consume(TokenKind::EqualGreater)?;
        let body = self.parse_expr()?;
        Ok(FuncDecl {
            name,
            params,
            effects,
            return_type,
            body,
        })
    }

    fn parse_api(&mut self) -> Result<ApiDecl, MtpError> {
        self.consume(TokenKind::Api)?;
        let method = self.parse_http_method()?;
        let path = self.consume_string()?;
        let effects = if self.match_token(TokenKind::Uses)? {
            self.parse_effects()?
        } else {
            Vec::new()
        };
        self.consume(TokenKind::LeftBrace)?;
        let body = self.parse_block()?;
        self.consume(TokenKind::RightBrace)?;
        Ok(ApiDecl {
            method,
            path,
            effects,
            body,
        })
    }

    fn parse_http_method(&mut self) -> Result<HttpMethod, MtpError> {
        match self.current_token().kind {
            TokenKind::Get => {
                self.advance();
                Ok(HttpMethod::Get)
            }
            TokenKind::Post => {
                self.advance();
                Ok(HttpMethod::Post)
            }
            TokenKind::Put => {
                self.advance();
                Ok(HttpMethod::Put)
            }
            TokenKind::Delete => {
                self.advance();
                Ok(HttpMethod::Delete)
            }
            TokenKind::Patch => {
                self.advance();
                Ok(HttpMethod::Patch)
            }
            _ => Err(MtpError::ParseError {
                message: "Expected HTTP method".to_string(),
            }),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, MtpError> {
        self.parse_match_expr()
    }

    fn parse_block(&mut self) -> Result<Expr, MtpError> {
        let mut exprs = Vec::new();
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            // Skip const declarations for now
            if self.match_token(TokenKind::Const)? {
                let _var_name = self.consume_identifier()?;
                self.consume(TokenKind::Equal)?;
                let _value = self.parse_expr()?;
                // Ignore const for now
            } else {
                exprs.push(self.parse_expr()?);
            }
            // Skip semicolons
            self.match_token(TokenKind::Semicolon)?;
        }
        Ok(Expr::Block(exprs))
    }

    fn parse_match_expr(&mut self) -> Result<Expr, MtpError> {
        if self.match_token(TokenKind::Match)? {
            let expr = self.parse_expr()?;
            self.consume(TokenKind::LeftBrace)?;
            let mut cases = Vec::new();
            while !self.check(TokenKind::RightBrace) {
                let pattern = self.parse_pattern()?;
                self.consume(TokenKind::EqualGreater)?;
                let case_expr = self.parse_expr()?;
                cases.push((pattern, case_expr));
                if !self.match_token(TokenKind::Comma)? {
                    break;
                }
            }
            self.consume(TokenKind::RightBrace)?;
            Ok(Expr::Match {
                expr: Box::new(expr),
                cases,
            })
        } else {
            self.parse_if_expr()
        }
    }

    fn parse_if_expr(&mut self) -> Result<Expr, MtpError> {
        if self.match_token(TokenKind::If)? {
            let cond = self.parse_expr()?;
            self.consume(TokenKind::Then)?;
            let then_branch = self.parse_expr()?;
            let else_branch = if self.match_token(TokenKind::Else)? {
                Some(self.parse_expr()?)
            } else {
                None
            };
            Ok(Expr::If {
                cond: Box::new(cond),
                then_branch: Box::new(then_branch),
                else_branch: else_branch.map(Box::new),
            })
        } else {
            self.parse_binary_expr()
        }
    }

    fn parse_binary_expr(&mut self) -> Result<Expr, MtpError> {
        let mut expr = self.parse_unary_expr()?;

        while self.is_binary_op() {
            let op = self.parse_binop()?;
            let right = self.parse_unary_expr()?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr, MtpError> {
        if self.match_token(TokenKind::Bang)? || self.match_token(TokenKind::Minus)? {
            let op = if self.previous().kind == TokenKind::Bang {
                UnOp::Not
            } else {
                UnOp::Neg
            };
            let expr = self.parse_unary_expr()?;
            Ok(Expr::Unary {
                op,
                expr: Box::new(expr),
            })
        } else {
            self.parse_call_expr()
        }
    }

    fn parse_call_expr(&mut self) -> Result<Expr, MtpError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(TokenKind::LeftParen)? {
                let mut args = Vec::new();
                if !self.check(TokenKind::RightParen) {
                    args.push(self.parse_expr()?);
                    while self.match_token(TokenKind::Comma)? {
                        args.push(self.parse_expr()?);
                    }
                }
                self.consume(TokenKind::RightParen)?;
                expr = Expr::Call {
                    func: self.extract_ident(&expr)?,
                    args,
                };
            } else if self.match_token(TokenKind::LeftBracket)? {
                let index = self.parse_expr()?;
                self.consume(TokenKind::RightBracket)?;
                // TODO: Implement array access
                expr = Expr::Call {
                    func: "array_get".to_string(),
                    args: vec![expr, index],
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, MtpError> {
        match &self.current_token().kind {
            TokenKind::String(s) => {
                self.advance();
                Ok(Expr::StringLit(s.clone()))
            }
            TokenKind::Number(n) => {
                self.advance();
                Ok(Expr::NumberLit(n.clone()))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::BoolLit(true))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::BoolLit(false))
            }
            TokenKind::Identifier(name) => {
                self.advance();
                Ok(Expr::Ident(name.clone()))
            }
            TokenKind::LeftBracket => {
                self.advance();
                let mut elements = Vec::new();
                if !self.check(TokenKind::RightBracket) {
                    elements.push(self.parse_expr()?);
                    while self.match_token(TokenKind::Comma)? {
                        elements.push(self.parse_expr()?);
                    }
                }
                self.consume(TokenKind::RightBracket)?;
                Ok(Expr::Array(elements))
            }
            TokenKind::LeftBrace => {
                self.advance();
                let mut fields = Vec::new();
                if !self.check(TokenKind::RightBrace) {
                    let key = self.consume_string()?;
                    self.consume(TokenKind::Colon)?;
                    let value = self.parse_expr()?;
                    fields.push((key, value));
                    while self.match_token(TokenKind::Comma)? {
                        let key = self.consume_string()?;
                        self.consume(TokenKind::Colon)?;
                        let value = self.parse_expr()?;
                        fields.push((key, value));
                    }
                }
                self.consume(TokenKind::RightBrace)?;
                Ok(Expr::Object(fields))
            }
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.consume(TokenKind::RightParen)?;
                Ok(expr)
            }
            _ => Err(MtpError::ParseError {
                message: format!("Unexpected token in primary: {:?}", self.current_token().kind),
            }),
        }
    }
            TokenKind::Number(n) => {
                self.advance();
                Ok(Expr::NumberLit(n.clone()))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::BoolLit(true))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::BoolLit(false))
            }
            TokenKind::Identifier(name) if name == "const" => {
                self.advance();
                let var_name = self.consume_identifier()?;
                self.consume(TokenKind::Equal)?;
                let value = self.parse_expr()?;
                // For now, treat const as a block with variable assignment
                Ok(Expr::Block(vec![Expr::Call {
                    func: format!("assign_{}", var_name),
                    args: vec![value],
                }]))
            }
            TokenKind::Identifier(name) => {
                self.advance();
                Ok(Expr::Ident(name.clone()))
            }
            TokenKind::LeftBracket => {
                self.advance();
                let mut elements = Vec::new();
                if !self.check(TokenKind::RightBracket) {
                    elements.push(self.parse_expr()?);
                    while self.match_token(TokenKind::Comma)? {
                        elements.push(self.parse_expr()?);
                    }
                }
                self.consume(TokenKind::RightBracket)?;
                Ok(Expr::Array(elements))
            }
            TokenKind::LeftBrace => {
                self.advance();
                let mut fields = Vec::new();
                if !self.check(TokenKind::RightBrace) {
                    let key = self.consume_string()?;
                    self.consume(TokenKind::Colon)?;
                    let value = self.parse_expr()?;
                    fields.push((key, value));
                    while self.match_token(TokenKind::Comma)? {
                        let key = self.consume_string()?;
                        self.consume(TokenKind::Colon)?;
                        let value = self.parse_expr()?;
                        fields.push((key, value));
                    }
                }
                self.consume(TokenKind::RightBrace)?;
                Ok(Expr::Object(fields))
            }
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.consume(TokenKind::RightParen)?;
                Ok(expr)
            }
            _ => Err(MtpError::ParseError {
                message: format!(
                    "Unexpected token in primary: {:?}",
                    self.current_token().kind
                ),
            }),
        }
    }

    fn parse_pattern(&mut self) -> Result<Pattern, MtpError> {
        match &self.current_token().kind {
            TokenKind::Identifier(name) => {
                self.advance();
                if name.chars().next().unwrap().is_uppercase() {
                    // ADT constructor
                    let mut sub_patterns = Vec::new();
                    if self.match_token(TokenKind::LeftParen)? {
                        sub_patterns.push(self.parse_pattern()?);
                        while self.match_token(TokenKind::Comma)? {
                            sub_patterns.push(self.parse_pattern()?);
                        }
                        self.consume(TokenKind::RightParen)?;
                    }
                    Ok(Pattern::Variant(name.clone(), sub_patterns))
                } else {
                    Ok(Pattern::Ident(name.clone()))
                }
            }
            TokenKind::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            _ => {
                let expr = self.parse_primary()?;
                Ok(Pattern::Lit(expr))
            }
        }
    }

    // Helper methods
    fn current_token(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || self.current_token().kind == TokenKind::Eof
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.current += 1;
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        !self.is_at_end() && self.current_token().kind == kind
    }

    fn match_token(&mut self, kind: TokenKind) -> Result<bool, MtpError> {
        if self.check(kind) {
            self.advance();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn consume(&mut self, kind: TokenKind) -> Result<(), MtpError> {
        if self.check(kind) {
            self.advance();
            Ok(())
        } else {
            Err(MtpError::ParseError {
                message: format!("Expected {:?}, found {:?}", kind, self.current_token().kind),
            })
        }
    }

    fn consume_identifier(&mut self) -> Result<String, MtpError> {
        if let TokenKind::Identifier(name) = &self.current_token().kind {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(MtpError::ParseError {
                message: "Expected identifier".to_string(),
            })
        }
    }

    fn consume_string(&mut self) -> Result<String, MtpError> {
        if let TokenKind::String(s) = &self.current_token().kind {
            let s = s.clone();
            self.advance();
            Ok(s)
        } else {
            Err(MtpError::ParseError {
                message: "Expected string".to_string(),
            })
        }
    }

    // Stub implementations for missing parts
    fn parse_type_params(&mut self) -> Result<Vec<String>, MtpError> {
        Ok(Vec::new())
    }

    fn parse_params(&mut self) -> Result<Vec<(String, Type)>, MtpError> {
        let mut params = Vec::new();
        if !self.check(TokenKind::RightParen) {
            let name = self.consume_identifier()?;
            self.consume(TokenKind::Colon)?;
            let ty = self.parse_type()?;
            params.push((name, ty));
            while self.match_token(TokenKind::Comma)? {
                let name = self.consume_identifier()?;
                self.consume(TokenKind::Colon)?;
                let ty = self.parse_type()?;
                params.push((name, ty));
            }
        }
        Ok(params)
    }

    fn parse_effects(&mut self) -> Result<Vec<String>, MtpError> {
        let mut effects = Vec::new();
        self.consume(TokenKind::LeftBrace)?;
        while !self.check(TokenKind::RightBrace) {
            effects.push(self.consume_identifier()?);
            if !self.match_token(TokenKind::Comma)? {
                break;
            }
        }
        self.consume(TokenKind::RightBrace)?;
        Ok(effects)
    }

    fn parse_type(&mut self) -> Result<Type, MtpError> {
        // Stub - return Number for now
        Ok(Type::Number)
    }

    fn is_binary_op(&self) -> bool {
        matches!(
            self.current_token().kind,
            TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::EqualEqual
                | TokenKind::BangEqual
                | TokenKind::Less
                | TokenKind::LessEqual
                | TokenKind::Greater
                | TokenKind::GreaterEqual
                | TokenKind::AmpAmp
                | TokenKind::PipePipe
                | TokenKind::PipeGreater
        )
    }

    fn parse_binop(&mut self) -> Result<BinOp, MtpError> {
        let op = match self.current_token().kind {
            TokenKind::Plus => BinOp::Add,
            TokenKind::Minus => BinOp::Sub,
            TokenKind::Star => BinOp::Mul,
            TokenKind::Slash => BinOp::Div,
            TokenKind::EqualEqual => BinOp::Eq,
            TokenKind::BangEqual => BinOp::Ne,
            TokenKind::Less => BinOp::Lt,
            TokenKind::LessEqual => BinOp::Le,
            TokenKind::Greater => BinOp::Gt,
            TokenKind::GreaterEqual => BinOp::Ge,
            TokenKind::AmpAmp => BinOp::And,
            TokenKind::PipePipe => BinOp::Or,
            TokenKind::PipeGreater => BinOp::Pipe,
            _ => {
                return Err(MtpError::ParseError {
                    message: "Expected binary operator".to_string(),
                })
            }
        };
        self.advance();
        Ok(op)
    }

    fn extract_ident(&self, expr: &Expr) -> Result<String, MtpError> {
        if let Expr::Ident(name) = expr {
            Ok(name.clone())
        } else {
            Err(MtpError::ParseError {
                message: "Expected identifier".to_string(),
            })
        }
    }
}
