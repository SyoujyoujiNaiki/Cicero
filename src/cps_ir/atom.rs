use super::IR;
use std::collections::HashMap;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Atom {
    Var(String),
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    Bool(bool),
    Char(char),
    StringLiteral(String),
    Lam(usize, Vec<String>, Box<IR>),
}

impl Atom {
    pub fn lam(label: usize, args: &[&str], body: IR) -> Self {
        Atom::Lam(
            label,
            args.iter().map(|s| s.to_string()).collect(),
            Box::new(body),
        )
    }
    pub fn v(s: &str) -> Self {
        Atom::Var(s.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum Value<'a> {
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    Bool(bool),
    Char(char),
    StringLiteral(String),
    Clo(&'a Vec<String>, &'a IR, HashMap<&'a str, usize>),
    Cont(&'a Vec<String>, &'a IR, HashMap<&'a str, usize>),
}

impl<'a> PartialEq for Value<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::I32(a), Value::I32(b)) => a == b,
            (Value::I64(a), Value::I64(b)) => a == b,
            (Value::U32(a), Value::U32(b)) => a == b,
            (Value::U64(a), Value::U64(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Char(a), Value::Char(b)) => a == b,
            (Value::StringLiteral(a), Value::StringLiteral(b)) => a == b,
            _ => false,
        }
    }
}
