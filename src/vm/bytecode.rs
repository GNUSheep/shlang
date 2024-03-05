use crate::vm::value::{Value, ValuesArray};

#[derive(Debug)]
pub enum OpCode {
    CONSTANT_FLOAT(usize),
    CONSTANT_INT(usize),
    RETURN,
}

#[derive(Debug)]
pub struct Instruction {
    pub op: OpCode,
    pub line: u32,
}

#[derive(Debug)]
pub struct Chunk { 
    code: Vec<Instruction>,
    values: ValuesArray,
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