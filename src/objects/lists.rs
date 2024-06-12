use std::{collections::HashMap, vec};

use crate::frontend::tokens::TokenType; 

use super::structs::Struct;

pub struct ListObj {}

impl ListObj {
    pub fn init() -> Struct {
        Struct {
            name: "List".to_string(),
            locals: vec![],
            output_type: TokenType::NULL,
            field_count: 0,
            methods: HashMap::new(),
            rc_counter: 1,
            index: 0,
        }
    }
}
