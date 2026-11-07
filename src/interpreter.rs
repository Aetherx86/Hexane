// Copyright (c) 2026 jk9576. All rights reserved.
// Hexane Language Toolchain
// Licensed under the MIT License.

use crate::ast::*;
use crate::diagnostics::{Diagnostic, Span};
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
enum ControlFlow {
    None,
    Return(Value),
    Break,
    Continue,
}

#[derive(Clone)]
pub(crate) struct Environment {
    pub(crate) parent: Option<Rc<RefCell<Environment>>>,
    pub(crate) variables: HashMap<String, Value>,
}

impl Environment {
    fn new() -> Self {
        Self {
            parent: None,
            variables: HashMap::new(),
        }
    }

    fn new_enclosed(parent: Rc<RefCell<Environment>>) -> Self {
        Self {
            parent: Some(parent),
            variables: HashMap::new(),
        }
    }

    fn define(&mut self, name: String, val: Value) {
        self.variables.insert(name, val);
    }

    fn get(&self, name: &str) -> Option<Value> {
        if let Some(val) = self.variables.get(name) {
            Some(val.clone())
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
    }

    fn assign(&mut self, name: &str, val: Value) -> bool {
        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), val);
            true
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().assign(name, val)
        } else {
            false
        }
    }
}

pub struct Interpreter {
    globals: Rc<RefCell<Environment>>,
    functions: HashMap<String, Function>,
    classes: HashMap<String, ClassDef>,
    filename: String,
    returned_var_name: Option<String>,
    call_depth: usize,
}

impl Interpreter {
    pub fn new(filename: impl Into<String>) -> Self {
        Self {
            globals: Rc::new(RefCell::new(Environment::new())),
            functions: HashMap::new(),
            classes: HashMap::new(),
            filename: filename.into(),
            returned_var_name: None,
            call_depth: 0,
        }
    }

    fn error(&self, span: Span, msg: impl Into<String>) -> Diagnostic {
        Diagnostic::error(&self.filename, span, msg)
    }

    pub fn load_program(&mut self, program: &Program) -> Result<(), Diagnostic> {
        for class in &program.classes {
            self.classes.insert(class.name.clone(), class.clone());
        }

        for func in &program.functions {
            self.functions.insert(func.name.clone(), func.clone());
        }

        for class in &program.classes {
            for method in &class.methods {
                let full_name = format!("{}.{}", class.name, method.name);
                self.functions.insert(full_name, method.clone());
            }
        }

        // Initialize globals
        let env = self.globals.clone();
        for global in &program.globals {
            self.exec_stmt(global, env.clone())?;
        }

        Ok(())
    }

    pub fn eval_repl_expr(&mut self, expr: &Expr) -> Result<Value, Diagnostic> {
        let env = self.globals.clone();
        self.eval_expr(expr, env)
    }

    #[allow(dead_code)]
    pub fn eval_repl_stmt(&mut self, stmt: &Stmt) -> Result<(), Diagnostic> {
        let env = self.globals.clone();
        self.exec_stmt(stmt, env)?;
        Ok(())
    }

    pub fn run_entry(&mut self) -> Result<Value, Diagnostic> {
        // Look for Main, Run, or main
        let entry_name = if self.functions.contains_key("Main") {
            "Main"
        } else if self.functions.contains_key("Run") {
            "Run"
        } else if self.functions.contains_key("main") {
            "main"
        } else {
            return Err(Diagnostic::error(
                &self.filename,
                Span::dummy(),
                "no entry point found: define 'fn Main()' or 'fn Run()'",
            ));
        };

        let func = self.functions.get(entry_name).unwrap().clone();
        self.call_function(&func, vec![])
    }

