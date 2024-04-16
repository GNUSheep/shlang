use std::collections::HashMap;

use crate::{
    objects::{functions::{Function, Local, NativeFn}, rc::Object, string::StringObj, structs::{Struct, StructInstance}}, vm::{bytecode::{Chunk, Instruction, OpCode}, value::{Convert, Value}
}};
use crate::frontend::tokens::{Token, TokenType, Keywords};

use super::errors::{self, error_message};

pub struct LoopInfo {
    pub start: usize,
    pub locals_start: usize,
}

impl LoopInfo {
    pub fn new() -> Self {
        LoopInfo {
            start: 0,
            locals_start: 0,
        }
    }
}

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

        (TokenType::KEYWORD(Keywords::RETURN), ParseRule { prefix: None, infix: None, prec: Precedence::NONE }),

        (TokenType::KEYWORD(Keywords::IF), ParseRule { prefix: None, infix: None, prec: Precedence::NONE }),

        (TokenType::KEYWORD(Keywords::WHILE), ParseRule { prefix: None, infix: None, prec: Precedence::NONE }),
        (TokenType::KEYWORD(Keywords::FOR), ParseRule { prefix: None, infix: None, prec: Precedence::NONE }),
        (TokenType::KEYWORD(Keywords::BREAK), ParseRule { prefix: None, infix: None, prec: Precedence::NONE }),

        (TokenType::STRING, ParseRule { prefix: Some(Compiler::string_parse), infix: None, prec: Precedence::NONE }),

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
        
        (TokenType::KEYWORD(Keywords::AND), ParseRule { prefix: None, infix: Some(Compiler::and_op), prec: Precedence::AND }),
        (TokenType::KEYWORD(Keywords::OR), ParseRule { prefix: None, infix: Some(Compiler::or_op), prec: Precedence::OR }),

        (TokenType::PLUS, ParseRule { prefix: None, infix: Some(Compiler::arithmetic), prec: Precedence::TERM }),
        (TokenType::MINUS, ParseRule { prefix: Some(Compiler::negation), infix: Some(Compiler::arithmetic), prec: Precedence::TERM }),
        (TokenType::STAR, ParseRule { prefix: None, infix: Some(Compiler::arithmetic), prec: Precedence::FACTOR }),
        (TokenType::SLASH, ParseRule { prefix: None, infix: Some(Compiler::arithmetic), prec: Precedence::FACTOR }),
        (TokenType::MOD, ParseRule { prefix: None, infix: Some(Compiler::arithmetic), prec: Precedence::FACTOR }),

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
        symbols.push(Symbol { name: "String".to_string(), symbol_type: TokenType::KEYWORD(Keywords::STRUCT), output_type: TokenType::STRING, arg_count: 1 });

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

                symbols.push(Symbol{name: fn_name, symbol_type: TokenType::KEYWORD(Keywords::FN), output_type: TokenType::NULL, arg_count: 0 });
            }

            if token_pair[0].token_type == TokenType::KEYWORD(Keywords::STRUCT) {
                let struct_name = token_pair[1].value.iter().collect::<String>();

                if symbols.iter().any(| symbol | symbol.name == struct_name) {
                    errors::error_message("COMPILER ERROR", format!("Struct: \"{}\" is already defined {}:", struct_name, token_pair[1].line));
                    std::process::exit(1);
                }

                symbols.push(Symbol{name: struct_name, symbol_type: TokenType::KEYWORD(Keywords::STRUCT), output_type: TokenType::KEYWORD(Keywords::NULL), arg_count: 0 });
            }

            let symbol_len = symbols.len();
            match token_pair[0].token_type {
                TokenType::KEYWORD(Keywords::INT) => symbols[symbol_len - 1].output_type = TokenType::INT,
                TokenType::KEYWORD(Keywords::FLOAT) => symbols[symbol_len - 1].output_type = TokenType::FLOAT,
                TokenType::KEYWORD(Keywords::BOOL) => symbols[symbol_len - 1].output_type = TokenType::BOOL,
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
    loop_info: LoopInfo,
    structs: HashMap<String, Struct>,
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
            loop_info: LoopInfo::new(),
            structs: HashMap::new(),
        }
    }

    pub fn get_cur_chunk(&mut self) -> &mut Chunk {
        self.cur_function.get_chunk()
    }

    pub fn get_cur_locals(&mut self) -> &mut Vec<Local> {
        self.cur_function.get_locals()
    }

    pub fn get_cur_instances(&mut self) -> &mut Vec<Local> {
        self.cur_function.get_instances()
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
                    TokenType::MOD => self.emit_byte(OpCode::MOD_INT, self.line),
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
                    TokenType::MOD => self.emit_byte(OpCode::MOD_FLOAT, self.line),
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

    pub fn string_parse(&mut self) {
        // todo tmp declare and dec rc 
        self.string_dec()
    }
 
    pub fn string_dec(&mut self) {
        let pos = self.get_struct_symbol_pos("String".to_string());

        let mut instance_obj = StructInstance::new(pos);

        let len = self.parser.symbols.len();
        instance_obj.set_index(len);

        let value = self.parser.prev.value.iter().collect::<String>();
        instance_obj.fields_values.push(Value::String(value.clone()));

        self.emit_byte(OpCode::STRING_DEC(instance_obj), self.line);

        self.get_cur_instances().push(Local{ name: String::new(), local_type: TokenType::KEYWORD(Keywords::INSTANCE(pos)), is_redirected: false, redirect_pos: 0, rf_index: len });

        self.parser.symbols.push(Symbol { name: String::new(), symbol_type: TokenType::KEYWORD(Keywords::INSTANCE(pos)), output_type: TokenType::KEYWORD(Keywords::NULL), arg_count: 0 })
    }

    pub fn identifier(&mut self) {
        if self.parser.cur.token_type == TokenType::EQ {
            self.var_assign();
            return
        }

        if self.parser.cur.token_type == TokenType::DOT {
            self.instance_call();
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
        
        let pos = self.get_local_pos(var_name);

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

        let pos = self.get_cur_instances()
            .iter()
            .enumerate()
            .find(|(_, local)| local.name == var_name)
            .map(|(index, _)| index as i32)
            .unwrap_or(-1);

        if pos != -1 {
            match self.get_cur_instances()[pos as usize].local_type {
                TokenType::KEYWORD(Keywords::INSTANCE(_)) => {
                    let pos = self.get_instance_local_pos(var_name);
                    
                    let heap_pos = self.get_cur_instances()[pos].rf_index;
                    
                    self.emit_byte(OpCode::GET_INSTANCE_RF(heap_pos), self.line);
                    
                    self.emit_byte(OpCode::INC_RC(pos as usize), self.line);
                    
                    return
                },
                _ => {},
            }
            
        }

        let pos = self.get_local_pos(var_name);
        match self.get_cur_locals()[pos as usize].local_type {
            TokenType::INT => {
                self.get_cur_chunk().push_value(Value::Int(0));
            },
            TokenType::FLOAT => {
                self.get_cur_chunk().push_value(Value::Float(0.0));
            },
            TokenType::BOOL => {
                self.get_cur_chunk().push_value(Value::Bool(true));
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
            errors::error_message("COMPILER ERROR", format!("Symbol: \"{}\" is already defined {}:", var_name, self.line));
            std::process::exit(1);
        }

        if self.get_cur_instances().iter().any(| local | local.name == var_name ) {
            errors::error_message("COMPILER ERROR", format!("Symbol: \"{}\" is already defined {}:", var_name, self.line));
            std::process::exit(1);
        }

        self.parser.consume(TokenType::COLON);
        match self.parser.cur.token_type {
            TokenType::KEYWORD(Keywords::INT) |
            TokenType::KEYWORD(Keywords::FLOAT) |
            TokenType::KEYWORD(Keywords::BOOL) |
            TokenType::IDENTIFIER => {},
            _ => {
                errors::error_message("COMPILER ERROR", format!("Expected var type after \":\" {}:", self.line));
                std::process::exit(1);
            },
        };

        let var_type = match self.parser.cur.token_type {
            TokenType::KEYWORD(keyword) => keyword.convert(),
            TokenType::IDENTIFIER => {
                let pos = self.get_struct_symbol_pos(self.parser.cur.value.iter().collect::<String>());

                TokenType::STRUCT(pos)
            }
            _ => {
                errors::error_message("COMPILER ERROR", format!("Expected var type after \":\" {}:", self.line));
                std::process::exit(1);
            },
        };
        self.parser.advance();

        match var_type {
            TokenType::STRUCT(pos) => {
                self.instance_declare(pos, var_name);

                return
            }
            _ => {},
        }

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

        self.get_cur_locals().push(Local { name: var_name, local_type: var_type, is_redirected: false, redirect_pos: 0, rf_index: 0 });
    }

    pub fn instance_call(&mut self) {
        let name = self.parser.prev.value.iter().collect::<String>();

        self.parser.consume(TokenType::DOT);

        let instance_pos = self.get_instance_local_pos(name.clone());

        self.parser.consume(TokenType::IDENTIFIER);
        let field_name = self.parser.prev.value.iter().collect::<String>();

        let root_struct_name = match self.get_cur_instances()[instance_pos].local_type {
            TokenType::KEYWORD(Keywords::INSTANCE(root_struct_pos)) => {
                self.parser.symbols[root_struct_pos].name.clone()
            },
            _ => {
                errors::error_message("COMPILING ERROR", format!("Cannot find root struct for instance \"{}\" {}:",
                    name,
                    self.line,
                ));
                std::process::exit(1);
            },
        };

        if self.parser.cur.token_type == TokenType::LEFT_PAREN {
            match self.structs.get(&root_struct_name).unwrap().methods.get(&field_name) {
                Some(mth) => {
                    self.mth_call(mth.output_type, mth.arg_count, name, mth.is_self_arg);
                },
                None => {
                    errors::error_message("COMPILING ERROR", format!("Method: \"{}\" is not declared in struct \"{}\" {}:",
                        field_name,
                        root_struct_name,
                        self.line,
                    ));
                    std::process::exit(1);
                },
            }
            
            match self.structs.get(&root_struct_name).unwrap().methods.get(&field_name) {
                Some(mth) => {
                    self.emit_byte(OpCode::METHOD_CALL(mth.clone()), self.line);
                },
                _ => {},
            }

            return
        }

        let field_index = self.structs.get(&root_struct_name).unwrap().locals
            .iter()
            .enumerate()
            .find(|(_, local)| *local.name == field_name)
            .map(|(index, _)| index as i32)
            .unwrap_or(-1);

        if field_index == -1 {
            errors::error_message("COMPILING ERROR", format!("Field: \"{}\" is not declared in struct \"{}\" {}:",
                field_name,
                root_struct_name,
                self.line,
            ));
            std::process::exit(1);
        }

        let pos = self.get_instance_local_pos(name);

        if self.parser.cur.token_type == TokenType::EQ {
            self.parser.consume(TokenType::EQ);

            self.expression();

            if self.get_cur_chunk().get_last_value().convert() != self.structs.get(&root_struct_name).unwrap().locals[field_index as usize].local_type {
                let value_type = self.get_cur_chunk().get_last_value().convert();

                errors::error_message("COMPILER ERROR",
                format!("Expected to find {:?} but found: {:?} {}:", 
                    self.structs.get(&root_struct_name).unwrap().locals[field_index as usize].local_type, 
                    value_type,
                    self.line
                ));
                std::process::exit(1);
            }

            self.emit_byte(OpCode::SET_INSTANCE_FIELD(pos as usize, field_index as usize), self.line);
        }else{
            match self.structs.get(&root_struct_name).unwrap().locals[field_index as usize].local_type {
                TokenType::INT => {
                    self.get_cur_chunk().push_value(Value::Int(0));
                },
                TokenType::FLOAT => {
                    self.get_cur_chunk().push_value(Value::Float(0.0));
                },
                TokenType::BOOL => {
                    self.get_cur_chunk().push_value(Value::Bool(true));
                },
                TokenType::NULL => {
                    self.get_cur_chunk().push_value(Value::Null);
                },
                _ => {},
            }

            self.emit_byte(OpCode::GET_INSTANCE_FIELD(pos as usize, field_index as usize), self.line);
        }


    }

    pub fn instance_declare(&mut self, pos: usize, name: String) {
        if self.parser.cur.token_type != TokenType::EQ {
            errors::error_message("COMPILING ERROR", format!("Struct cannot be left undeclared {}:",
                self.line,
            ));
            std::process::exit(1);
        }
        self.parser.consume(TokenType::EQ);

        if self.parser.cur.token_type != TokenType::LEFT_BRACE {
            let value = self.parser.cur.value.iter().collect::<String>();

            let pos = self.get_instance_local_pos(value);
            let local_type = self.get_cur_instances()[pos].local_type;
            let local_rf_pos = self.get_cur_instances()[pos].rf_index;
            
            self.get_cur_instances().push(Local{ name: name, local_type: local_type, is_redirected: true, redirect_pos: pos, rf_index: local_rf_pos });

            return
        }
        self.parser.consume(TokenType::LEFT_BRACE);
        
        let mut field_counts = 0;

        let root_struct_name = self.parser.symbols[pos].name.clone();
        while self.parser.cur.token_type != TokenType::RIGHT_BRACE {
            self.expression();

            if self.get_cur_chunk().get_last_value().convert() != self.structs.get(&root_struct_name).unwrap().locals[field_counts].local_type {
                let value_type = self.get_cur_chunk().get_last_value().convert();

                errors::error_message("COMPILER ERROR",
                format!("Expected to find {:?} but found: {:?} {}:", 
                    self.structs.get(&root_struct_name).unwrap().locals[field_counts].local_type, 
                    value_type,
                    self.line
                ));
                std::process::exit(1);
            }
            
            if self.parser.cur.token_type == TokenType::COMMA {
                self.parser.consume(TokenType::COMMA);
            }
            field_counts += 1;
        }
        self.parser.consume(TokenType::RIGHT_BRACE);

        let mut instance_obj = StructInstance::new(pos);

        if field_counts != self.parser.symbols[pos].arg_count {
            errors::error_message("COMPILER ERROR",
            format!("Expected to find {} fields but found: {} {}:", self.parser.symbols[pos].arg_count, field_counts, self.line));
            std::process::exit(1);
        }
        let len = self.parser.symbols.len();
        instance_obj.set_index(len);

        self.emit_byte(OpCode::INSTANCE_DEC(instance_obj), self.line);

        self.get_cur_instances().push(Local{ name: name, local_type: TokenType::KEYWORD(Keywords::INSTANCE(pos)), is_redirected: false, redirect_pos: 0, rf_index: len });

        self.parser.symbols.push(Symbol { name: String::new(), symbol_type: TokenType::KEYWORD(Keywords::INSTANCE(pos)), output_type: TokenType::KEYWORD(Keywords::NULL), arg_count: 0 })
    }

    pub fn struct_declare(&mut self) {
        self.parser.consume(TokenType::IDENTIFIER);

        let name = self.parser.prev.value.iter().collect::<String>();

        if self.scope_depth != 0 {
            errors::error_message("COMPILE ERROR", format!("Struct \"{}\" declaration inside bounds {}:", name, self.line));
            std::process::exit(1)
        }

        let mut struct_obj = Struct::new(name.clone());

        self.scope_depth += 1;

        self.parser.consume(TokenType::LEFT_BRACE);
        while self.parser.cur.token_type != TokenType::RIGHT_BRACE && self.parser.cur.token_type != TokenType::KEYWORD(Keywords::METHODS) {
            self.parser.consume(TokenType::IDENTIFIER);

            let field_name = self.parser.prev.value.iter().collect::<String>();

            self.parser.consume(TokenType::COLON);

            let field_type = match self.parser.cur.token_type {
                TokenType::KEYWORD(keyword) => keyword.convert(),
                _ => {
                    errors::error_message("COMPILER ERROR", format!("Expected field type after \":\" {}:", self.line));
                    std::process::exit(1);
                },
            };
            self.parser.advance();

            self.parser.consume(TokenType::COMMA);

            struct_obj.locals.push(Local { name: field_name, local_type: field_type, is_redirected: false, redirect_pos: 0, rf_index: 0 });
        }

        // need to do that, because methods will not be compiled otherwise
        self.structs.insert(name.clone(), struct_obj.clone());

        if self.parser.cur.token_type == TokenType::KEYWORD(Keywords::METHODS) {
            self.parser.advance();
            self.mth_stmt(&mut struct_obj);
        }

        self.parser.consume(TokenType::RIGHT_BRACE);
        
        struct_obj.field_count = struct_obj.locals.len();

        let pos = self.get_struct_symbol_pos(name.clone());
        self.parser.symbols[pos].arg_count = struct_obj.locals.len();

        self.structs.insert(name.clone(), struct_obj.clone());

        self.emit_byte(OpCode::STRUCT_DEC(struct_obj), self.line);
        
        self.scope_depth -= 1;
    }

    pub fn mth_call(&mut self, output_type: TokenType, mth_arg_count: usize, instance_name: String, is_self: bool) {
        self.parser.consume(TokenType::LEFT_PAREN);
        if is_self {
            let pos = self.get_instance_local_pos(instance_name);

            let heap_pos = self.get_cur_instances()[pos].rf_index;
            self.emit_byte(OpCode::GET_INSTANCE_RF(heap_pos), self.line);

            self.emit_byte(OpCode::INC_RC(pos as usize), self.line);
        }

        let mut arg_count = 0;
        while self.parser.cur.token_type != TokenType::RIGHT_PAREN {
            arg_count += 1;
            
            self.expression();

            if self.parser.cur.token_type == TokenType::COMMA {
                self.parser.consume(TokenType::COMMA);
            }
        }
        self.parser.consume(TokenType::RIGHT_PAREN);

        if arg_count != mth_arg_count {
            errors::error_message("COMPILER ERROR",
            format!("Expected to find {} arguments but found: {} {}:", mth_arg_count, arg_count, self.line));
            std::process::exit(1);
        }

        match output_type {
            TokenType::INT => {
                self.get_cur_chunk().push_value(Value::Int(0));
            },
            TokenType::FLOAT => {
                self.get_cur_chunk().push_value(Value::Float(0.0));
            },
            TokenType::BOOL => {
                self.get_cur_chunk().push_value(Value::Bool(true));
            },
            TokenType::NULL => {
                self.get_cur_chunk().push_value(Value::Null);
            },
            output_type => {
                errors::error_message("COMPILER ERROR", format!("Unexpected output type \"{:?}\" {}:", output_type, self.line));
                std::process::exit(1);
            }
        };
    }

    pub fn mth_stmt(&mut self, struct_obj: &mut Struct) {
        self.parser.consume(TokenType::LEFT_BRACE);

        while self.parser.cur.token_type != TokenType::RIGHT_BRACE {
            let name = self.parser.cur.value.iter().collect::<String>();

            if struct_obj.methods.contains_key(&name) {
                errors::error_message("COMPILER ERROR", format!("Method: \"{}\" is already defined for struct: \"{}\" {}:", name, struct_obj.name, self.line));
                std::process::exit(1);
            }

            let root_struct_pos = self.get_struct_symbol_pos(struct_obj.name.clone());
            let mth = self.fn_declare(true, root_struct_pos);

            struct_obj.methods.insert(name, mth);
        }

        self.parser.consume(TokenType::RIGHT_BRACE);
    }

    pub fn get_fn_symbol_pos(&mut self, fn_name: String) -> usize {
        let pos = self.parser.symbols
            .iter()
            .enumerate()
            .find(|(_, name)| *name.name == fn_name && name.symbol_type != TokenType::KEYWORD(Keywords::STRUCT))
            .map(|(index, _)| index as i32)
            .unwrap_or(-1);

        if pos == -1 {
            errors::error_message("COMPILER ERROR",
            format!("Symbol: \"{}\" is not defined as function in this scope {}:", fn_name, self.line));
            std::process::exit(1);
        }

        pos as usize
    }
    
    pub fn get_struct_symbol_pos(&mut self, struct_name: String) -> usize {
        let pos = self.parser.symbols
            .iter()
            .enumerate()
            .find(|(_, name)| *name.name == struct_name && name.symbol_type == TokenType::KEYWORD(Keywords::STRUCT))
            .map(|(index, _)| index as i32)
            .unwrap_or(-1);

        if pos == -1 {
            errors::error_message("COMPILER ERROR",
            format!("Symbol: \"{}\" is not defined as struct in this scope {}:", struct_name, self.line));
            std::process::exit(1);
        }

        pos as usize
    }

    pub fn get_local_pos(&mut self, name: String) -> usize {
        let pos = self.get_cur_locals()
            .iter()
            .enumerate()
            .find(|(_, local)| local.name == name)
            .map(|(index, _)| index as i32)
            .unwrap_or(-1);

        if pos == -1 {
            errors::error_message("COMPILER ERROR",
            format!("Symbol: \"{}\" is not defined as var in this scope {}:", name, self.line));
            std::process::exit(1);
        }

        pos as usize
    }
    
    pub fn get_instance_local_pos(&mut self, instance_name: String) -> usize {
        let pos = self.get_cur_instances()
            .iter()
            .enumerate()
            .find(|(_, local)| {
                local.name == instance_name &&
                matches!(local.local_type, TokenType::KEYWORD(Keywords::INSTANCE(_)))
            })
            .map(|(index, _)| index as i32)
            .unwrap_or(-1);

        if pos == -1 {
            errors::error_message("COMPILER ERROR",
            format!("Local: \"{}\" is not defined as instance in this scope {}:", instance_name, self.line));
            std::process::exit(1);
        }

        if self.get_cur_instances()[pos as usize].is_redirected {
            return self.get_cur_instances()[pos as usize].redirect_pos
        }

        pos as usize
    }

    pub fn fn_call(&mut self) {
        let mut arg_count: usize = 0;
        
        let symbol_to_hold_enclosing = self.symbol_to_hold;
        while self.parser.cur.token_type != TokenType::RIGHT_PAREN {
            arg_count += 1;

            self.expression();

            if self.parser.cur.token_type == TokenType::COMMA {
                self.parser.consume(TokenType::COMMA);
            }
        }
        self.parser.consume(TokenType::RIGHT_PAREN);
        self.symbol_to_hold = symbol_to_hold_enclosing;

        if self.parser.symbols[self.symbol_to_hold].name == "print" || self.parser.symbols[self.symbol_to_hold].name == "println" {
            self.emit_byte(OpCode::PRINT_FN_CALL(self.symbol_to_hold, arg_count), self.line);
            return
        }

        if arg_count != self.parser.symbols[self.symbol_to_hold].arg_count {
            errors::error_message("COMPILER ERROR",
            format!("Expected to find {} arguments but found: {} {}:", self.parser.symbols[self.symbol_to_hold].arg_count, arg_count, self.line));
            std::process::exit(1);
        }

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
                TokenType::BOOL => {
                    self.get_cur_chunk().push_value(Value::Bool(true));
                },
                TokenType::NULL => {
                    self.get_cur_chunk().push_value(Value::Null);
                },
                output_type => {
                    errors::error_message("COMPILER ERROR", format!("Unexpected output type \"{:?}\" {}:", output_type, self.line));
                    std::process::exit(1);
                }
            };
        }
    }

    pub fn fn_declare(&mut self, is_mth: bool, root_struct_pos: usize) -> Function {
        let name = self.parser.cur.value.iter().collect::<String>();

        if (self.scope_depth != 0 && !is_mth) || (self.scope_depth == 0 && is_mth) {
            errors::error_message("COMPILE ERROR", format!("Function/Method \"{}\" declaration inside bounds {}:", name, self.line));
            std::process::exit(1)
        }
        let mut function = Function::new(name.clone());

        self.parser.advance();

        self.parser.consume(TokenType::LEFT_PAREN);

        while self.parser.cur.token_type != TokenType::RIGHT_PAREN {
            function.arg_count += 1;

            self.parser.consume(TokenType::IDENTIFIER);
            let arg_name = self.parser.prev.value.iter().collect::<String>();

            if arg_name == "self" && is_mth {
                if function.arg_count != 1 {
                    errors::error_message("COMPILE ERROR", format!("\"self\" keyword need to be first in argument list {}:", self.line));
                    std::process::exit(1)
                }

                function.is_self_arg =  true;
                function.arg_count -= 1;

                if self.parser.cur.token_type == TokenType::COMMA {
                    self.parser.consume(TokenType::COMMA);
                }

                function.instances.push(Local { name: "self".to_string(), local_type: TokenType::KEYWORD(Keywords::INSTANCE(root_struct_pos)), is_redirected: false, redirect_pos: 0, rf_index: 0 });

                continue;
            }

            self.parser.consume(TokenType::COLON);
            let arg_type = match self.parser.cur.token_type {
                TokenType::KEYWORD(keyword) => keyword.convert(),
                TokenType::IDENTIFIER => {
                    let value = self.parser.cur.value.iter().collect::<String>();
                    let pos = self.get_struct_symbol_pos(value);

                    TokenType::KEYWORD(Keywords::INSTANCE(pos))
                }
                _ => {
                    errors::error_message("COMPILER ERROR", format!("Expected arg type after \":\" {}:", self.line));
                    std::process::exit(1);
                }
            };
            self.parser.advance();

            if self.parser.cur.token_type == TokenType::COMMA {
                self.parser.consume(TokenType::COMMA);
            }

            if matches!(arg_type, TokenType::KEYWORD(Keywords::INSTANCE(_))) {
                function.instances.push(Local { name: arg_name, local_type: arg_type , is_redirected: false, redirect_pos: 0, rf_index: 0 });
            }else {
                function.locals.push(Local { name: arg_name, local_type: arg_type , is_redirected: false, redirect_pos: 0, rf_index: 0 });
            }

        }
        self.parser.consume(TokenType::RIGHT_PAREN);

        if !is_mth {
            let pos = self.get_fn_symbol_pos(name.clone());        
            self.parser.symbols[pos].arg_count = function.arg_count;
        }

        match self.parser.cur.token_type {
            TokenType::KEYWORD(keyword) => {
                function.output_type = keyword.convert();
                self.parser.consume(TokenType::KEYWORD(keyword))
            },
            _ => {
                function.output_type = TokenType::NULL;
            }
        };

        self.parser.consume(TokenType::LEFT_BRACE);

        self.scope_depth += 1;

        let enclosing = self.cur_function.clone();
        self.cur_function = function;

        self.block();

        let pos = self.get_cur_chunk().push_value(Value::Null);
        self.emit_byte(OpCode::CONSTANT_NULL(pos), self.line);

        self.emit_byte(OpCode::RETURN, self.line);

        for index in 0..self.get_cur_instances().len() {
            match self.get_cur_instances()[index].local_type.clone() {
                TokenType::KEYWORD(Keywords::INSTANCE(_)) => {
                    self.emit_byte(OpCode::DEC_RC(index), self.line);
                },
                _ => {},
            }
        }

        self.emit_byte(OpCode::END_OF_FN, self.line);

        if is_mth {
            let fun = self.cur_function.clone();
            self.cur_function = enclosing;
            self.scope_depth -= 1;

            return fun
        }

        let op_code = OpCode::FUNCTION_DEC(self.cur_function.clone());

        self.cur_function = enclosing;

        self.emit_byte(op_code, self.line);

        self.scope_depth -= 1;

        Function::new(String::new())
    }

    pub fn declare(&mut self) {
        match self.parser.prev.token_type {
            TokenType::KEYWORD(Keywords::FN) => {
                let _ = self.fn_declare(false, 0);
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

    pub fn if_stmt(&mut self) {
        if self.parser.cur.token_type == TokenType::LEFT_BRACE {
            errors::error_message("COMPILING ERROR", format!("Expected to find expression after {} statement {}:",
                self.parser.prev.value.iter().collect::<String>().to_ascii_uppercase(),
                self.line,
            ));
            std::process::exit(1);
        }

        self.expression();

        if self.parser.symbols.len() > 1 && 
        self.parser.symbols[self.symbol_to_hold].symbol_type == TokenType::KEYWORD(Keywords::FN) &&
        self.parser.symbols[self.symbol_to_hold].output_type != TokenType::BOOL {
            errors::error_message("COMPILING ERROR", format!("Expected to find BOOL but found {:?} {}:",
                self.parser.symbols[self.symbol_to_hold].output_type,
                self.line,
            ));
            std::process::exit(1);
        }

        let index_jump_to_stmt = self.get_cur_chunk().code.len();
        self.emit_byte(OpCode::IF_STMT_OFFSET(0), self.line);

        self.emit_byte(OpCode::POP, self.line);

        self.parser.consume(TokenType::LEFT_BRACE);

        let local_counter = self.get_cur_locals().len();

        self.block();

        for _ in 0..self.get_cur_locals().len() - local_counter {
            self.emit_byte(OpCode::POP, self.line);
            self.get_cur_locals().pop();
        }

        let index_exit_if = self.get_cur_chunk().code.len();
        self.emit_byte(OpCode::JUMP(0), self.line);

        let offset_stmt = (self.get_cur_chunk().code.len() - index_jump_to_stmt) - 1;
        self.get_cur_chunk().code[index_jump_to_stmt] = Instruction { op: OpCode::IF_STMT_OFFSET(offset_stmt), line: self.line };

        self.emit_byte(OpCode::POP, self.line);

        if self.parser.cur.token_type == TokenType::KEYWORD(Keywords::ELIF) || self.parser.cur.token_type == TokenType::KEYWORD(Keywords::ELSE) {
            self.compile_line();
        }

        let offset_exit_if = (self.get_cur_chunk().code.len() - index_exit_if) - 1;
        self.get_cur_chunk().code[index_exit_if] = Instruction { op: OpCode::JUMP(offset_exit_if), line: self.line };
    }

    pub fn else_stmt(&mut self) {
        self.parser.consume(TokenType::LEFT_BRACE);
        self.block();
    }

    pub fn while_stmt(&mut self) {
        let loop_start_index = self.get_cur_chunk().code.len();

        if self.parser.cur.token_type == TokenType::LEFT_BRACE {
            errors::error_message("COMPILING ERROR", format!("Expected to find expression after {} statement {}:",
                self.parser.prev.value.iter().collect::<String>().to_ascii_uppercase(),
                self.line,
            ));
            std::process::exit(1);
        }

        self.expression();

        if self.parser.symbols.len() > 1 && 
        self.parser.symbols[self.symbol_to_hold].symbol_type == TokenType::KEYWORD(Keywords::FN) &&
        self.parser.symbols[self.symbol_to_hold].output_type != TokenType::BOOL {
            errors::error_message("COMPILING ERROR", format!("Expected to find BOOL but found {:?} {}:",
                self.parser.symbols[self.symbol_to_hold].output_type,
                self.line,
            ));
            std::process::exit(1);
        };

        let index_exit_stmt = self.get_cur_chunk().code.len();
        self.emit_byte(OpCode::IF_STMT_OFFSET(0), self.line);
        self.emit_byte(OpCode::POP, self.line);

        self.parser.consume(TokenType::LEFT_BRACE);

        let local_counter = self.get_cur_locals().len();
        self.scope_depth += 1;

        self.loop_info.start = loop_start_index;

        self.block();

        self.loop_info.start = loop_start_index;
        self.scope_depth -= 1;

        for _ in 0..self.get_cur_locals().len() - local_counter {
            self.emit_byte(OpCode::POP, self.line);
            self.get_cur_locals().pop();
        }

        let offset_loop = (self.get_cur_chunk().code.len() - loop_start_index) + 1;
        self.emit_byte(OpCode::LOOP(offset_loop), self.line);

        let offset_stmt = (self.get_cur_chunk().code.len() - index_exit_stmt) - 1;
        self.get_cur_chunk().code[index_exit_stmt] = Instruction { op: OpCode::IF_STMT_OFFSET(offset_stmt), line: self.line };

        self.emit_byte(OpCode::POP, self.line);
    }

    pub fn for_stmt(&mut self) {
        self.parser.consume(TokenType::IDENTIFIER);

        let identifier = self.parser.prev.value.iter().collect::<String>();
        self.get_cur_locals().push(Local { name: identifier, local_type: TokenType::INT, is_redirected: false, redirect_pos: 0, rf_index: 0 });

        self.parser.consume(TokenType::KEYWORD(Keywords::IN));

        // in future there need to check if I got a range or vec list to iterate on.
        self.parser.consume(TokenType::LEFT_PAREN);
        
        self.expression();

        self.parser.consume(TokenType::COMMA);

        self.expression();

        self.get_cur_locals().push(Local { name: "".to_string(), local_type: TokenType::INT, is_redirected: false, redirect_pos: 0, rf_index: 0 });

        if self.parser.cur.token_type != TokenType::RIGHT_PAREN {
            self.parser.consume(TokenType::COMMA);

            self.expression();

            match self.get_cur_chunk().get_last_instruction().op {
                OpCode::FUNCTION_CALL(_) => {
                    errors::error_message("COMPILING ERROR", format!("Functions cannot be used as STEP BY argument {}:",
                        self.line,
                    ));
                    std::process::exit(1);
                },
                _ => {},
            }
        }else {
            let pos = self.get_cur_chunk().push_value(Value::Int(1));
            self.emit_byte(OpCode::CONSTANT_INT(pos), self.line);
        }

        self.get_cur_locals().push(Local { name: "".to_string(), local_type: TokenType::INT, is_redirected: false, redirect_pos: 0, rf_index: 0 });

        self.parser.consume(TokenType::RIGHT_PAREN);

        let loop_start_index = self.get_cur_chunk().code.len();

        // check if condition is still true
        let len_locals = self.get_cur_locals().len();

        self.emit_byte(OpCode::VAR_CALL(len_locals - 3), self.line);
        self.emit_byte(OpCode::VAR_CALL(len_locals - 2), self.line);

        self.emit_byte(OpCode::NEG_EQ_INT, self.line);
        //

        let index_exit_stmt = self.get_cur_chunk().code.len();
        self.emit_byte(OpCode::IF_STMT_OFFSET(0), self.line);
        self.emit_byte(OpCode::POP, self.line);

        self.parser.consume(TokenType::LEFT_BRACE);

        let local_counter = self.get_cur_locals().len();
        self.scope_depth += 1;

        self.loop_info.locals_start = local_counter;
        self.loop_info.start = loop_start_index;

        self.block();

        self.loop_info.start = loop_start_index;
        self.loop_info.locals_start = local_counter;
        self.scope_depth -= 1;

        // adding
        self.emit_byte(OpCode::VAR_CALL(len_locals - 3), self.line);

        self.emit_byte(OpCode::VAR_CALL(len_locals - 1), self.line);

        self.emit_byte(OpCode::ADD_INT, self.line);

        self.emit_byte(OpCode::VAR_SET(len_locals - 3), self.line);
        //

        for _ in 0..self.get_cur_locals().len() - local_counter {
            self.emit_byte(OpCode::POP, self.line);
            self.get_cur_locals().pop();
        }

        let offset_loop = (self.get_cur_chunk().code.len() - loop_start_index) + 1;
        self.emit_byte(OpCode::LOOP(offset_loop), self.line);

        let offset_stmt = (self.get_cur_chunk().code.len() - index_exit_stmt) - 1;
        self.get_cur_chunk().code[index_exit_stmt] = Instruction { op: OpCode::IF_STMT_OFFSET(offset_stmt), line: self.line };

        self.emit_byte(OpCode::POP, self.line);

        for _ in 0..3{
            self.emit_byte(OpCode::POP, self.line);
            self.get_cur_locals().pop();
        }
    }

    pub fn and_op(&mut self) {
        let index = self.get_cur_chunk().code.len();
        self.emit_byte(OpCode::IF_STMT_OFFSET(0), self.line);

        if self.parser.cur.token_type == TokenType::LEFT_BRACE {
            errors::error_message("COMPILING ERROR", format!("Expected to find expression after {} statement {}:",
                self.parser.prev.value.iter().collect::<String>().to_ascii_uppercase(),
                self.line,
            ));
            std::process::exit(1);
        };
        self.emit_byte(OpCode::POP, self.line);
        self.parse(Precedence::AND);

        let offset = (self.get_cur_chunk().code.len() - index) - 1;
        self.get_cur_chunk().code[index] = Instruction { op: OpCode::IF_STMT_OFFSET(offset), line: self.line };
    }

    pub fn or_op(&mut self) {
        let index = self.get_cur_chunk().code.len();

        self.emit_byte(OpCode::IF_STMT_OFFSET(0), self.line);

        let index_or = self.get_cur_chunk().code.len();
        self.emit_byte(OpCode::JUMP(0), self.line);

        if self.parser.cur.token_type == TokenType::LEFT_BRACE {
            errors::error_message("COMPILING ERROR", format!("Expected to find expression after {} statement {}:",
                self.parser.prev.value.iter().collect::<String>().to_ascii_uppercase(),
                self.line,
            ));
            std::process::exit(1);
        };
        let offset = (self.get_cur_chunk().code.len() - index) - 1;
        self.get_cur_chunk().code[index] = Instruction { op: OpCode::IF_STMT_OFFSET(offset), line: self.line };

        self.emit_byte(OpCode::POP, self.line);

        self.parse(Precedence::OR);

        let offset = (self.get_cur_chunk().code.len() - index_or) - 1;
        self.get_cur_chunk().code[index_or] = Instruction { op: OpCode::JUMP(offset), line: self.line };
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
            TokenType::KEYWORD(Keywords::STRUCT) => {
                self.parser.advance();
                self.struct_declare();
            },
            TokenType::KEYWORD(Keywords::IF) => {
                self.parser.advance();
                self.if_stmt();
            },
            TokenType::KEYWORD(Keywords::ELIF) => {
                if self.parser.prev.token_type != TokenType::RIGHT_BRACE {
                    error_message("COMPILER ERROR", format!("Expected to find }} before ELIF statment {}:", self.line));
                    std::process::exit(1);
                }
                self.parser.advance();
                self.if_stmt();
            },
            TokenType::KEYWORD(Keywords::ELSE) => {
                if self.parser.prev.token_type != TokenType::RIGHT_BRACE {
                    error_message("COMPILER ERROR", format!("Expected to find }} before ELSE statment {}:", self.line));
                    std::process::exit(1);
                }
                self.parser.advance();
                self.else_stmt();
            },
            TokenType::KEYWORD(Keywords::WHILE) => {
                self.parser.advance();
                self.while_stmt();
            },
            TokenType::KEYWORD(Keywords::FOR) => {
                self.parser.advance();
                self.for_stmt();
            },
            TokenType::KEYWORD(Keywords::BREAK) => {
                self.parser.advance();

                if self.scope_depth <= 1 {
                    errors::error_message("COMPILING ERROR", format!("BREAK statment used out of loop {}:",
                        self.line,
                    ));
                    std::process::exit(1);
                };

                self.emit_byte(OpCode::BREAK, self.line);

                let offset = (self.get_cur_chunk().code.len() - self.loop_info.start) + 1;
                self.emit_byte(OpCode::LOOP(offset), self.line);
            },
            TokenType::KEYWORD(Keywords::CONTINUE) => {
                self.parser.advance();

                if self.scope_depth <= 1 {
                    errors::error_message("COMPILING ERROR", format!("CONTINUE statment used out of loop {}:",
                        self.line,
                    ));
                    std::process::exit(1);
                };

                self.emit_byte(OpCode::VAR_CALL(self.loop_info.locals_start - 3), self.line);

                self.emit_byte(OpCode::VAR_CALL(self.loop_info.locals_start - 1), self.line);
        
                self.emit_byte(OpCode::ADD_INT, self.line);
        
                self.emit_byte(OpCode::VAR_SET(self.loop_info.locals_start - 3), self.line);

                let offset = (self.get_cur_chunk().code.len() - self.loop_info.start) + 1;
                self.emit_byte(OpCode::LOOP(offset), self.line);
            },
            _ => {
                self.expression();
                self.emit_byte(OpCode::POP, self.line);
            },
        }
    }

    pub fn compile(&mut self) -> Chunk {
        // more native types
        let string_type = StringObj::init();
        self.get_cur_chunk().push(Instruction { op: OpCode::STRUCT_DEC(string_type.clone()), line: 0 });
        self.structs.insert("String".to_string(), string_type.clone());

        self.parser.advance();
        loop {
            self.line = self.parser.cur.line;
            if self.parser.check_if_eof() {
                break;
            }
            self.compile_line();
            self.loop_info = LoopInfo::new();
        }
        // Dunno if that help with memory
        self.structs = HashMap::new();

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