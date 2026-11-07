// Copyright (c) 2026 jk9576. All rights reserved.
// Hexane Language Toolchain
// Licensed under the MIT License.

use crate::diagnostics::Span;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    Int,
    F32,
    F64,
    Bool,
    String,
    Void,
    Auto,
    Array(Box<Type>, usize),
    Pointer(Box<Type>),
    Custom(String),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::I8 => write!(f, "i8"),
            Type::I16 => write!(f, "i16"),
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            Type::U8 => write!(f, "u8"),
            Type::U16 => write!(f, "u16"),
            Type::U32 => write!(f, "u32"),
            Type::U64 => write!(f, "u64"),
            Type::Int => write!(f, "int"),
            Type::F32 => write!(f, "f32"),
            Type::F64 => write!(f, "f64"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "String"),
            Type::Void => write!(f, "void"),
            Type::Auto => write!(f, "auto"),
            Type::Array(inner, size) => write!(f, "array({}, {})", inner, size),
            Type::Pointer(inner) => write!(f, "{}*", inner),
            Type::Custom(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
    PreInc,
    PreDec,
    PostInc,
    PostDec,
    Deref,
    AddressOf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    RemAssign,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
    ShlAssign,
    ShrAssign,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Integer(i64, Span),
    Float(f64, Span),
    Bool(bool, Span),
    StringLit(String, Span),
    Ident(String, Span),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    Index {
        array: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    MemberAccess {
        object: Box<Expr>,
        member: String,
        span: Span,
    },
    Assign {
        target: Box<Expr>,
        op: AssignOp,
        value: Box<Expr>,
        span: Span,
    },
    ArrayLit {
        elements: Vec<Expr>,
        span: Span,
    },
    StructLit {
        name: String,
        fields: Vec<(String, Expr)>,
        span: Span,
    },
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Integer(_, s)
            | Expr::Float(_, s)
            | Expr::Bool(_, s)
            | Expr::StringLit(_, s)
            | Expr::Ident(_, s)
            | Expr::Unary { span: s, .. }
            | Expr::Binary { span: s, .. }
            | Expr::Call { span: s, .. }
            | Expr::Index { span: s, .. }
            | Expr::MemberAccess { span: s, .. }
            | Expr::Assign { span: s, .. }
            | Expr::ArrayLit { span: s, .. }
            | Expr::StructLit { span: s, .. }
            | Expr::Range { span: s, .. } => *s,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    VarDecl {
        name: String,
        ty: Option<Type>,
        init: Option<Expr>,
        is_returned: bool,
        is_let: bool,
        span: Span,
    },
    Expr(Expr),
    If {
        cond: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
        span: Span,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    For {
        var_name: String,
        var_ty: Option<Type>,
        iterable: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    Return {
        value: Option<Expr>,
        is_return_var: bool,
        span: Span,
    },
    Break(Span),
    Continue(Span),
    Block(Vec<Stmt>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub is_ref: bool,
    pub is_self: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub body: Vec<Stmt>,
    pub is_extern: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDef {
    pub name: String,
    pub fields: Vec<(String, Type)>,
    pub methods: Vec<Function>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PackageDecl {
    pub name: String,
    pub library: Option<String>,
    pub is_api: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportDecl {
    pub package: String,
    pub library: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub packages: Vec<PackageDecl>,
    pub imports: Vec<ImportDecl>,
    pub classes: Vec<ClassDef>,
    pub functions: Vec<Function>,
    pub globals: Vec<Stmt>,
}

impl Program {
    pub fn new() -> Self {
        Self {
            packages: Vec::new(),
            imports: Vec::new(),
            classes: Vec::new(),
            functions: Vec::new(),
            globals: Vec::new(),
        }
    }

    pub fn merge(&mut self, other: Program) {
        self.packages.extend(other.packages);
        self.imports.extend(other.imports);
        self.classes.extend(other.classes);
        self.functions.extend(other.functions);
        self.globals.extend(other.globals);
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}
