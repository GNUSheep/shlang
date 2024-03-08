use std::collections::HashMap;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::vm::{
    value::Value,
    bytecode::{Chunk, OpCode, Instruction},
};
use crate::frontend::tokens::{Token, TokenType};

#[derive(Debug)]
pub struct ParseRule {
    prefix: Option<fn(&mut Compiler)>,
    infix: Option<fn(&mut Compiler)>,
    prec: Precedence,
}

pub fn init_rules() -> HashMap<TokenType, ParseRule> {
    HashMap::from([
        (TokenType::INT, ParseRule { prefix: Some(Compiler::number), infix: None, prec: Precedence::NONE }),
        (TokenType::FLOAT, ParseRule { prefix: Some(Compiler::number), infix: None, prec: Precedence::NONE }),

        (TokenType::PLUS, ParseRule { prefix: None, infix: Some(Compiler::arithmetic), prec: Precedence::TERM }),
        (TokenType::STAR, ParseRule { prefix: None, infix: Some(Compiler::arithmetic), prec: Precedence::FACTOR }),
        (TokenType::SLASH, ParseRule { prefix: None, infix: Some(Compiler::arithmetic), prec: Precedence::FACTOR }),

        (TokenType::EOF, ParseRule { prefix: None, infix: None, prec: Precedence::NONE }),
    ])
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy, FromPrimitive)]
pub enum Precedence {
    NONE = 0,
    ASSIGNMENT,  
    OR,
    AND,
    EQUALITY,
    COMPARISON,
    TERM,
    FACTOR,
    UNARY,
    CALL,
    PRIMARY
}

pub struct Parser {
    tokens: Vec<Token>,
    cur: usize,
    prev: usize,
    rules: HashMap<TokenType, ParseRule>,
}

impl Parser {
    pub fn next(&mut self) -> &Token {
        if self.cur != 0 {
            if self.check_if_eof() {
                return  &self.tokens[self.cur - 1];
            }

            self.prev += 1;
        }
        self.cur += 1;
        if self.tokens[self.cur - 1].token_type == TokenType::ERROR {
            // better handle errors
            panic!("Error: {:?}", self.tokens[self.cur - 1]);
        }
        &self.tokens[self.cur - 1]
    }

    pub fn next_token(&mut self) -> TokenType {
        if self.check_if_eof() {
            return TokenType::EOF;
        }

        if self.cur != 0 {
            self.prev += 1;
        }
        self.cur += 1;
        self.tokens[self.cur - 1].token_type
    }

    pub fn prev_token(&mut self) -> TokenType {
        if self.prev == 0 {
            return self.tokens[self.prev].token_type;
        }
        self.tokens[self.prev - 1].token_type
    }

    pub fn check_if_eof(&mut self) -> bool {
        if self.tokens[self.cur - 1].token_type == TokenType::EOF {
            return true;
        }
        false
    }

    pub fn get_rule(&self, token_type: &TokenType) -> &ParseRule {
        &self.rules[token_type]
    }

    pub fn peek(&mut self) -> &Token {
        &self.tokens[self.cur]
    }

    pub fn peek_token_with_index(&self, index: usize) -> TokenType {
        self.tokens[index].token_type
    }

    pub fn peek_prev(&self) -> &Token {
        if self.prev == 0 {
            return &self.tokens[self.prev];
        }
        &self.tokens[self.prev - 1]
    }
}

pub struct Compiler {
    parser: Parser,
    chunk: Chunk,
}

impl Compiler {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            parser: Parser {
                tokens: tokens,
                cur: 0,
                prev: 0,
                rules: init_rules(),
            },
            chunk: Chunk::new(),
        }
    }

    pub fn number(&mut self) {
        let token = self.parser.peek_prev();

        match token.token_type {
            TokenType::INT => {
                let value: i64 = token.value.iter().collect::<String>().parse().unwrap();
                let pos = self.chunk.push_value(Value::Int(value));
                let line = token.line;

                self.emit_byte(OpCode::CONSTANT_INT(pos), line);
            }
            TokenType::FLOAT => {
                let value: f64 = token.value.iter().collect::<String>().parse().unwrap();
                let pos = self.chunk.push_value(Value::Float(value));
                let line = token.line;

                self.emit_byte(OpCode::CONSTANT_INT(pos), line);
            }
            // Better handling errors
            _ => panic!("ERROR"),
        }
    }

    pub fn arithmetic(&mut self) {
        let arithmetic_token = self.parser.peek_prev();
        let arithmetic_token_type = arithmetic_token.token_type;
        let line = arithmetic_token.line;

        if !self.check_num_types(self.parser.peek_token_with_index(self.parser.cur - 1), &self.chunk.last().op) {
            panic!("WRONG TYPES");
        }
        let constants_type = self.parser.peek_token_with_index(self.parser.cur - 1);

        let rule = self.parser.get_rule(&arithmetic_token_type);

        match Precedence::from_u32(rule.prec as u32 + 1) {
            Some(prec) => {
                self.parse(prec)
            },
            None => panic!("ERROR"),
        };

        match arithmetic_token_type {
            TokenType::PLUS => {
                match constants_type {
                    TokenType::INT => self.emit_byte(OpCode::ADD_INT, line),
                    TokenType::FLOAT => self.emit_byte(OpCode::ADD_FLOAT, line),
                    _ => panic!("ERROR"),
                }
            },
            TokenType::STAR => {
                match constants_type {
                    TokenType::INT => self.emit_byte(OpCode::MUL_INT, line),
                    TokenType::FLOAT => self.emit_byte(OpCode::MUL_FLOAT, line),
                    _ => panic!("ERROR"),
                }
            },
            _ => panic!("ERROR"),
        };
    }

    pub fn check_num_types(&self, a_type: TokenType, b_type: &OpCode) -> bool {
        let b_token_type = match b_type {
            OpCode::CONSTANT_INT(_) => TokenType::INT,
            OpCode::CONSTANT_FLOAT(_) => TokenType::FLOAT,
            _ => panic!("ERROR"),
        };

        if a_type == b_token_type {
            return true;
        }
        false
    }

    pub fn expression(&mut self) {
        self.parse(Precedence::ASSIGNMENT);   
    }

    pub fn compile(&mut self) -> Chunk {
        loop {
            self.parser.next();
            self.expression();

            if self.parser.check_if_eof() {
                break;
            }
        }
        let line = self.parser.peek_prev().line;
        self.emit_byte(OpCode::RETURN, line);

        self.chunk.clone()
    }
    
    pub fn parse(&mut self, prec: Precedence) {
        let mut token_type = self.parser.next_token();

        let prev_token_type = self.parser.prev_token();
        if !self.parser.rules.contains_key(&prev_token_type) {
            // Better error
            panic!("ERROR");
        }
        let rule = self.parser.get_rule(&prev_token_type);

        println!("{:?}, {:?}, {:?}", prev_token_type, token_type, prec);

        match rule.prefix {
            Some(f) => f(self),
            _ => panic!("ERROR"),
        };

        while prec <= self.parser.get_rule(&token_type).prec {
            token_type = self.parser.next_token();
            
            let prev_token_type = self.parser.prev_token();
            let rule = self.parser.get_rule(&prev_token_type);
            match rule.infix {
                Some(f) => f(self),
                _ => panic!("ERROR"),
            }
        }
    }

    pub fn emit_byte(&mut self, op: OpCode, line: i32) {
        self.chunk.push(Instruction{ op: op, line: line as u32 });
    }
}