use crate::{
    objects::rc, vm::{bytecode, value::Value},
    frontend::tokens::{TokenType, Keywords},
    compiler::compiler::Symbol,
    std,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Local {
    pub name: String,
    pub local_type: TokenType,
    pub is_redirected: bool,
    pub redirect_pos: usize,
    pub rf_index: usize,
    pub is_string: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub chunk: bytecode::Chunk,
    pub locals: Vec<Local>,
    pub instances: Vec<Local>,
    pub output_type: TokenType,
    pub arg_count: usize,
    pub is_self_arg: bool,
    rc_counter: usize,
    index: usize,
}

impl rc::Object for Function {
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
        vec![Value::Chunk(self.chunk.clone())]
    }

    fn set_value(&mut self, _pos: usize, _value: Value) {
    }

    fn get_arg_count(&self) -> usize {
        self.arg_count
    }
}

impl Function {
    pub fn new(name: String) -> Self {
        Self {
            name: name,
            chunk: bytecode::Chunk::new(),
            locals: vec![],
            instances: vec![],
            output_type: TokenType::KEYWORD(Keywords::NULL),
            arg_count: 0,
            is_self_arg: false,
            rc_counter: 1,
            index: 0,
        }
    }

    pub fn get_chunk(&mut self) -> &mut bytecode::Chunk {
        &mut self.chunk
    }

    pub fn get_locals(&mut self) -> &mut Vec<Local> {
        &mut self.locals
    }

    pub fn get_instances(&mut self) -> &mut Vec<Local> {
        &mut self.instances
    }
}

pub struct NativeFn {
    pub name: String,
    pub function: fn(Vec<Value>) -> Value,
    pub arg_count: usize,
    pub rc_counter: usize,
    pub index: usize,
}

impl NativeFn {
    pub fn get_natives_symbols() -> Vec<Symbol> {
        vec![
            Symbol { name: "print".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::KEYWORD(Keywords::NULL), arg_count: 1 },
            Symbol { name: "println".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::KEYWORD(Keywords::NULL), arg_count: 1 },
            Symbol { name: "input".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::STRING, arg_count: 1 },
            Symbol { name: "convINT".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::INT, arg_count: 1 },
            Symbol { name: "convSTR".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::STRING, arg_count: 1 },
        ]
    }

    pub fn get_natives_fn() -> Vec<NativeFn> {
        vec![
            NativeFn { name: "print".to_string(), function: std::print::print, arg_count: 1, rc_counter: 1, index: 0 },
            NativeFn { name: "println".to_string(), function: std::print::println, arg_count: 1, rc_counter: 1, index: 0 },
            NativeFn { name: "input".to_string(), function: std::input::input, arg_count: 1, rc_counter: 1, index: 0 },
            NativeFn { name: "convINT".to_string(), function: std::conv::conv_to_int, arg_count: 1, rc_counter: 1, index: 0 },
            NativeFn { name: "convSTR".to_string(), function: std::conv::conv_to_string, arg_count: 1, rc_counter: 1, index: 0 },
        ]
    }
}

impl rc::Object for NativeFn {
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
        vec![Value::Fn(self.function)]
    }

    fn set_value(&mut self, _pos: usize, _value: Value) {
    }
    
    fn get_arg_count(&self) -> usize {
        self.arg_count
    }
}