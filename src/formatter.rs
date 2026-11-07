// Copyright (c) 2026 jk9576. All rights reserved.
// Carbonation / Hexene Language Toolchain
// Licensed under the MIT License.

//! Carbon Source Code Formatter
//! Formats Carbon source code following standard style guidelines
//! (2-space indentation, normalized spacing, clean declaration layout).

use crate::ast::*;

pub struct Formatter {
    indent: usize,
    buffer: String,
}

impl Formatter {
    pub fn new() -> Self {
        Self {
            indent: 0,
            buffer: String::new(),
        }
    }

    fn push_indent(&mut self) {
        self.indent += 2;
    }

    fn pop_indent(&mut self) {
        if self.indent >= 2 {
            self.indent -= 2;
        }
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.buffer.push(' ');
        }
    }

    fn writeln(&mut self, s: &str) {
        self.write_indent();
        self.buffer.push_str(s);
        self.buffer.push('\n');
    }

    pub fn format_program(&mut self, program: &Program) -> String {
        self.buffer.clear();

        for pkg in &program.packages {
            if let Some(ref lib) = pkg.library {
                self.writeln(&format!("package {} library \"{}\";", pkg.name, lib));
            } else if pkg.is_api {
                self.writeln(&format!("package {} api;", pkg.name));
            } else {
                self.writeln(&format!("package {};", pkg.name));
            }
        }
        if !program.packages.is_empty() {
            self.writeln("");
        }

        for imp in &program.imports {
            if let Some(ref lib) = imp.library {
                self.writeln(&format!("import {} library \"{}\";", imp.package, lib));
            } else {
                self.writeln(&format!("import {};", imp.package));
            }
        }
        if !program.imports.is_empty() {
            self.writeln("");
        }

        for global in &program.globals {
            self.format_stmt(global);
        }
        if !program.globals.is_empty() {
            self.writeln("");
        }

        for class in &program.classes {
            self.format_class(class);
            self.writeln("");
        }

        for func in &program.functions {
            self.format_function(func);
            self.writeln("");
        }

        self.buffer.trim_end().to_string() + "\n"
    }

    fn format_class(&mut self, class: &ClassDef) {
        self.writeln(&format!("class {} {{", class.name));
        self.push_indent();

        for (fname, fty) in &class.fields {
            self.writeln(&format!("var {}: {};", fname, fty));
        }

        for method in &class.methods {
            self.writeln("");
            self.format_function(method);
        }

        self.pop_indent();
        self.writeln("}");
    }

    fn format_function(&mut self, func: &Function) {
        let mut params = Vec::new();
        for p in &func.params {
            if p.is_self {
                let ref_str = if p.is_ref { "ref " } else { "" };
                params.push(format!("{}self", ref_str));
            } else {
                let ref_str = if p.is_ref { "ref " } else { "" };
                params.push(format!("{}{}: {}", ref_str, p.name, p.ty));
            }
        }
        let param_str = params.join(", ");
        let ret_str = if func.return_type == Type::Void {
            String::new()
        } else {
            format!(" -> {}", func.return_type)
        };

        if func.is_extern {
            self.writeln(&format!(
                "extern fn {}({}){};",
                func.name, param_str, ret_str
            ));
            return;
        }

        self.writeln(&format!("fn {}({}){} {{", func.name, param_str, ret_str));
        self.push_indent();

        for stmt in &func.body {
            self.format_stmt(stmt);
        }

        self.pop_indent();
        self.writeln("}");
    }

    fn format_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl {
                name,
                ty,
                init,
                is_let,
                is_returned,
                ..
            } => {
                let kw = if *is_let { "let" } else { "var" };
                let ret_prefix = if *is_returned { "returned " } else { "" };
                let ty_str = ty.as_ref().map(|t| format!(": {}", t)).unwrap_or_default();
                if let Some(init_e) = init {
                    self.writeln(&format!(
                        "{}{} {}{} = {};",
                        ret_prefix,
                        kw,
                        name,
                        ty_str,
                        format_expr(init_e)
                    ));
                } else {
                    self.writeln(&format!("{}{} {}{};", ret_prefix, kw, name, ty_str));
                }
            }
            Stmt::Expr(e) => {
                self.writeln(&format!("{};", format_expr(e)));
            }
            Stmt::Return {
                value,
                is_return_var,
                ..
            } => {
                if *is_return_var {
                    self.writeln("return var;");
                } else if let Some(v) = value {
                    self.writeln(&format!("return {};", format_expr(v)));
                } else {
                    self.writeln("return;");
                }
            }
            Stmt::If {
                cond,
                then_branch,
                else_branch,
                ..
            } => {
                self.writeln(&format!("if ({}) {{", format_expr(cond)));
                self.push_indent();
                for s in then_branch {
                    self.format_stmt(s);
                }
                self.pop_indent();
                if let Some(else_b) = else_branch {
                    self.writeln("} else {");
                    self.push_indent();
                    for s in else_b {
                        self.format_stmt(s);
                    }
                    self.pop_indent();
                }
                self.writeln("}");
            }
            Stmt::While { cond, body, .. } => {
                self.writeln(&format!("while ({}) {{", format_expr(cond)));
                self.push_indent();
                for s in body {
                    self.format_stmt(s);
                }
                self.pop_indent();
                self.writeln("}");
            }
            Stmt::For {
                var_name,
                var_ty,
                iterable,
                body,
                ..
            } => {
                let ty_str = var_ty
                    .as_ref()
                    .map(|t| format!(": {}", t))
                    .unwrap_or_default();
                self.writeln(&format!(
                    "for (var {}{} in {}) {{",
                    var_name,
                    ty_str,
                    format_expr(iterable)
                ));
                self.push_indent();
                for s in body {
                    self.format_stmt(s);
                }
                self.pop_indent();
                self.writeln("}");
            }
            Stmt::Break(_) => self.writeln("break;"),
            Stmt::Continue(_) => self.writeln("continue;"),
            Stmt::Block(stmts) => {
                self.writeln("{");
                self.push_indent();
                for s in stmts {
                    self.format_stmt(s);
                }
                self.pop_indent();
                self.writeln("}");
            }
        }
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

