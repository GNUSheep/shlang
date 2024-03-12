use crate::frontend::tokens::TokenType;

use crate::compiler::errors;

pub use std::ops::Neg;

#[derive(Debug, Clone)]
pub enum Value {
    Float(f64),
    Int(i64),
    Bool(bool),
    String(String),
}

impl Value {
    pub fn get_float(&self) -> f64 {
        match self {
            Value::Float(val) => return *val,
            _ => {
                errors::conversion_error("Enum Value<_>", "f64");
                std::process::exit(1);
            },
        }
    }

    pub fn get_int(&self) -> i64 {
        match self {
            Value::Int(val) => return *val,
            _ => {
                errors::conversion_error("Enum Value<_>", "i64");
                std::process::exit(1);
            },
        }
    }
}

impl Neg for Value {
    type Output = Self;

    fn neg(self) -> Self {
        let value: Value; 
        match self {
            Value::Float(val) => {
                value = Value::Float(-val);
            }
            Value::Int(val) => {
                value = Value::Int(-val);
            }
            Value::Bool(val) => {
                value = Value::Bool(!val);
            }
            _ => {
                errors::conversion_error("Enum Value<_>", "NEG Enum Value<_>");
                std::process::exit(1);
            },
        };

        value
    }
}

pub trait Convert {
    fn convert(&self) -> TokenType;
}

impl Convert for Value {
    fn convert(&self) -> TokenType {
        match *self {
            Value::Float(_) => TokenType::FLOAT,
            Value::Int(_) => TokenType::INT,
            _ => {
                errors::conversion_error("Enum Value<_>", "TokenType");
                std::process::exit(1);
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValuesArray {
    values: Vec<Value>,
}

impl ValuesArray {
    pub fn init() -> Self {
        Self {
            values: vec![],
        }
    }

    pub fn push(&mut self, value: Value) {
        self.values.push(value);
    }

    pub fn get(&self, index: usize) -> Value {
        self.values[index].clone()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

