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
}

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub chunk: bytecode::Chunk,
    pub locals: Vec<Local>,
    pub output_type: TokenType,
    pub arg_count: usize,
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

    fn get_value(&self) -> Value {
        Value::Chunk(self.chunk.clone())
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
            output_type: TokenType::KEYWORD(Keywords::NULL),
            arg_count: 0,
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
}

pub struct NativeFn {
    pub name: String,
    pub function: fn(Vec<Value>) -> Value,
    pub arg_count: usize,
    rc_counter: usize,
    index: usize,
}

impl NativeFn {
    pub fn get_natives_symbols() -> Vec<Symbol> {
        vec![
            Symbol { name: "print".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::KEYWORD(Keywords::NULL), arg_count: 1 },
            Symbol { name: "println".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::KEYWORD(Keywords::NULL), arg_count: 1 },
            Symbol { name: "test_returning".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::INT, arg_count: 0 },
        ]
    }

    pub fn get_natives_fn() -> Vec<NativeFn> {
        vec![
            NativeFn { name: "print".to_string(), function: std::print::print, arg_count: 1, rc_counter: 1, index: 0 },
            NativeFn { name: "println".to_string(), function: std::print::println, arg_count: 1, rc_counter: 1, index: 0 },
            NativeFn { name: "test_returning".to_string(), function: std::print::test_returning, arg_count: 0, rc_counter: 1, index: 0 },
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

    fn get_value(&self) -> Value {
        Value::Fn(self.function)
    }

    fn get_arg_count(&self) -> usize {
        self.arg_count
    }
}