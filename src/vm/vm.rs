use std::collections::VecDeque;

use crate::vm::{
    bytecode::{Chunk, OpCode, Instruction},
    value::Value,
};

pub enum Error {
    Undefined(String),
}

pub enum Result {
    Ok,
    Err(Error),
}

pub struct VM {
    chunk: Chunk,
    stack: VecDeque<Value>,
    ip: usize,
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk: chunk,
            stack: VecDeque::new(),
            ip: 0,
        }
    }

    pub fn get_instruction(&mut self) -> &Instruction {
        self.ip += 1;
        self.chunk.get_instruction(self.ip - 1)
    }

    pub fn run(&mut self) -> Result {
        loop {
            let instruction = self.get_instruction();
            match instruction.op { 
                OpCode::CONSTANT_FLOAT(index) | OpCode::CONSTANT_INT(index) => {
                    self.stack.push_back(self.chunk.get_value(index));

                }

                OpCode::RETURN => {
                    match self.stack.pop_back() {
                        Some(constant) => {
                            match constant {
                                Value::Float(val) => println!("Float: {}", val),
                                Value::Int(val) => println!("Int: {}", val),
                            }
                        }
                        None => {},
                    }
                    
                    return Result::Ok
                },
            }
        }
    }


}