use std::{collections::HashMap, vec};
use regex::Regex;

use crate::{
    frontend::tokens::{Keywords, TokenType}, 
    vm::{bytecode::{Instruction, OpCode}, value::Value
}};

use super::{functions::{Function, Local, NativeFn, SpecialType}, structs::{Struct, StructInstance}};

pub struct StringObj {}

impl StringObj {
    pub fn init(string_pos: usize) -> Struct {
        let mut mths = StringMethods { cur_pos: string_pos };

        Struct {
            name: "String".to_string(),
            locals: vec![Local { name: "value".to_string(), local_type: TokenType::STRING, is_special: SpecialType::String }],
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
            ("count".to_string(), self.pack_into_fn("count".to_string(), TokenType::INT, 2, TokenType::STRING)),
            ("find".to_string(), self.pack_into_fn("find".to_string(), TokenType::INT, 2, TokenType::STRING)),
            ("isChar".to_string(), self.pack_into_fn("isChar".to_string(), TokenType::BOOL, 1, TokenType::NULL)),
            ("isDigit".to_string(), self.pack_into_fn("isDigit".to_string(), TokenType::BOOL, 1, TokenType::NULL)),
            ("trim".to_string(), self.pack_into_fn("trim".to_string(), TokenType::STRING, 1, TokenType::NULL)),
            ("trimLeft".to_string(), self.pack_into_fn("trimLeft".to_string(), TokenType::STRING, 1, TokenType::NULL)),
            ("trimRight".to_string(), self.pack_into_fn("trimRight".to_string(), TokenType::STRING, 1, TokenType::NULL)),
            ("replace".to_string(), self.pack_into_fn("replace".to_string(), TokenType::STRING, 3, TokenType::STRING)),
        ])
    }

    pub fn get_methods_rc() -> Vec<NativeFn> {
        vec![
            NativeFn { function: StringMethods::len, arg_count: 1, rc_counter: 1 },
            NativeFn { function: StringMethods::to_lower, arg_count: 1, rc_counter: 1 },
            NativeFn { function: StringMethods::to_upper, arg_count: 1, rc_counter: 1 },
            NativeFn { function: StringMethods::get, arg_count: 2, rc_counter: 1 },
            NativeFn { function: StringMethods::count, arg_count: 2, rc_counter: 1 },
            NativeFn { function: StringMethods::find, arg_count: 2, rc_counter: 1 },
            NativeFn { function: StringMethods::is_char, arg_count: 1, rc_counter: 1 },
            NativeFn { function: StringMethods::is_digit, arg_count: 1, rc_counter: 1 },
            NativeFn { function: StringMethods::trim, arg_count: 1, rc_counter: 1 },
            NativeFn { function: StringMethods::trim_left, arg_count: 1, rc_counter: 1 },
            NativeFn { function: StringMethods::trim_right, arg_count: 1, rc_counter: 1 },
            NativeFn { function: StringMethods::replace, arg_count: 3, rc_counter: 1 },
        ]
    }

    pub fn pack_into_fn(&mut self, name: String, out_type: TokenType, arg_count: usize, arg_type: TokenType) -> Function {
        self.cur_pos += 1;

        let mut function = Function::new(name);

        function.output_type = out_type;
        function.is_self_arg = true;
        function.arg_count = arg_count - 1;

        for _ in 1..arg_count {
            function.arg_type.push(arg_type);
        }

        function.chunk.push_value(Value::String(String::new()));
        function.locals.push(Local { name: "self".to_string(), local_type: TokenType::KEYWORD(Keywords::INSTANCE(3)), is_special: SpecialType::String });
        function.chunk.push(Instruction { op: OpCode::GET_INSTANCE_RF(0), line: 1});
        function.chunk.push(Instruction { op: OpCode::INC_RC(0), line: 1});
        function.chunk.push(Instruction { op: OpCode::GET_INSTANCE_FIELD(0), line: 1});
        
        if arg_type == TokenType::STRING {
            for i in 1..arg_count {
                function.locals.push(Local { name: "".to_string(), local_type: TokenType::KEYWORD(Keywords::INSTANCE(3)), is_special: SpecialType::String });
                function.chunk.push(Instruction { op: OpCode::GET_INSTANCE_RF(i), line: 1});
                function.chunk.push(Instruction { op: OpCode::INC_RC(i), line: 1});
            }
        }

        function.chunk.push(Instruction { op: OpCode::NATIVE_FN_CALL(self.cur_pos), line: 1});

        if out_type == TokenType::STRING {
            let instance_object = StructInstance::new();

            function.chunk.push(Instruction { op: OpCode::STRING_DEC_VALUE(instance_object), line: 2 });
        }

        if out_type != TokenType::NULL {
            function.chunk.push(Instruction { op: OpCode::RETURN, line: 2});
        }

        function.chunk.push(Instruction { op: OpCode::CONSTANT_NULL(1), line: 2});
        function.chunk.push(Instruction { op: OpCode::RETURN, line: 2});

        function.chunk.push(Instruction { op: OpCode::DEC_RC(0), line: 2});

        if arg_type == TokenType::STRING {
            for i in 0..function.arg_count {
                function.chunk.push(Instruction { op: OpCode::DEC_RC(i+1), line: 2});
            }
        }

        
        function.chunk.push(Instruction { op: OpCode::END_OF_FN, line: 2});

        function
    }

    fn len(args: Vec<Value>) -> Value {
        Value::Int(args[0].get_string().len() as i128)
    }

    fn to_upper(args: Vec<Value>) -> Value {
        Value::String(args[0].get_string().to_ascii_uppercase())
    }

    fn to_lower(args: Vec<Value>) -> Value {
        Value::String(args[0].get_string().to_ascii_lowercase())
    }

    fn get(args: Vec<Value>) -> Value {
        println!("{:?}", args);
        Value::String(String::from_utf8(vec![args[1].get_string().as_bytes()[args[0].get_int() as usize]]).unwrap())
    }

    fn count(args: Vec<Value>) -> Value {
        let str = args[0].get_string();

        let vec_indices = str.match_indices(&args[1].get_string()).collect::<Vec<_>>();

        Value::Int(vec_indices.len() as i128)
    }

    fn find(args: Vec<Value>) -> Value {
        let str = args[0].get_string();

        match str.find(&args[1].get_string()) {
            Some(val) => Value::Int(val as i128),
            None => Value::Int(-1),
        }
    }

    fn is_char(args: Vec<Value>) -> Value {
        let pattern = Regex::new(r"^[^0-9]*$").unwrap();

        Value::Bool(pattern.is_match(&args[0].get_string()))
    }

    pub fn is_digit(args: Vec<Value>) -> Value {
        let pattern = Regex::new(r"^[^a-zA-Z]*$").unwrap();

        Value::Bool(pattern.is_match(&args[0].get_string()))
    }

    fn trim(args: Vec<Value>) -> Value {
        Value::String(args[0].get_string().trim().to_string())
    }

    fn trim_left(args: Vec<Value>) -> Value {
        Value::String(args[0].get_string().trim_start().to_string())
    }

    fn trim_right(args: Vec<Value>) -> Value {
        Value::String(args[0].get_string().trim_end().to_string())
    }

    fn replace(args: Vec<Value>) -> Value {
        Value::String(args[0].get_string().replace(&args[1].get_string(), &args[2].get_string()))
    }
}
