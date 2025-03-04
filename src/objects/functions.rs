use crate::{
    objects::rc, vm::{bytecode, value::Value},
    frontend::tokens::{TokenType, Keywords},
    compiler::compiler::Symbol,
    std,
};

#[derive(Clone, Debug, PartialEq)]
pub enum SpecialType {
    String,
    List(Value),
    Null,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Local {
    pub name: String,
    pub local_type: TokenType,
    pub is_special: SpecialType,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub chunk: bytecode::Chunk,
    pub locals: Vec<Local>,
    pub instances: Vec<Local>,
    pub output_type: TokenType,
    pub arg_count: usize,
    pub arg_type: Vec<TokenType>,
    pub is_self_arg: bool,
    rc_counter: usize,
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

    fn is_empty_obj(&self) -> bool {
        false
    }

    fn get_value(&self, _pos: usize) -> Value {
        Value::Chunk(self.chunk.clone())
    }
    
    fn get_values(&self) -> Vec<Value> {
        vec![Value::Chunk(self.chunk.clone())]
    }

    fn set_value(&mut self, _pos: usize, _value: Value) {}

    fn replace_values(&mut self, _value: Vec<Value>) {}

    fn get_values_len(&self) -> usize {
        1
    }

    fn get_arg_count(&self) -> usize {
        self.arg_count
    }
}

impl Function {
    pub fn new(name: String) -> Self {
        Self {
            name,
            chunk: bytecode::Chunk::new(),
            locals: vec![],
            instances: vec![],
            output_type: TokenType::KEYWORD(Keywords::NULL),
            arg_count: 0,
            arg_type: vec![],
            is_self_arg: false,
            rc_counter: 1,
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
    pub function: fn(Vec<Value>) -> Value,
    pub arg_count: usize,
    pub rc_counter: usize,
}

impl NativeFn {
    pub fn get_natives_symbols() -> Vec<Symbol> {
        vec![
            Symbol { name: "print".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::KEYWORD(Keywords::NULL), arg_count: 1 },
            Symbol { name: "println".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::KEYWORD(Keywords::NULL), arg_count: 1 },
            Symbol { name: "input".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::STRING, arg_count: 1 },
            Symbol { name: "conv".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::INT, arg_count: 1 },
            Symbol { name: "convf".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::FLOAT, arg_count: 1 },
            Symbol { name: "convstr".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::STRING, arg_count: 1 },
            Symbol { name: "abs".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::INT, arg_count: 1 },
            Symbol { name: "absf".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::FLOAT, arg_count: 1 },
            Symbol { name: "pow".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::INT, arg_count: 2 },
            Symbol { name: "powf".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::FLOAT, arg_count: 2 },
            Symbol { name: "min".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::INT, arg_count: 2 },
            Symbol { name: "minf".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::FLOAT, arg_count: 2 },
            Symbol { name: "max".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::INT, arg_count: 2 },
            Symbol { name: "maxf".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::FLOAT, arg_count: 2 },
            Symbol { name: "sqrt".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::INT, arg_count: 1 },
            Symbol { name: "sqrtf".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::FLOAT, arg_count: 1 },
            Symbol { name: "roundf".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::FLOAT, arg_count: 2 },
            Symbol { name: "floorf".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::FLOAT, arg_count: 2 },
            Symbol { name: "ceilf".to_string(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::FLOAT, arg_count: 2 },  
        ]
    }

    pub fn get_natives_fn() -> Vec<NativeFn> {
        vec![
            NativeFn { function: std::print::print, arg_count: 1, rc_counter: 1 },
            NativeFn { function: std::print::println, arg_count: 1, rc_counter: 1 },
            NativeFn { function: std::input::input, arg_count: 1, rc_counter: 1 },
            NativeFn { function: std::conv::conv_to_int, arg_count: 1, rc_counter: 1 },
            NativeFn { function: std::conv::conv_to_float, arg_count: 1, rc_counter: 1 },
            NativeFn { function: std::conv::conv_to_string, arg_count: 1, rc_counter: 1 },
            NativeFn { function: std::math::abs_int, arg_count: 1, rc_counter: 1 },
            NativeFn { function: std::math::abs_float, arg_count: 1, rc_counter: 1 },
            NativeFn { function: std::math::pow_int, arg_count: 2, rc_counter: 1 },
            NativeFn { function: std::math::pow_float, arg_count: 2, rc_counter: 1 },
            NativeFn { function: std::math::min_int, arg_count: 2, rc_counter: 1 },
            NativeFn { function: std::math::min_float, arg_count: 2, rc_counter: 1 },
            NativeFn { function: std::math::max_int, arg_count: 2, rc_counter: 1 },
            NativeFn { function: std::math::max_float, arg_count: 2, rc_counter: 1 },
            NativeFn { function: std::math::sqrt_int, arg_count: 1, rc_counter: 1 },
            NativeFn { function: std::math::sqrt_float, arg_count: 1, rc_counter: 1 },
            NativeFn { function: std::math::round, arg_count: 2, rc_counter: 1 },
            NativeFn { function: std::math::floor, arg_count: 2, rc_counter: 1 },
            NativeFn { function: std::math::ceil, arg_count: 2, rc_counter: 1 },
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

    fn is_empty_obj(&self) -> bool {
        false
    }

    fn get_value(&self, _pos: usize) -> Value {
        Value::Fn(self.function)
    }
    
    fn get_values(&self) -> Vec<Value> {
        vec![Value::Fn(self.function)]
    }

    fn set_value(&mut self, _pos: usize, _value: Value) {}

    fn replace_values(&mut self, _value: Vec<Value>) {}

    fn get_values_len(&self) -> usize {
        1
    }
    
    fn get_arg_count(&self) -> usize {
        self.arg_count
    }
}
