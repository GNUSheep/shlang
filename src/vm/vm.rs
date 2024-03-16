use crate::vm::{
    bytecode::{Chunk, OpCode, Instruction},
    value::Value,
};

use crate::objects::rc;

use super::bytecode;

pub struct Frame {
    pub chunk: Chunk,
    pub stack: Vec<Value>,
    pub ip: usize,
}

pub struct VM {
    pub frames: Vec<Frame>,
    pub ip: usize,
    pub rc: rc::ReferenceCounter,
}

impl VM {
    pub fn new() -> Self {
        Self {
            frames: vec![],
            ip: 0,
            rc: rc::ReferenceCounter::init(),
        }
    }

    pub fn get_instruction(&mut self) -> &Instruction {
        let frame = &mut self.frames[self.ip];
        frame.ip += 1;
        frame.chunk.get_instruction(frame.ip - 1)
    }

    pub fn declare_all(&mut self, chunk: Chunk) -> Frame {
        let mut main_function_index: i32 = -1;
        for instruction in chunk.code {
            match instruction.op {
                OpCode::FUNCTION_DEC(function) => {
                    if function.name.to_ascii_lowercase() == "main" {
                        main_function_index = self.rc.heap.len() as i32;
                    }
                    self.rc.push(Box::new(function));
                },
                _ => panic!("ERROR"),
            }
        }

        if main_function_index == -1 {
            panic!("CANNOT FIND MAIN FUNC");
        }

        Frame{chunk: self.rc.get_object(main_function_index as usize).get_value().get_chunk(), stack: vec![], ip: 0}
    }