    pub fn call_function(
        &mut self,
        func: &Function,
        args: Vec<Value>,
    ) -> Result<Value, Diagnostic> {
        if func.is_extern {
            match func.name.as_str() {
                "abs" => {
                    if let Some(Value::Int(n)) = args.first() {
                        return Ok(Value::Int(n.abs()));
                    }
                }
                "puts" => {
                    if let Some(Value::Str(s)) = args.first() {
                        println!("{}", s);
                        return Ok(Value::Int(0));
                    }
                }
                "putchar" => {
                    if let Some(Value::Int(c)) = args.first() {
                        print!("{}", *c as u8 as char);
                        return Ok(Value::Int(*c));
                    }
                }
                "sqrt" => {
                    if let Some(Value::Float(f)) = args.first() {
                        return Ok(Value::Float(f.sqrt()));
                    } else if let Some(Value::Int(n)) = args.first() {
                        return Ok(Value::Float((*n as f64).sqrt()));
                    }
                }
                _ => {}
            }
            return Err(self.error(
                func.span,
                format!(
                    "cannot interpret external function '{}' without native linking (use 'carbonation compile' to link with native binaries)",
                    func.name
                ),
            ));
        }

        if self.call_depth >= 50 {
            return Err(self.error(func.span, "maximum recursion depth exceeded (50 frames)"));
        }
        self.call_depth += 1;

        let env = Rc::new(RefCell::new(Environment::new_enclosed(
            self.globals.clone(),
        )));

        for (i, param) in func.params.iter().enumerate() {
            let val = if i < args.len() {
                args[i].clone()
            } else {
                Value::Void
            };
            env.borrow_mut().define(param.name.clone(), val);
        }

        let old_returned_var = self.returned_var_name.clone();
        self.returned_var_name = None;

        let mut return_val = Value::Void;
        for stmt in &func.body {
            match self.exec_stmt(stmt, env.clone()) {
                Ok(ControlFlow::Return(v)) => {
                    return_val = v;
                    break;
                }
                Ok(ControlFlow::Break) => {
                    self.call_depth -= 1;
                    return Err(self.error(stmt_span(stmt), "'break' outside of loop"));
                }
                Ok(ControlFlow::Continue) => {
                    self.call_depth -= 1;
                    return Err(self.error(stmt_span(stmt), "'continue' outside of loop"));
                }
                Ok(ControlFlow::None) => {}
                Err(err) => {
                    self.call_depth -= 1;
                    return Err(err);
                }
            }
        }

        self.returned_var_name = old_returned_var;
        self.call_depth -= 1;
        Ok(return_val)
    }

