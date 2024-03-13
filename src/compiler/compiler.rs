use std::collections::HashMap;

use crate::vm::{
    value::{Value, Convert},
    bytecode::{Chunk, OpCode, Instruction},
};
use crate::frontend::tokens::{Token, TokenType, Keywords};

use super::errors;

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
        
        (TokenType::KEYWORD(Keywords::TRUE), ParseRule { prefix: Some(Compiler::bool), infix: None, prec: Precedence::NONE }),
        (TokenType::KEYWORD(Keywords::FALSE), ParseRule { prefix: Some(Compiler::bool), infix: None, prec: Precedence::NONE }),

        (TokenType::INTERJ, ParseRule { prefix: Some(Compiler::negation), infix: None, prec: Precedence::NONE }),

        (TokenType::INTERJ_EQ, ParseRule { prefix: None, infix: Some(Compiler::logic_operator), prec: Precedence::EQUALITY }),
        (TokenType::EQ_EQ, ParseRule { prefix: None, infix: Some(Compiler::logic_operator), prec: Precedence::EQUALITY }),
        (TokenType::GREATER, ParseRule { prefix: None, infix: Some(Compiler::logic_operator), prec: Precedence::COMPARISON }),
        (TokenType::GREATER_EQ, ParseRule { prefix: None, infix: Some(Compiler::logic_operator), prec: Precedence::COMPARISON }),
        (TokenType::LESS, ParseRule { prefix: None, infix: Some(Compiler::logic_operator), prec: Precedence::COMPARISON }),
        (TokenType::LESS_EQ, ParseRule { prefix: None, infix: Some(Compiler::logic_operator), prec: Precedence::COMPARISON }),

        (TokenType::PLUS, ParseRule { prefix: None, infix: Some(Compiler::arithmetic), prec: Precedence::TERM }),
        (TokenType::MINUS, ParseRule { prefix: Some(Compiler::negation), infix: Some(Compiler::arithmetic), prec: Precedence::TERM }),
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
            _ => {
                errors::conversion_error("u32", "Precedence");
                std::process::exit(1);
            }
            
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
            errors::token_error(self.cur.clone());
        }
    }

    //pub fn check_if_eof(&mut self) -> bool {
    //    if self.cur.token_type == TokenType::EOF {
    //        return true;
    //    }
    //    false
    //}

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

    pub fn negation(&mut self) {
        let negation_token = self.parser.prev.clone();

        self.parse(Precedence::UNARY);

        match negation_token.token_type {
            TokenType::MINUS => self.emit_byte(OpCode::NEGATE, negation_token.line),
            TokenType::INTERJ => self.emit_byte(OpCode::NEGATE, negation_token.line),
            _ => {
                errors::error_unexpected(self.parser.prev.clone(), "negation function");
                std::process::exit(1);
            }
        }
    }

    pub fn logic_operator(&mut self) {
        let logic_token = self.parser.prev.clone();
        let left_side = self.chunk.get_value(self.chunk.values.len() - 1).convert();
        
        let rule = self.parser.get_rule(&logic_token.token_type);

        self.parse((rule.prec as u32 + 1).into());

        let constants_type = self.check_static_types(&self.parser.prev, left_side, &logic_token);

        match constants_type {
            TokenType::INT => {
                match logic_token.token_type {
                    TokenType::EQ_EQ => self.emit_byte(OpCode::EQ_INT, logic_token.line),
                    TokenType::INTERJ_EQ => self.emit_byte(OpCode::NEG_EQ_INT, logic_token.line),
                    TokenType::GREATER => self.emit_byte(OpCode::GREATER_INT, logic_token.line),
                    TokenType::GREATER_EQ => self.emit_byte(OpCode::EQ_GREATER_INT, logic_token.line),
                    TokenType::LESS => self.emit_byte(OpCode::LESS_INT, logic_token.line),
                    TokenType::LESS_EQ => self.emit_byte(OpCode::EQ_LESS_INT, logic_token.line),
                    _ => {
                        errors::error_unexpected(logic_token, "logic operator function");
                        std::process::exit(1);
                    }
                };
            },
            TokenType::FLOAT => {
                match logic_token.token_type {
                    TokenType::EQ_EQ => self.emit_byte(OpCode::EQ_FLOAT, logic_token.line),
                    TokenType::INTERJ_EQ => self.emit_byte(OpCode::NEG_EQ_FLOAT, logic_token.line),
                    TokenType::GREATER => self.emit_byte(OpCode::GREATER_FLOAT, logic_token.line),
                    TokenType::GREATER_EQ => self.emit_byte(OpCode::EQ_GREATER_FLOAT, logic_token.line),
                    TokenType::LESS => self.emit_byte(OpCode::LESS_FLOAT, logic_token.line),
                    TokenType::LESS_EQ => self.emit_byte(OpCode::EQ_LESS_FLOAT, logic_token.line),
                    _ => {
                        errors::error_unexpected(logic_token, "logic operator function");
                        std::process::exit(1);
                    }
                };
            },
            TokenType::BOOL => {
                match logic_token.token_type {
                    TokenType::EQ_EQ => self.emit_byte(OpCode::EQ_BOOL, logic_token.line),
                    TokenType::INTERJ_EQ => self.emit_byte(OpCode::NEG_EQ_BOOL, logic_token.line),
                    _ => {
                        errors::error_unexpected(logic_token, "logic operator function");
                        std::process::exit(1);
                    }
                };
            }
            _ => {
                errors::error_unexpected_token_type(constants_type, logic_token.line, "logic operator function");
                std::process::exit(1);
            }
        };
    }

    pub fn bool(&mut self) {
        match self.parser.prev.token_type {
            TokenType::KEYWORD(val) => {
                match val {
                    Keywords::TRUE => {
                        let pos = self.chunk.push_value(Value::Bool(true));
                        let line = self.parser.prev.line;

                        self.emit_byte(OpCode::CONSTANT_BOOL(pos), line);
                    },
                    Keywords::FALSE => {
                        let pos = self.chunk.push_value(Value::Bool(false));
                        let line = self.parser.prev.line;

                        self.emit_byte(OpCode::CONSTANT_BOOL(pos), line);
                    }, 
                    _ => {
                        errors::error_unexpected_keyword(val, self.parser.prev.line, "bool function");
                        std::process::exit(1);
                    }
                }
            }
            _ => {
                errors::error_unexpected(self.parser.prev.clone(), "bool function");
                std::process::exit(1);
            }
        };
    }

    pub fn number(&mut self) {
        match self.parser.prev.token_type {
            TokenType::INT => {
                let value: i64 = match self.parser.prev.value.iter().collect::<String>().parse() {
                    Ok(v) => v,
                    Err(_) => {
                        errors::conversion_error("Vec<char>", "i64");
                        std::process::exit(1);
                    },
                };

                let pos = self.chunk.push_value(Value::Int(value));
                let line = self.parser.prev.line;

                self.emit_byte(OpCode::CONSTANT_INT(pos), line);
            }
            TokenType::FLOAT => {
                let value: f64 = match self.parser.prev.value.iter().collect::<String>().parse() {
                    Ok(v) => v,
                    Err(_) => {
                        errors::conversion_error("Vec<char>", "f64");
                        std::process::exit(1);
                    },
                };

                let pos = self.chunk.push_value(Value::Float(value));
                let line = self.parser.prev.line;

                self.emit_byte(OpCode::CONSTANT_FLOAT(pos), line);
            }
            // Better handling errors
            _ => {
                errors::error_unexpected(self.parser.prev.clone(), "number function");
                std::process::exit(1);
            },
        }
    }

    pub fn arithmetic(&mut self) {
        let arithmetic_token = self.parser.prev.clone();
        let left_side = self.chunk.get_value(self.chunk.values.len() - 1).convert();
        
        let rule = self.parser.get_rule(&arithmetic_token.token_type);

        self.parse((rule.prec as u32 + 1).into());

        let constants_type = self.check_static_types(&self.parser.prev, left_side, &arithmetic_token);

        match constants_type {
            TokenType::INT => {
                match arithmetic_token.token_type {
                    TokenType::PLUS => self.emit_byte(OpCode::ADD_INT, arithmetic_token.line),
                    TokenType::MINUS => self.emit_byte(OpCode::SUB_INT, arithmetic_token.line),
                    TokenType::STAR => self.emit_byte(OpCode::MUL_INT, arithmetic_token.line),
                    TokenType::SLASH => self.emit_byte(OpCode::DIV_INT, arithmetic_token.line),
                    _ => {
                        errors::error_unexpected(arithmetic_token, "arithmetic function");
                        std::process::exit(1);
                    }
                };
            },
            TokenType::FLOAT => {
                match arithmetic_token.token_type {
                    TokenType::PLUS => self.emit_byte(OpCode::ADD_FLOAT, arithmetic_token.line),
                    TokenType::MINUS => self.emit_byte(OpCode::SUB_FLOAT, arithmetic_token.line),
                    TokenType::STAR => self.emit_byte(OpCode::MUL_FLOAT, arithmetic_token.line),
                    TokenType::SLASH => self.emit_byte(OpCode::DIV_FLOAT, arithmetic_token.line),
                    _ => {
                        errors::error_unexpected(arithmetic_token, "arithmetic function");
                        std::process::exit(1);
                    }
                };
            },
            _ => {
                errors::error_unexpected_token_type(constants_type, arithmetic_token.line, "arithmetic function");
                std::process::exit(1);
            }
        };
    }

    pub fn check_static_types(&self, a_token: &Token, b_type: TokenType, op: &Token) -> TokenType {
        let a_token_type = match a_token.token_type {
            TokenType::KEYWORD(_) => TokenType::BOOL,
            token_type => token_type,
        };

        if !self.check_num_types(a_token_type, b_type) {
            errors::error_message("COMPILING ERROR", format!("Mismatched types: {:?} {} {:?} {}:",
                b_type,
                op.value.iter().collect::<String>(),
                a_token_type,
                a_token.line,
            ));
            std::process::exit(1);
        }
        a_token_type
    }

    pub fn check_num_types(&self, a_type: TokenType, b_type: TokenType) -> bool {
        if a_type == b_type {
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
            errors::error_message("PARSING ERROR", format!("Cannot get a parse rule for: {:?}: \"{}\", {}:", 
                self.parser.prev.token_type, 
                self.parser.prev.value.iter().collect::<String>(), 
                self.parser.prev.line,
            ));
            std::process::exit(1);
        }
        let rule = self.parser.get_rule(&self.parser.prev.token_type);

        match rule.prefix {
            Some(f) => f(self),
            _ => {
                errors::error_message("PARSING ERROR", format!("Expected prefix for: {:?}, {}:", self.parser.prev.token_type, self.parser.prev.line));
                std::process::exit(1);
            },
        };

        while prec <= self.parser.get_rule(&self.parser.cur.token_type).prec {
            self.parser.advance();

            if !self.parser.rules.contains_key(&self.parser.prev.token_type) {
                errors::error_message("PARSING ERROR", format!("Cannot get a parse rule for: {:?}: \"{}\", {}:", 
                    self.parser.prev.token_type, 
                    self.parser.prev.value.iter().collect::<String>(), 
                    self.parser.prev.line,
                ));
                std::process::exit(1);
            }
            let rule = self.parser.get_rule(&self.parser.prev.token_type);
            match rule.infix {
                Some(f) => f(self),
                _ => {
                    errors::error_message("PARSING ERROR", format!("Expected infix for: {:?}, {}:", self.parser.prev.token_type, self.parser.prev.line));
                    std::process::exit(1);
                },
            }
        }
    }

    pub fn emit_byte(&mut self, op: OpCode, line: u32) {
        self.chunk.push(Instruction{ op: op, line: line });
    }
}