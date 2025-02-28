use crate::frontend::tokens::{Keywords, TokenType};

use crate::compiler::errors;

use crate::vm::bytecode::Chunk;

use std::fmt;
pub use std::ops::Neg;
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Float(f64),
    Int(i64),
    Bool(bool),
    Null,
    String(String),
    Chunk(Chunk),
    List(TokenType),
    InstanceRef(usize),
    StringRef(usize),
    Fn(fn(Vec<Value>) -> Value),
    EmptyObj,
}

impl Value {
    pub fn get_float(&self) -> f64 {
        match self {
            Value::Float(val) => return *val,
            _ => {
                errors::conversion_error(&format!("Enum Value<{:?}>", self), "f64");
                std::process::exit(1);
            },
        }
    }

    pub fn get_int(&self) -> i64 {
        match self {
            Value::Int(val) => return *val,
            _ => {
                errors::conversion_error(&format!("Enum Value<{:?}>", self), "i64");
                std::process::exit(1);
            },
        }
    }

    pub fn get_bool(&self) -> bool {
        match self {
            Value::Bool(val) => return *val,
            _ => {
                errors::conversion_error(&format!("Enum Value<{:?}>", self), "bool");
                std::process::exit(1);
            },
        }
    }

    pub fn get_chunk(&self) -> Chunk {
        match self {
            Value::Chunk(val) => return val.clone(),
            _ => {
                errors::conversion_error(&format!("Enum Value<{:?}>", self), "chunk");
                std::process::exit(1);
            },
        }
    }

    pub fn get_fn(&self) -> fn(Vec<Value>) -> Value {
        match self {
            Value::Fn(val) => return *val,
            _val => {
                errors::conversion_error(&format!("Enum Value<{:?}>", self), "fn");
                std::process::exit(1);
            },
        }
    }

    pub fn get_string(&self) -> String {
        match self {
            Value::String(val) => return val.clone(),
            _ => {
                println!("{:?}", self);
                errors::conversion_error(&format!("Enum Value<{:?}>", self), "String");
                std::process::exit(1);
            },
        }
    }

    pub fn sort(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
            _ => Ordering::Equal,
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
        // println!("{:?}", self);
        match *self {
            Value::Float(_) => TokenType::FLOAT,
            Value::Int(_) => TokenType::INT,
            Value::Bool(_) => TokenType::BOOL,
            Value::Null => TokenType::NULL,
            Value::String(_) => TokenType::STRING,
            Value::List(val) => {
                let val_converted = match val {
                    TokenType::INT => Keywords::INT,
                    TokenType::FLOAT => Keywords::FLOAT,
                    TokenType::BOOL => Keywords::BOOL,
                    TokenType::STRING => Keywords::STRING,
                    TokenType::STRUCT(pos) => Keywords::INSTANCE(pos), 
                    _ => {
                        errors::conversion_error("Enum Value<_>", "TokenType");
                        std::process::exit(1);
                    },
                };

                TokenType::LIST(val_converted)
            },
            Value::InstanceRef(val) => TokenType::STRUCT(val), 
            _ => {
                errors::conversion_error("Enum Value<_>", "TokenType");
                std::process::exit(1);
            },
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, output: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Float(val) => write!(output, "{}", val),
            Value::Int(val) => write!(output, "{}", val),
            Value::Bool(val) => write!(output, "{}", val),
            Value::String(val) => write!(output, "{}", val),
            Value::Null => write!(output, "null"),
            Value::InstanceRef(_) => {
                errors::error_message("DISPLAY NOT IMPLEMENTED", format!("Writing \"Struct/List Object\" to stdout is not allowed"));
                std::process::exit(1);
            },
            v => {
                errors::error_message("DISPLAY NOT IMPLEMENTED", format!("Writing \"{:?}\" to stdout is not allowed", v));
                std::process::exit(1);
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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