    fn exec_stmt(
        &mut self,
        stmt: &Stmt,
        env: Rc<RefCell<Environment>>,
    ) -> Result<ControlFlow, Diagnostic> {
        match stmt {
            Stmt::VarDecl {
                name,
                ty,
                init,
                is_returned,
                ..
            } => {
                let val = if let Some(init_expr) = init {
                    self.eval_expr(init_expr, env.clone())?
                } else if let Some(type_decl) = ty {
                    self.instantiate_type_default(type_decl)
                } else {
                    Value::Void
                };

                if *is_returned {
                    self.returned_var_name = Some(name.clone());
                }

                env.borrow_mut().define(name.clone(), val);
                Ok(ControlFlow::None)
            }
            Stmt::Expr(expr) => {
                self.eval_expr(expr, env)?;
                Ok(ControlFlow::None)
            }
            Stmt::If {
                cond,
                then_branch,
                else_branch,
                ..
            } => {
                let cond_val = self.eval_expr(cond, env.clone())?;
                if cond_val.is_truthy() {
                    let inner_env = Rc::new(RefCell::new(Environment::new_enclosed(env)));
                    for s in then_branch {
                        let cf = self.exec_stmt(s, inner_env.clone())?;
                        if !matches!(cf, ControlFlow::None) {
                            return Ok(cf);
                        }
                    }
                } else if let Some(else_stmts) = else_branch {
                    let inner_env = Rc::new(RefCell::new(Environment::new_enclosed(env)));
                    for s in else_stmts {
                        let cf = self.exec_stmt(s, inner_env.clone())?;
                        if !matches!(cf, ControlFlow::None) {
                            return Ok(cf);
                        }
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::While { cond, body, .. } => {
                loop {
                    let cond_val = self.eval_expr(cond, env.clone())?;
                    if !cond_val.is_truthy() {
                        break;
                    }

                    let inner_env = Rc::new(RefCell::new(Environment::new_enclosed(env.clone())));
                    let mut should_break = false;
                    for s in body {
                        match self.exec_stmt(s, inner_env.clone())? {
                            ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                            ControlFlow::Break => {
                                should_break = true;
                                break;
                            }
                            ControlFlow::Continue => {
                                break;
                            }
                            ControlFlow::None => {}
                        }
                    }
                    if should_break {
                        break;
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::For {
                var_name,
                iterable,
                body,
                ..
            } => {
                let iter_val = self.eval_expr(iterable, env.clone())?;
                let items: Vec<Value> = match iter_val {
                    Value::Array(arr) => arr.borrow().clone(),
                    _ => Vec::new(),
                };

                for item in items {
                    let inner_env = Rc::new(RefCell::new(Environment::new_enclosed(env.clone())));
                    inner_env.borrow_mut().define(var_name.clone(), item);

                    let mut should_break = false;
                    for s in body {
                        match self.exec_stmt(s, inner_env.clone())? {
                            ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                            ControlFlow::Break => {
                                should_break = true;
                                break;
                            }
                            ControlFlow::Continue => {
                                break;
                            }
                            ControlFlow::None => {}
                        }
                    }
                    if should_break {
                        break;
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::Return {
                value,
                is_return_var,
                span,
            } => {
                if *is_return_var {
                    if let Some(ref name) = self.returned_var_name
                        && let Some(val) = env.borrow().get(name)
                    {
                        return Ok(ControlFlow::Return(val));
                    }
                    return Err(
                        self.error(*span, "'return var;' used with no active returned variable")
                    );
                }

                let ret = if let Some(v_expr) = value {
                    self.eval_expr(v_expr, env)?
                } else {
                    Value::Void
                };
                Ok(ControlFlow::Return(ret))
            }
            Stmt::Break(_) => Ok(ControlFlow::Break),
            Stmt::Continue(_) => Ok(ControlFlow::Continue),
            Stmt::Block(stmts) => {
                let inner_env = Rc::new(RefCell::new(Environment::new_enclosed(env)));
                for s in stmts {
                    let cf = self.exec_stmt(s, inner_env.clone())?;
                    if !matches!(cf, ControlFlow::None) {
                        return Ok(cf);
                    }
                }
                Ok(ControlFlow::None)
            }
        }
    }

    pub(crate) fn eval_expr(
        &mut self,
        expr: &Expr,
        env: Rc<RefCell<Environment>>,
    ) -> Result<Value, Diagnostic> {
        match expr {
            Expr::Integer(n, _) => Ok(Value::Int(*n)),
            Expr::Float(f, _) => Ok(Value::Float(*f)),
            Expr::Bool(b, _) => Ok(Value::Bool(*b)),
            Expr::StringLit(s, _) => Ok(Value::Str(s.clone())),
            Expr::Ident(name, span) => {
                if name == "Core" {
                    return Ok(Value::Str("Core".to_string()));
                }
                if let Some(val) = env.borrow().get(name) {
                    Ok(val)
                } else if self.classes.contains_key(name) {
                    Ok(Value::Str(name.clone()))
                } else {
                    Err(self.error(*span, format!("undefined variable '{}'", name)))
                }
            }
            Expr::Unary {
                op,
                expr: inner,
                span,
            } => {
                let val = self.eval_expr(inner, env.clone())?;
                match op {
                    UnaryOp::Neg => match val {
                        Value::Int(n) => Ok(Value::Int(n.wrapping_neg())),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        _ => Err(self.error(*span, "invalid operand for unary '-'")),
                    },
                    UnaryOp::Not => Ok(Value::Bool(!val.is_truthy())),
                    UnaryOp::BitNot => match val {
                        Value::Int(n) => Ok(Value::Int(!n)),
                        _ => Err(self.error(*span, "invalid operand for unary '~'")),
                    },
                    UnaryOp::PreInc => {
                        let new_val = match val {
                            Value::Int(n) => Value::Int(n.wrapping_add(1)),
                            Value::Float(f) => Value::Float(f + 1.0),
                            _ => {
                                return Err(self.error(*span, "cannot increment non-numeric value"));
                            }
                        };
                        self.assign_target(inner, new_val.clone(), env)?;
                        Ok(new_val)
                    }
                    UnaryOp::PreDec => {
                        let new_val = match val {
                            Value::Int(n) => Value::Int(n.wrapping_sub(1)),
                            Value::Float(f) => Value::Float(f - 1.0),
                            _ => {
                                return Err(self.error(*span, "cannot decrement non-numeric value"));
                            }
                        };
                        self.assign_target(inner, new_val.clone(), env)?;
                        Ok(new_val)
                    }
                    UnaryOp::PostInc => {
                        let old_val = val.clone();
                        let new_val = match val {
                            Value::Int(n) => Value::Int(n.wrapping_add(1)),
                            Value::Float(f) => Value::Float(f + 1.0),
                            _ => {
                                return Err(self.error(*span, "cannot increment non-numeric value"));
                            }
                        };
                        self.assign_target(inner, new_val, env)?;
                        Ok(old_val)
                    }
                    UnaryOp::PostDec => {
                        let old_val = val.clone();
                        let new_val = match val {
                            Value::Int(n) => Value::Int(n.wrapping_sub(1)),
                            Value::Float(f) => Value::Float(f - 1.0),
                            _ => {
                                return Err(self.error(*span, "cannot decrement non-numeric value"));
                            }
                        };
                        self.assign_target(inner, new_val, env)?;
                        Ok(old_val)
                    }
                    UnaryOp::AddressOf | UnaryOp::Deref => Ok(val),
                }
            }
            Expr::Binary {
                op,
                left,
                right,
                span,
            } => {
                let l = self.eval_expr(left, env.clone())?;
                let r = self.eval_expr(right, env)?;

                match op {
                    BinaryOp::Add => match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_add(b))),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                        (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 + b)),
                        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + b as f64)),
                        (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{}{}", a, b))),
                        _ => Err(self.error(*span, "unsupported operands for '+'")),
                    },
                    BinaryOp::Sub => match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_sub(b))),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                        (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 - b)),
                        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - b as f64)),
                        _ => Err(self.error(*span, "unsupported operands for '-'")),
                    },
                    BinaryOp::Mul => match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_mul(b))),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                        (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 * b)),
                        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * b as f64)),
                        _ => Err(self.error(*span, "unsupported operands for '*'")),
                    },
                    BinaryOp::Div => match (l, r) {
                        (Value::Int(a), Value::Int(b)) => {
                            if b == 0 {
                                Err(self.error(*span, "division by zero"))
                            } else if a == i64::MIN && b == -1 {
                                Ok(Value::Int(i64::MIN))
                            } else {
                                Ok(Value::Int(a / b))
                            }
                        }
                        (Value::Float(a), Value::Float(b)) => {
                            if b == 0.0 {
                                Err(self.error(*span, "float division by zero"))
                            } else {
                                Ok(Value::Float(a / b))
                            }
                        }
                        (Value::Int(a), Value::Float(b)) => {
                            if b == 0.0 {
                                Err(self.error(*span, "float division by zero"))
                            } else {
                                Ok(Value::Float(a as f64 / b))
                            }
                        }
                        (Value::Float(a), Value::Int(b)) => {
                            if b == 0 {
                                Err(self.error(*span, "float division by zero"))
                            } else {
                                Ok(Value::Float(a / b as f64))
                            }
                        }
                        _ => Err(self.error(*span, "unsupported operands for '/'")),
                    },
                    BinaryOp::Rem => match (l, r) {
                        (Value::Int(a), Value::Int(b)) => {
                            if b == 0 {
                                Err(self.error(*span, "modulo by zero"))
                            } else if a == i64::MIN && b == -1 {
                                Ok(Value::Int(0))
                            } else {
                                Ok(Value::Int(a % b))
                            }
                        }
                        _ => Err(self.error(*span, "unsupported operands for '%'")),
                    },
                    BinaryOp::Eq => Ok(Value::Bool(l == r)),
                    BinaryOp::NotEq => Ok(Value::Bool(l != r)),
                    BinaryOp::Lt => match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
                        _ => Err(self.error(*span, "unsupported operands for '<'")),
                    },
                    BinaryOp::LtEq => match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
                        _ => Err(self.error(*span, "unsupported operands for '<='")),
                    },
                    BinaryOp::Gt => match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
                        _ => Err(self.error(*span, "unsupported operands for '>'")),
                    },
                    BinaryOp::GtEq => match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
                        _ => Err(self.error(*span, "unsupported operands for '>='")),
                    },
                    BinaryOp::And => Ok(Value::Bool(l.is_truthy() && r.is_truthy())),
                    BinaryOp::Or => Ok(Value::Bool(l.is_truthy() || r.is_truthy())),
                    BinaryOp::BitAnd => match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a & b)),
                        _ => Err(self.error(*span, "unsupported operands for '&'")),
                    },
                    BinaryOp::BitOr => match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a | b)),
                        _ => Err(self.error(*span, "unsupported operands for '|'")),
                    },
                    BinaryOp::BitXor => match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a ^ b)),
                        _ => Err(self.error(*span, "unsupported operands for '^'")),
                    },
                    BinaryOp::Shl => match (l, r) {
                        (Value::Int(a), Value::Int(b)) => {
                            let shift = (b & 63) as u32;
                            Ok(Value::Int(a.wrapping_shl(shift)))
                        }
                        _ => Err(self.error(*span, "unsupported operands for '<<'")),
                    },
                    BinaryOp::Shr => match (l, r) {
                        (Value::Int(a), Value::Int(b)) => {
                            let shift = (b & 63) as u32;
                            Ok(Value::Int(a.wrapping_shr(shift)))
                        }
                        _ => Err(self.error(*span, "unsupported operands for '>>'")),
                    },
                }
            }
            Expr::Assign {
                target,
                op,
                value,
                span,
            } => {
                let right_val = self.eval_expr(value, env.clone())?;
                let final_val = match op {
                    AssignOp::Assign => right_val,
                    _ => {
                        let current_val = self.eval_expr(target, env.clone())?;
                        match (op, current_val, right_val) {
                            (AssignOp::AddAssign, Value::Int(a), Value::Int(b)) => {
                                Value::Int(a.wrapping_add(b))
                            }
                            (AssignOp::SubAssign, Value::Int(a), Value::Int(b)) => {
                                Value::Int(a.wrapping_sub(b))
                            }
                            (AssignOp::MulAssign, Value::Int(a), Value::Int(b)) => {
                                Value::Int(a.wrapping_mul(b))
                            }
                            (AssignOp::DivAssign, Value::Int(a), Value::Int(b)) => {
                                if b == 0 {
                                    return Err(self.error(*span, "division by zero in '/='"));
                                } else if a == i64::MIN && b == -1 {
                                    Value::Int(i64::MIN)
                                } else {
                                    Value::Int(a / b)
                                }
                            }
                            (AssignOp::RemAssign, Value::Int(a), Value::Int(b)) => {
                                if b == 0 {
                                    return Err(self.error(*span, "modulo by zero in '%='"));
                                } else if a == i64::MIN && b == -1 {
                                    Value::Int(0)
                                } else {
                                    Value::Int(a % b)
                                }
                            }
                            (AssignOp::BitAndAssign, Value::Int(a), Value::Int(b)) => {
                                Value::Int(a & b)
                            }
                            (AssignOp::BitOrAssign, Value::Int(a), Value::Int(b)) => {
                                Value::Int(a | b)
                            }
                            (AssignOp::BitXorAssign, Value::Int(a), Value::Int(b)) => {
                                Value::Int(a ^ b)
                            }
                            (AssignOp::ShlAssign, Value::Int(a), Value::Int(b)) => {
                                let shift = (b & 63) as u32;
                                Value::Int(a.wrapping_shl(shift))
                            }
                            (AssignOp::ShrAssign, Value::Int(a), Value::Int(b)) => {
                                let shift = (b & 63) as u32;
                                Value::Int(a.wrapping_shr(shift))
                            }
                            _ => return Err(self.error(*span, "unsupported compound assignment")),
                        }
                    }
                };

                self.assign_target(target, final_val.clone(), env)?;
                Ok(final_val)
            }
            Expr::Call { callee, args, span } => {
                let mut arg_vals = Vec::new();
                for arg in args {
                    arg_vals.push(self.eval_expr(arg, env.clone())?);
                }

                // Core.Print / Core.PrintStr
                if let Expr::MemberAccess { object, member, .. } = &**callee
                    && let Expr::Ident(obj_name, _) = &**object
                    && obj_name == "Core"
                {
                    match member.as_str() {
                        "Print" => {
                            for (i, val) in arg_vals.iter().enumerate() {
                                if i > 0 {
                                    print!(" ");
                                }
                                print!("{}", val);
                            }
                            println!();
                            return Ok(Value::Void);
                        }
                        "PrintStr" => {
                            for val in &arg_vals {
                                match val {
                                    Value::Str(s) => print!("{}", s),
                                    other => print!("{}", other),
                                }
                            }
                            return Ok(Value::Void);
                        }
                        _ => {}
                    }
                }

                if let Expr::Ident(func_name, _) = &**callee {
                    if func_name == "Print" || func_name == "PrintInt" || func_name == "println" {
                        for (i, val) in arg_vals.iter().enumerate() {
                            if i > 0 {
                                print!(" ");
                            }
                            print!("{}", val);
                        }
                        println!();
                        return Ok(Value::Void);
                    } else if func_name == "PrintStr" {
                        for val in &arg_vals {
                            match val {
                                Value::Str(s) => print!("{}", s),
                                other => print!("{}", other),
                            }
                        }
                        return Ok(Value::Void);
                    }
                }

                // Static Class method Sieve.Make()
                if let Expr::MemberAccess { object, member, .. } = &**callee
                    && let Expr::Ident(class_name, _) = &**object
                {
                    let full_name = format!("{}.{}", class_name, member);
                    if let Some(func) = self.functions.get(&full_name).cloned() {
                        return self.call_function(&func, arg_vals);
                    }
                }

                // Instance method call obj.method(args)
                if let Expr::MemberAccess { object, member, .. } = &**callee {
                    let obj_val = self.eval_expr(object, env.clone())?;
                    if let Value::Struct(_fields) = &obj_val {
                        // Look for method across classes
                        for class in self.classes.values() {
                            for method in &class.methods {
                                if &method.name == member {
                                    let mut full_args = vec![obj_val.clone()];
                                    full_args.extend(arg_vals);
                                    let m = method.clone();
                                    return self.call_function(&m, full_args);
                                }
                            }
                        }
                    }
                }

                // Global function call
                if let Expr::Ident(func_name, _) = &**callee
                    && let Some(func) = self.functions.get(func_name).cloned()
                {
                    return self.call_function(&func, arg_vals);
                }

                Err(self.error(*span, "call to uncallable expression"))
            }
            Expr::Index { array, index, span } => {
                let arr_val = self.eval_expr(array, env.clone())?;
                let idx_val = self.eval_expr(index, env)?;

                let idx = match idx_val {
                    Value::Int(n) if n >= 0 => n as usize,
                    _ => return Err(self.error(*span, "array index must be non-negative integer")),
                };

                match arr_val {
                    Value::Array(a) => {
                        let items = a.borrow();
                        if idx < items.len() {
                            Ok(items[idx].clone())
                        } else {
                            Err(self.error(
                                *span,
                                format!("array index out of bounds: {} >= {}", idx, items.len()),
                            ))
                        }
                    }
                    _ => Err(self.error(*span, "cannot index non-array value")),
                }
            }
            Expr::MemberAccess {
                object,
                member,
                span,
            } => {
                let obj_val = self.eval_expr(object, env)?;
                match obj_val {
                    Value::Struct(fields) => {
                        if let Some(val) = fields.borrow().get(member) {
                            Ok(val.clone())
                        } else {
                            Err(self
                                .error(*span, format!("field '{}' not found in struct", member)))
                        }
                    }
                    _ => Ok(Value::Str(member.clone())),
                }
            }
            Expr::ArrayLit { elements, .. } => {
                let mut vals = Vec::new();
                for elem in elements {
                    vals.push(self.eval_expr(elem, env.clone())?);
                }
                Ok(Value::Array(Rc::new(RefCell::new(vals))))
            }
            Expr::StructLit {
                name: _, fields, ..
            } => {
                let mut map = HashMap::new();
                for (fname, fexpr) in fields {
                    map.insert(fname.clone(), self.eval_expr(fexpr, env.clone())?);
                }
                Ok(Value::Struct(Rc::new(RefCell::new(map))))
            }
            Expr::Range {
                start,
                end,
                inclusive,
                ..
            } => {
                let s_val = self.eval_expr(start, env.clone())?;
                let e_val = self.eval_expr(end, env)?;
                let s = match s_val {
                    Value::Int(n) => n,
                    _ => 0,
                };
                let e = match e_val {
                    Value::Int(n) => n,
                    _ => 0,
                };
                let mut items = Vec::new();
                if *inclusive {
                    for i in s..=e {
                        items.push(Value::Int(i));
                    }
                } else {
                    for i in s..e {
                        items.push(Value::Int(i));
                    }
                }
                Ok(Value::Array(Rc::new(RefCell::new(items))))
            }
        }
    }

    fn assign_target(
        &mut self,
        target: &Expr,
        val: Value,
        env: Rc<RefCell<Environment>>,
    ) -> Result<(), Diagnostic> {
        match target {
            Expr::Ident(name, span) => {
                if !env.borrow_mut().assign(name, val) {
                    return Err(self.error(
                        *span,
                        format!("cannot assign to undefined variable '{}'", name),
                    ));
                }
                Ok(())
            }
            Expr::Index { array, index, span } => {
                let arr_val = self.eval_expr(array, env.clone())?;
                let idx_val = self.eval_expr(index, env)?;
                let idx = match idx_val {
                    Value::Int(n) if n >= 0 => n as usize,
                    _ => return Err(self.error(*span, "array index must be non-negative integer")),
                };

                match arr_val {
                    Value::Array(a) => {
                        let mut items = a.borrow_mut();
                        if idx < items.len() {
                            items[idx] = val;
                            Ok(())
                        } else {
                            Err(self.error(
                                *span,
                                format!("array index out of bounds: {} >= {}", idx, items.len()),
                            ))
                        }
                    }
                    _ => Err(self.error(*span, "cannot index non-array")),
                }
            }
            Expr::MemberAccess {
                object,
                member,
                span,
            } => {
                let obj_val = self.eval_expr(object, env)?;
                match obj_val {
                    Value::Struct(fields) => {
                        fields.borrow_mut().insert(member.clone(), val);
                        Ok(())
                    }
                    _ => Err(self.error(*span, "cannot assign field on non-struct")),
                }
            }
            _ => Err(self.error(target.span(), "invalid assignment target")),
        }
    }

    fn instantiate_type_default(&self, ty: &Type) -> Value {
        match ty {
            Type::I8
            | Type::I16
            | Type::I32
            | Type::I64
            | Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Int => Value::Int(0),
            Type::F32 | Type::F64 => Value::Float(0.0),
            Type::Bool => Value::Bool(false),
            Type::String => Value::Str(String::new()),
            Type::Array(inner, size) => {
                let default_elem = self.instantiate_type_default(inner);
                Value::Array(Rc::new(RefCell::new(vec![default_elem; *size])))
            }
            Type::Custom(name) => {
                if let Some(class) = self.classes.get(name) {
                    let mut fields = HashMap::new();
                    for (fname, fty) in &class.fields {
                        fields.insert(fname.clone(), self.instantiate_type_default(fty));
                    }
                    Value::Struct(Rc::new(RefCell::new(fields)))
                } else {
                    Value::Struct(Rc::new(RefCell::new(HashMap::new())))
                }
            }
            _ => Value::Void,
        }
    }
}

fn stmt_span(stmt: &Stmt) -> Span {
    match stmt {
        Stmt::VarDecl { span, .. }
        | Stmt::If { span, .. }
        | Stmt::While { span, .. }
        | Stmt::For { span, .. }
        | Stmt::Return { span, .. }
        | Stmt::Break(span)
        | Stmt::Continue(span) => *span,
        Stmt::Expr(e) => e.span(),
        Stmt::Block(stmts) => stmts.first().map(stmt_span).unwrap_or_else(Span::dummy),
    }
}
