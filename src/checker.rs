// Copyright (c) 2026 jk9576. All rights reserved.
// Hexane Language Toolchain
// Licensed under the MIT License.

use crate::ast::*;
use crate::diagnostics::{Diagnostic, Span};
use std::collections::HashMap;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Symbol {
    name: String,
    ty: Option<Type>,
    is_let: bool,
}

#[derive(Debug)]
struct Scope {
    symbols: HashMap<String, Symbol>,
}

impl Scope {
    fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }
}

pub struct Checker {
    scopes: Vec<Scope>,
    functions: HashMap<String, Function>,
    classes: HashMap<String, ClassDef>,
    filename: String,
    current_return_type: Option<Type>,
    returned_var: Option<(String, Option<Type>)>,
}

impl Checker {
    pub fn new(filename: impl Into<String>) -> Self {
        Self {
            scopes: vec![Scope::new()],
            functions: HashMap::new(),
            classes: HashMap::new(),
            filename: filename.into(),
            current_return_type: None,
            returned_var: None,
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn define_symbol(&mut self, name: &str, ty: Option<Type>, is_let: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.symbols.insert(
                name.to_string(),
                Symbol {
                    name: name.to_string(),
                    ty,
                    is_let,
                },
            );
        }
    }

    fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.symbols.get(name) {
                return Some(sym);
            }
        }
        None
    }

    fn error(&self, span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic::error(&self.filename, span, message)
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), Vec<Diagnostic>> {
        let mut errors = Vec::new();

        // 1. Register classes
        for class in &program.classes {
            if self.classes.contains_key(&class.name) {
                errors.push(self.error(class.span, format!("duplicate class '{}'", class.name)));
            } else {
                self.classes.insert(class.name.clone(), class.clone());
            }
        }

        // 2. Register top-level functions and class methods
        for func in &program.functions {
            if self.functions.contains_key(&func.name) {
                errors.push(self.error(func.span, format!("duplicate function '{}'", func.name)));
            } else {
                self.functions.insert(func.name.clone(), func.clone());
            }
        }

        // Register class methods
        for class in &program.classes {
            for method in &class.methods {
                let full_name = format!("{}.{}", class.name, method.name);
                self.functions.insert(full_name, method.clone());
            }
        }

        // 3. Register globals
        for global in &program.globals {
            if let Stmt::VarDecl {
                name,
                ty,
                is_let,
                span,
                ..
            } = global
            {
                if self.lookup_symbol(name).is_some() {
                    errors.push(self.error(*span, format!("duplicate global variable '{}'", name)));
                } else {
                    self.define_symbol(name, ty.clone(), *is_let);
                }
            }
        }

        // 4. Check global initializers
        for global in &program.globals {
            if let Err(err) = self.check_stmt(global) {
                errors.push(err);
            }
        }

        // 5. Check functions
        for func in &program.functions {
            if let Err(mut errs) = self.check_function(func) {
                errors.append(&mut errs);
            }
        }

        // 6. Check class methods
        for class in &program.classes {
            for method in &class.methods {
                if let Err(mut errs) = self.check_function(method) {
                    errors.append(&mut errs);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn check_function(&mut self, func: &Function) -> Result<(), Vec<Diagnostic>> {
        if func.is_extern {
            return Ok(());
        }
        let mut errors = Vec::new();
        self.push_scope();
        self.current_return_type = Some(func.return_type.clone());
        self.returned_var = None;

        for param in &func.params {
            self.define_symbol(&param.name, Some(param.ty.clone()), false);
        }

        for stmt in &func.body {
            if let Err(err) = self.check_stmt(stmt) {
                errors.push(err);
            }
        }

        self.current_return_type = None;
        self.returned_var = None;
        self.pop_scope();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), Diagnostic> {
        match stmt {
            Stmt::VarDecl {
                name,
                ty,
                init,
                is_returned,
                is_let,
                span,
            } => {
                if let Some(init_expr) = init {
                    let init_ty = self.check_expr(init_expr)?;
                    if let Some(expected_ty) = ty
                        && !self.types_compatible(expected_ty, &init_ty)
                    {
                        return Err(self.error(
                                *span,
                                format!(
                                    "cannot initialize variable '{}' of type '{}' with value of type '{}'",
                                    name, expected_ty, init_ty
                                ),
                            ));
                    }
                }
                if *is_returned {
                    self.returned_var = Some((name.clone(), ty.clone()));
                }
                self.define_symbol(name, ty.clone(), *is_let);
                Ok(())
            }
            Stmt::Expr(expr) => {
                self.check_expr(expr)?;
                Ok(())
            }
            Stmt::If {
                cond,
                then_branch,
                else_branch,
                ..
            } => {
                let cond_ty = self.check_expr(cond)?;
                if cond_ty != Type::Bool && cond_ty != Type::Auto && cond_ty != Type::I32 {
                    // Non-boolean allowed if convertible or auto
                }
                self.push_scope();
                for s in then_branch {
                    self.check_stmt(s)?;
                }
                self.pop_scope();

                if let Some(else_stmts) = else_branch {
                    self.push_scope();
                    for s in else_stmts {
                        self.check_stmt(s)?;
                    }
                    self.pop_scope();
                }
                Ok(())
            }
            Stmt::While { cond, body, .. } => {
                self.check_expr(cond)?;
                self.push_scope();
                for s in body {
                    self.check_stmt(s)?;
                }
                self.pop_scope();
                Ok(())
            }
            Stmt::For {
                var_name,
                var_ty,
                iterable,
                body,
                ..
            } => {
                let iter_ty = self.check_expr(iterable)?;
                self.push_scope();
                let elem_ty = match iter_ty {
                    Type::Array(inner, _) => *inner,
                    _ => var_ty.clone().unwrap_or(Type::I32),
                };
                self.define_symbol(var_name, Some(elem_ty), false);
                for s in body {
                    self.check_stmt(s)?;
                }
                self.pop_scope();
                Ok(())
            }
            Stmt::Return {
                value,
                is_return_var,
                span,
            } => {
                if *is_return_var {
                    if self.returned_var.is_none() {
                        return Err(self.error(
                            *span,
                            "'return var;' used without 'returned var' declaration",
                        ));
                    }
                    return Ok(());
                }

                if let Some(val_expr) = value {
                    let val_ty = self.check_expr(val_expr)?;
                    if let Some(expected) = &self.current_return_type
                        && !self.types_compatible(expected, &val_ty)
                    {
                        return Err(self.error(
                            *span,
                            format!(
                                "return type mismatch: expected '{}', found '{}'",
                                expected, val_ty
                            ),
                        ));
                    }
                } else if let Some(expected) = &self.current_return_type
                    && *expected != Type::Void
                    && *expected != Type::Auto
                {
                    return Err(self.error(
                        *span,
                        format!(
                            "non-void function must return a value of type '{}'",
                            expected
                        ),
                    ));
                }
                Ok(())
            }
            Stmt::Break(_) | Stmt::Continue(_) => Ok(()),
            Stmt::Block(stmts) => {
                self.push_scope();
                for s in stmts {
                    self.check_stmt(s)?;
                }
                self.pop_scope();
                Ok(())
            }
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Result<Type, Diagnostic> {
        match expr {
            Expr::Integer(_, _) => Ok(Type::I32),
            Expr::Float(_, _) => Ok(Type::F64),
            Expr::Bool(_, _) => Ok(Type::Bool),
            Expr::StringLit(_, _) => Ok(Type::String),
            Expr::Ident(name, span) => {
                if name == "Core" {
                    return Ok(Type::Custom("Core".to_string()));
                }
                if let Some(sym) = self.lookup_symbol(name) {
                    Ok(sym.ty.clone().unwrap_or(Type::Auto))
                } else if self.classes.contains_key(name) {
                    Ok(Type::Custom(name.clone()))
                } else if self.functions.contains_key(name) {
                    Ok(Type::Auto)
                } else {
                    Err(self.error(*span, format!("undefined identifier '{}'", name)))
                }
            }
            Expr::Unary {
                op,
                expr: inner,
                span,
            } => {
                let inner_ty = self.check_expr(inner)?;
                match op {
                    UnaryOp::Neg | UnaryOp::BitNot => Ok(inner_ty),
                    UnaryOp::Not => Ok(Type::Bool),
                    UnaryOp::PreInc | UnaryOp::PreDec | UnaryOp::PostInc | UnaryOp::PostDec => {
                        Ok(inner_ty)
                    }
                    UnaryOp::AddressOf => Ok(Type::Pointer(Box::new(inner_ty))),
                    UnaryOp::Deref => {
                        if let Type::Pointer(pointee) = inner_ty {
                            Ok(*pointee)
                        } else {
                            Err(self.error(
                                *span,
                                format!("cannot dereference non-pointer type '{}'", inner_ty),
                            ))
                        }
                    }
                }
            }
            Expr::Binary {
                op, left, right, ..
            } => {
                let left_ty = self.check_expr(left)?;
                let right_ty = self.check_expr(right)?;
                match op {
                    BinaryOp::Add
                    | BinaryOp::Sub
                    | BinaryOp::Mul
                    | BinaryOp::Div
                    | BinaryOp::Rem
                    | BinaryOp::BitAnd
                    | BinaryOp::BitOr
                    | BinaryOp::BitXor
                    | BinaryOp::Shl
                    | BinaryOp::Shr => {
                        if left_ty == Type::F64 || right_ty == Type::F64 {
                            Ok(Type::F64)
                        } else if left_ty == Type::I64 || right_ty == Type::I64 {
                            Ok(Type::I64)
                        } else {
                            Ok(Type::I32)
                        }
                    }
                    BinaryOp::Eq
                    | BinaryOp::NotEq
                    | BinaryOp::Lt
                    | BinaryOp::LtEq
                    | BinaryOp::Gt
                    | BinaryOp::GtEq
                    | BinaryOp::And
                    | BinaryOp::Or => Ok(Type::Bool),
                }
            }
            Expr::Assign {
                target,
                value,
                span,
                ..
            } => {
                let target_ty = self.check_expr(target)?;
                let val_ty = self.check_expr(value)?;

                // Check mutability if target is a simple variable
                if let Expr::Ident(name, _) = &**target
                    && let Some(sym) = self.lookup_symbol(name)
                    && sym.is_let
                {
                    return Err(self.error(
                        *span,
                        format!("cannot assign to immutable 'let' binding '{}'", name),
                    ));
                }

                if !self.types_compatible(&target_ty, &val_ty) {
                    return Err(self.error(
                        *span,
                        format!(
                            "cannot assign value of type '{}' to target of type '{}'",
                            val_ty, target_ty
                        ),
                    ));
                }
                Ok(target_ty)
            }
            Expr::Call { callee, args, .. } => {
                // Check built-in functions
                if let Expr::MemberAccess { object, member, .. } = &**callee
                    && let Expr::Ident(obj_name, _) = &**object
                    && obj_name == "Core"
                {
                    match member.as_str() {
                        "Print" | "PrintStr" => {
                            for arg in args {
                                self.check_expr(arg)?;
                            }
                            return Ok(Type::Void);
                        }
                        "Range" | "InclusiveRange" => {
                            for arg in args {
                                self.check_expr(arg)?;
                            }
                            return Ok(Type::Auto);
                        }
                        _ => {}
                    }
                }

                if let Expr::Ident(func_name, _) = &**callee
                    && (func_name == "Print"
                        || func_name == "PrintInt"
                        || func_name == "PrintStr"
                        || func_name == "println")
                {
                    for arg in args {
                        self.check_expr(arg)?;
                    }
                    return Ok(Type::Void);
                }

                // Check static class methods: Class.Method(args)
                if let Expr::MemberAccess { object, member, .. } = &**callee
                    && let Expr::Ident(class_name, _) = &**object
                {
                    let full_name = format!("{}.{}", class_name, member);
                    if let Some(func) = self.functions.get(&full_name) {
                        let ret = func.return_type.clone();
                        for arg in args {
                            self.check_expr(arg)?;
                        }
                        return Ok(ret);
                    }
                }

                // Normal function call
                if let Expr::Ident(func_name, _) = &**callee
                    && let Some(func) = self.functions.get(func_name)
                {
                    let ret = func.return_type.clone();
                    for arg in args {
                        self.check_expr(arg)?;
                    }
                    return Ok(ret);
                }

                // Instance method call obj.method(args)
                if let Expr::MemberAccess { object, member, .. } = &**callee {
                    let obj_ty = self.check_expr(object)?;
                    if let Type::Custom(class_name) = obj_ty {
                        let full_name = format!("{}.{}", class_name, member);
                        if let Some(func) = self.functions.get(&full_name) {
                            let ret = func.return_type.clone();
                            for arg in args {
                                self.check_expr(arg)?;
                            }
                            return Ok(ret);
                        }
                    }
                }

                // Fallback for general expressions
                self.check_expr(callee)?;
                for arg in args {
                    self.check_expr(arg)?;
                }
                Ok(Type::Auto)
            }
            Expr::Index { array, index, span } => {
                let arr_ty = self.check_expr(array)?;
                let idx_ty = self.check_expr(index)?;
                if !self.is_integer_type(&idx_ty) && idx_ty != Type::Auto {
                    return Err(self.error(
                        *span,
                        format!("array index must be integer, found '{}'", idx_ty),
                    ));
                }
                match arr_ty {
                    Type::Array(elem_ty, _) => Ok(*elem_ty),
                    Type::Auto => Ok(Type::Auto),
                    _ => Ok(Type::Auto),
                }
            }
            Expr::MemberAccess {
                object,
                member,
                span,
            } => {
                let obj_ty = self.check_expr(object)?;
                match obj_ty {
                    Type::Custom(class_name) => {
                        if class_name == "Core" {
                            return Ok(Type::Auto);
                        }
                        if let Some(class) = self.classes.get(&class_name) {
                            for (field_name, field_ty) in &class.fields {
                                if field_name == member {
                                    return Ok(field_ty.clone());
                                }
                            }
                            // Might be a method reference
                            let method_name = format!("{}.{}", class_name, member);
                            if self.functions.contains_key(&method_name) {
                                return Ok(Type::Auto);
                            }
                            return Err(self.error(
                                *span,
                                format!(
                                    "class '{}' has no field or method '{}'",
                                    class_name, member
                                ),
                            ));
                        }
                        Ok(Type::Auto)
                    }
                    _ => Ok(Type::Auto),
                }
            }
            Expr::ArrayLit { elements, .. } => {
                let mut elem_ty = Type::Auto;
                for elem in elements {
                    elem_ty = self.check_expr(elem)?;
                }
                Ok(Type::Array(Box::new(elem_ty), elements.len()))
            }
            Expr::StructLit { name, fields, .. } => {
                for (_, val) in fields {
                    self.check_expr(val)?;
                }
                Ok(Type::Custom(name.clone()))
            }
            Expr::Range { start, end, .. } => {
                self.check_expr(start)?;
                self.check_expr(end)?;
                Ok(Type::Auto)
            }
        }
    }

    fn is_integer_type(&self, ty: &Type) -> bool {
        matches!(
            ty,
            Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::Int
        )
    }

    fn types_compatible(&self, expected: &Type, actual: &Type) -> bool {
        if expected == actual || *expected == Type::Auto || *actual == Type::Auto {
            return true;
        }
        if self.is_integer_type(expected) && self.is_integer_type(actual) {
            return true;
        }
        if (expected == &Type::F64 || expected == &Type::F32)
            && (actual == &Type::F64 || actual == &Type::F32 || self.is_integer_type(actual))
        {
            return true;
        }
        false
    }
}
