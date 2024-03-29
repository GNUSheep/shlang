use crate::vm::{
    bytecode::{Chunk, Instruction, OpCode},
    value::Value,
};

use crate::objects::{rc, functions::NativeFn};
use crate::compiler::errors;

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

    pub fn declare_native(&mut self) {
        let natvies_fn = NativeFn::get_natives_fn();

        for native in natvies_fn {
            self.rc.push(Box::new(native));
        }
    }

    pub fn declare_all(&mut self, chunk: Chunk) -> Frame {
        self.declare_native();

        let mut main_function_index: usize = 0;
        for instruction in chunk.code {
            match instruction.op {
                OpCode::FUNCTION_DEC(function) => {
                    if function.name.to_ascii_lowercase() == "main" {
                        main_function_index = self.rc.heap.len();
                    }
                    self.rc.push(Box::new(function));
                },
                _ => errors::error_message("RUNTIME ERROR", format!("Declare all - this error should never prints out")),
            }
        }

        Frame{chunk: self.rc.get_object(main_function_index).get_value().get_chunk(), stack: vec![], ip: 0}
    }

    pub fn run(&mut self) {
        loop {
            let instruction = self.get_instruction().clone();

            match instruction.op {
                OpCode::RETURN => {
                    if self.ip == 0 {
                        println!("Stack: {:?}", self.frames[self.ip].stack);
                        break
                    }
                    let return_val = self.frames[self.ip].stack.pop().unwrap();
                    self.frames.pop();
                    
                    self.ip -= 1;
        
                    self.frames[self.ip].stack.push(return_val);
                },
                _ => self.run_instruction(instruction),
            };
        }
        self.rc.remove_all();
    }

    fn run_instruction(&mut self, instruction: Instruction) {
        match instruction.op { 
            OpCode::CONSTANT_FLOAT(index) | OpCode::CONSTANT_INT(index) | OpCode::CONSTANT_BOOL(index)  | OpCode::CONSTANT_NULL(index) => {
                let frame = &mut self.frames[self.ip];
                frame.stack.push(frame.chunk.get_value(index));
    
            },
    
            OpCode::FUNCTION_CALL(index) => {
                let chunk = self.rc.get_object(index).get_value();
                
                let mut stack: Vec<Value> = vec![];
                for _ in 0..self.rc.get_object(index).get_arg_count() {
                    stack.push(self.frames[self.ip].stack.pop().unwrap());
                }
                stack.reverse();
                
                self.frames.push(Frame { chunk: chunk.get_chunk().clone(), stack: stack, ip: 0 });

                self.ip += 1;
            },
    
            OpCode::NATIVE_FN_CALL(index) => {
                let native_fn = self.rc.get_object(index).get_value().get_fn();
                
                let mut stack: Vec<Value> = vec![];
                for _ in 0..self.rc.get_object(index).get_arg_count() {
                    stack.push(self.frames[self.ip].stack.pop().unwrap());
                }
                stack.reverse();

                let output = native_fn(stack);
                if output != Value::Null {
                    self.frames[self.ip].stack.push(output);
                }
            },

            OpCode::PRINT_FN_CALL(index, arg_count) => {
                let native_fn = self.rc.get_object(index).get_value().get_fn();

                let mut stack: Vec<Value> = vec![];
                for _ in 0..arg_count {
                    stack.push(self.frames[self.ip].stack.pop().unwrap());
                }
                stack.reverse();

                let output = native_fn(stack);
                if output != Value::Null {
                    self.frames[self.ip].stack.push(output);
                }
            },

            OpCode::IF_STMT_OFFSET(offset) => {
                let index = self.frames[self.ip].stack.len();
                if self.frames[self.ip].stack[index - 1].get_bool() == false {
                    self.frames[self.ip].ip += offset;
                }
            },

            OpCode::JUMP(offset) => {
                self.frames[self.ip].ip += offset;
            },

            OpCode::POP => {
                self.frames[self.ip].stack.pop();
            }

            OpCode::VAR_CALL(index) => {
                let value = self.frames[self.ip].stack[index].clone();
                self.frames[self.ip].stack.push(value);
            },
            OpCode::VAR_SET(index) => {
                let value = self.frames[self.ip].stack.pop().unwrap();
                self.frames[self.ip].stack[index] = value;
            }
    
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
    
            _ => errors::error_message("RUNTIME - VM ERROR", format!("VM - this error should never prints out")),
        }
    }
}