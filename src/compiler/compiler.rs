use std::collections::HashMap;

use crate::{
    objects::functions::{Function, Local, NativeFn},
    vm::{bytecode::{Chunk, Instruction, OpCode}, value::{Convert, Value}
}};
use crate::frontend::tokens::{Token, TokenType, Keywords};

use super::errors;

#[derive(PartialEq, Debug)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: TokenType,
    pub output_type: TokenType,
    pub arg_count: usize,
}

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

        (TokenType::IDENTIFIER, ParseRule { prefix: Some(Compiler::identifier), infix: None, prec: Precedence::NONE }),

        (TokenType::KEYWORD(Keywords::TRUE), ParseRule { prefix: Some(Compiler::bool), infix: None, prec: Precedence::NONE }),
        (TokenType::KEYWORD(Keywords::FALSE), ParseRule { prefix: Some(Compiler::bool), infix: None, prec: Precedence::NONE }),
        (TokenType::KEYWORD(Keywords::NULL), ParseRule { prefix: Some(Compiler::bool), infix: None, prec: Precedence::NONE }),

        (TokenType::KEYWORD(Keywords::FN), ParseRule { prefix: None, infix: None, prec: Precedence::NONE }),
        (TokenType::KEYWORD(Keywords::VAR), ParseRule { prefix: None, infix: None, prec: Precedence::NONE }),

        (TokenType::RIGHT_BRACE, ParseRule { prefix: None, infix: None, prec: Precedence::NONE }),
        (TokenType::LEFT_BRACE, ParseRule { prefix: None, infix: None, prec: Precedence::NONE }),

        (TokenType::COMMA, ParseRule { prefix: None, infix: None, prec: Precedence::NONE }),

        (TokenType::LEFT_PAREN, ParseRule { prefix: None, infix: Some(Compiler::fn_call), prec: Precedence::CALL }),
        (TokenType::RIGHT_PAREN, ParseRule { prefix: None, infix: None, prec: Precedence::NONE }),

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
    symbols: Vec<Symbol>,
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

    pub fn check_if_eof(&mut self) -> bool {
        if self.cur.token_type == TokenType::EOF {
            return true;
        }
        false
    }

    pub fn consume(&mut self, token_type: TokenType) {
        if self.cur.token_type != token_type {
            errors::error_message("PARSER ERROR", format!("Expected to find a {:?}", token_type));
            std::process::exit(1);
        }
        self.advance();
    }

    pub fn get_symbols(&mut self) {
        let mut symbols: Vec<Symbol> = NativeFn::get_natives_symbols();
        let mut is_main_fn_found = false;

        for token_pair in self.tokens.clone().windows(2) {
            if token_pair[0].token_type == TokenType::KEYWORD(Keywords::FN) {
                let fn_name = token_pair[1].value.iter().collect::<String>();

                if symbols.iter().any(| symbol | symbol.name == fn_name) {
                    errors::error_message("COMPILER ERROR", format!("Function: \"{}\" is already defined {}:", fn_name, token_pair[1].line));
                    std::process::exit(1);
                }

                if fn_name == "main".to_ascii_lowercase() {
                    is_main_fn_found = true;
                }

                symbols.push(Symbol{name: fn_name, symbol_type: TokenType::KEYWORD(Keywords::FN), output_type: TokenType::KEYWORD(Keywords::NULL), arg_count: 0 });
            }

            let symbol_len = symbols.len();
            match token_pair[0].token_type {
                TokenType::KEYWORD(Keywords::INT) => symbols[symbol_len - 1].output_type = TokenType::INT,
                TokenType::KEYWORD(Keywords::FLOAT) => symbols[symbol_len - 1].output_type = TokenType::FLOAT,
                _ => {},
            }
        }

        if !is_main_fn_found {
            errors::error_message("COMPILE ERROR", format!("Cannot find \"main\" function"));
            std::process::exit(1);
        }

        self.symbols = symbols;
    }

    pub fn get_rule(&self, token_type: &TokenType) -> &ParseRule {
        &self.rules[token_type]
    }
}

