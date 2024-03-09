use std::collections::HashMap;

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
        (TokenType::MINUS, ParseRule { prefix: None, infix: Some(Compiler::arithmetic), prec: Precedence::TERM }),
        (TokenType::STAR, ParseRule { prefix: None, infix: Some(Compiler::arithmetic), prec: Precedence::FACTOR }),
        (TokenType::SLASH, ParseRule { prefix: None, infix: Some(Compiler::arithmetic), prec: Precedence::FACTOR }),

        (TokenType::EOF, ParseRule { prefix: None, infix: None, prec: Precedence::NONE }),
    ])
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub enum Precedence {
    NONE,
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

impl From<u32> for Precedence {
    fn from(value: u32) -> Self {
        match value {
            0 => Precedence::NONE,
            1 => Precedence::ASSIGNMENT,
            2 => Precedence::OR,
            3 => Precedence::AND,
            4 => Precedence::EQUALITY,
            5 => Precedence::COMPARISON,
            6 => Precedence::TERM,
            7 => Precedence::FACTOR,
            8 => Precedence::UNARY,
            9 => Precedence::CALL,
            10 => Precedence::PRIMARY,
            _ => panic!("Error"),
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    cur: Token,
    prev: Token,
    index: usize,
    rules: HashMap<TokenType, ParseRule>,
}

impl Parser {
    pub fn advance(&mut self) {
        self.prev = self.cur.clone();
        self.cur = self.tokens[self.index].clone();
        self.index += 1;

        if self.cur.token_type == TokenType::ERROR {
            panic!("ERROR");
        }
    }

    pub fn check_if_eof(&mut self) -> bool {
        if self.cur.token_type == TokenType::EOF {
            return true;
        }
        false
    }

    pub fn get_rule(&self, token_type: &TokenType) -> &ParseRule {
        &self.rules[token_type]
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
                cur: Token { token_type: TokenType::ERROR, value: vec![], line: 0},
                prev: Token { token_type: TokenType::ERROR, value: vec![], line: 0},
                index: 0,
                rules: init_rules(),
            },
            chunk: Chunk::new(),
        }
    }

    pub fn number(&mut self) {
        match self.parser.prev.token_type {
            TokenType::INT => {
                let value: i64 = self.parser.prev.value.iter().collect::<String>().parse().unwrap();
                let pos = self.chunk.push_value(Value::Int(value));
                let line = self.parser.prev.line;

                self.emit_byte(OpCode::CONSTANT_INT(pos), line);
            }
            TokenType::FLOAT => {
                let value: f64 = self.parser.prev.value.iter().collect::<String>().parse().unwrap();
                let pos = self.chunk.push_value(Value::Float(value));
                let line = self.parser.prev.line;

                self.emit_byte(OpCode::CONSTANT_FLOAT(pos), line);
            }
            // Better handling errors
            _ => panic!("ERROR"),
        }
    }

    pub fn arithmetic(&mut self) {
        let arithmetic_token = self.parser.prev.clone();

        if !self.check_num_types(self.parser.cur.token_type, &self.chunk.get_value(self.chunk.values.len() - 1)) {
            panic!("WRONG TYPES");
        }
        let constants_type = self.parser.cur.token_type;

        let rule = self.parser.get_rule(&arithmetic_token.token_type);

        self.parse((rule.prec as u32 + 1).into());

        match arithmetic_token.token_type {
            TokenType::PLUS => {
                match constants_type {
                    TokenType::INT => self.emit_byte(OpCode::ADD_INT, arithmetic_token.line),
                    TokenType::FLOAT => self.emit_byte(OpCode::ADD_FLOAT, arithmetic_token.line),
                    _ => panic!("ERROR"),
                }
            },
            TokenType::MINUS => {
                match constants_type {
                    TokenType::INT => self.emit_byte(OpCode::SUB_INT, arithmetic_token.line),
                    TokenType::FLOAT => self.emit_byte(OpCode::SUB_FLOAT, arithmetic_token.line),
                    _ => panic!("ERROR"),
                }
            },
            TokenType::STAR => {
                match constants_type {
                    TokenType::INT => self.emit_byte(OpCode::MUL_INT, arithmetic_token.line),
                    TokenType::FLOAT => self.emit_byte(OpCode::MUL_FLOAT, arithmetic_token.line),
                    _ => panic!("ERROR"),
                }
            },
            TokenType::SLASH => {
                match constants_type {
                    TokenType::INT => self.emit_byte(OpCode::DIV_INT, arithmetic_token.line),
                    TokenType::FLOAT => self.emit_byte(OpCode::DIV_FLOAT, arithmetic_token.line),
                    _ => panic!("ERROR"),
                }
            },
            _ => panic!("ERROR"),
        };
    }

    pub fn check_num_types(&self, a_type: TokenType, b_type: &Value) -> bool {
        let b_token_type = match b_type {
            Value::Int(_) => TokenType::INT,
            Value::Float(_) => TokenType::FLOAT,
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
        self.parser.advance();
        self.expression();
        let line = self.chunk.get_instruction(self.chunk.len() - 1).line;
        self.emit_byte(OpCode::RETURN, line);

        self.chunk.clone()
    }
    
    pub fn parse(&mut self, prec: Precedence) {
        self.parser.advance();

        if !self.parser.rules.contains_key(&self.parser.prev.token_type) {
            // Better error
            panic!("ERROR");
        }
        let rule = self.parser.get_rule(&self.parser.prev.token_type);

        match rule.prefix {
            Some(f) => f(self),
            _ => panic!("ERROR"),
        };

        while prec <= self.parser.get_rule(&self.parser.cur.token_type).prec {
            self.parser.advance();

            let rule = self.parser.get_rule(&self.parser.prev.token_type);
            match rule.infix {
                Some(f) => f(self),
                _ => panic!("ERROR"),
            }
        }
    }

    pub fn emit_byte(&mut self, op: OpCode, line: u32) {
        self.chunk.push(Instruction{ op: op, line: line });
    }
}