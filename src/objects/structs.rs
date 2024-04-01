use crate::{
    frontend::tokens::{TokenType, Keywords},
    vm::value::Value,
    objects::{rc::Object, functions::Local},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub name: String,
    pub locals: Vec<Local>,
    pub output_type: TokenType,
    pub field_count: usize,
    rc_counter: usize,
    index: usize,
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

    fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn get_values(&self) -> Vec<Value> {
        vec![Value::Bool(true)]
    }

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
            rc_counter: 1,
            index: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructInstance {
    pub root_struct_pos: usize,
    pub fields_values: Vec<Value>,
    rc_counter: usize,
    index: usize,
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

    fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn get_values(&self) -> Vec<Value> {
        self.fields_values.clone()
    }

    fn get_arg_count(&self) -> usize {
        0
    }
}

impl StructInstance {
    pub fn new(pos: usize) -> Self {
        Self {
            root_struct_pos: pos,
            fields_values: vec![],
            rc_counter: 0,
            index: 0,
        }
    }
}