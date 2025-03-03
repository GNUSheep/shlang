use crate::{
    objects::{string::StringMethods, structs::StructInstance}, vm::{bytecode::{Chunk, Instruction, OpCode},
    value::Value,
}};

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
                _ => errors::error_message("shlang/vm/vm.rs".to_string(), "RUNTIME ERROR", format!("Declare all - this error should never prints out")),
            }
        }

        Frame{chunk: self.rc.get_object(main_function_index).get_values()[0].get_chunk(), stack: vec![], ip: 0}
    }

    pub fn run(&mut self) {
        loop {
            let instruction = self.get_instruction().clone();
            match instruction.op {
                OpCode::RETURN => {
                    if self.ip == 0 {
                        // println!("Stack: {:?}", self.frames[self.ip].stack);
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
                    self.frames[self.ip].stack.push(return_val);
                },
                _ => {
                    self.run_instruction(instruction);
                },
            };
        }
        self.rc.remove_all();
    }

    fn run_instruction(&mut self, instruction: Instruction) {
        // println!("{:?}", instruction);
        match instruction.op {
            OpCode::CONSTANT_FLOAT(index) | OpCode::CONSTANT_INT(index) | OpCode::CONSTANT_BOOL(index)  | OpCode::CONSTANT_NULL(index) => {
                let frame = &mut self.frames[self.ip];
                frame.stack.push(frame.chunk.get_value(index));
            },

            OpCode::STRING_DEC(instance) => {
                let heap_pos = self.rc.heap.len();
                self.rc.push(Box::new(instance));
                self.frames[self.ip].stack.push(Value::StringRef(heap_pos));
            },
            OpCode::STRING_DEC_VALUE(mut instance) => {
                let heap_pos = self.rc.heap.len();
                instance.fields_values.push(self.frames[self.ip].stack.pop().unwrap());
                self.rc.push(Box::new(instance));
                self.frames[self.ip].stack.push(Value::StringRef(heap_pos));
            },

            OpCode::INSTANCE_DEC(mut instance, field_count) => {
                for _ in 0..field_count {
                    instance.fields_values.push(self.frames[self.ip].stack.pop().unwrap())
                }
                instance.fields_values.reverse();

                // Make it look for empty spaces
                let heap_pos = self.rc.heap.len();
                self.rc.push(Box::new(instance));
                self.frames[self.ip].stack.push(Value::InstanceRef(heap_pos));
            },
            OpCode::GET_INSTANCE_FIELD(field_pos) => {
                let instance_obj = match self.frames[self.ip].stack.pop().unwrap() {
                    Value::InstanceRef(heap_pos) | Value::StringRef(heap_pos) => {
                        self.rc.get_object(heap_pos)
                    }
                    _ => panic!("Error, this type of value shoudnt be here"),
                };
                instance_obj.dec_counter();

                let field_value = instance_obj.get_values()[field_pos].clone();
                match field_value {
                    Value::StringRef(heap_pos) | Value::InstanceRef(heap_pos) => self.rc.get_object(heap_pos).inc_counter(),
                    _ => {},
                }
                
                self.frames[self.ip].stack.push(field_value);
            },
            OpCode::SET_INSTANCE_FIELD(field_pos) => {
                let value = self.frames[self.ip].stack.pop().unwrap();

                let (heap_pos, field) = match self.frames[self.ip].stack.pop().unwrap() {
                    Value::InstanceRef(heap_pos) | Value::StringRef(heap_pos) => {
                        (heap_pos, self.rc.get_object(heap_pos).get_values()[field_pos].clone())
                    },
                    _ => panic!("Error, this type of value shoudnt be here"),
                };

                self.rc.get_object(heap_pos).dec_counter();
                
                match field {
                    Value::StringRef(field_heap_pos) | Value::InstanceRef(field_heap_pos) => {
                        self.rc.get_object(field_heap_pos).dec_counter();
                    },
                    _ => {}
                }

                self.rc.get_object(heap_pos).set_value(field_pos, value);

            },
            OpCode::GET_INSTANCE_RF(pos) => {
                let instance_rf = self.frames[self.ip].stack[pos].clone();
                self.frames[self.ip].stack.push(instance_rf);
            },
 
            OpCode::GET_LIST_FIELD => {
                let field_pos = match self.frames[self.ip].stack.pop() {
                    Some(Value::Int(val)) => {
                        if val < 0 {     
                            errors::error_message("shlang/vm/vm.rs".to_string(), "RUNTIME - VM ERROR", 
                                format!("VM - Index cannot be negative {}:", instruction.line));
                            std::process::exit(1);
                        };
                        
                        val as usize
                    }
                    _ => {                        
                        errors::error_message("shlang/vm/vm.rs".to_string(), "RUNTIME - VM ERROR", format!("VM - this error should never prints out: run out of stack {}:", instruction.line));
                        std::process::exit(1);
                    },
                };

                let instance_obj = match self.frames[self.ip].stack.pop().unwrap() {
                    Value::InstanceRef(heap_pos) => {
                        self.rc.get_object(heap_pos)
                    }
                    _ => panic!("Error, this type of value shoudnt be here: GET LIST FIELD"),
                };

                instance_obj.dec_counter();

                if field_pos as usize >= instance_obj.get_values().len() {
                    errors::error_message("shlang/vm/vm.rs".to_string(), "RUNTIME - VM ERROR", 
                        format!("VM - List index out of range  {}/{} {}:", field_pos, instance_obj.get_values().len(), instruction.line));
                    std::process::exit(1);
                }

                let field_value = instance_obj.get_values()[field_pos].clone();

                match field_value {
                    Value::StringRef(heap_pos) | Value::InstanceRef(heap_pos) => self.rc.get_object(heap_pos).inc_counter(),
                    _ => {},
                }
                
                self.frames[self.ip].stack.push(field_value);
            },
            OpCode::SET_LIST_FIELD => {
                let value = self.frames[self.ip].stack.pop().unwrap();

                let field_pos = match self.frames[self.ip].stack.pop() {
                    Some(Value::Int(val)) => {
                        if val < 0 {     
                            errors::error_message("shlang/vm/vm.rs".to_string(), "RUNTIME - VM ERROR", 
                                format!("VM - Index cannot be negative {}:", instruction.line));
                            std::process::exit(1);
                        };
                       
                        val as usize
                    }
                    _ => {                        
                        errors::error_message("shlang/vm/vm.rs".to_string(), "RUNTIME - VM ERROR", format!("VM - this error should never prints out: run out of stack {}:", instruction.line));
                        std::process::exit(1);
                    },
                };

                let (heap_pos, field_value) = match self.frames[self.ip].stack.pop().unwrap() {
                    Value::InstanceRef(heap_pos) => {
                        (heap_pos, self.rc.get_object(heap_pos).get_values()[field_pos].clone())
                    }
                    _ => panic!("Error, this type of value shoudnt be here: SET LIST FIELD"),
                };

                self.rc.get_object(heap_pos).dec_counter();

                if field_pos >= self.rc.get_object(heap_pos).get_values().len() {
                    errors::error_message("shlang/vm/vm.rs".to_string(), "RUNTIME - VM ERROR", 
                        format!("VM - List index out of range  {}/{} {}:", field_pos, self.rc.get_object(heap_pos).get_values().len(), instruction.line));
                    std::process::exit(1);
                }
                
                match field_value {
                    Value::StringRef(heap_pos) | Value::InstanceRef(heap_pos) => self.rc.get_object(heap_pos).dec_counter(),
                    _ => {},
                }
                
                self.rc.get_object(heap_pos).set_value(field_pos, value);
            },
            OpCode::LIST_PUSH => {
                let instance_obj = match self.frames[self.ip].stack.pop().unwrap() {
                    Value::InstanceRef(heap_pos) => {
                        self.rc.get_object(heap_pos)
                    }
                    _ => panic!("Error, this type of value shoudnt be here: LIST PUSH"),
                };

                let value = self.frames[self.ip].stack.pop().unwrap();

                let mut list_values = instance_obj.get_values();

                list_values.push(value);

                instance_obj.replace_values(list_values);
            },
            OpCode::LIST_POP => {
                let instance_obj = match self.frames[self.ip].stack.pop().unwrap() {
                    Value::InstanceRef(heap_pos) => {
                        self.rc.get_object(heap_pos)
                    }
                    _ => panic!("Error, this type of value shoudnt be here: LIST PUSH"),
                };

                let mut list_values = instance_obj.get_values();

                match list_values.pop() {
                    Some(val) => {
                        self.frames[self.ip].stack.push(val);
                    }
                    None => {
                        errors::error_message("shlang/vm/vm.rs".to_string(), "RUNTIME - VM ERROR", 
                            format!("VM - List out of element cannot POP  {}:", instruction.line));
                        std::process::exit(1);
                    }
                }

                instance_obj.replace_values(list_values);
            },
            OpCode::LIST_INSERT => {
                let heap_pos = match self.frames[self.ip].stack.pop().unwrap() {
                    Value::InstanceRef(heap_pos) => heap_pos,
                    _ => panic!("Error, this type of value shoudnt be here: LIST PUSH"),
                };

                let field_pos = match self.frames[self.ip].stack.pop() {
                    Some(Value::Int(val)) => {
                        if val < 0 {     
                            errors::error_message("shlang/vm/vm.rs".to_string(), "RUNTIME - VM ERROR", 
                                format!("VM - Index cannot be negative {}:", instruction.line));
                            std::process::exit(1);
                        };
                       
                        val as usize
                    }
                    _ => {                        
                        errors::error_message("shlang/vm/vm.rs".to_string(), "RUNTIME - VM ERROR", format!("VM - this error should never prints out: run out of stack {}:", instruction.line));
                        std::process::exit(1);
                    },
                };

                if field_pos >= self.rc.get_object(heap_pos).get_values().len() {
                    errors::error_message("shlang/vm/vm.rs".to_string(), "RUNTIME - VM ERROR", 
                        format!("VM - List index out of range  {}/{} {}:", field_pos, self.rc.get_object(heap_pos).get_values().len(), instruction.line));
                    std::process::exit(1);
                }

                let value = self.frames[self.ip].stack.pop().unwrap();

                let mut list_values = self.rc.get_object(heap_pos).get_values();

                list_values.insert(field_pos, value);

                self.rc.get_object(heap_pos).replace_values(list_values);
            },
            OpCode::LIST_REMOVE => {
                let heap_pos = match self.frames[self.ip].stack.pop().unwrap() {
                    Value::InstanceRef(heap_pos) => heap_pos,
                    _ => panic!("Error, this type of value shoudnt be here: LIST PUSH"),
                };

                let field_pos = match self.frames[self.ip].stack.pop() {
                    Some(Value::Int(val)) => {
                        if val < 0 {     
                            errors::error_message("shlang/vm/vm.rs".to_string(), "RUNTIME - VM ERROR", 
                                format!("VM - Index cannot be negative {}:", instruction.line));
                            std::process::exit(1);
                        };
                       
                        val as usize
                    }
                    _ => {                        
                        errors::error_message("shlang/vm/vm.rs".to_string(), "RUNTIME - VM ERROR", format!("VM - this error should never prints out: run out of stack {}:", instruction.line));
                        std::process::exit(1);
                    },
                };

                if field_pos >= self.rc.get_object(heap_pos).get_values().len() {
                    errors::error_message("shlang/vm/vm.rs".to_string(), "RUNTIME - VM ERROR", 
                        format!("VM - List index out of range  {}/{} {}:", field_pos, self.rc.get_object(heap_pos).get_values().len(), instruction.line));
                    std::process::exit(1);
                }

                let mut list_values = self.rc.get_object(heap_pos).get_values();

                match list_values[field_pos] {
                    Value::StringRef(heap_pos) | Value::InstanceRef(heap_pos) => self.rc.get_object(heap_pos).dec_counter(),
                    _ => {},
                }
                
                list_values.remove(field_pos);

                self.rc.get_object(heap_pos).replace_values(list_values);
            },
            OpCode::LIST_LEN => {
                let instance_obj = match self.frames[self.ip].stack.pop().unwrap() {
                    Value::InstanceRef(heap_pos) => {
                        self.rc.get_object(heap_pos)
                    }
                    _ => panic!("Error, this type of value shoudnt be here: LIST PUSH"),
                };

                let list_values = instance_obj.get_values();

                let list_values_len = list_values.len();

                self.frames[self.ip].stack.push(Value::Int(list_values_len as i128));
            },
            OpCode::LIST_SORT => {
                let instance_obj = match self.frames[self.ip].stack.pop().unwrap() {
                    Value::InstanceRef(heap_pos) => {
                        self.rc.get_object(heap_pos)
                    }
                    _ => panic!("Error, this type of value shoudnt be here: LIST PUSH"),
                };

                let mut list_values = instance_obj.get_values();

                list_values.sort_by(|a, b| a.sort(b));

                instance_obj.replace_values(list_values);
            },

            OpCode::METHOD_CALL(mth) => {
                let mut stack: Vec<Value> = vec![];

                let adder: usize = if mth.is_self_arg { 1 } else { 0 };
                for _ in 0..mth.arg_count + adder {
                    let value = self.frames[self.ip].stack.pop().unwrap();
                    stack.push(value);
                }
                stack.reverse();

                self.frames.push(Frame { chunk: mth.chunk, stack, ip: 0});

                self.ip += 1;
            }

            OpCode::FUNCTION_CALL(index) => {
                let chunk = self.rc.get_object(index).get_values()[0].clone();

                let mut stack: Vec<Value> = vec![];

                for _ in 0..self.rc.get_object(index).get_arg_count() {
                    let value = self.frames[self.ip].stack.pop().unwrap();
                    stack.push(value);
                }
                stack.reverse();

                self.frames.push(Frame { chunk: chunk.get_chunk().clone(), stack, ip: 0});
                
                self.ip += 1;
            },
            OpCode::NATIVE_FN_CALL(index) => {
                let native_fn = self.rc.get_object(index).get_values()[0].get_fn();

                let mut stack: Vec<Value> = vec![];
                let len = self.frames[self.ip].stack.len() - 1;

                let arg_count = self.rc.get_object(index).get_arg_count();

                for i in 0..arg_count {
                    let mut value = self.frames[self.ip].stack[len - i].clone();
                    match value {
                        Value::StringRef(index) => {
                            value = self.rc.get_object(index).get_values()[0].clone()
                        },
                        _ => {}
                    }
                    stack.push(value)
                }
                stack.reverse();
                let output = native_fn(stack);

                for i in 0..arg_count {
                    match self.frames[self.ip].stack[len - i].clone() {
                        Value::InstanceRef(_) | Value::StringRef(_) => {
                            self.run_instruction(Instruction { op: OpCode::DEC_RC(len - i), line: instruction.line });
                        }
                        _ => {},
                    }
                    self.frames[self.ip].stack.pop();
                }

                self.rc.remove();
                
                self.frames[self.ip].stack.push(output);
            },
            OpCode::IO_FN_CALL(index, arg_count) => {
                let native_fn = self.rc.get_object(index).get_values()[0].get_fn();
                let mut stack: Vec<Value> = vec![];
                let len: usize = if self.frames[self.ip].stack.len() != 0 {
                    self.frames[self.ip].stack.len() - 1
                }else { 0 };

                for i in 0..arg_count {
                    let mut value = self.frames[self.ip].stack[len - i].clone();
                    match value {
                        Value::StringRef(index) => {
                            value = self.rc.get_object(index).get_values()[0].clone()
                        },
                        _ => {}
                    }
                    stack.push(value);
                }
                stack.reverse();

                let output = native_fn(stack);
                // For know this is okay, but for futrue, IO functioncs need to be rewrited and done as Natvie Functions
                for i in 0..arg_count {
                    match self.frames[self.ip].stack[len - i].clone() {
                        Value::InstanceRef(_) | Value::StringRef(_) => {
                            self.run_instruction(Instruction { op: OpCode::DEC_RC(len - i), line: instruction.line });
                        }
                        _ => {},
                    }
                    self.run_instruction(Instruction { op: OpCode::POP, line: instruction.line });
                }

                self.rc.remove();

                match output {
                    Value::String(_) => {
                        let mut instance = StructInstance::new();
                        instance.fields_values.push(output);

                        let heap_pos = self.rc.heap.len();
                        self.rc.push(Box::new(instance));
                        self.frames[self.ip].stack.push(Value::StringRef(heap_pos));
                    }
                    output => self.frames[self.ip].stack.push(output),                  
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
            OpCode::POP_UNUSED => {
                let value = self.frames[self.ip].stack.pop().unwrap();

                match value {
                    Value::StringRef(heap_pos) | Value::InstanceRef(heap_pos) => {
                        self.rc.get_object(heap_pos).dec_counter();
                    }
                    _ => {},
                }
            }

            OpCode::DEC_RC(pos) => {
                match self.frames[self.ip].stack[pos] {
                    Value::InstanceRef(heap_pos) | Value::StringRef(heap_pos) => {
                        self.rc.dec_counter(heap_pos);
                    }
                    _ => panic!("Error, this type of value shoudnt be here DEC RC"),
                };
            },
            OpCode::INC_RC(pos) => {
                match self.frames[self.ip].stack[pos] {
                    Value::InstanceRef(heap_pos) | Value::StringRef(heap_pos) => {
                        self.rc.inc_counter(heap_pos);
                    }
                    _ => panic!("Error, this type of value shoudnt be here INC RC"),
                };        
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
                self.frames[self.ip].stack.push(Value::Float(b.rem_euclid(a)));
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
                self.frames[self.ip].stack.push(Value::Int(b.rem_euclid(a)));
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
                let stack_pos = self.frames[self.ip].stack.len() - 1;
                self.run_instruction(Instruction { op: OpCode::DEC_RC(stack_pos), line: instruction.line });
                self.run_instruction(Instruction { op: OpCode::DEC_RC(stack_pos - 1), line: instruction.line });
                
                let a = match self.frames[self.ip].stack.pop().unwrap() {
                    Value::StringRef(pos) => {
                        let fields = self.rc.get_object(pos).get_values();
                        fields[0].clone()
                    },
                    _ => panic!("Ref not found"),
                };
                let b = match self.frames[self.ip].stack.pop().unwrap() {
                    Value::StringRef(pos) => {
                        let fields = self.rc.get_object(pos).get_values();
                        fields[0].clone()
                    },
                    _ => panic!("Ref not found")
                };
    
                let heap_pos = self.rc.heap.len();
                let mut instance = StructInstance::new();
            
                instance.fields_values.push(Value::String(b.get_string()+&a.get_string()));
                self.rc.push(Box::new(instance));
                self.frames[self.ip].stack.push(Value::StringRef(heap_pos));
                
            },
            OpCode::EQ_STRING => {
                let heap_pos = self.frames[self.ip].stack.len() - 1;
                self.run_instruction(Instruction { op: OpCode::DEC_RC(heap_pos), line: instruction.line });
                self.run_instruction(Instruction { op: OpCode::DEC_RC(heap_pos - 1), line: instruction.line });
                
                let a = match self.frames[self.ip].stack.pop().unwrap() {
                    Value::StringRef(pos) => {
                        let fields = self.rc.get_object(pos).get_values();
                        fields[0].clone()
                    },
                    _ => panic!("Ref not found"),
                };
               
                let b = match self.frames[self.ip].stack.pop().unwrap() {
                    Value::StringRef(pos) => {
                        let fields = self.rc.get_object(pos).get_values();
                        fields[0].clone()
                    },
                    _ => panic!("Ref not found")
                };
                
                self.frames[self.ip].stack.push(Value::Bool(a.get_string()==b.get_string()));
            },
            OpCode::NEG_EQ_STRING => {    
                let heap_pos = self.frames[self.ip].stack.len() - 1;
                self.run_instruction(Instruction { op: OpCode::DEC_RC(heap_pos), line: instruction.line });
                self.run_instruction(Instruction { op: OpCode::DEC_RC(heap_pos - 1), line: instruction.line });
                
                let a = match self.frames[self.ip].stack.pop().unwrap() {
                    Value::StringRef(pos) => {
                        let fields = self.rc.get_object(pos).get_values();
                        fields[0].clone()
                    },
                    _ => panic!("Ref not found"),
                };
               
                let b = match self.frames[self.ip].stack.pop().unwrap() {
                    Value::StringRef(pos) => {
                        let fields = self.rc.get_object(pos).get_values();
                        fields[0].clone()
                    },
                    _ => panic!("Ref not found")
                };

                self.frames[self.ip].stack.push(Value::Bool(a.get_string()!=b.get_string()));
            },

            opcode => errors::error_message("shlang/vm/vm.rs".to_string(), "RUNTIME - VM ERROR", format!("VM - this error should never prints out: {:?}", opcode)),
        }
    }
}
