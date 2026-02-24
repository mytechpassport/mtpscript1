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
            // Handle const declarations
            if self.match_token(TokenKind::Const)? {
                let var_name = self.consume_identifier()?;
                self.consume(TokenKind::Equal)?;
                let value = self.parse_expr()?;
                exprs.push(Expr::Call {
                    func: format!("const_{}", var_name),
                    args: vec![value],
                });
            } else if self.check(TokenKind::Function) {
                // Handle local function declarations
                self.advance();
                // Check if it's an anonymous function (lambda) or named function
                if self.check(TokenKind::LeftParen) {
                    // Anonymous function - treat as expression
                    self.consume(TokenKind::LeftParen)?;
                    let params = self.parse_lambda_params()?;
                    self.consume(TokenKind::RightParen)?;
                    self.consume(TokenKind::LeftBrace)?;
                    let body = self.parse_block()?;
                    self.consume(TokenKind::RightBrace)?;
                    exprs.push(Expr::Lambda {
                        params,
                        body: Box::new(body),
                    });
                } else {
                    // Named function declaration
                    let func_name = self.consume_identifier()?;
                    self.consume(TokenKind::LeftParen)?;
                    let params = self.parse_lambda_params()?;
                    self.consume(TokenKind::RightParen)?;
                    // Optional return type
                    if self.match_token(TokenKind::Colon)? {
                        self.skip_type_annotation()?;
                    }
                    self.consume(TokenKind::LeftBrace)?;
                    let body = self.parse_block()?;
                    self.consume(TokenKind::RightBrace)?;
                    exprs.push(Expr::Call {
                        func: format!("defn_{}", func_name),
                        args: vec![Expr::Lambda {
                            params,
                            body: Box::new(body),
                        }],
                    });
                }
            } else if self.check(TokenKind::Type) {
                // Handle local type declarations
                self.advance();
                let type_name = self.consume_identifier()?;
                // Skip the type body
                if self.match_token(TokenKind::Equal)? {
                    // ADT type: type Foo = Bar | Baz
                    self.skip_type_body()?;
                } else if self.check(TokenKind::LeftBrace) {
                    // Record type: type Foo { field: Type }
                    self.advance();
                    while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
                        self.consume_identifier()?; // field name
                        self.consume(TokenKind::Colon)?;
                        self.skip_type_annotation()?;
                        // Fields can be separated by whitespace or comma
                        self.match_token(TokenKind::Comma)?;
                    }
                    self.consume(TokenKind::RightBrace)?;
                }
                // Record type declaration as a call
                exprs.push(Expr::Call {
                    func: format!("type_{}", type_name),
                    args: vec![],
                });
            } else {
                exprs.push(self.parse_expr()?);
            }
            // Skip optional semicolons
            self.match_token(TokenKind::Semicolon)?;
        }
        Ok(Expr::Block(exprs))
    }

    fn skip_type_body(&mut self) -> Result<(), MtpError> {
        // Skip ADT variant definitions: Foo | Bar(Type) | Baz
        loop {
            // Skip variant name
            if let TokenKind::Identifier(_) = &self.current_token().kind {
                self.advance();
            } else {
                break;
            }

            // Check for payload type
            if self.check(TokenKind::LeftParen) {
                self.advance();
                let mut depth = 1;
                while depth > 0 && !self.is_at_end() {
                    match self.current_token().kind {
                        TokenKind::LeftParen => depth += 1,
                        TokenKind::RightParen => depth -= 1,
                        _ => {}
                    }
                    self.advance();
                }
            }

            // Check for more variants
            if !self.match_token(TokenKind::PipePipe)? {
                // Also try single pipe (|)
                if let TokenKind::Identifier(name) = &self.current_token().kind {
                    if name == "|" {
                        self.advance();
                        continue;
                    }
                }
                break;
            }
        }
        Ok(())
    }

    fn parse_match_expr(&mut self) -> Result<Expr, MtpError> {
        if self.match_token(TokenKind::Match)? {
            let expr = self.parse_expr()?;
            self.consume(TokenKind::LeftBrace)?;
            let mut cases = Vec::new();
            while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
                let pattern = self.parse_pattern()?;
                self.consume(TokenKind::EqualGreater)?;

                // Match arm body can be a block or a simple expression
                let case_expr = if self.check(TokenKind::LeftBrace) {
                    // Check if it's an object literal or a block
                    let saved = self.current;
                    self.advance();
                    if self.check(TokenKind::RightBrace) {
                        // Empty block
                        self.advance();
                        Expr::Block(Vec::new())
                    } else if self.is_object_literal() {
                        // Object literal - go back and parse as expression
                        self.current = saved;
                        self.parse_expr()?
                    } else {
                        // Block expression
                        let block = self.parse_block()?;
                        self.consume(TokenKind::RightBrace)?;
                        block
                    }
                } else {
                    self.parse_expr()?
                };

                cases.push((pattern, case_expr));

                // Match cases are separated by commas or newlines (we allow optional comma)
                self.match_token(TokenKind::Comma)?;
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
            // Handle parenthesized condition: if (cond) { ... }
            let has_paren = self.match_token(TokenKind::LeftParen)?;
            let cond = self.parse_expr()?;
            if has_paren {
                self.consume(TokenKind::RightParen)?;
            }

            // Handle both 'then' keyword and block-based if expressions
            let then_branch = if self.match_token(TokenKind::Then)? {
                self.parse_expr()?
            } else if self.check(TokenKind::LeftBrace) {
                self.advance();
                let block = self.parse_block()?;
                self.consume(TokenKind::RightBrace)?;
                block
            } else {
                self.parse_expr()?
            };

            let else_branch = if self.match_token(TokenKind::Else)? {
                if self.check(TokenKind::LeftBrace) {
                    self.advance();
                    let block = self.parse_block()?;
                    self.consume(TokenKind::RightBrace)?;
                    Some(block)
                } else {
                    Some(self.parse_expr()?)
                }
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
                let s = s.clone();
                self.advance();
                Ok(Expr::StringLit(s))
            }
            TokenKind::Number(n) => {
                let n = n.clone();
                self.advance();
                Ok(Expr::NumberLit(n))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::BoolLit(true))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::BoolLit(false))
            }
            // Handle ADT constructors (Ok, Err, Some, None)
            TokenKind::Ok | TokenKind::Err | TokenKind::Some | TokenKind::None => {
                let name = match &self.current_token().kind {
                    TokenKind::Ok => "Ok".to_string(),
                    TokenKind::Err => "Err".to_string(),
                    TokenKind::Some => "Some".to_string(),
                    TokenKind::None => "None".to_string(),
                    _ => unreachable!(),
                };
                self.advance();
                // Check if it has arguments
                if self.check(TokenKind::LeftParen) {
                    self.advance();
                    let mut args = Vec::new();
                    if !self.check(TokenKind::RightParen) {
                        args.push(self.parse_expr()?);
                        while self.match_token(TokenKind::Comma)? {
                            args.push(self.parse_expr()?);
                        }
                    }
                    self.consume(TokenKind::RightParen)?;
                    Ok(Expr::Call { func: name, args })
                } else {
                    Ok(Expr::Ident(name))
                }
            }
            // Handle inline function expressions
            TokenKind::Function => {
                self.advance();
                self.consume(TokenKind::LeftParen)?;
                let params = self.parse_lambda_params()?;
                self.consume(TokenKind::RightParen)?;
                self.consume(TokenKind::LeftBrace)?;
                let body = self.parse_block()?;
                self.consume(TokenKind::RightBrace)?;
                Ok(Expr::Lambda {
                    params,
                    body: Box::new(body),
                })
            }
            TokenKind::Respond => {
                self.advance();
                let expr = self.parse_expr()?;
                Ok(Expr::Respond(Box::new(expr)))
            }
            TokenKind::Await => {
                self.advance();
                let expr = self.parse_expr()?;
                Ok(Expr::Call {
                    func: "await".to_string(),
                    args: vec![expr],
                })
            }
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                // Check for dot access (method calls or field access)
                if self.check(TokenKind::Dot) {
                    self.advance();
                    let field = self.consume_identifier()?;
                    // Check if it's a method call
                    if self.check(TokenKind::LeftParen) {
                        self.advance();
                        let mut args = Vec::new();
                        if !self.check(TokenKind::RightParen) {
                            args.push(self.parse_expr()?);
                            while self.match_token(TokenKind::Comma)? {
                                args.push(self.parse_expr()?);
                            }
                        }
                        self.consume(TokenKind::RightParen)?;
                        // Convert to namespaced function call: Decimal.add -> Decimal_add
                        Ok(Expr::Call {
                            func: format!("{}.{}", name, field),
                            args,
                        })
                    } else {
                        // Field access - convert to property_get call
                        Ok(Expr::Call {
                            func: "property_get".to_string(),
                            args: vec![Expr::Ident(name), Expr::StringLit(field)],
                        })
                    }
                } else {
                    Ok(Expr::Ident(name))
                }
            }
            TokenKind::LeftBracket => {
                self.advance();
                let mut elements = Vec::new();
                if !self.check(TokenKind::RightBracket) {
                    elements.push(self.parse_expr()?);
                    while self.match_token(TokenKind::Comma)? {
                        if self.check(TokenKind::RightBracket) {
                            break; // Allow trailing comma
                        }
                        elements.push(self.parse_expr()?);
                    }
                }
                self.consume(TokenKind::RightBracket)?;
                Ok(Expr::Array(elements))
            }
            TokenKind::LeftBrace => {
                self.advance();
                // Check for empty block/object
                if self.check(TokenKind::RightBrace) {
                    self.advance();
                    return Ok(Expr::Block(Vec::new()));
                }

                // Try to determine if it's an object literal or block expression
                // Object literals start with "key": value or identifier: value
                let is_object = self.is_object_literal();

                if is_object {
                    let mut fields = Vec::new();
                    loop {
                        if self.check(TokenKind::RightBrace) {
                            break;
                        }
                        // Key can be string or identifier
                        let key = if let TokenKind::String(s) = &self.current_token().kind {
                            let s = s.clone();
                            self.advance();
                            s
                        } else {
                            self.consume_identifier()?
                        };
                        self.consume(TokenKind::Colon)?;
                        let value = self.parse_expr()?;
                        fields.push((key, value));
                        if !self.match_token(TokenKind::Comma)? {
                            break;
                        }
                    }
                    self.consume(TokenKind::RightBrace)?;
                    Ok(Expr::Object(fields))
                } else {
                    // It's a block expression
                    let mut exprs = Vec::new();
                    while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
                        // Handle const declarations
                        if self.match_token(TokenKind::Const)? {
                            let var_name = self.consume_identifier()?;
                            self.consume(TokenKind::Equal)?;
                            let value = self.parse_expr()?;
                            exprs.push(Expr::Call {
                                func: format!("const_{}", var_name),
                                args: vec![value],
                            });
                        } else if self.check(TokenKind::Function) {
                            // Handle local function declarations
                            self.advance();
                            let func_name = self.consume_identifier()?;
                            self.consume(TokenKind::LeftParen)?;
                            let params = self.parse_lambda_params()?;
                            self.consume(TokenKind::RightParen)?;
                            self.consume(TokenKind::LeftBrace)?;
                            let body = self.parse_block()?;
                            self.consume(TokenKind::RightBrace)?;
                            exprs.push(Expr::Call {
                                func: format!("defn_{}", func_name),
                                args: vec![Expr::Lambda {
                                    params,
                                    body: Box::new(body),
                                }],
                            });
                        } else {
                            exprs.push(self.parse_expr()?);
                        }
                        // Skip optional semicolons
                        self.match_token(TokenKind::Semicolon)?;
                    }
                    self.consume(TokenKind::RightBrace)?;
                    Ok(Expr::Block(exprs))
                }
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

    fn is_object_literal(&self) -> bool {
        // Look ahead to determine if this is an object literal
        // Object literal: { "key": value } or { key: value }
        // Block: { expr; expr } or { const x = ...; expr }

        let saved_pos = self.current;

        // Check first token
        match &self.tokens.get(saved_pos).map(|t| &t.kind) {
            Some(TokenKind::String(_)) => {
                // Check if next is colon
                matches!(
                    self.tokens.get(saved_pos + 1).map(|t| &t.kind),
                    Some(TokenKind::Colon)
                )
            }
            Some(TokenKind::Identifier(_)) => {
                // Check if next is colon (not function call or assignment)
                matches!(
                    self.tokens.get(saved_pos + 1).map(|t| &t.kind),
                    Some(TokenKind::Colon)
                )
            }
            _ => false,
        }
    }

    fn parse_lambda_params(&mut self) -> Result<Vec<String>, MtpError> {
        let mut params = Vec::new();
        if !self.check(TokenKind::RightParen) {
            let name = self.consume_identifier()?;
            // Skip type annotation if present
            if self.match_token(TokenKind::Colon)? {
                self.skip_type_annotation()?;
            }
            params.push(name);
            while self.match_token(TokenKind::Comma)? {
                let name = self.consume_identifier()?;
                if self.match_token(TokenKind::Colon)? {
                    self.skip_type_annotation()?;
                }
                params.push(name);
            }
        }
        Ok(params)
    }

    fn skip_type_annotation(&mut self) -> Result<(), MtpError> {
        // Skip type annotations like: number, string, List<number>, function(number): number
        loop {
            match &self.current_token().kind {
                TokenKind::Identifier(_) | TokenKind::Number(_) => {
                    self.advance();
                }
                TokenKind::Less => {
                    // Generic type args
                    self.advance();
                    let mut depth = 1;
                    while depth > 0 && !self.is_at_end() {
                        match self.current_token().kind {
                            TokenKind::Less => depth += 1,
                            TokenKind::Greater => depth -= 1,
                            _ => {}
                        }
                        self.advance();
                    }
                }
                TokenKind::LeftParen => {
                    // Function type
                    self.advance();
                    let mut depth = 1;
                    while depth > 0 && !self.is_at_end() {
                        match self.current_token().kind {
                            TokenKind::LeftParen => depth += 1,
                            TokenKind::RightParen => depth -= 1,
                            _ => {}
                        }
                        self.advance();
                    }
                    // Check for return type
                    if self.match_token(TokenKind::Colon)? {
                        self.skip_type_annotation()?;
                    }
                    return Ok(());
                }
                _ => break,
            }
        }
        Ok(())
    }

    fn parse_pattern(&mut self) -> Result<Pattern, MtpError> {
        match &self.current_token().kind {
            TokenKind::Ok | TokenKind::Err | TokenKind::Some | TokenKind::None => {
                let name = match &self.current_token().kind {
                    TokenKind::Ok => "Ok".to_string(),
                    TokenKind::Err => "Err".to_string(),
                    TokenKind::Some => "Some".to_string(),
                    TokenKind::None => "None".to_string(),
                    _ => unreachable!(),
                };
                self.advance();
                let mut sub_patterns = Vec::new();
                if self.match_token(TokenKind::LeftParen)? {
                    if !self.check(TokenKind::RightParen) {
                        sub_patterns.push(self.parse_pattern()?);
                        while self.match_token(TokenKind::Comma)? {
                            sub_patterns.push(self.parse_pattern()?);
                        }
                    }
                    self.consume(TokenKind::RightParen)?;
                }
                Ok(Pattern::Variant(name, sub_patterns))
            }
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                if name.chars().next().unwrap().is_uppercase() {
                    // ADT constructor or record pattern
                    if self.check(TokenKind::LeftBrace) {
                        // Record pattern: Person { name: n, age: a }
                        self.advance();
                        let mut fields = Vec::new();
                        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
                            let field_name = self.consume_identifier()?;
                            self.consume(TokenKind::Colon)?;
                            let field_pattern = self.parse_pattern()?;
                            fields.push((field_name, field_pattern));
                            if !self.match_token(TokenKind::Comma)? {
                                break;
                            }
                        }
                        self.consume(TokenKind::RightBrace)?;
                        Ok(Pattern::Record(fields))
                    } else {
                        // ADT constructor
                        let mut sub_patterns = Vec::new();
                        if self.match_token(TokenKind::LeftParen)? {
                            if !self.check(TokenKind::RightParen) {
                                sub_patterns.push(self.parse_pattern()?);
                                while self.match_token(TokenKind::Comma)? {
                                    sub_patterns.push(self.parse_pattern()?);
                                }
                            }
                            self.consume(TokenKind::RightParen)?;
                        }
                        Ok(Pattern::Variant(name, sub_patterns))
                    }
                } else {
                    Ok(Pattern::Ident(name))
                }
            }
            TokenKind::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            TokenKind::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(Pattern::Lit(Expr::StringLit(s)))
            }
            TokenKind::Number(n) => {
                let n = n.clone();
                self.advance();
                Ok(Pattern::Lit(Expr::NumberLit(n)))
            }
            TokenKind::True => {
                self.advance();
                Ok(Pattern::Lit(Expr::BoolLit(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Pattern::Lit(Expr::BoolLit(false)))
            }
            _ => Err(MtpError::ParseError {
                message: format!("Unexpected token in pattern: {:?}", self.current_token().kind),
            }),
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
        if self.is_at_end() {
            return false;
        }
        match (&kind, &self.current_token().kind) {
            (TokenKind::Equal, TokenKind::Equal) => true,
            (TokenKind::Underscore, TokenKind::Underscore) => true,
            (TokenKind::Then, TokenKind::Then) => true,
            (TokenKind::Ok, TokenKind::Ok) => true,
            (TokenKind::Err, TokenKind::Err) => true,
            (TokenKind::Some, TokenKind::Some) => true,
            (TokenKind::None, TokenKind::None) => true,
            _ => self.current_token().kind == kind,
        }
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
        // Handle special case for Equal token matching
        let matches = match (&kind, &self.current_token().kind) {
            (TokenKind::Equal, TokenKind::Equal) => true,
            _ => self.check(kind.clone()),
        };

        if matches {
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
