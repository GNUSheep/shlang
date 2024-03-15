use crate::vm::{
    bytecode::{Chunk, OpCode, Instruction},
    value::Value,
};

use crate::objects::rc;

pub struct VM {
    chunk: Chunk,
    stack: Vec<Value>,
    ip: usize,
    rc: rc::ReferenceCounter,
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk: chunk,
            stack: vec![],
            ip: 0,
            rc: rc::ReferenceCounter::init(),
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
                OpCode::CONSTANT_FLOAT(index) | OpCode::CONSTANT_INT(index) | OpCode::CONSTANT_BOOL(index) => {
                    self.stack.push(self.chunk.get_value(index));

                },

                OpCode::ADD_FLOAT => {
                    let a = self.stack.pop().unwrap().get_float();
                    let b = self.stack.pop().unwrap().get_float();
                    self.stack.push(Value::Float(b+a));
                },
                OpCode::SUB_FLOAT => {
                    let a = self.stack.pop().unwrap().get_float();
                    let b = self.stack.pop().unwrap().get_float();
                    self.stack.push(Value::Float(b-a));
                },
                OpCode::MUL_FLOAT => {
                    let a = self.stack.pop().unwrap().get_float();
                    let b = self.stack.pop().unwrap().get_float();
                    self.stack.push(Value::Float(b*a));
                },
                OpCode::DIV_FLOAT => {
                    let a = self.stack.pop().unwrap().get_float();
                    let b = self.stack.pop().unwrap().get_float();
                    self.stack.push(Value::Float(b/a));
                },
                OpCode::EQ_FLOAT => {
                    let a = self.stack.pop().unwrap().get_float();
                    let b = self.stack.pop().unwrap().get_float();

                    self.stack.push(Value::Bool(a==b));
                }
                OpCode::NEG_EQ_FLOAT => {
                    let a = self.stack.pop().unwrap().get_float();
                    let b = self.stack.pop().unwrap().get_float();

                    self.stack.push(Value::Bool(a!=b));
                }
                OpCode::GREATER_FLOAT => {
                    let a = self.stack.pop().unwrap().get_float();
                    let b = self.stack.pop().unwrap().get_float();

                    self.stack.push(Value::Bool(b>a));
                }
                OpCode::EQ_GREATER_FLOAT => {
                    let a = self.stack.pop().unwrap().get_float();
                    let b = self.stack.pop().unwrap().get_float();

                    self.stack.push(Value::Bool(b>=a));
                }
                OpCode::LESS_FLOAT => {
                    let a = self.stack.pop().unwrap().get_float();
                    let b = self.stack.pop().unwrap().get_float();

                    self.stack.push(Value::Bool(b<a));
                }
                OpCode::EQ_LESS_FLOAT => {
                    let a = self.stack.pop().unwrap().get_float();
                    let b = self.stack.pop().unwrap().get_float();

                    self.stack.push(Value::Bool(b<=a));
                }
                
                OpCode::ADD_INT => {
                    let a = self.stack.pop().unwrap().get_int();
                    let b = self.stack.pop().unwrap().get_int();
                    self.stack.push(Value::Int(b+a));
                },
                OpCode::SUB_INT => {
                    let a = self.stack.pop().unwrap().get_int();
                    let b = self.stack.pop().unwrap().get_int();
                    self.stack.push(Value::Int(b-a));
                },
                OpCode::MUL_INT => {
                    let a = self.stack.pop().unwrap().get_int();
                    let b = self.stack.pop().unwrap().get_int();
                    self.stack.push(Value::Int(b*a));
                },
                OpCode::DIV_INT => {
                    let a = self.stack.pop().unwrap().get_int();
                    let b = self.stack.pop().unwrap().get_int();
                    self.stack.push(Value::Int(b/a));
                },
                OpCode::EQ_INT => {
                    let a = self.stack.pop().unwrap().get_int();
                    let b = self.stack.pop().unwrap().get_int();

                    self.stack.push(Value::Bool(a==b));
                }
                OpCode::NEG_EQ_INT => {
                    let a = self.stack.pop().unwrap().get_int();
                    let b = self.stack.pop().unwrap().get_int();

                    self.stack.push(Value::Bool(a!=b));
                }
                OpCode::GREATER_INT => {
                    let a = self.stack.pop().unwrap().get_int();
                    let b = self.stack.pop().unwrap().get_int();

                    self.stack.push(Value::Bool(b>a));
                }
                OpCode::EQ_GREATER_INT => {
                    let a = self.stack.pop().unwrap().get_int();
                    let b = self.stack.pop().unwrap().get_int();

                    self.stack.push(Value::Bool(b>=a));
                }
                OpCode::LESS_INT => {
                    let a = self.stack.pop().unwrap().get_int();
                    let b = self.stack.pop().unwrap().get_int();

                    self.stack.push(Value::Bool(b<a));
                }
                OpCode::EQ_LESS_INT => {
                    let a = self.stack.pop().unwrap().get_int();
                    let b = self.stack.pop().unwrap().get_int();

                    self.stack.push(Value::Bool(b<=a));
                }

                OpCode::NEGATE => {
                    let a = self.stack.pop().unwrap();
                    self.stack.push(-a);
                },

                OpCode::EQ_BOOL => {
                    let a = self.stack.pop().unwrap().get_bool();
                    let b = self.stack.pop().unwrap().get_bool();

                    self.stack.push(Value::Bool(a==b));
                }
                OpCode::NEG_EQ_BOOL => {
                    let a = self.stack.pop().unwrap().get_bool();
                    let b = self.stack.pop().unwrap().get_bool();

                    self.stack.push(Value::Bool(a!=b));
                }

                OpCode::RETURN => {
                    println!("Stack: {:?}", self.stack);
                    
                    break
                },
            }
        }
        self.rc.remove_all();
    }


}