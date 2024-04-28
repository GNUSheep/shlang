use std::{collections::HashMap, vec};

use crate::{
    frontend::tokens::{Keywords, TokenType}, 
    vm::{bytecode::{Instruction, OpCode}, value::Value
}};

use super::{functions::{Function, NativeFn, Local}, structs::Struct};

pub struct StringObj {}

impl StringObj {
    pub fn init(string_pos: usize) -> Struct {
        let mut mths = StringMethods { cur_pos: string_pos };

        Struct {
            name: "String".to_string(),
            locals: vec![Local { name: "value".to_string(), local_type: TokenType::STRING, is_redirected: false, redirect_pos: 0, rf_index: 0, is_string: true }],
            output_type: TokenType::NULL,
            field_count: 1,
            methods: mths.get_methods(),
            rc_counter: 1,
            index: 0,
        }
    }
}

pub struct StringMethods {
    cur_pos: usize,
}

impl StringMethods {
    pub fn get_methods(&mut self) -> HashMap<String, Function> {
        HashMap::from([
            ("len".to_string(), self.pack_into_fn("len".to_string(), TokenType::INT, 1, TokenType::NULL)),
            ("toLower".to_string(), self.pack_into_fn("toLower".to_string(), TokenType::STRING, 1, TokenType::NULL)),
            ("toUpper".to_string(), self.pack_into_fn("toUpper".to_string(), TokenType::STRING, 1, TokenType::NULL)),
            ("get".to_string(), self.pack_into_fn("get".to_string(), TokenType::STRING, 2, TokenType::INT)),
            ("count".to_string(), self.pack_into_fn("count".to_string(), TokenType::STRING, 2, TokenType::STRING)),
        ])
    }

    pub fn get_methods_rc() -> Vec<NativeFn> {
        vec![
            NativeFn { name: "len".to_string(), function: StringMethods::len, arg_count: 1, rc_counter: 1, index: 0 },
            NativeFn { name: "toLower".to_string(), function: StringMethods::to_lower, arg_count: 1, rc_counter: 1, index: 0 },
            NativeFn { name: "toUpper".to_string(), function: StringMethods::to_upper, arg_count: 1, rc_counter: 1, index: 0 },
            NativeFn { name: "get".to_string(), function: StringMethods::get, arg_count: 2, rc_counter: 1, index: 0 },
            NativeFn { name: "count".to_string(), function: StringMethods::count, arg_count: 2, rc_counter: 1, index: 0 },
        ]
    }

    pub fn pack_into_fn(&mut self, name: String, out_type: TokenType, arg_count: usize, arg_type: TokenType) -> Function {
        self.cur_pos += 1;

        let mut function = Function::new(name);

        function.chunk.push_value(Value::String(String::new()));
        function.chunk.push_value(Value::Null);

        function.output_type = out_type;
        function.is_self_arg = true;
        function.arg_count = arg_count - 1;

        function.instances.push(Local { name: "self".to_string(), local_type: TokenType::KEYWORD(Keywords::INSTANCE(3)), is_redirected: false, redirect_pos: 0, rf_index: 0, is_string: false });
        
        if arg_type == TokenType::STRING {
            for i in 1..arg_count {
                function.instances.push(Local { name: "".to_string(), local_type: TokenType::KEYWORD(Keywords::INSTANCE(3)), is_redirected: false, redirect_pos: 0, rf_index: 0, is_string: true });
                function.chunk.push(Instruction { op: OpCode::GET_INSTANCE_FIELD(i, 0), line: 1});
            }
        }

        function.chunk.push(Instruction { op: OpCode::GET_INSTANCE_FIELD(0, 0), line: 1});
        function.chunk.push(Instruction { op: OpCode::NATIVE_FN_CALL(self.cur_pos), line: 1});

        if out_type != TokenType::NULL {
            function.chunk.push(Instruction { op: OpCode::RETURN, line: 1});
        }

        function.chunk.push(Instruction { op: OpCode::CONSTANT_NULL(1), line: 1});
        function.chunk.push(Instruction { op: OpCode::RETURN, line: 1});
        function.chunk.push(Instruction { op: OpCode::DEC_RC(0), line: 1});
        function.chunk.push(Instruction { op: OpCode::END_OF_FN, line: 1});

        function
    }

    fn len(args: Vec<Value>) -> Value {
        Value::Int(args[0].get_string().len() as i64)
    }

    fn to_upper(args: Vec<Value>) -> Value {
        Value::String(args[0].get_string().to_ascii_uppercase())
    }

    fn to_lower(args: Vec<Value>) -> Value {
        Value::String(args[0].get_string().to_ascii_lowercase())
    }

    fn get(args: Vec<Value>) -> Value {
        Value::String(String::from_utf8(vec![args[1].get_string().as_bytes()[args[0].get_int() as usize]]).unwrap())
    }

    fn count(args: Vec<Value>) -> Value {
        let str = args[1].get_string();

        let vec_indices = str.match_indices(&args[0].get_string()).collect::<Vec<_>>();

        Value::Int(vec_indices.len() as i64)
    }
}