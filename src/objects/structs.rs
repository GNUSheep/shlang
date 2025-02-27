use std::collections::HashMap;

use crate::{
    frontend::tokens::{TokenType, Keywords},
    vm::value::Value,
    objects::{rc::Object, functions::Local},
};

use super::functions::Function;

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub name: String,
    pub locals: Vec<Local>,
    pub output_type: TokenType,
    pub field_count: usize,
    pub methods: HashMap<String, Function>,
    pub rc_counter: usize,
    pub index: usize,
}

impl Object for Struct {
    fn inc_counter(&mut self) {
        self.rc_counter += 1;
    }
    
    fn dec_counter(&mut self) {
        self.rc_counter -= 1;
    }

    fn get_rc_counter(&self) -> usize {
        self.rc_counter
    }

    fn get_values(&self) -> Vec<Value> {
        vec![Value::String(self.name.clone())]
    }

    fn set_value(&mut self, _pos: usize, _value: Value) {}

    fn replace_values(&mut self, _value: Vec<Value>) {}

    fn get_arg_count(&self) -> usize {
        self.field_count
    }
}

impl Struct {
    pub fn new(name: String) -> Self {
        Self {
            name: name,
            locals: vec![],
            output_type: TokenType::KEYWORD(Keywords::NULL),
            field_count: 0,
            methods: HashMap::new(),
            rc_counter: 1,
            index: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructInstance {
    pub fields_values: Vec<Value>,
    rc_counter: usize,
}

impl Object for StructInstance {
    fn inc_counter(&mut self) {
        self.rc_counter += 1;
    }
    
    fn dec_counter(&mut self) {
        self.rc_counter -= 1;
    }

    fn get_rc_counter(&self) -> usize {
        self.rc_counter
    }

    fn get_values(&self) -> Vec<Value> {
        self.fields_values.clone()
    }

    fn set_value(&mut self, pos: usize, value: Value) {
        self.fields_values[pos] = value;
    }

    fn replace_values(&mut self, value: Vec<Value>) {
        self.fields_values = value;
    }

    fn get_arg_count(&self) -> usize {
        0
    }
}

impl StructInstance {
    pub fn new() -> Self {
        Self {
            fields_values: vec![],
            rc_counter: 1,
        }
    }
}
