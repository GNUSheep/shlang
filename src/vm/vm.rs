use crate::{
    objects::rc::RefObject, 
    vm::{bytecode::{Chunk, Instruction, OpCode},
    value::Value,
}};

use crate::objects::{rc, functions::NativeFn};
use crate::compiler::errors;

pub struct Frame {
    pub chunk: Chunk,
    pub stack: Vec<Value>,
    pub ip: usize,
    pub offset: usize,
}

pub struct VM {
    pub frames: Vec<Frame>,
    pub ip: usize,
    pub rc: rc::ReferenceCounter,
    break_loop: bool,
}

impl VM {
    pub fn new() -> Self {
        Self {
            frames: vec![],
            ip: 0,
            rc: rc::ReferenceCounter::init(),
            break_loop: false,
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
                OpCode::STRUCT_DEC(struct_) => {
                    self.rc.push(Box::new(struct_));
                },
                _ => errors::error_message("RUNTIME ERROR", format!("Declare all - this error should never prints out")),
            }
        }

        Frame{chunk: self.rc.get_object(main_function_index).get_values()[0].get_chunk(), stack: vec![], ip: 0, offset: 0 }
    }

    pub fn run(&mut self) {
        self.frames[self.ip].offset = self.rc.heap.len();
        loop {
            let instruction = self.get_instruction().clone();

            match instruction.op {
                OpCode::RETURN => {
                    if self.ip == 0 {
                        println!("Stack: {:?}", self.frames[self.ip].stack);
                        break
                    }

                    let return_val = self.frames[self.ip].stack.pop().unwrap();
                    
                    let mut instr = self.get_instruction().clone();

                    while matches!(instr.op, OpCode::DEC_RC(_)) && instr.op != OpCode::END_OF_FN {
                        self.run_instruction(instr);

                        instr = self.get_instruction().clone();
                    }
                    
                    self.frames.pop();

                    self.rc.remove();

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

            OpCode::INSTANCE_DEC(mut instance) => {
                let field_count = self.rc.get_object(instance.root_struct_pos).get_arg_count();
                for _ in 0..field_count {
                    instance.fields_values.push(self.frames[self.ip].stack.pop().unwrap())
                }
                instance.fields_values.reverse();

                self.rc.push(Box::new(instance));
            },
            OpCode::GET_INSTANCE_FIELD(pos, field_pos) => {
                let instance_fields = self.rc.get_object(pos+self.frames[self.ip].offset).get_values();
                match instance_fields[0] {
                    Value::InstanceRef(index) => {
                        let pos = self.rc.find_object(index);

                        let fields = self.rc.get_object(pos).get_values();
                        self.frames[self.ip].stack.push(fields[field_pos].clone());
                        return
                    },
                    _ => {},
                };
                self.frames[self.ip].stack.push(instance_fields[field_pos].clone());
            },
            OpCode::SET_INSTANCE_FIELD(pos, field_pos) => {
                let len = self.frames[self.ip].stack.len() - 1;
                let value = self.frames[self.ip].stack[len].clone();

                match self.rc.get_object(self.frames[self.ip].offset + pos).get_values()[0] {
                    Value::InstanceRef(index) => {
                        let pos = self.rc.find_object(index);

                        self.rc.get_object(pos).set_value(field_pos, value);
                        return
                    },
                    _ => {},
                };

                self.rc.get_object(self.frames[self.ip].offset + pos).set_value(field_pos, value);
            },
            OpCode::GET_INSTANCE_RF(pos) => {
                // need to find if other method with using it, would be better
                self.rc.push(Box::new(RefObject { ref_index: pos, rc_counter: 1, index: 0}));
                self.frames[self.ip].stack.push(Value::InstanceRef(0));
            },
            OpCode::METHOD_CALL(mth) => {
                let mut stack: Vec<Value> = vec![];
                let mut instance_rf_count = 0;

                let adder: usize = if mth.is_self_arg { 1 }else { 0 };
                for _ in 0..mth.arg_count + adder {
                    let value = self.frames[self.ip].stack.pop().unwrap();
                    if matches!(value, Value::InstanceRef(_)) {
                        instance_rf_count += 1;
                    }else {
                        stack.push(value);
                    }
                }
                stack.reverse();

                self.frames.push(Frame { chunk: mth.chunk, stack: stack, ip: 0, offset: self.rc.heap.len() - instance_rf_count });

                self.ip += 1;
            }

            OpCode::FUNCTION_CALL(index) => {
                let chunk = self.rc.get_object(index).get_values()[0].clone();

                let mut stack: Vec<Value> = vec![];
                let mut instance_rf_count = 0;

                for _ in 0..self.rc.get_object(index).get_arg_count() {
                    let value = self.frames[self.ip].stack.pop().unwrap();
                    if matches!(value, Value::InstanceRef(_)) {
                        instance_rf_count += 1;
                    }else {
                        stack.push(value);
                    }
                }
                stack.reverse();
                
                self.frames.push(Frame { chunk: chunk.get_chunk().clone(), stack: stack, ip: 0, offset: self.rc.heap.len() - instance_rf_count });
                
                self.ip += 1;
            },
            OpCode::NATIVE_FN_CALL(index) => {
                let native_fn = self.rc.get_object(index).get_values()[0].get_fn();

                let mut stack: Vec<Value> = vec![];
                let len = self.frames[self.ip].stack.len() - 1;
                for i in 0..self.rc.get_object(index).get_arg_count() {
                    stack.push(self.frames[self.ip].stack[len - i].clone());
                }
                stack.reverse();

                let output = native_fn(stack);
                if output != Value::Null {
                    self.frames[self.ip].stack.push(output);
                }
            },
            OpCode::PRINT_FN_CALL(index, arg_count) => {
                let native_fn = self.rc.get_object(index).get_values()[0].get_fn();

                let mut stack: Vec<Value> = vec![];
                let len = self.frames[self.ip].stack.len() - 1;
                for i in 0..arg_count {
                    stack.push(self.frames[self.ip].stack[len - i].clone());
                }
                stack.reverse();

                let output = native_fn(stack);
                if output != Value::Null {
                    self.frames[self.ip].stack.push(output);
                }
            },

            OpCode::IF_STMT_OFFSET(offset) => {
                let index = self.frames[self.ip].stack.len();
                if self.frames[self.ip].stack[index - 1].get_bool() == false || self.break_loop {
                    self.frames[self.ip].ip += offset;
                    self.break_loop = false;
                }
            },

            OpCode::JUMP(offset) => {
                self.frames[self.ip].ip += offset;
            },

            OpCode::LOOP(offset) => {
                self.frames[self.ip].ip -= offset;
            },

            OpCode::BREAK => {
                self.break_loop = true;
            }

            OpCode::POP => {
                self.frames[self.ip].stack.pop();
            },

            OpCode::DEC_RC(pos) => {
                self.rc.dec_counter(self.frames[self.ip].offset+pos);
            },
            OpCode::INC_RC(pos) => {
                self.rc.inc_counter(self.frames[self.ip].offset+pos);
            }

            OpCode::VAR_CALL(index) => {
                let value = self.frames[self.ip].stack[index].clone();
                self.frames[self.ip].stack.push(value);
            },
            OpCode::VAR_SET(index) => {
                let len = self.frames[self.ip].stack.len();
                let value = self.frames[self.ip].stack[len - 1].clone();
                self.frames[self.ip].stack[index] = value;
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
            OpCode::MOD_FLOAT => {
                let a = self.frames[self.ip].stack.pop().unwrap().get_float();
                let b = self.frames[self.ip].stack.pop().unwrap().get_float();
                self.frames[self.ip].stack.push(Value::Float(b%a));
            },       
            OpCode::EQ_FLOAT => {
                let a = self.frames[self.ip].stack.pop().unwrap().get_float();
                let b = self.frames[self.ip].stack.pop().unwrap().get_float();
    
                self.frames[self.ip].stack.push(Value::Bool(a==b));
            },
            OpCode::NEG_EQ_FLOAT => {
                let a = self.frames[self.ip].stack.pop().unwrap().get_float();
                let b = self.frames[self.ip].stack.pop().unwrap().get_float();
    
                self.frames[self.ip].stack.push(Value::Bool(a!=b));
            },
            OpCode::GREATER_FLOAT => {
                let a = self.frames[self.ip].stack.pop().unwrap().get_float();
                let b = self.frames[self.ip].stack.pop().unwrap().get_float();
    
                self.frames[self.ip].stack.push(Value::Bool(b>a));
            },
            OpCode::EQ_GREATER_FLOAT => {
                let a = self.frames[self.ip].stack.pop().unwrap().get_float();
                let b = self.frames[self.ip].stack.pop().unwrap().get_float();
    
                self.frames[self.ip].stack.push(Value::Bool(b>=a));
            },
            OpCode::LESS_FLOAT => {
                let a = self.frames[self.ip].stack.pop().unwrap().get_float();
                let b = self.frames[self.ip].stack.pop().unwrap().get_float();
    
                self.frames[self.ip].stack.push(Value::Bool(b<a));
            },
            OpCode::EQ_LESS_FLOAT => {
                let a = self.frames[self.ip].stack.pop().unwrap().get_float();
                let b = self.frames[self.ip].stack.pop().unwrap().get_float();
    
                self.frames[self.ip].stack.push(Value::Bool(b<=a));
            },
            
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
            OpCode::MOD_INT => {
                let a = self.frames[self.ip].stack.pop().unwrap().get_int();
                let b = self.frames[self.ip].stack.pop().unwrap().get_int();
                self.frames[self.ip].stack.push(Value::Int(b%a));
            },
            OpCode::EQ_INT => {
                let a = self.frames[self.ip].stack.pop().unwrap().get_int();
                let b = self.frames[self.ip].stack.pop().unwrap().get_int();
    
                self.frames[self.ip].stack.push(Value::Bool(a==b));
            },
            OpCode::NEG_EQ_INT => {
                let a = self.frames[self.ip].stack.pop().unwrap().get_int();
                let b = self.frames[self.ip].stack.pop().unwrap().get_int();
    
                self.frames[self.ip].stack.push(Value::Bool(a!=b));
            },
            OpCode::GREATER_INT => {
                let a = self.frames[self.ip].stack.pop().unwrap().get_int();
                let b = self.frames[self.ip].stack.pop().unwrap().get_int();
    
                self.frames[self.ip].stack.push(Value::Bool(b>a));
            },
            OpCode::EQ_GREATER_INT => {
                let a = self.frames[self.ip].stack.pop().unwrap().get_int();
                let b = self.frames[self.ip].stack.pop().unwrap().get_int();
    
                self.frames[self.ip].stack.push(Value::Bool(b>=a));
            },
            OpCode::LESS_INT => {
                let a = self.frames[self.ip].stack.pop().unwrap().get_int();
                let b = self.frames[self.ip].stack.pop().unwrap().get_int();
    
                self.frames[self.ip].stack.push(Value::Bool(b<a));
            },
            OpCode::EQ_LESS_INT => {
                let a = self.frames[self.ip].stack.pop().unwrap().get_int();
                let b = self.frames[self.ip].stack.pop().unwrap().get_int();
    
                self.frames[self.ip].stack.push(Value::Bool(b<=a));
            },
    
            OpCode::NEGATE => {
                let a = self.frames[self.ip].stack.pop().unwrap();
                self.frames[self.ip].stack.push(-a);
            },
    
            OpCode::EQ_BOOL => {
                let a = self.frames[self.ip].stack.pop().unwrap().get_bool();
                let b = self.frames[self.ip].stack.pop().unwrap().get_bool();
    
                self.frames[self.ip].stack.push(Value::Bool(a==b));
            },
            OpCode::NEG_EQ_BOOL => {
                let a = self.frames[self.ip].stack.pop().unwrap().get_bool();
                let b = self.frames[self.ip].stack.pop().unwrap().get_bool();
    
                self.frames[self.ip].stack.push(Value::Bool(a!=b));
            },
    
            opcode => errors::error_message("RUNTIME - VM ERROR", format!("VM - this error should never prints out: {:?}", opcode)),
        }
    }
}