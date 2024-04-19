use std::collections::HashMap;

use crate::frontend::tokens::TokenType;

use super::{functions::Local, structs::Struct};

pub struct StringObj {}

impl StringObj {
    pub fn init() -> Struct {
        Struct {
            name: "String".to_string(),
            locals: vec![Local { name: "value".to_string(), local_type: TokenType::STRING, is_redirected: false, redirect_pos: 0, rf_index: 0, is_string: true }],
            output_type: TokenType::NULL,
            field_count: 1,
            methods: HashMap::new(),
            rc_counter: 1,
            index: 0,
        }
    }
}