    pub fn run(&mut self) {
        loop {
            let instruction = self.get_instruction();
            match instruction.op.clone() { 
                OpCode::CONSTANT_FLOAT(index) | OpCode::CONSTANT_INT(index) | OpCode::CONSTANT_BOOL(index) => {
                    let frame = &mut self.frames[self.ip];
                    frame.stack.push(frame.chunk.get_value(index));

                },

                OpCode::ADD_FLOAT => {
                    let a = self.frames[self.ip].stack.pop().unwrap().get_float();
                    let b = self.frames[self.ip].stack.pop().unwrap().get_float();
                    self.frames[self.ip].stack.push(Value::Float(b+a));
                },
                OpCode::SUB_FLOAT => {
                    let a = self.frames[self.ip].stack.pop().unwrap().get_float();
                    let b = self.frames[self.ip].stack.pop().unwrap().get_float();
                    self.frames[self.ip].stack.push(Value::Float(b-a));
                },
                OpCode::MUL_FLOAT => {
                    let a = self.frames[self.ip].stack.pop().unwrap().get_float();
                    let b = self.frames[self.ip].stack.pop().unwrap().get_float();
                    self.frames[self.ip].stack.push(Value::Float(b*a));
                },
                OpCode::DIV_FLOAT => {
                    let a = self.frames[self.ip].stack.pop().unwrap().get_float();
                    let b = self.frames[self.ip].stack.pop().unwrap().get_float();
                    self.frames[self.ip].stack.push(Value::Float(b/a));
                },
                OpCode::EQ_FLOAT => {
                    let a = self.frames[self.ip].stack.pop().unwrap().get_float();
                    let b = self.frames[self.ip].stack.pop().unwrap().get_float();

                    self.frames[self.ip].stack.push(Value::Bool(a==b));
                }
                OpCode::NEG_EQ_FLOAT => {
                    let a = self.frames[self.ip].stack.pop().unwrap().get_float();
                    let b = self.frames[self.ip].stack.pop().unwrap().get_float();

                    self.frames[self.ip].stack.push(Value::Bool(a!=b));
                }
                OpCode::GREATER_FLOAT => {
                    let a = self.frames[self.ip].stack.pop().unwrap().get_float();
                    let b = self.frames[self.ip].stack.pop().unwrap().get_float();

                    self.frames[self.ip].stack.push(Value::Bool(b>a));
                }
                OpCode::EQ_GREATER_FLOAT => {
                    let a = self.frames[self.ip].stack.pop().unwrap().get_float();
                    let b = self.frames[self.ip].stack.pop().unwrap().get_float();

                    self.frames[self.ip].stack.push(Value::Bool(b>=a));
                }
                OpCode::LESS_FLOAT => {
                    let a = self.frames[self.ip].stack.pop().unwrap().get_float();
                    let b = self.frames[self.ip].stack.pop().unwrap().get_float();

                    self.frames[self.ip].stack.push(Value::Bool(b<a));
                }
                OpCode::EQ_LESS_FLOAT => {
                    let a = self.frames[self.ip].stack.pop().unwrap().get_float();
                    let b = self.frames[self.ip].stack.pop().unwrap().get_float();

                    self.frames[self.ip].stack.push(Value::Bool(b<=a));
                }
                
                OpCode::ADD_INT => {
                    let a = self.frames[self.ip].stack.pop().unwrap().get_int();
                    let b = self.frames[self.ip].stack.pop().unwrap().get_int();
                    self.frames[self.ip].stack.push(Value::Int(b+a));
                },
                OpCode::SUB_INT => {
                    let a = self.frames[self.ip].stack.pop().unwrap().get_int();
                    let b = self.frames[self.ip].stack.pop().unwrap().get_int();
                    self.frames[self.ip].stack.push(Value::Int(b-a));
                },
                OpCode::MUL_INT => {
                    let a = self.frames[self.ip].stack.pop().unwrap().get_int();
                    let b = self.frames[self.ip].stack.pop().unwrap().get_int();
                    self.frames[self.ip].stack.push(Value::Int(b*a));
                },
                OpCode::DIV_INT => {
                    let a = self.frames[self.ip].stack.pop().unwrap().get_int();
                    let b = self.frames[self.ip].stack.pop().unwrap().get_int();
                    self.frames[self.ip].stack.push(Value::Int(b/a));
                },
                OpCode::EQ_INT => {
                    let a = self.frames[self.ip].stack.pop().unwrap().get_int();
                    let b = self.frames[self.ip].stack.pop().unwrap().get_int();

                    self.frames[self.ip].stack.push(Value::Bool(a==b));
                }
                OpCode::NEG_EQ_INT => {
                    let a = self.frames[self.ip].stack.pop().unwrap().get_int();
                    let b = self.frames[self.ip].stack.pop().unwrap().get_int();

                    self.frames[self.ip].stack.push(Value::Bool(a!=b));
                }
                OpCode::GREATER_INT => {
                    let a = self.frames[self.ip].stack.pop().unwrap().get_int();
                    let b = self.frames[self.ip].stack.pop().unwrap().get_int();

                    self.frames[self.ip].stack.push(Value::Bool(b>a));
                }
                OpCode::EQ_GREATER_INT => {
                    let a = self.frames[self.ip].stack.pop().unwrap().get_int();
                    let b = self.frames[self.ip].stack.pop().unwrap().get_int();

                    self.frames[self.ip].stack.push(Value::Bool(b>=a));
                }
                OpCode::LESS_INT => {
                    let a = self.frames[self.ip].stack.pop().unwrap().get_int();
                    let b = self.frames[self.ip].stack.pop().unwrap().get_int();

                    self.frames[self.ip].stack.push(Value::Bool(b<a));
                }
                OpCode::EQ_LESS_INT => {
                    let a = self.frames[self.ip].stack.pop().unwrap().get_int();
                    let b = self.frames[self.ip].stack.pop().unwrap().get_int();

                    self.frames[self.ip].stack.push(Value::Bool(b<=a));
                }

                OpCode::NEGATE => {
                    let a = self.frames[self.ip].stack.pop().unwrap();
                    self.frames[self.ip].stack.push(-a);
                },

                OpCode::EQ_BOOL => {
                    let a = self.frames[self.ip].stack.pop().unwrap().get_bool();
                    let b = self.frames[self.ip].stack.pop().unwrap().get_bool();

                    self.frames[self.ip].stack.push(Value::Bool(a==b));
                }
                OpCode::NEG_EQ_BOOL => {
                    let a = self.frames[self.ip].stack.pop().unwrap().get_bool();
                    let b = self.frames[self.ip].stack.pop().unwrap().get_bool();

                    self.frames[self.ip].stack.push(Value::Bool(a!=b));
                }

                OpCode::RETURN => {
                    println!("Stack: {:?}", self.frames[self.ip].stack);
                    
                    break
                },

                _ => panic!("ERROR"),
            }
        }
        self.rc.remove_all();
    }


}