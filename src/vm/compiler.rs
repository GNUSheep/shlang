use std::collections::HashMap;

use crate::vm::{
    value::Value,
    bytecode::{Chunk, OpCode, Instruction},
};
use crate::frontend::tokens::{Token, TokenType};

pub struct ParseRule {
    prefix: Option<fn(&mut Compiler)>,
    infix: Option<fn(&mut Compiler)>,
    prec: Precedence,
}

pub fn init_rules() -> HashMap<TokenType, ParseRule> {
    HashMap::from([
        (TokenType::NUMBER, ParseRule { prefix: Some(Compiler::number), infix: None, prec: Precedence::NONE }),
    ])
}

#[derive(PartialEq, PartialOrd)]
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
        self.prev = self.cur;
        
        self.cur += 1;
        if self.tokens[self.cur - 1].token_type == TokenType::ERROR {
            // better handle errors
            panic!("Error: {:?}", self.tokens[self.cur - 1]);
        }
        &self.tokens[self.cur - 1]
    }

    pub fn next_token(&mut self) -> TokenType {
        self.prev = self.cur;
        self.cur += 1;
        self.tokens[self.cur - 1].token_type
    }

    pub fn prev_token(&mut self) -> TokenType {
        self.tokens[self.prev].token_type
    }

    pub fn get_rule(&self, token_type: &TokenType) -> &ParseRule {
        &self.rules[token_type]
    }

    pub fn peek(&mut self) -> &Token {
        &self.tokens[self.cur]
    }

    pub fn peek_prev(&self) -> &Token {
        &self.tokens[self.prev]
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
            // i64
            TokenType::NUMBER => {
                let value: i64 = token.value.iter().collect::<String>().parse().unwrap();
                let pos = self.chunk.push_value(Value::Int(value));
                let line = token.line;

                self.emit_byte(OpCode::CONSTANT_INT(pos), line);
            }
            // Better handling errors
            _ => panic!("ERROR"),
        }
    }

    pub fn binary(&mut self) {
    }

    pub fn expression(&mut self) {
        self.parse(Precedence::ASSIGNMENT);   
    }

    pub fn compile(&mut self) -> Chunk {
        loop {
            if self.parser.peek().token_type == TokenType::EOF {
                break;
            }

            self.expression();
        }
        let line = self.parser.peek().line;
        self.emit_byte(OpCode::RETURN, line);

        self.chunk.clone()
    }

    pub fn parse(&mut self, prec: Precedence) {
        let mut token_type = self.parser.next_token();

        let prev_token_type = self.parser.prev_token();
        if !self.parser.rules.contains_key(&prev_token) {
            // Better error
            panic!("ERROR");
        }
        let rule = self.parser.get_rule(&prev_token);

        match rule.prefix {
            Some(f) => f(self),
            _ => panic!("ERROR"),
        };

        while prec <= self.parser.get_rule(&token_type).prec {
            let token_type = self.parser.next_token();

            let prev_token_type = self.parser.prev_token();
            let rule = self.parser.get_rule(&prev_token);
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