use std::collections::VecDeque;

use crate::vm::{
    bytecode::{Chunk, OpCode, Instruction},
    value::Value,
};

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

    pub fn run(&mut self) {
        loop {
            let instruction = self.get_instruction();
            match instruction.op { 
                OpCode::CONSTANT_FLOAT(index) | OpCode::CONSTANT_INT(index) => {
                    self.stack.push_back(self.chunk.get_value(index));

                },

                OpCode::ADD_FLOAT => {
                    let a = self.stack.pop_front().unwrap().get_float();
                    let b = self.stack.pop_front().unwrap().get_float();
                    self.stack.push_back(Value::Float(a+b));
                },
                OpCode::SUB_FLOAT => {
                    let a = self.stack.pop_front().unwrap().get_float();
                    let b = self.stack.pop_front().unwrap().get_float();
                    self.stack.push_back(Value::Float(a-b));
                },
                OpCode::MUL_FLOAT => {
                    let a = self.stack.pop_front().unwrap().get_float();
                    let b = self.stack.pop_front().unwrap().get_float();
                    self.stack.push_back(Value::Float(a*b));
                },
                OpCode::DIV_FLOAT => {
                    let a = self.stack.pop_front().unwrap().get_float();
                    let b = self.stack.pop_front().unwrap().get_float();
                    self.stack.push_back(Value::Float(a/b));
                },

                OpCode::ADD_INT => {
                    let a = self.stack.pop_front().unwrap().get_int();
                    let b = self.stack.pop_front().unwrap().get_int();
                    self.stack.push_back(Value::Int(a+b));
                },
                OpCode::SUB_INT => {
                    let a = self.stack.pop_front().unwrap().get_int();
                    let b = self.stack.pop_front().unwrap().get_int();
                    self.stack.push_back(Value::Int(a-b));
                },
                OpCode::MUL_INT => {
                    let a = self.stack.pop_front().unwrap().get_int();
                    let b = self.stack.pop_front().unwrap().get_int();
                    self.stack.push_back(Value::Int(a*b));
                },
                OpCode::DIV_INT => {
                    let a = self.stack.pop_front().unwrap().get_int();
                    let b = self.stack.pop_front().unwrap().get_int();
                    self.stack.push_back(Value::Int(a/b));
                },

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
                    
                    break
                },
            }
        }
    }


}