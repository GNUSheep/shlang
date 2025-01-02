use crate::{
    objects::{rc::RefObject, string::StringMethods}, vm::{bytecode::{Chunk, Instruction, OpCode},
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
        let natives_fn = NativeFn::get_natives_fn();

        for native in natives_fn {
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
                    let name = struct_.name == "String";

                    self.rc.push(Box::new(struct_));

                    if name {
                        let mths_string = StringMethods::get_methods_rc();

                        for obj in mths_string {
                            self.rc.push(Box::new(obj));
                        }
                    }
                    
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

                    while instr.op != OpCode::END_OF_FN {
                        if matches!(instr.op, OpCode::DEC_RC(_)) || matches!(instr.op, OpCode::POP) {
                            self.run_instruction(instr);
                        }
                        
                        instr = self.get_instruction().clone();
                    }
                    self.frames.pop();
                    
                    self.rc.remove();

                    self.ip -= 1;

                    if !matches!(return_val, Value::InstanceRef(_)) {
                        self.frames[self.ip].stack.push(return_val);
                    }
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

            OpCode::STRING_DEC(instance) => {
                self.rc.push(Box::new(instance));
            },
            OpCode::STRING_DEC_VALUE(mut instance) => {
                instance.fields_values.push(self.frames[self.ip].stack.pop().unwrap());
                self.rc.push(Box::new(instance));
            },

            OpCode::INSTANCE_DEC(mut instance, field_count) => {
                for _ in 0..field_count {
                    instance.fields_values.push(self.frames[self.ip].stack.pop().unwrap())
                }
                instance.fields_values.reverse();
                
                self.rc.push(Box::new(instance));
            },
            OpCode::GET_INSTANCE_FIELD(pos, field_pos) => {
                let instance_fields = self.rc.get_object(self.frames[self.ip].offset+pos).get_values();

                match instance_fields[0] {
                    Value::InstanceRef(index) | Value::StringRef(index)  => {
                        let fields = self.rc.get_object(index).get_values();
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
                    Value::InstanceRef(index) | Value::StringRef(index) => {
                        self.rc.get_object(index).set_value(field_pos, value);
                        return
                    },
                    _ => {},
                };

                self.rc.get_object(self.frames[self.ip].offset + pos).set_value(field_pos, value);
            },
            OpCode::GET_INSTANCE_W_OFFSET_RF(index) => {
                let mut offset = self.frames[self.ip].offset + index;
                while matches!(self.rc.get_object(offset).get_values()[0], Value::InstanceRef(_)) {
                    match self.rc.get_object(offset).get_values()[0] {
                        Value::InstanceRef(pos) => {
                            offset = pos;
                        }
                        _ => {},
                    }
                }
                self.rc.push(Box::new(RefObject { ref_index: offset, rc_counter: 1, index: 0}));
                self.frames[self.ip].stack.push(Value::InstanceRef(offset));
            },
            OpCode::GET_INSTANCE_RF(pos) => {
                // need to find if other method with using it, would be better
                let offset = self.frames[self.ip].offset;
                
                self.rc.push(Box::new(RefObject { ref_index: offset+pos, rc_counter: 1, index: 0}));
                self.frames[self.ip].stack.push(Value::InstanceRef(offset+pos));
                println!("{:?}", offset+pos)
            },

            OpCode::GET_LIST_FIELD(pos) => {
                let list_fields = self.rc.get_object(self.frames[self.ip].offset+pos).get_values();

                let field_pos = match self.frames[self.ip].stack.pop() {
                    Some(Value::Int(val)) => {
                        if val < 0 {     
                            errors::error_message("RUNTIME - VM ERROR", 
                                format!("VM - Index cannot be negative {}:", instruction.line));
                            std::process::exit(1);
                        };
                        val as usize
                    }
                    _ => {                        
                        errors::error_message("RUNTIME - VM ERROR", format!("VM - this error should never prints out: run out of stack {}:", instruction.line));
                        std::process::exit(1);
                    },
                };

                if field_pos >= list_fields.len() {                
                    errors::error_message("RUNTIME - VM ERROR", 
                        format!("VM - List index out of range  {}/{} {}:", field_pos, list_fields.len(), instruction.line));
                    std::process::exit(1);
                };
                
                self.frames[self.ip].stack.push(list_fields[field_pos].clone());
            },
            OpCode::GET_LIST(pos) => {
                let list_fields = self.rc.get_object(self.frames[self.ip].offset+pos).get_values();

                let mut list_fields_unwrap = vec![];
                for field in list_fields {
                    match field {
                        Value::InstanceRef(index) => {
                            list_fields_unwrap.push(Value::InstanceObj(self.rc.get_object(index).get_values()));   
                        },
                        Value::StringRef(index) => {
                            list_fields_unwrap.push(self.rc.get_object(index).get_values()[0].clone());
                        },
                        _ => {
                            list_fields_unwrap.push(field);
                        },
                    }
                };
                
                self.frames[self.ip].stack.push(Value::ListObj(list_fields_unwrap));
            },
            OpCode::SET_LIST_FIELD(pos) => {                
                let len = self.frames[self.ip].stack.len() - 1;
                
                let value = match self.frames[self.ip].stack.pop() {
                    Some(val) => val,
                    _ => {
                        errors::error_message("RUNTIME - VM ERROR", format!("VM - this error should never prints out: missing value on stack {}:", instruction.line));
                        std::process::exit(1);
                    }    
                };

                let field_pos = match self.frames[self.ip].stack[len - 1].clone() {
                    Value::Int(val) => {
                        if val < 0 {     
                            errors::error_message("RUNTIME - VM ERROR", 
                                format!("VM - Index cannot be negative {}:", instruction.line));
                        };
                        val as usize
                    }
                    _ => {                        
                        errors::error_message("RUNTIME - VM ERROR", format!("VM - this error should never prints out: bad value on stack {}:", instruction.line));
                        std::process::exit(1);
                    },
                };

                self.rc.get_object(self.frames[self.ip].offset + pos).set_value(field_pos, value);
            },

            OpCode::METHOD_CALL(mth) => {
                let mut stack: Vec<Value> = vec![];
                let mut instance_rf_count = 0;

                let adder: usize = if mth.is_self_arg { 1 }else { 0 };
                for _ in 0..mth.arg_count + adder {
                    let value = self.frames[self.ip].stack.pop().unwrap();
                    if matches!(value, Value::InstanceRef(_)) || matches!(value, Value::StringRef(_)) {
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
                    if matches!(value, Value::InstanceRef(_)) || matches!(value, Value::StringRef(_)) {
                        instance_rf_count += 1;
                    } else {
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
                    let value = self.frames[self.ip].stack[len - i].clone();
                    match value {
                        Value::StringRef(index) => {
                            let fields = self.rc.get_object(index).get_values();

                            stack.push(fields[0].clone());
                        },
                        _ => stack.push(value),
                    }
                }
                stack.reverse();
                let output = native_fn(stack);
                if output != Value::Null {
                    for _ in 0..self.rc.get_object(index).get_arg_count() { self.frames[self.ip].stack.pop(); }; 

                    self.frames[self.ip].stack.push(output);
                }
            },
            OpCode::IO_FN_CALL(index, arg_count) => {
                let native_fn = self.rc.get_object(index).get_values()[0].get_fn();

                let mut stack: Vec<Value> = vec![];
                let len: usize = if self.frames[self.ip].stack.len() != 0 {
                    self.frames[self.ip].stack.len() - 1
                }else { 0 };

                for i in 0..arg_count {
                    let value = self.frames[self.ip].stack[len - i].clone();
                    match value {
                        Value::StringRef(index) => {
                            let pos = self.rc.find_object(index);
                            let fields = self.rc.get_object(pos).get_values();
                            stack.push(fields[0].clone());
                        },
                        _ => {
                            stack.push(value);
                        }
                    }
                }
                stack.reverse();

                let output = native_fn(stack);
                if output != Value::Null {
                    self.frames[self.ip].stack.pop();
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
                let mut offset = self.frames[self.ip].offset+pos;
                while matches!(self.rc.get_object(offset).get_values()[0], Value::InstanceRef(_)) ||
                    matches!(self.rc.get_object(offset).get_values()[0], Value::StringRef(_))
                {
                    match self.rc.get_object(offset).get_values()[0] {
                        Value::InstanceRef(pos) | Value::StringRef(pos) => {
                            self.rc.dec_counter(offset);
                            offset = pos;
                        }
                        _ => {},
                    }
                }
                self.rc.dec_counter(offset);
            },
            OpCode::DEC_TO(index) => {
                for i in (self.frames[self.ip].offset+index..self.rc.heap.len()).rev() {
                    self.rc.dec_counter(i);
                }
            },
            OpCode::INC_RC(pos) => {
                let mut offset = self.frames[self.ip].offset+pos;
                while matches!(self.rc.get_object(offset).get_values()[0], Value::InstanceRef(_)) ||
                    matches!(self.rc.get_object(offset).get_values()[0], Value::StringRef(_))
                {
                    match self.rc.get_object(offset).get_values()[0] {
                        Value::InstanceRef(pos) | Value::StringRef(pos) => {
                            offset = pos;
                        }
                        _ => {},
                    }
                }
                self.rc.inc_counter(offset);
            },
            OpCode::PUSH_STACK(val) => {
                self.frames[self.ip].stack.push(val);
            },
            OpCode::RF_REMOVE => {
                self.rc.remove();
            },

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
    
            OpCode::ADD_STRING => {
                let a = match self.frames[self.ip].stack.pop().unwrap() {
                    Value::StringRef(index) => {
                        let pos = self.rc.find_object(index);

                        let fields = self.rc.get_object(pos).get_values();

                        fields[0].clone()
                    },
                    _ => Value::Null,
                };
                let b = match self.frames[self.ip].stack.pop().unwrap() {
                    Value::StringRef(index) => {
                        let pos = self.rc.find_object(index);

                        let fields = self.rc.get_object(pos).get_values();

                        fields[0].clone()
                    },
                    Value::String(val) => Value::String(val),
                    _ => {
                        Value::Null
                    },
                };
    
                self.frames[self.ip].stack.push(Value::String(b.get_string()+&a.get_string()));
            },
            OpCode::EQ_STRING => {
                let a = match self.frames[self.ip].stack.pop().unwrap() {
                    Value::StringRef(index) => {
                        let pos = self.rc.find_object(index);

                        let fields = self.rc.get_object(pos).get_values();

                        fields[0].clone()
                    },
                    Value::String(val) => Value::String(val),
                    _ => Value::Null,
                };
                let b = match self.frames[self.ip].stack.pop().unwrap() {
                    Value::StringRef(index) => {
                        let pos = self.rc.find_object(index);

                        let fields = self.rc.get_object(pos).get_values();

                        fields[0].clone()
                    },
                    Value::String(val) => Value::String(val),
                    _ => Value::Null,
                };

                self.frames[self.ip].stack.push(Value::Bool(a.get_string()==b.get_string()));
            },
            OpCode::NEG_EQ_STRING => {
                let a = match self.frames[self.ip].stack.pop().unwrap() {
                    Value::StringRef(index) => {
                        let pos = self.rc.find_object(index);

                        let fields = self.rc.get_object(pos).get_values();

                        fields[0].clone()
                    },
                    _ => Value::Null,
                };
                let b = match self.frames[self.ip].stack.pop().unwrap() {
                    Value::StringRef(index) => {
                        let pos = self.rc.find_object(index);

                        let fields = self.rc.get_object(pos).get_values();

                        fields[0].clone()
                    },
                    _ => Value::Null,
                };
    
                self.frames[self.ip].stack.push(Value::Bool(a!=b));
            },

            opcode => errors::error_message("RUNTIME - VM ERROR", format!("VM - this error should never prints out: {:?}", opcode)),
        }
    }
}