pub struct Compiler {
    pub parser: Parser,
    cur_function: Function,
    scope_depth: u32,
    line: u32,
    symbol_to_hold: usize,
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
                symbols: vec![],
            },
            cur_function: Function::new(String::new()),
            scope_depth: 0,
            line: 0,
            symbol_to_hold: 0,
        }
    }

    pub fn get_cur_chunk(&mut self) -> &mut Chunk {
        self.cur_function.get_chunk()
    }

    pub fn get_cur_locals(&mut self) -> &mut Vec<Local> {
        self.cur_function.get_locals()
    }

    pub fn negation(&mut self) {
        let negation_token = self.parser.prev.clone();

        self.parse(Precedence::UNARY);

        match negation_token.token_type {
            TokenType::MINUS => self.emit_byte(OpCode::NEGATE, self.line),
            TokenType::INTERJ => self.emit_byte(OpCode::NEGATE, self.line),
            _ => {
                errors::error_unexpected(self.parser.prev.clone(), "negation function");
                std::process::exit(1);
            }
        }
    }

    pub fn logic_operator(&mut self) {
        let logic_token = self.parser.prev.clone();

        let chunk = self.get_cur_chunk();
        let left_side = chunk.get_value(chunk.values.len() - 1).convert();

        let rule = self.parser.get_rule(&logic_token.token_type);

        self.parse((rule.prec as u32 + 1).into());

        let values_len = self.get_cur_chunk().values.len();
        let right_side = self.get_cur_chunk().values.get(values_len - 1).convert();

        let constants_type = self.check_static_types(&right_side, left_side, &logic_token);

        match constants_type {
            TokenType::INT => {
                match logic_token.token_type {
                    TokenType::EQ_EQ => self.emit_byte(OpCode::EQ_INT, self.line),
                    TokenType::INTERJ_EQ => self.emit_byte(OpCode::NEG_EQ_INT, self.line),
                    TokenType::GREATER => self.emit_byte(OpCode::GREATER_INT, self.line),
                    TokenType::GREATER_EQ => self.emit_byte(OpCode::EQ_GREATER_INT, self.line),
                    TokenType::LESS => self.emit_byte(OpCode::LESS_INT, self.line),
                    TokenType::LESS_EQ => self.emit_byte(OpCode::EQ_LESS_INT, self.line),
                    _ => {
                        errors::error_unexpected(logic_token, "logic operator function");
                        std::process::exit(1);
                    }
                };
            },
            TokenType::FLOAT => {
                match logic_token.token_type {
                    TokenType::EQ_EQ => self.emit_byte(OpCode::EQ_FLOAT, self.line),
                    TokenType::INTERJ_EQ => self.emit_byte(OpCode::NEG_EQ_FLOAT, self.line),
                    TokenType::GREATER => self.emit_byte(OpCode::GREATER_FLOAT, self.line),
                    TokenType::GREATER_EQ => self.emit_byte(OpCode::EQ_GREATER_FLOAT, self.line),
                    TokenType::LESS => self.emit_byte(OpCode::LESS_FLOAT, self.line),
                    TokenType::LESS_EQ => self.emit_byte(OpCode::EQ_LESS_FLOAT, self.line),
                    _ => {
                        errors::error_unexpected(logic_token, "logic operator function");
                        std::process::exit(1);
                    }
                };
            },
            TokenType::BOOL => {
                match logic_token.token_type {
                    TokenType::EQ_EQ => self.emit_byte(OpCode::EQ_BOOL, self.line),
                    TokenType::INTERJ_EQ => self.emit_byte(OpCode::NEG_EQ_BOOL, self.line),
                    _ => {
                        errors::error_unexpected(logic_token, "logic operator function");
                        std::process::exit(1);
                    }
                };
            }
            _ => {
                errors::error_unexpected_token_type(constants_type, self.line, "logic operator function");
                std::process::exit(1);
            }
        };
    }

    pub fn bool(&mut self) {
        match self.parser.prev.token_type {
            TokenType::KEYWORD(val) => {
                match val {
                    Keywords::TRUE => {
                        let pos = self.get_cur_chunk().push_value(Value::Bool(true));

                        self.emit_byte(OpCode::CONSTANT_BOOL(pos), self.line);
                    },
                    Keywords::FALSE => {
                        let pos = self.get_cur_chunk().push_value(Value::Bool(false));

                        self.emit_byte(OpCode::CONSTANT_BOOL(pos), self.line);
                    },
                    Keywords::NULL => {
                        let pos = self.get_cur_chunk().push_value(Value::Null);

                        self.emit_byte(OpCode::CONSTANT_NULL(pos), self.line);
                    },
                    _ => {
                        errors::error_unexpected_keyword(val, self.line, "bool function");
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

                let pos = self.get_cur_chunk().push_value(Value::Int(value));

                self.emit_byte(OpCode::CONSTANT_INT(pos), self.line);
            }
            TokenType::FLOAT => {
                let value: f64 = match self.parser.prev.value.iter().collect::<String>().parse() {
                    Ok(v) => v,
                    Err(_) => {
                        errors::conversion_error("Vec<char>", "f64");
                        std::process::exit(1);
                    },
                };

                let pos = self.get_cur_chunk().push_value(Value::Float(value));

                self.emit_byte(OpCode::CONSTANT_FLOAT(pos), self.line);
            }
            _ => {
                errors::error_unexpected(self.parser.prev.clone(), "number function");
                std::process::exit(1);
            },
        }
    }

    pub fn arithmetic(&mut self) {
        let arithmetic_token = self.parser.prev.clone();

        let chunk = self.get_cur_chunk();
        let left_side = chunk.get_value(chunk.values.len() - 1).convert();

        let rule = self.parser.get_rule(&arithmetic_token.token_type);

        self.parse((rule.prec as u32 + 1).into());

        let values_len = self.get_cur_chunk().values.len();
        
        let right_side = self.get_cur_chunk().values.get(values_len - 1).convert();

        let constants_type = self.check_static_types(&right_side, left_side, &arithmetic_token);

        match constants_type {
            TokenType::INT => {
                match arithmetic_token.token_type {
                    TokenType::PLUS => self.emit_byte(OpCode::ADD_INT, self.line),
                    TokenType::MINUS => self.emit_byte(OpCode::SUB_INT, self.line),
                    TokenType::STAR => self.emit_byte(OpCode::MUL_INT, self.line),
                    TokenType::SLASH => self.emit_byte(OpCode::DIV_INT, self.line),
                    _ => {
                        errors::error_unexpected(arithmetic_token, "arithmetic function");
                        std::process::exit(1);
                    }
                };
            },
            TokenType::FLOAT => {
                match arithmetic_token.token_type {
                    TokenType::PLUS => self.emit_byte(OpCode::ADD_FLOAT, self.line),
                    TokenType::MINUS => self.emit_byte(OpCode::SUB_FLOAT, self.line),
                    TokenType::STAR => self.emit_byte(OpCode::MUL_FLOAT, self.line),
                    TokenType::SLASH => self.emit_byte(OpCode::DIV_FLOAT, self.line),
                    _ => {
                        errors::error_unexpected(arithmetic_token, "arithmetic function");
                        std::process::exit(1);
                    }
                };
            },
            _ => {
                errors::error_unexpected_token_type(constants_type, self.line, "arithmetic function");
                std::process::exit(1);
            }
        };
    }

    pub fn check_static_types(&self, a_token_type: &TokenType, b_type: TokenType, op: &Token) -> TokenType {
        if !self.check_num_types(a_token_type.clone(), b_type) {
            errors::error_message("COMPILING ERROR", format!("Mismatched types: {:?} {} {:?} {}:",
                b_type,
                op.value.iter().collect::<String>(),
                a_token_type,
                self.line,
            ));
            std::process::exit(1);
        }
        a_token_type.clone()
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

    pub fn block(&mut self) {
        while !(self.parser.cur.token_type == TokenType::RIGHT_BRACE) && !self.parser.check_if_eof() {
            self.compile_line();
        }

        self.parser.consume(TokenType::RIGHT_BRACE);
    }

    pub fn identifier(&mut self) {
        if self.parser.cur.token_type == TokenType::EQ {
            self.var_assign();
            return
        }

        if self.parser.cur.token_type != TokenType::LEFT_PAREN {
            self.var_call();
            return
        } 

        let pos = self.get_fn_symbol_pos(self.parser.prev.value.iter().collect::<String>());

        self.symbol_to_hold = pos;
    }

    pub fn var_assign(&mut self) {
        let var_name = self.parser.prev.value.iter().collect::<String>();
        self.parser.consume(TokenType::EQ);

        self.expression();
        
        let pos = self.get_cur_locals()
        .iter()
        .enumerate()
        .find(|(_, local)| local.name == var_name)
        .map(|(index, _)| index as i32)
        .unwrap_or(-1);

        if pos == -1 {
            errors::error_message("COMPILER ERROR",
            format!("Symbol: \"{}\" is not defined as var in this scope {}:", var_name, self.line));
            std::process::exit(1);
        }

        let value_type = self.get_cur_chunk().get_last_value().convert();
        let var_type = self.get_cur_locals()[pos as usize].local_type;
        if value_type != var_type {
            errors::error_message("COMPILING ERROR", format!("Mismatched types while assigning var, expected: {:?} found: {:?} {}:",
                var_type,
                value_type,
                self.line,
            ));
            std::process::exit(1);
        }

        self.emit_byte(OpCode::VAR_SET(pos as usize), self.line);
    }

    pub fn var_call(&mut self) {
        let var_name = self.parser.prev.value.iter().collect::<String>();

        let pos = self.get_cur_locals()
            .iter()
            .enumerate()
            .find(|(_, local)| local.name == var_name)
            .map(|(index, _)| index as i32)
            .unwrap_or(-1);

        if pos == -1 {
            errors::error_message("COMPILER ERROR",
            format!("Symbol: \"{}\" is not defined as var in this scope {}:", var_name, self.line));
            std::process::exit(1);
        }

        match self.get_cur_locals()[pos as usize].local_type {
            TokenType::INT => {
                self.get_cur_chunk().push_value(Value::Int(0));
            },
            TokenType::FLOAT => {
                self.get_cur_chunk().push_value(Value::Float(0.0));
            },
            local_type => {
                errors::error_message("COMPILER ERROR", format!("Unexpected local type \"{:?}\" {}:", local_type, self.line));
                std::process::exit(1);
            }
        };
        
        self.emit_byte(OpCode::VAR_CALL(pos as usize), self.line);
    }

    pub fn var_declare(&mut self) {
        self.parser.consume(TokenType::IDENTIFIER);

        let var_name = self.parser.prev.value.iter().collect::<String>();
        if self.get_cur_locals().iter().any(| local | local.name == var_name ) {
            errors::error_message("COMPILER ERROR", format!("Symnbol: \"{}\" is already defined {}:", var_name, self.line));
            std::process::exit(1);
        }

        self.parser.consume(TokenType::COLON);
        if self.parser.cur.token_type != TokenType::KEYWORD(Keywords::INT) {
            errors::error_message("COMPILER ERROR", format!("Expected var type after \":\" {}:", self.line));
            std::process::exit(1);
        }
        let var_type = match self.parser.cur.token_type {
            TokenType::KEYWORD(keyword) => keyword.convert(),
            _ => {
                errors::error_message("COMPILER ERROR", format!("Expected var type after \":\" {}:", self.line));
                std::process::exit(1);
            },
        };
        self.parser.advance();

        if self.parser.cur.token_type == TokenType::EQ {
            self.parser.advance();
            self.expression();

            let value_type = self.get_cur_chunk().get_last_value().convert();
            if value_type != var_type {
                errors::error_message("COMPILING ERROR", format!("Mismatched types while declaring var, expected: {:?} found: {:?} {}:",
                    var_type,
                    value_type,
                    self.line,
                ));
                std::process::exit(1);
            }
        }else {
            let pos = self.get_cur_chunk().push_value(Value::Null);
            self.emit_byte(OpCode::CONSTANT_NULL(pos), self.line);
        }

        self.get_cur_locals().push(Local { name: var_name, local_type: var_type});
    }

    pub fn get_fn_symbol_pos(&mut self, fn_name: String) -> usize {
        let pos = self.parser.symbols
            .iter()
            .enumerate()
            .find(|(_, name)| *name.name == fn_name)
            .map(|(index, _)| index as i32)
            .unwrap_or(-1);

        if pos == -1 {
            errors::error_message("COMPILER ERROR",
            format!("Symbol: \"{}\" is not defined as function in this scope {}:", fn_name, self.line));
            std::process::exit(1);
        }

        pos as usize
    }

    pub fn fn_call(&mut self) {
        if self.parser.symbols[self.symbol_to_hold].symbol_type == TokenType::NATIVE_FN {
            self.emit_byte(OpCode::NATIVE_FN_CALL(self.symbol_to_hold), self.line);
        }else{
            self.emit_byte(OpCode::FUNCTION_CALL(self.symbol_to_hold), self.line);

            match self.parser.symbols[self.symbol_to_hold].output_type {
                TokenType::INT => {
                    self.get_cur_chunk().push_value(Value::Int(0));
                },
                TokenType::FLOAT => {
                    self.get_cur_chunk().push_value(Value::Float(0.0));
                },
                output_type => {
                    errors::error_message("COMPILER ERROR", format!("Unexpected output type \"{:?}\" {}:", output_type, self.line));
                    std::process::exit(1);
                }
            };
        }

        let mut arg_count: usize = 0;
        while self.parser.cur.token_type != TokenType::RIGHT_PAREN {
            arg_count += 1;

            self.expression();
            if self.parser.cur.token_type == TokenType::COMMA {
                self.parser.consume(TokenType::COMMA);
            }
        }
        
        if arg_count != self.parser.symbols[self.symbol_to_hold].arg_count {
            errors::error_message("COMPILER ERROR",
            format!("Expected to find {} arguments but found: {} {}:", self.parser.symbols[self.symbol_to_hold].arg_count, arg_count, self.line));
            std::process::exit(1);
        }
        self.parser.consume(TokenType::RIGHT_PAREN);
    }

    pub fn fn_declare(&mut self) {
        let name = self.parser.cur.value.iter().collect::<String>();

        if self.scope_depth != 0 {
            errors::error_message("COMPILE ERROR", format!("Function \"{}\" declaration inside bounds {}:", name, self.line));
            std::process::exit(1)
        }

        let mut function = Function::new(name.clone());

        self.parser.advance();

        self.parser.consume(TokenType::LEFT_PAREN);

        while self.parser.cur.token_type != TokenType::RIGHT_PAREN {
            function.arg_count += 1;

            self.parser.consume(TokenType::IDENTIFIER);
            let arg_name = self.parser.prev.value.iter().collect::<String>();

            self.parser.consume(TokenType::COLON);
            let arg_type = match self.parser.cur.token_type {
                TokenType::KEYWORD(keyword) => keyword.convert(),
                _ => {
                    errors::error_message("COMPILER ERROR", format!("Expected arg type after \":\" {}:", self.line));
                    std::process::exit(1);
                }
            };
            self.parser.advance();

            if self.parser.cur.token_type == TokenType::COMMA {
                self.parser.consume(TokenType::COMMA);
            }

            function.locals.push(Local { name: arg_name, local_type: arg_type });
        }
        self.parser.consume(TokenType::RIGHT_PAREN);

        let pos = self.get_fn_symbol_pos(name);        
        self.parser.symbols[pos].arg_count = function.arg_count;

        match self.parser.cur.token_type {
            TokenType::KEYWORD(keyword) => {
                function.output_type = keyword.convert();
                self.parser.consume(TokenType::KEYWORD(keyword))
            },
            _ => {},
        };

        self.parser.consume(TokenType::LEFT_BRACE);

        self.scope_depth += 1;

        let enclosing = self.cur_function.clone();
        self.cur_function = function;

        self.block();

        let pos = self.get_cur_chunk().push_value(Value::Null);
        self.emit_byte(OpCode::CONSTANT_NULL(pos), self.line);

        self.emit_byte(OpCode::RETURN, self.line);
        let op_code = OpCode::FUNCTION_DEC(self.cur_function.clone());

        self.cur_function = enclosing;

        self.emit_byte(op_code, self.line);

        self.scope_depth -= 1;
    }

    pub fn declare(&mut self) {
        match self.parser.prev.token_type {
            TokenType::KEYWORD(Keywords::FN) => {
                self.fn_declare();
            },
            TokenType::KEYWORD(Keywords::VAR) => {
                self.var_declare();
            },
            _ => errors::error_unexpected(self.parser.prev.clone(), "declare function"),
        }
    }

    pub fn return_stmt(&mut self) {
        self.expression();

        if let OpCode::VAR_CALL(index) = self.get_cur_chunk().get_last_instruction().op {
            let var_type = self.get_cur_locals()[index].local_type; 
            if var_type != self.cur_function.output_type {
                errors::error_message("COMPILING ERROR", format!("Mismatched types while returning function, expected: {:?} found: {:?} {}:",
                    self.cur_function.output_type,
                    var_type,
                    self.line,
                ));
                std::process::exit(1);
            }
        }else {
            let value_type = self.get_cur_chunk().get_last_value().convert();
            if value_type != self.cur_function.output_type {
                errors::error_message("COMPILING ERROR", format!("Mismatched types while returning function, expected: {:?} found: {:?} {}:",
                    self.cur_function.output_type,
                    value_type,
                    self.line,
                ));
                std::process::exit(1);
            }
        }


        self.emit_byte(OpCode::RETURN, self.line);
    }

    fn compile_line(&mut self) {
        match self.parser.cur.token_type {
            TokenType::KEYWORD(Keywords::FN) | TokenType::KEYWORD(Keywords::VAR) => {
                self.parser.advance();
                self.declare();
            },
            TokenType::KEYWORD(Keywords::RETURN) => {
                self.parser.advance();
                self.return_stmt();
            },
            _ => self.expression(),
        }
    }

    pub fn compile(&mut self) -> Chunk {
        self.parser.advance();
        loop {
            self.line = self.parser.cur.line;
            if self.parser.check_if_eof() {
                break;
            }
            self.compile_line();
        }

        self.get_cur_chunk().clone()
    }

    pub fn parse(&mut self, prec: Precedence) {
        self.parser.advance();

        if !self.parser.rules.contains_key(&self.parser.prev.token_type) {
            errors::error_message("PARSING ERROR", format!("Cannot get a parse rule for: {:?}: \"{}\", {}:",
                self.parser.prev.token_type,
                self.parser.prev.value.iter().collect::<String>(),
                self.line,
            ));
            std::process::exit(1);
        }
        let rule = self.parser.get_rule(&self.parser.prev.token_type);

        match rule.prefix {
            Some(f) => f(self),
            _ => {
                errors::error_message("PARSING ERROR", format!("Expected prefix for: {:?}, {}:", self.parser.prev.token_type, self.line));
                std::process::exit(1);
            },
        };

        while prec <= self.parser.get_rule(&self.parser.cur.token_type).prec {
            self.parser.advance();

            if !self.parser.rules.contains_key(&self.parser.prev.token_type) {
                errors::error_message("PARSING ERROR", format!("Cannot get a parse rule for: {:?}: \"{}\", {}:",
                    self.parser.prev.token_type,
                    self.parser.prev.value.iter().collect::<String>(),
                    self.line,
                ));
                std::process::exit(1);
            }
            let rule = self.parser.get_rule(&self.parser.prev.token_type);
            match rule.infix {
                Some(f) => f(self),
                _ => {
                    errors::error_message("PARSING ERROR", format!("Expected infix for: {:?}, {}:", self.parser.prev.token_type, self.line));
                    std::process::exit(1);
                },
            }
        }
    }

    pub fn emit_byte(&mut self, op: OpCode, line: u32) {
        if self.scope_depth == 0 {
            errors::error_message("PARSER ERROR", format!("Expression found outside of bounds {}:",self.line));
            std::process::exit(1)
        }
        self.get_cur_chunk().push(Instruction{ op: op, line: line });
    }
}