fn format_expr(expr: &Expr) -> String {
    match expr {
        Expr::Integer(n, _) => n.to_string(),
        Expr::Float(f, _) => f.to_string(),
        Expr::Bool(b, _) => b.to_string(),
        Expr::StringLit(s, _) => format!("{:?}", s),
        Expr::Ident(name, _) => name.clone(),
        Expr::Binary {
            op, left, right, ..
        } => {
            let op_str = match op {
                BinaryOp::Add => "+",
                BinaryOp::Sub => "-",
                BinaryOp::Mul => "*",
                BinaryOp::Div => "/",
                BinaryOp::Rem => "%",
                BinaryOp::Eq => "==",
                BinaryOp::NotEq => "!=",
                BinaryOp::Lt => "<",
                BinaryOp::LtEq => "<=",
                BinaryOp::Gt => ">",
                BinaryOp::GtEq => ">=",
                BinaryOp::And => "and",
                BinaryOp::Or => "or",
                BinaryOp::BitAnd => "&",
                BinaryOp::BitOr => "|",
                BinaryOp::BitXor => "^",
                BinaryOp::Shl => "<<",
                BinaryOp::Shr => ">>",
            };
            format!("{} {} {}", format_expr(left), op_str, format_expr(right))
        }
        Expr::Unary {
            op, expr: inner, ..
        } => {
            let op_str = match op {
                UnaryOp::Neg => "-",
                UnaryOp::Not => "not ",
                UnaryOp::BitNot => "~",
                UnaryOp::PreInc => "++",
                UnaryOp::PreDec => "--",
                UnaryOp::PostInc | UnaryOp::PostDec => "",
                UnaryOp::Deref => "*",
                UnaryOp::AddressOf => "&",
            };
            format!("{}{}", op_str, format_expr(inner))
        }
        Expr::Assign {
            target, op, value, ..
        } => {
            let op_str = match op {
                AssignOp::Assign => "=",
                AssignOp::AddAssign => "+=",
                AssignOp::SubAssign => "-=",
                AssignOp::MulAssign => "*=",
                AssignOp::DivAssign => "/=",
                AssignOp::RemAssign => "%=",
                AssignOp::BitAndAssign => "&=",
                AssignOp::BitOrAssign => "|=",
                AssignOp::BitXorAssign => "^=",
                AssignOp::ShlAssign => "<<=",
                AssignOp::ShrAssign => ">>=",
            };
            format!("{} {} {}", format_expr(target), op_str, format_expr(value))
        }
        Expr::Call { callee, args, .. } => {
            let args_s: Vec<String> = args.iter().map(format_expr).collect();
            format!("{}({})", format_expr(callee), args_s.join(", "))
        }
        Expr::MemberAccess { object, member, .. } => {
            format!("{}.{}", format_expr(object), member)
        }
        Expr::Index { array, index, .. } => {
            format!("{}[{}]", format_expr(array), format_expr(index))
        }
        Expr::Range {
            start,
            end,
            inclusive,
            ..
        } => {
            if *inclusive {
                format!(
                    "Core.InclusiveRange({}, {})",
                    format_expr(start),
                    format_expr(end)
                )
            } else {
                format!("Core.Range({}, {})", format_expr(start), format_expr(end))
            }
        }
        Expr::ArrayLit { elements, .. } => {
            let elems: Vec<String> = elements.iter().map(format_expr).collect();
            format!("({})", elems.join(", "))
        }
        Expr::StructLit { name, fields, .. } => {
            let flds: Vec<String> = fields
                .iter()
                .map(|(n, e)| format!(".{} = {}", n, format_expr(e)))
                .collect();
            format!("{} {{{}}}", name, flds.join(", "))
        }
    }
}
