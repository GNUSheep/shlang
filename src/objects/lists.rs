use std::{collections::HashMap, vec};

use crate::{frontend::tokens::{Keywords, TokenType}, vm::{bytecode::{Instruction, OpCode}, value::Value}}; 

use super::{functions::{Function, Local, SpecialType}, structs::Struct};

pub struct ListObj {}

impl ListObj {
    pub fn init(list_pos: usize) -> Struct {
        let mut mths = ListMethods { cur_pos: list_pos };
        
        Struct {
            name: "List".to_string(),
            locals: vec![],
            output_type: TokenType::NULL,
            field_count: 0,
            methods: mths.get_methods(),
            rc_counter: 1,
            index: 0,
        }
    }
}

pub struct ListMethods {
    cur_pos: usize,
}

impl ListMethods {
    pub fn get_methods(&mut self) -> HashMap<String, Function> {
        HashMap::from([
            ("push".to_string(), self.generate_method_function("push".to_string(), TokenType::NULL, 1, vec![TokenType::LIST_ELEMENT], OpCode::LIST_PUSH)),
            ("pop".to_string(), self.generate_method_function("pop".to_string(), TokenType::LIST_ELEMENT, 0, vec![], OpCode::LIST_POP)),
            ("insert".to_string(), self.generate_method_function("insert".to_string(), TokenType::NULL, 2, vec![TokenType::LIST_ELEMENT, TokenType::INT], OpCode::LIST_INSERT)),
            ("remove".to_string(), self.generate_method_function("remove".to_string(), TokenType::NULL, 1, vec![TokenType::INT], OpCode::LIST_REMOVE)),
            ("len".to_string(), self.generate_method_function("len".to_string(), TokenType::INT, 0, vec![], OpCode::LIST_LEN)),
            ("sort".to_string(), self.generate_method_function("sort".to_string(), TokenType::NULL, 0, vec![], OpCode::LIST_SORT)),
        ])
    }

    pub fn generate_method_function(&mut self, name: String, output_type: TokenType, arg_count: usize, arg_types: Vec<TokenType>,  method_bytecode: OpCode) -> Function {
        self.cur_pos += 1;

        let mut function = Function::new(name);

        function.output_type = output_type;
        function.is_self_arg = true;
        function.arg_count = arg_count;

        function.arg_type = arg_types;

        function.locals.push(Local { name: "self".to_string(), local_type: TokenType::LIST(Keywords::LIST), is_special: SpecialType::List(Value::List(TokenType::LIST(Keywords::LIST))) });
        function.chunk.push(Instruction { op: OpCode::GET_INSTANCE_RF(0), line: 1});

        function.chunk.push(Instruction { op: method_bytecode, line: 1 });

        
        if output_type != TokenType::NULL {
            function.chunk.push(Instruction { op: OpCode::RETURN, line: 2});
        }
        
        function.chunk.push_value(Value::Null);
        function.chunk.push(Instruction { op: OpCode::CONSTANT_NULL(0), line: 2});
        function.chunk.push(Instruction { op: OpCode::RETURN, line: 2});

        function.chunk.push(Instruction { op: OpCode::DEC_RC(0), line: 2});
       
        function.chunk.push(Instruction { op: OpCode::END_OF_FN, line: 2});

        function
    }
}
