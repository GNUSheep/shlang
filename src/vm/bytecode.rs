use crate::vm::value::{Value, ValuesArray};
use crate::objects::functions;

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum OpCode {
    FUNCTION_DEC(functions::Function),

    CONSTANT_BOOL(usize),
    EQ_BOOL,
    NEG_EQ_BOOL,

    CONSTANT_FLOAT(usize),
    ADD_FLOAT,
    SUB_FLOAT,
    MUL_FLOAT,
    DIV_FLOAT,
    EQ_FLOAT,
    NEG_EQ_FLOAT,
    GREATER_FLOAT,
    EQ_GREATER_FLOAT,
    LESS_FLOAT,
    EQ_LESS_FLOAT,
    
    CONSTANT_INT(usize),
    ADD_INT,
    SUB_INT,
    MUL_INT,
    DIV_INT,
    EQ_INT,
    NEG_EQ_INT,
    GREATER_INT,
    EQ_GREATER_INT,
    LESS_INT,
    EQ_LESS_INT,

    NEGATE,

    RETURN,
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub op: OpCode,
    pub line: u32,
}

#[derive(Debug, Clone)]
pub struct Chunk { 
    pub code: Vec<Instruction>,
    pub values: ValuesArray,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: vec![],
            values: ValuesArray::init(),
        }
    }

    pub fn push(&mut self, instruction: Instruction) {
        self.code.push(instruction);
    }

    pub fn push_value(&mut self, value: Value) -> usize {
        self.values.push(value);
        self.values.len() - 1
    }

    pub fn get_instruction(&self, offset: usize) -> &Instruction {
        &self.code[offset]
    }

    pub fn get_value(&self, index: usize) -> Value {
        self.values.get(index)
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }
} 