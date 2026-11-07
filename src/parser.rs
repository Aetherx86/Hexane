// Copyright (c) 2026 jk9576. All rights reserved.
// Hexane Language Toolchain
// Licensed under the MIT License.

use crate::ast::*;
use crate::diagnostics::{Diagnostic, Span};
use crate::token::{Token, TokenKind};

pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
    filename: String,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token], filename: impl Into<String>) -> Self {
        Self {
            tokens,
            current: 0,
            filename: filename.into(),
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    fn is_at_end(&self) -> bool {
        self.peek_kind() == &TokenKind::Eof
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek_kind() == kind
        }
    }

    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn error(&self, span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic::error(&self.filename, span, message)
    }

    fn expect(&mut self, kind: &TokenKind, msg: &str) -> Result<&Token, Diagnostic> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            let token = self.peek();
            Err(self.error(
                token.span,
                format!("expected '{}', found '{}' ({})", kind, token.kind, msg),
            ))
        }
    }

    // Top level
    pub fn parse_program(&mut self) -> Result<Program, Diagnostic> {
        let mut program = Program::new();

        while !self.is_at_end() {
            match self.peek_kind() {
                TokenKind::Package => {
                    program.packages.push(self.parse_package()?);
                }
                TokenKind::Import => {
                    program.imports.push(self.parse_import()?);
                }
                TokenKind::Class | TokenKind::Struct => {
                    program.classes.push(self.parse_class()?);
                }
                TokenKind::Extern => {
                    self.advance();
                    if let TokenKind::StringLit(_) = self.peek_kind() {
                        self.advance();
                    }
                    program.functions.push(self.parse_function(true)?);
                }
                TokenKind::Fn => {
                    program.functions.push(self.parse_function(false)?);
                }
                TokenKind::Var | TokenKind::Let => {
                    program.globals.push(self.parse_var_decl()?);
                }
                TokenKind::Semicolon => {
                    self.advance();
                }
                _ => {
                    let token = self.peek();
                    return Err(self.error(
                        token.span,
                        format!("unexpected top-level item '{}'", token.kind),
                    ));
                }
            }
        }

        Ok(program)
    }

    fn parse_package(&mut self) -> Result<PackageDecl, Diagnostic> {
        self.expect(&TokenKind::Package, "package declaration")?;
        let name = self.parse_identifier("package name")?;
        let mut library = None;
        let mut is_api = false;

        if self.match_token(&TokenKind::Api) {
            is_api = true;
        } else if self.match_token(&TokenKind::Library) {
            if let TokenKind::StringLit(lib) = self.peek_kind().clone() {
                self.advance();
                library = Some(lib);
            } else {
                return Err(self.error(self.peek().span, "expected library name string"));
            }
        }

        self.expect(&TokenKind::Semicolon, "after package declaration")?;
        Ok(PackageDecl {
            name,
            library,
            is_api,
        })
    }

    fn parse_import(&mut self) -> Result<ImportDecl, Diagnostic> {
        self.expect(&TokenKind::Import, "import declaration")?;
        let package = self.parse_identifier("import package name")?;
        let mut library = None;

        if self.match_token(&TokenKind::Library) {
            if let TokenKind::StringLit(lib) = self.peek_kind().clone() {
                self.advance();
                library = Some(lib);
            } else {
                return Err(self.error(self.peek().span, "expected library name string"));
            }
        }

        self.expect(&TokenKind::Semicolon, "after import declaration")?;
        Ok(ImportDecl { package, library })
    }

    fn parse_class(&mut self) -> Result<ClassDef, Diagnostic> {
        let is_class = self.match_token(&TokenKind::Class);
        if !is_class {
            self.expect(&TokenKind::Struct, "class or struct")?;
        }
        let start_span = self.tokens[self.current - 1].span;
        let name = self.parse_identifier("class/struct name")?;
        self.expect(&TokenKind::LBrace, "opening '{' of class body")?;

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            if self.match_token(&TokenKind::Impl) {
                // Carbon "impl as ..."
                // Read until '{' and matching '}'
                while !self.check(&TokenKind::LBrace) && !self.is_at_end() {
                    self.advance();
                }
                if self.match_token(&TokenKind::LBrace) {
                    let mut depth = 1;
                    while depth > 0 && !self.is_at_end() {
                        if self.check(&TokenKind::LBrace) {
                            depth += 1;
                        } else if self.check(&TokenKind::RBrace) {
                            depth -= 1;
                        }
                        self.advance();
                    }
                }
            } else if self.check(&TokenKind::Fn) {
                methods.push(self.parse_function(false)?);
            } else if self.match_token(&TokenKind::Var) {
                let field_name = self.parse_identifier("field name")?;
                self.expect(&TokenKind::Colon, "':' after field name")?;
                let ty = self.parse_type()?;
                self.expect(&TokenKind::Semicolon, "';' after field declaration")?;
                fields.push((field_name, ty));
            } else if self.match_token(&TokenKind::Semicolon) {
                // empty statement
            } else {
                let token = self.peek();
                return Err(self.error(
                    token.span,
                    format!("unexpected token '{}' in class body", token.kind),
                ));
            }
        }

        let end_token = self.expect(&TokenKind::RBrace, "closing '}' of class body")?;
        let span = Span::new(
            start_span.line,
            start_span.col,
            start_span.start,
            end_token.span.end,
        );

        Ok(ClassDef {
            name,
            fields,
            methods,
            span,
        })
    }

    fn parse_function(&mut self, mut is_extern: bool) -> Result<Function, Diagnostic> {
        let fn_token = self.expect(&TokenKind::Fn, "function declaration")?;
        let start_span = fn_token.span;
        let name = self.parse_identifier("function name")?;

        self.expect(&TokenKind::LParen, "'(' before parameters")?;
        let mut params = Vec::new();

        if !self.check(&TokenKind::RParen) {
            loop {
                let mut is_ref = false;
                if self.match_token(&TokenKind::Ref) {
                    is_ref = true;
                }
                if self.match_token(&TokenKind::Var) {
                    // Carbon syntax for mutable parameter, e.g. fn Foo(var n: i32)
                }

                if self.match_token(&TokenKind::SelfKw) {
                    params.push(Param {
                        name: "self".to_string(),
                        ty: Type::Custom("self".to_string()),
                        is_ref,
                        is_self: true,
                    });
                } else {
                    let param_name = self.parse_identifier("parameter name")?;
                    self.expect(&TokenKind::Colon, "':' after parameter name")?;
                    let ty = self.parse_type()?;
                    params.push(Param {
                        name: param_name,
                        ty,
                        is_ref,
                        is_self: false,
                    });
                }

                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(&TokenKind::RParen, "')' after parameters")?;

        let mut return_type = Type::Void;
        if self.match_token(&TokenKind::Arrow) {
            return_type = self.parse_type()?;
        }

        let mut body = Vec::new();
        let end_span;

        if is_extern || self.check(&TokenKind::Semicolon) {
            let semi = self.expect(&TokenKind::Semicolon, "';' after function declaration")?;
            is_extern = true;
            end_span = semi.span.end;
        } else {
            self.expect(&TokenKind::LBrace, "'{' before function body")?;
            while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                body.push(self.parse_statement()?);
            }
            let end_token = self.expect(&TokenKind::RBrace, "'}' after function body")?;
            end_span = end_token.span.end;
        }

        let span = Span::new(start_span.line, start_span.col, start_span.start, end_span);

        Ok(Function {
            name,
            params,
            return_type,
            body,
            is_extern,
            span,
        })
    }

    fn parse_type(&mut self) -> Result<Type, Diagnostic> {
        let token = self.advance().clone();
        let mut ty = match token.kind {
            TokenKind::I8 => Type::I8,
            TokenKind::I16 => Type::I16,
            TokenKind::I32 => Type::I32,
            TokenKind::I64 => Type::I64,
            TokenKind::U8 => Type::U8,
            TokenKind::U16 => Type::U16,
            TokenKind::U32 => Type::U32,
            TokenKind::U64 => Type::U64,
            TokenKind::Int => Type::Int,
            TokenKind::F32 => Type::F32,
            TokenKind::F64 => Type::F64,
            TokenKind::Float => Type::F64,
            TokenKind::BoolType => Type::Bool,
            TokenKind::StringType | TokenKind::StringViewType => Type::String,
            TokenKind::Auto => Type::Auto,
            TokenKind::Void => Type::Void,
            TokenKind::Array => {
                self.expect(&TokenKind::LParen, "'(' after array")?;
                let elem_type = self.parse_type()?;
                self.expect(&TokenKind::Comma, "',' in array(Type, Size)")?;
                let size_tok = self.advance().clone();
                let size = match size_tok.kind {
                    TokenKind::Integer(n) if n >= 0 => n as usize,
                    _ => {
                        return Err(
                            self.error(size_tok.span, "array size must be a non-negative integer")
                        );
                    }
                };
                self.expect(&TokenKind::RParen, "')' closing array(Type, Size)")?;
                Type::Array(Box::new(elem_type), size)
            }
            TokenKind::LBracket => {
                // Carbon [Type; Size]
                let elem_type = self.parse_type()?;
                self.expect(&TokenKind::Semicolon, "';' in [Type; Size]")?;
                let size_tok = self.advance().clone();
                let size = match size_tok.kind {
                    TokenKind::Integer(n) if n >= 0 => n as usize,
                    _ => {
                        return Err(self.error(size_tok.span, "array size must be an integer"));
                    }
                };
                self.expect(&TokenKind::RBracket, "']' in [Type; Size]")?;
                Type::Array(Box::new(elem_type), size)
            }
            TokenKind::Ident(ref name) => Type::Custom(name.clone()),
            _ => {
                return Err(self.error(
                    token.span,
                    format!("expected type name, found '{}'", token.kind),
                ));
            }
        };

        // Pointer postfix *
        while self.match_token(&TokenKind::Star) {
            ty = Type::Pointer(Box::new(ty));
        }

        Ok(ty)
    }

    fn parse_statement(&mut self) -> Result<Stmt, Diagnostic> {
        match self.peek_kind() {
            TokenKind::Var | TokenKind::Let | TokenKind::Returned => self.parse_var_decl(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::For => self.parse_for(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Break => {
                let token = self.advance().clone();
                self.expect(&TokenKind::Semicolon, "';' after break")?;
                Ok(Stmt::Break(token.span))
            }
            TokenKind::Continue => {
                let token = self.advance().clone();
                self.expect(&TokenKind::Semicolon, "';' after continue")?;
                Ok(Stmt::Continue(token.span))
            }
            TokenKind::LBrace => {
                self.advance();
                let mut stmts = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                    stmts.push(self.parse_statement()?);
                }
                self.expect(&TokenKind::RBrace, "'}' closing block")?;
                Ok(Stmt::Block(stmts))
            }
            _ => {
                let expr = self.parse_expression()?;
                self.expect(&TokenKind::Semicolon, "';' after expression")?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    fn parse_var_decl(&mut self) -> Result<Stmt, Diagnostic> {
        let is_returned = self.match_token(&TokenKind::Returned);
        let is_let = self.match_token(&TokenKind::Let);
        if !is_let && !is_returned {
            self.expect(&TokenKind::Var, "variable declaration")?;
        } else if is_returned {
            self.expect(&TokenKind::Var, "'var' after 'returned'")?;
        }

        let start_span = self.tokens[self.current - 1].span;
        let name = self.parse_identifier("variable name")?;

        let mut ty = None;
        if self.match_token(&TokenKind::Colon) {
            ty = Some(self.parse_type()?);
        }

        let mut init = None;
        if self.match_token(&TokenKind::Eq) {
            init = Some(self.parse_expression()?);
        }

        let end_token = self.expect(&TokenKind::Semicolon, "';' after variable declaration")?;
        let span = Span::new(
            start_span.line,
            start_span.col,
            start_span.start,
            end_token.span.end,
        );

        Ok(Stmt::VarDecl {
            name,
            ty,
            init,
            is_returned,
            is_let,
            span,
        })
    }

    fn parse_if(&mut self) -> Result<Stmt, Diagnostic> {
        let if_tok = self.expect(&TokenKind::If, "'if'")?;
        let start_span = if_tok.span;

        self.expect(&TokenKind::LParen, "'(' before if condition")?;
        let cond = self.parse_expression()?;
        self.expect(&TokenKind::RParen, "')' after if condition")?;

        self.expect(&TokenKind::LBrace, "'{' before then branch")?;
        let mut then_branch = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            then_branch.push(self.parse_statement()?);
        }
        let rbrace = self.expect(&TokenKind::RBrace, "'}' after then branch")?;

        let mut else_branch = None;
        let mut end_span = rbrace.span;

        if self.match_token(&TokenKind::Else) {
            if self.check(&TokenKind::If) {
                let else_if_stmt = self.parse_if()?;
                end_span = match &else_if_stmt {
                    Stmt::If { span, .. } => *span,
                    _ => unreachable!(),
                };
                else_branch = Some(vec![else_if_stmt]);
            } else {
                self.expect(&TokenKind::LBrace, "'{' before else branch")?;
                let mut stmts = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                    stmts.push(self.parse_statement()?);
                }
                let end_brace = self.expect(&TokenKind::RBrace, "'}' after else branch")?;
                end_span = end_brace.span;
                else_branch = Some(stmts);
            }
        }

        let span = Span::new(
            start_span.line,
            start_span.col,
            start_span.start,
            end_span.end,
        );

        Ok(Stmt::If {
            cond,
            then_branch,
            else_branch,
            span,
        })
    }

    fn parse_while(&mut self) -> Result<Stmt, Diagnostic> {
        let while_tok = self.expect(&TokenKind::While, "'while'")?;
        let start_span = while_tok.span;

        self.expect(&TokenKind::LParen, "'(' before while condition")?;
        let cond = self.parse_expression()?;
        self.expect(&TokenKind::RParen, "')' after while condition")?;

        self.expect(&TokenKind::LBrace, "'{' before while body")?;
        let mut body = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            body.push(self.parse_statement()?);
        }
        let end_tok = self.expect(&TokenKind::RBrace, "'}' after while body")?;

        let span = Span::new(
            start_span.line,
            start_span.col,
            start_span.start,
            end_tok.span.end,
        );

        Ok(Stmt::While { cond, body, span })
    }

    fn parse_for(&mut self) -> Result<Stmt, Diagnostic> {
        let for_tok = self.expect(&TokenKind::For, "'for'")?;
        let start_span = for_tok.span;

        self.expect(&TokenKind::LParen, "'(' before for loop header")?;
        let _ = self.match_token(&TokenKind::Var) || self.match_token(&TokenKind::Let);
        let var_name = self.parse_identifier("loop variable name")?;

        let mut var_ty = None;
        if self.match_token(&TokenKind::Colon) {
            var_ty = Some(self.parse_type()?);
        }

        self.expect(&TokenKind::In, "'in' in for loop")?;
        let iterable = self.parse_expression()?;
        self.expect(&TokenKind::RParen, "')' after for loop header")?;

        self.expect(&TokenKind::LBrace, "'{' before for loop body")?;
        let mut body = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            body.push(self.parse_statement()?);
        }
        let end_tok = self.expect(&TokenKind::RBrace, "'}' after for loop body")?;

        let span = Span::new(
            start_span.line,
            start_span.col,
            start_span.start,
            end_tok.span.end,
        );

        Ok(Stmt::For {
            var_name,
            var_ty,
            iterable,
            body,
            span,
        })
    }

    fn parse_return(&mut self) -> Result<Stmt, Diagnostic> {
        let ret_tok = self.expect(&TokenKind::Return, "'return'")?;
        let start_span = ret_tok.span;

        if self.match_token(&TokenKind::Var) {
            // Carbon "return var;"
            let end_tok = self.expect(&TokenKind::Semicolon, "';' after 'return var'")?;
            return Ok(Stmt::Return {
                value: None,
                is_return_var: true,
                span: Span::new(
                    start_span.line,
                    start_span.col,
                    start_span.start,
                    end_tok.span.end,
                ),
            });
        }

        let mut value = None;
        if !self.check(&TokenKind::Semicolon) {
            value = Some(self.parse_expression()?);
        }

        let end_tok = self.expect(&TokenKind::Semicolon, "';' after return statement")?;
        let span = Span::new(
            start_span.line,
            start_span.col,
            start_span.start,
            end_tok.span.end,
        );

        Ok(Stmt::Return {
            value,
            is_return_var: false,
            span,
        })
    }

    // Expressions
    pub fn parse_expression(&mut self) -> Result<Expr, Diagnostic> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr, Diagnostic> {
        let expr = self.parse_logical_or()?;

        if let Some(op) = self.match_assignment_op() {
            let value = self.parse_assignment()?;
            let span = Span::new(
                expr.span().line,
                expr.span().col,
                expr.span().start,
                value.span().end,
            );
            return Ok(Expr::Assign {
                target: Box::new(expr),
                op,
                value: Box::new(value),
                span,
            });
        }

        Ok(expr)
    }

    fn match_assignment_op(&mut self) -> Option<AssignOp> {
        let op = match self.peek_kind() {
            TokenKind::Eq => Some(AssignOp::Assign),
            TokenKind::PlusEq => Some(AssignOp::AddAssign),
            TokenKind::MinusEq => Some(AssignOp::SubAssign),
            TokenKind::StarEq => Some(AssignOp::MulAssign),
            TokenKind::SlashEq => Some(AssignOp::DivAssign),
            TokenKind::PercentEq => Some(AssignOp::RemAssign),
            TokenKind::AmpEq => Some(AssignOp::BitAndAssign),
            TokenKind::PipeEq => Some(AssignOp::BitOrAssign),
            TokenKind::CaretEq => Some(AssignOp::BitXorAssign),
            TokenKind::ShlEq => Some(AssignOp::ShlAssign),
            TokenKind::ShrEq => Some(AssignOp::ShrAssign),
            _ => None,
        };

        if op.is_some() {
            self.advance();
        }
        op
    }

    fn parse_logical_or(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_logical_and()?;

        while self.match_token(&TokenKind::OrOr) {
            let right = self.parse_logical_and()?;
            let span = Span::new(
                expr.span().line,
                expr.span().col,
                expr.span().start,
                right.span().end,
            );
            expr = Expr::Binary {
                op: BinaryOp::Or,
                left: Box::new(expr),
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_bitwise_or()?;

        while self.match_token(&TokenKind::AndAnd) {
            let right = self.parse_bitwise_or()?;
            let span = Span::new(
                expr.span().line,
                expr.span().col,
                expr.span().start,
                right.span().end,
            );
            expr = Expr::Binary {
                op: BinaryOp::And,
                left: Box::new(expr),
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    fn parse_bitwise_or(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_bitwise_xor()?;

        while self.match_token(&TokenKind::Pipe) {
            let right = self.parse_bitwise_xor()?;
            let span = Span::new(
                expr.span().line,
                expr.span().col,
                expr.span().start,
                right.span().end,
            );
            expr = Expr::Binary {
                op: BinaryOp::BitOr,
                left: Box::new(expr),
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    fn parse_bitwise_xor(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_bitwise_and()?;

        while self.match_token(&TokenKind::Caret) {
            let right = self.parse_bitwise_and()?;
            let span = Span::new(
                expr.span().line,
                expr.span().col,
                expr.span().start,
                right.span().end,
            );
            expr = Expr::Binary {
                op: BinaryOp::BitXor,
                left: Box::new(expr),
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    fn parse_bitwise_and(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_equality()?;

        while self.match_token(&TokenKind::Amp) {
            let right = self.parse_equality()?;
            let span = Span::new(
                expr.span().line,
                expr.span().col,
                expr.span().start,
                right.span().end,
            );
            expr = Expr::Binary {
                op: BinaryOp::BitAnd,
                left: Box::new(expr),
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_relational()?;

        while let Some(op) = match self.peek_kind() {
            TokenKind::EqEq => Some(BinaryOp::Eq),
            TokenKind::NotEq => Some(BinaryOp::NotEq),
            _ => None,
        } {
            self.advance();
            let right = self.parse_relational()?;
            let span = Span::new(
                expr.span().line,
                expr.span().col,
                expr.span().start,
                right.span().end,
            );
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    fn parse_relational(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_shift()?;

        while let Some(op) = match self.peek_kind() {
            TokenKind::Lt => Some(BinaryOp::Lt),
            TokenKind::LtEq => Some(BinaryOp::LtEq),
            TokenKind::Gt => Some(BinaryOp::Gt),
            TokenKind::GtEq => Some(BinaryOp::GtEq),
            _ => None,
        } {
            self.advance();
            let right = self.parse_shift()?;
            let span = Span::new(
                expr.span().line,
                expr.span().col,
                expr.span().start,
                right.span().end,
            );
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    fn parse_shift(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_additive()?;

        while let Some(op) = match self.peek_kind() {
            TokenKind::Shl => Some(BinaryOp::Shl),
            TokenKind::Shr => Some(BinaryOp::Shr),
            _ => None,
        } {
            self.advance();
            let right = self.parse_additive()?;
            let span = Span::new(
                expr.span().line,
                expr.span().col,
                expr.span().start,
                right.span().end,
            );
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    fn parse_additive(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_multiplicative()?;

        while let Some(op) = match self.peek_kind() {
            TokenKind::Plus => Some(BinaryOp::Add),
            TokenKind::Minus => Some(BinaryOp::Sub),
            _ => None,
        } {
            self.advance();
            let right = self.parse_multiplicative()?;
            let span = Span::new(
                expr.span().line,
                expr.span().col,
                expr.span().start,
                right.span().end,
            );
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_unary()?;

        while let Some(op) = match self.peek_kind() {
            TokenKind::Star => Some(BinaryOp::Mul),
            TokenKind::Slash => Some(BinaryOp::Div),
            TokenKind::Percent => Some(BinaryOp::Rem),
            _ => None,
        } {
            self.advance();
            let right = self.parse_unary()?;
            let span = Span::new(
                expr.span().line,
                expr.span().col,
                expr.span().start,
                right.span().end,
            );
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, Diagnostic> {
        if let Some(op) = match self.peek_kind() {
            TokenKind::Minus => Some(UnaryOp::Neg),
            TokenKind::Bang => Some(UnaryOp::Not),
            TokenKind::Tilde => Some(UnaryOp::BitNot),
            TokenKind::PlusPlus => Some(UnaryOp::PreInc),
            TokenKind::MinusMinus => Some(UnaryOp::PreDec),
            TokenKind::Star => Some(UnaryOp::Deref),
            TokenKind::Amp => Some(UnaryOp::AddressOf),
            _ => None,
        } {
            let op_tok = self.advance().clone();
            let right = self.parse_unary()?;
            let span = Span::new(
                op_tok.span.line,
                op_tok.span.col,
                op_tok.span.start,
                right.span().end,
            );
            return Ok(Expr::Unary {
                op,
                expr: Box::new(right),
                span,
            });
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(&TokenKind::LParen) {
                // Function call
                let mut args = Vec::new();
                if !self.check(&TokenKind::RParen) {
                    loop {
                        args.push(self.parse_expression()?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                }
                let end_tok = self.expect(&TokenKind::RParen, "')' after function arguments")?;
                let span = Span::new(
                    expr.span().line,
                    expr.span().col,
                    expr.span().start,
                    end_tok.span.end,
                );

                // Special handling for Core.Range and Core.InclusiveRange
                if let Expr::MemberAccess { object, member, .. } = &expr
                    && let Expr::Ident(obj_name, _) = &**object
                    && obj_name == "Core"
                {
                    if member == "Range" && args.len() == 1 {
                        let start = Box::new(Expr::Integer(0, expr.span()));
                        let end = Box::new(args.remove(0));
                        expr = Expr::Range {
                            start,
                            end,
                            inclusive: false,
                            span,
                        };
                        continue;
                    } else if member == "Range" && args.len() == 2 {
                        let start = Box::new(args.remove(0));
                        let end = Box::new(args.remove(0));
                        expr = Expr::Range {
                            start,
                            end,
                            inclusive: false,
                            span,
                        };
                        continue;
                    } else if member == "InclusiveRange" && args.len() == 2 {
                        let start = Box::new(args.remove(0));
                        let end = Box::new(args.remove(0));
                        expr = Expr::Range {
                            start,
                            end,
                            inclusive: true,
                            span,
                        };
                        continue;
                    }
                }

                expr = Expr::Call {
                    callee: Box::new(expr),
                    args,
                    span,
                };
            } else if self.match_token(&TokenKind::LBracket) {
                // Array index
                let index = self.parse_expression()?;
                let end_tok = self.expect(&TokenKind::RBracket, "']' after index")?;
                let span = Span::new(
                    expr.span().line,
                    expr.span().col,
                    expr.span().start,
                    end_tok.span.end,
                );
                expr = Expr::Index {
                    array: Box::new(expr),
                    index: Box::new(index),
                    span,
                };
            } else if self.match_token(&TokenKind::Dot) {
                // Member access
                let member = self.parse_identifier("member name after '.'")?;
                let span = Span::new(
                    expr.span().line,
                    expr.span().col,
                    expr.span().start,
                    self.tokens[self.current - 1].span.end,
                );
                expr = Expr::MemberAccess {
                    object: Box::new(expr),
                    member,
                    span,
                };
            } else if self.match_token(&TokenKind::PlusPlus) {
                let end_span = self.tokens[self.current - 1].span;
                let span = Span::new(
                    expr.span().line,
                    expr.span().col,
                    expr.span().start,
                    end_span.end,
                );
                expr = Expr::Unary {
                    op: UnaryOp::PostInc,
                    expr: Box::new(expr),
                    span,
                };
            } else if self.match_token(&TokenKind::MinusMinus) {
                let end_span = self.tokens[self.current - 1].span;
                let span = Span::new(
                    expr.span().line,
                    expr.span().col,
                    expr.span().start,
                    end_span.end,
                );
                expr = Expr::Unary {
                    op: UnaryOp::PostDec,
                    expr: Box::new(expr),
                    span,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, Diagnostic> {
        let token = self.peek().clone();
        match token.kind {
            TokenKind::Integer(v) => {
                self.advance();
                Ok(Expr::Integer(v, token.span))
            }
            TokenKind::FloatLit(v) => {
                self.advance();
                Ok(Expr::Float(v, token.span))
            }
            TokenKind::BoolLit(v) => {
                self.advance();
                Ok(Expr::Bool(v, token.span))
            }
            TokenKind::StringLit(ref s) => {
                self.advance();
                Ok(Expr::StringLit(s.clone(), token.span))
            }
            TokenKind::Ident(ref s) => {
                self.advance();
                Ok(Expr::Ident(s.clone(), token.span))
            }
            TokenKind::SelfKw => {
                self.advance();
                Ok(Expr::Ident("self".to_string(), token.span))
            }
            TokenKind::LParen => {
                self.advance();
                let inner = self.parse_expression()?;
                self.expect(&TokenKind::RParen, "')' closing grouped expression")?;
                Ok(inner)
            }
            TokenKind::LBracket => {
                // Array literal [a, b, c]
                self.advance();
                let mut elements = Vec::new();
                if !self.check(&TokenKind::RBracket) {
                    loop {
                        elements.push(self.parse_expression()?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                }
                let end_tok = self.expect(&TokenKind::RBracket, "']' closing array literal")?;
                let span = Span::new(
                    token.span.line,
                    token.span.col,
                    token.span.start,
                    end_tok.span.end,
                );
                Ok(Expr::ArrayLit { elements, span })
            }
            _ => Err(self.error(
                token.span,
                format!("expected expression, found '{}'", token.kind),
            )),
        }
    }

    fn parse_identifier(&mut self, context: &str) -> Result<String, Diagnostic> {
        if let TokenKind::Ident(ref s) = self.peek_kind().clone() {
            let name = s.clone();
            self.advance();
            Ok(name)
        } else {
            let token = self.peek();
            Err(self.error(
                token.span,
                format!("expected identifier in {}, found '{}'", context, token.kind),
            ))
        }
    }
}
