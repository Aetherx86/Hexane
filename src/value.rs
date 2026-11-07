// Copyright (c) 2026 jk9576. All rights reserved.
// Hexane Language Toolchain
// Licensed under the MIT License.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Array(Rc<RefCell<Vec<Value>>>),
    Struct(Rc<RefCell<HashMap<String, Value>>>),
    Void,
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::Array(a) => !a.borrow().is_empty(),
            Value::Struct(_) => true,
            Value::Void => false,
        }
    }

    #[allow(dead_code)]
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Bool(_) => "bool",
            Value::Str(_) => "String",
            Value::Array(_) => "array",
            Value::Struct(_) => "struct",
            Value::Void => "void",
        }
    }

    #[allow(dead_code)]
    pub fn default_for_type(ty_name: &str, size: usize) -> Self {
        match ty_name {
            "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "int" => Value::Int(0),
            "f32" | "f64" | "float" => Value::Float(0.0),
            "bool" => Value::Bool(false),
            "String" | "StringView" => Value::Str(String::new()),
            "array" => Value::Array(Rc::new(RefCell::new(vec![Value::Int(0); size]))),
            _ => Value::Struct(Rc::new(RefCell::new(HashMap::new()))),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Str(s) => write!(f, "{}", s),
            Value::Array(a) => {
                let items = a.borrow();
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Value::Struct(s) => {
                let map = s.borrow();
                write!(f, "{{")?;
                let mut first = true;
                for (k, v) in map.iter() {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                    first = false;
                }
                write!(f, "}}")
            }
            Value::Void => write!(f, "()"),
        }
    }
}
