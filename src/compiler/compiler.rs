use std::collections::HashMap;

use crate::{
    frontend, objects::{functions::{Function, Local, NativeFn, SpecialType}, lists::ListObj, string::StringObj, structs::{Struct, StructInstance}}, vm::{bytecode::{Chunk, Instruction, OpCode}, value::{Convert, Value}
}};
use crate::frontend::tokens::{Token, TokenType, Keywords};

use super::errors::{self, error_message};

pub struct LoopInfo {
    pub loop_type: TokenType,
    pub start: usize,
    pub locals_start: usize,
    pub end_stmt_pos: usize,
}

impl LoopInfo {
    pub fn new() -> Self {
        LoopInfo {
            loop_type: TokenType::NULL,
            start: 0,
            locals_start: 0,
            end_stmt_pos: 0,
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
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
        (TokenType::KEYWORD(Keywords::CONTINUE), ParseRule { prefix: None, infix: None, prec: Precedence::NONE }),

        (TokenType::STRING, ParseRule { prefix: Some(Compiler::string_dec), infix: None, prec: Precedence::NONE }),

        (TokenType::RIGHT_BRACE, ParseRule { prefix: None, infix: None, prec: Precedence::NONE }),
        (TokenType::LEFT_BRACE, ParseRule { prefix: None, infix: None, prec: Precedence::NONE }),

        (TokenType::RIGHT_BRACKET, ParseRule { prefix: None, infix: None, prec: Precedence::NONE }),
        (TokenType::LEFT_BRACKET, ParseRule { prefix: None, infix: Some(Compiler::list_call), prec: Precedence::CALL }),

        (TokenType::COMMA, ParseRule { prefix: None, infix: None, prec: Precedence::NONE }),
        (TokenType::DOT, ParseRule { prefix: None, infix: Some(Compiler::instance_call), prec: Precedence::CALL }),

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
                errors::conversion_error("shlang/compiler.rs".to_string(), "u32", "Precedence");
                std::process::exit(1);
            }
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    cur: Token,
    prev: Token,
    line: u32,
    index: usize,
    rules: HashMap<TokenType, ParseRule>,
    symbols: Vec<Symbol>,
    base_dir: String,
    imported_files: HashMap<String, Vec<Token>>,
    cur_file: String,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, base_dir: String, imported_files: HashMap<String, Vec<Token>>, cur_file: String) -> Self {       
        Self {
            tokens,
            cur: Token { token_type: TokenType::ERROR, value: vec![], line: 0},
            prev: Token { token_type: TokenType::ERROR, value: vec![], line: 0},
            line: 0,
            index: 0,
            rules: init_rules(),
            symbols: vec![],
            base_dir,
            imported_files,
            cur_file,
        }
    }
    
    pub fn advance(&mut self) {
        self.prev = self.cur.clone();
        self.cur = self.tokens[self.index].clone();
        self.line = self.prev.line;
        self.index += 1;

        if self.cur.token_type == TokenType::ERROR {
            errors::token_error(self.cur_file.clone(), self.cur.clone());
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
            errors::error_message(self.cur_file.clone(), "PARSER ERROR", format!("Expected to find a {:?}, but found: {:?} {}:", token_type, self.cur.token_type, self.line));
            std::process::exit(1);
        }
        self.advance();
    }

    pub fn list_type_value(symbols: &Vec<Symbol>, token: Token, obj_name: String, line: u32, cur_file: String) -> Keywords {
        let list_type = match token.token_type {
            TokenType::KEYWORD(key) => key,
            TokenType::IDENTIFIER => {
                let struct_name = token.value.iter().collect::<String>();

                let pos = symbols
                    .iter()
                    .enumerate()
                    .find(|(_, name)| *name.name == struct_name && name.symbol_type == TokenType::KEYWORD(Keywords::STRUCT))
                    .map(|(index, _)| index as i32)
                    .unwrap_or(-1);
                 
                if pos == -1 {
                    errors::error_message(cur_file, "COMPILER ERROR",
                    format!("Symbol: \"{}\" is not defined as struct in this scope, failed to create a function with that output type {}:", obj_name, line));
                    std::process::exit(1);
                }

                Keywords::INSTANCE(pos as usize)
            },
            t => {
                errors::error_message(cur_file, "COMPILER ERROR",
                    format!("Unexpected list type {:?} {}:", t, line));
                std::process::exit(1);
            }
        };

        list_type
    }

    pub fn get_token_type(val: Token, iter: &mut std::slice::IterMut<Token>, symbols: &Vec<Symbol>, line: u32, cur_file: String) -> TokenType { 
        match val.token_type {
            TokenType::KEYWORD(Keywords::INT) => TokenType::INT,
            TokenType::KEYWORD(Keywords::FLOAT) => TokenType::FLOAT,
            TokenType::KEYWORD(Keywords::BOOL) => TokenType::BOOL,
            TokenType::KEYWORD(Keywords::STRING) => TokenType::STRING,
            TokenType::IDENTIFIER => {
                let obj_name = val.value.iter().collect::<String>();

                if obj_name == "List" {
                    iter.next();

                    let token = if let Some(token) = iter.next() { token }
                    else {
                        errors::error_message(cur_file, "COMPILER ERROR",
                            format!("Expected to find a list type after \"<\" {}:", line));
                        std::process::exit(1);
                    };

                    let list_type = Self::list_type_value(&symbols, token.clone(), obj_name, line.clone(), cur_file.clone()).clone();
                    iter.next();                                        
                    TokenType::LIST(list_type)
                } else {
                    let pos = symbols
                        .iter()
                        .enumerate()
                        .find(|(_, name)| *name.name == obj_name && name.symbol_type == TokenType::KEYWORD(Keywords::STRUCT))
                        .map(|(index, _)| index as i32)
                        .unwrap_or(-1);

                    if pos == -1 {
                        errors::error_message(cur_file, "COMPILER ERROR",
                        format!("Symbol: \"{}\" is not defined as struct in this scope, failed to create a function with that output type {}:", obj_name, line));
                        std::process::exit(1);
                    }

                    TokenType::STRUCT(pos as usize)
                }
            }
            _ => TokenType::NULL,
        }                        
    }

    pub fn get_builtin_symbols(&mut self, string_mths_offset: usize) {
        self.symbols = NativeFn::get_natives_symbols();
                
        self.symbols.push(Symbol { name: "String".to_string(), symbol_type: TokenType::KEYWORD(Keywords::STRUCT), output_type: TokenType::STRING, arg_count: 1 });

        for _ in 0..string_mths_offset { 
            self.symbols.push(Symbol { name: String::new(), symbol_type: TokenType::NATIVE_FN, output_type: TokenType::KEYWORD(Keywords::NULL), arg_count: 1 });
        }

        self.symbols.push(Symbol { name: "List".to_string(), symbol_type: TokenType::KEYWORD(Keywords::STRUCT), output_type: TokenType::INT, arg_count: 0 });
    }

    pub fn get_symbols_and_functions(&mut self, check_for_main: bool, main_file: String) -> HashMap<String, Function> {
        let mut functions: HashMap<String, Function> = HashMap::new();

        let mut is_main_fn_found = false;

        let mut iter = self.tokens.iter_mut();
        'l: while let Some(token) = iter.next()  {
            if token.token_type == TokenType::KEYWORD(Keywords::IMPORT) {
                let file_path_token = iter.next().unwrap();

                if file_path_token.token_type != TokenType::STRING {
                    error_message(self.cur_file.clone(), "COMPILER ERROR", format!("Expected to find path to file after \"import\" keyword {}:", token.line));
                    std::process::exit(1);
                }
                
                let mut file_path = file_path_token.value.iter().collect::<String>();
                file_path = self.base_dir.clone() + &"/" + &file_path;

                if file_path == self.base_dir.clone() + &"/" + &main_file {    
                    error_message(self.cur_file.clone(), "COMPILER ERROR", format!("Cannot import main file passed as argument to interpreter {}:", token.line));
                    std::process::exit(1);
                }

                if file_path == self.cur_file {
                    error_message(file_path, "COMPILER ERROR", format!("Cannot import itself {}:", token.line));
                    std::process::exit(1);
                }
                
                if self.imported_files.contains_key(&file_path) {
                    continue
                }
                let (source_code, _) = frontend::lexer::get_file(&file_path);

                let mut scanner = frontend::lexer::Scanner::init(&source_code);
                let tokens = scanner.get_tokens();

                self.imported_files.insert(file_path.clone(), tokens.clone());
                let mut new_parser = Parser::new(tokens, self.base_dir.clone(), self.imported_files.clone(), file_path);

                new_parser.symbols = self.symbols.to_vec();

                functions.extend(new_parser.get_symbols_and_functions(false, main_file.clone()));

                self.symbols = new_parser.symbols;

                self.imported_files = new_parser.imported_files;
            }
            
            if token.token_type == TokenType::KEYWORD(Keywords::FN) {               
                let fn_name = match iter.next() {
                    Some(val) => {
                        if val.token_type == TokenType::EOF { break 'l };
                        val.value.iter().collect::<String>()
                    },
                    None => break 'l,
                };
                let mut function = Function::new(fn_name.clone());

                if self.symbols.iter().any(| symbol | symbol.name == fn_name) {
                    errors::error_message(self.cur_file.clone(), "COMPILER ERROR", format!("Function: \"{}\" is already defined {}:", fn_name, token.line));
                    std::process::exit(1);
                }

                if fn_name == "main".to_ascii_lowercase() {
                    is_main_fn_found = true;
                }

                let mut arg_count = 0;
                'args: while let Some(tok) = iter.next() {
                    match tok.token_type {
                        TokenType::COLON => {
                            if let Some(val) = iter.next() {
                                let arg_type = Parser::get_token_type(val.clone(), &mut iter, &self.symbols, self.line, self.cur_file.clone());
                                function.arg_type.push(arg_type);
                            }
                            
                            arg_count += 1;
                        },
                        TokenType::RIGHT_PAREN | TokenType::EOF => break 'args,
                        _ => {},
                    }
                }

                let out_type = match iter.next() {
                    Some(val) => {
                        if val.token_type == TokenType::EOF { break 'l };
                        Parser::get_token_type(val.clone(), &mut iter, &self.symbols, self.line, self.cur_file.clone())
                    },
                    None => break 'l,
                };

                functions.insert(fn_name.clone(), function);
                self.symbols.push(Symbol{name: fn_name, symbol_type: TokenType::KEYWORD(Keywords::FN), output_type: out_type, arg_count });
            }

            if token.token_type == TokenType::KEYWORD(Keywords::STRUCT) {
                let struct_name = match iter.next() {
                    Some(val) => {
                        if val.token_type == TokenType::EOF { break 'l };
                        val.value.iter().collect::<String>()
                    },
                    None => break 'l,
                };

                if self.symbols.iter().any(| symbol | symbol.name == struct_name) {
                    errors::error_message(self.cur_file.clone(), "COMPILER ERROR", format!("Struct: \"{}\" is already defined {}:", struct_name, token.line));
                    std::process::exit(1);
                }

                self.symbols.push(Symbol{name: struct_name, symbol_type: TokenType::KEYWORD(Keywords::STRUCT), output_type: TokenType::KEYWORD(Keywords::NULL), arg_count: 0 });
            }
        }

        if check_for_main && !is_main_fn_found {
            errors::error_message(self.cur_file.clone(), "COMPILE ERROR", format!("Cannot find \"main\" function"));
            std::process::exit(1);
        }

        if !check_for_main && is_main_fn_found {
                errors::error_message(self.cur_file.clone(), "COMPILE ERROR", format!("Function \"main\" should be found in file passed as argument to intepreter"));
            std::process::exit(1);
        }
        functions
    }

    pub fn get_rule(&self, token_type: &TokenType) -> &ParseRule {
        &self.rules[token_type]
    }
}

pub struct Compiler {
    pub parser: Parser,
    cur_function: Function,
    functions: HashMap<String, Function>,
    scope_depth: u32,
    symbol_to_hold: usize,
    loop_info: LoopInfo,
    structs: HashMap<String, Struct>,
    changing_fn: bool,
    main_file: String,
    cur_filepath: String,
    compiled_files: HashMap<String, i32>,
}

impl Compiler {
    pub fn new(tokens: Vec<Token>, base_dir: String, cur_filepath: String, main_file: String) -> Self {
        Self {
            parser: Parser::new(tokens, base_dir, HashMap::new(), cur_filepath.clone()),
            cur_function: Function::new(String::new()),
            functions: HashMap::new(),
            scope_depth: 0,
            symbol_to_hold: 0,
            loop_info: LoopInfo::new(),
            structs: HashMap::new(),
            changing_fn: false,
            main_file,
            cur_filepath,
            compiled_files: HashMap::new(),
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
            TokenType::MINUS => self.emit_byte(OpCode::NEGATE, self.parser.line),
            TokenType::INTERJ => self.emit_byte(OpCode::NEGATE, self.parser.line),
            _ => {
                errors::error_unexpected(self.cur_filepath.clone(), self.parser.prev.clone(), "negation function");
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

        self.get_cur_chunk().push_value(Value::Bool(false));

        match constants_type {
            TokenType::INT => {
                match logic_token.token_type {
                    TokenType::EQ_EQ => self.emit_byte(OpCode::EQ_INT, self.parser.line),
                    TokenType::INTERJ_EQ => self.emit_byte(OpCode::NEG_EQ_INT, self.parser.line),
                    TokenType::GREATER => self.emit_byte(OpCode::GREATER_INT, self.parser.line),
                    TokenType::GREATER_EQ => self.emit_byte(OpCode::EQ_GREATER_INT, self.parser.line),
                    TokenType::LESS => self.emit_byte(OpCode::LESS_INT, self.parser.line),
                    TokenType::LESS_EQ => self.emit_byte(OpCode::EQ_LESS_INT, self.parser.line),
                    _ => {
                        errors::error_unexpected(self.cur_filepath.clone(), logic_token, "logic operator function");
                        std::process::exit(1);
                    }
                };
            },
            TokenType::FLOAT => {
                match logic_token.token_type {
                    TokenType::EQ_EQ => self.emit_byte(OpCode::EQ_FLOAT, self.parser.line),
                    TokenType::INTERJ_EQ => self.emit_byte(OpCode::NEG_EQ_FLOAT, self.parser.line),
                    TokenType::GREATER => self.emit_byte(OpCode::GREATER_FLOAT, self.parser.line),
                    TokenType::GREATER_EQ => self.emit_byte(OpCode::EQ_GREATER_FLOAT, self.parser.line),
                    TokenType::LESS => self.emit_byte(OpCode::LESS_FLOAT, self.parser.line),
                    TokenType::LESS_EQ => self.emit_byte(OpCode::EQ_LESS_FLOAT, self.parser.line),
                    _ => {
                        errors::error_unexpected(self.cur_filepath.clone(), logic_token, "logic operator function");
                        std::process::exit(1);
                    }
                };
            },
            TokenType::BOOL => {
                match logic_token.token_type {
                    TokenType::EQ_EQ => self.emit_byte(OpCode::EQ_BOOL, self.parser.line),
                    TokenType::INTERJ_EQ => self.emit_byte(OpCode::NEG_EQ_BOOL, self.parser.line),
                    _ => {
                        errors::error_unexpected(self.cur_filepath.clone(), logic_token, "logic operator function");
                        std::process::exit(1);
                    }
                };
            },
            TokenType::STRING => {
                match logic_token.token_type {
                    TokenType::EQ_EQ => self.emit_byte(OpCode::EQ_STRING, self.parser.line),
                    TokenType::INTERJ_EQ => self.emit_byte(OpCode::NEG_EQ_STRING, self.parser.line),
                    _ => {
                        errors::error_unexpected(self.cur_filepath.clone(), logic_token, "logic operator function");
                        std::process::exit(1);
                    }
                };
            }
            _ => {
                errors::error_unexpected_token_type(self.cur_filepath.clone(), constants_type, self.parser.line, "logic operator function");
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

                        self.emit_byte(OpCode::CONSTANT_BOOL(pos), self.parser.line);
                    },
                    Keywords::FALSE => {
                        let pos = self.get_cur_chunk().push_value(Value::Bool(false));

                        self.emit_byte(OpCode::CONSTANT_BOOL(pos), self.parser.line);
                    },
                    Keywords::NULL => {
                        let pos = self.get_cur_chunk().push_value(Value::Null);

                        self.emit_byte(OpCode::CONSTANT_NULL(pos), self.parser.line);
                    },
                    _ => {
                        errors::error_unexpected_keyword(self.cur_filepath.clone(), val, self.parser.line, "bool function");
                        std::process::exit(1);
                    }
                }
            }
            _ => {
                errors::error_unexpected(self.cur_filepath.clone(), self.parser.prev.clone(), "bool function");
                std::process::exit(1);
            }
        };
    }

    pub fn number(&mut self) {
        match self.parser.prev.token_type {
            TokenType::INT => {
                let value: i128 = match self.parser.prev.value.iter().collect::<String>().parse() {
                    Ok(v) => v,
                    Err(_) => {
                        errors::conversion_error(self.cur_filepath.clone(), "Vec<char>", "i128");
                        std::process::exit(1);
                    },
                };

                let pos = self.get_cur_chunk().push_value(Value::Int(value));

                self.emit_byte(OpCode::CONSTANT_INT(pos), self.parser.line);
            }
            TokenType::FLOAT => {
                let value: f64 = match self.parser.prev.value.iter().collect::<String>().parse() {
                    Ok(v) => v,
                    Err(_) => {
                        errors::conversion_error(self.cur_filepath.clone(), "Vec<char>", "f64");
                        std::process::exit(1);
                    },
                };

                let pos = self.get_cur_chunk().push_value(Value::Float(value));

                self.emit_byte(OpCode::CONSTANT_FLOAT(pos), self.parser.line);
            }
            _ => {
                errors::error_unexpected(self.cur_filepath.clone(), self.parser.prev.clone(), "number function");
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
                    TokenType::PLUS => self.emit_byte(OpCode::ADD_INT, self.parser.line),
                    TokenType::MINUS => self.emit_byte(OpCode::SUB_INT, self.parser.line),
                    TokenType::STAR => self.emit_byte(OpCode::MUL_INT, self.parser.line),
                    TokenType::SLASH => self.emit_byte(OpCode::DIV_INT, self.parser.line),
                    TokenType::MOD => self.emit_byte(OpCode::MOD_INT, self.parser.line),
                    _ => {
                        errors::error_unexpected(self.cur_filepath.clone(), arithmetic_token, "arithmetic function");
                        std::process::exit(1);
                    }
                };
            },
            TokenType::FLOAT => {
                match arithmetic_token.token_type {
                    TokenType::PLUS => self.emit_byte(OpCode::ADD_FLOAT, self.parser.line),
                    TokenType::MINUS => self.emit_byte(OpCode::SUB_FLOAT, self.parser.line),
                    TokenType::STAR => self.emit_byte(OpCode::MUL_FLOAT, self.parser.line),
                    TokenType::SLASH => self.emit_byte(OpCode::DIV_FLOAT, self.parser.line),
                    TokenType::MOD => self.emit_byte(OpCode::MOD_FLOAT, self.parser.line),
                    _ => {
                        errors::error_unexpected(self.cur_filepath.clone(), arithmetic_token, "arithmetic function");
                        std::process::exit(1);
                    }
                };
            },
            TokenType::STRING => {
                match arithmetic_token.token_type {
                    TokenType::PLUS => self.emit_byte(OpCode::ADD_STRING, self.parser.line),
                    _ => {
                        errors::error_unexpected(self.cur_filepath.clone(), arithmetic_token, "arithmetic function");
                        std::process::exit(1);
                    }        
                };
            },
            _ => {
                errors::error_unexpected_token_type(self.cur_filepath.clone(), constants_type, self.parser.line, "arithmetic function");
                std::process::exit(1);
            }
        };
    }

    pub fn check_static_types(&self, a_token_type: &TokenType, b_type: TokenType, op: &Token) -> TokenType {
        if !self.check_num_types(a_token_type.clone(), b_type) {
            errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Mismatched types: {:?} {} {:?} {}:",
                b_type,
                op.value.iter().collect::<String>(),
                a_token_type,
                self.parser.line,
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
 
    pub fn string_dec(&mut self) {
        let mut instance_obj = StructInstance::new();

        let value = self.parser.prev.value.iter().collect::<String>();
        instance_obj.fields_values.push(Value::String(value.clone()));
        
        self.emit_byte(OpCode::STRING_DEC(instance_obj), self.parser.line);

        self.get_cur_chunk().push_value(Value::String(String::new()));
    }

    pub fn list_dec(&mut self, name: String) {
        let list_type_token = self.parser.cur.clone();

        let list_type = match self.parser.cur.token_type {
            TokenType::KEYWORD(keyword) => keyword.convert(),
            TokenType::IDENTIFIER => {
                let struct_name = self.parser.cur.value.iter().collect::<String>();
                let struct_pos = self.get_struct_symbol_pos(struct_name);
                
                TokenType::STRUCT(struct_pos)                
            }, 
            list_type => list_type,
        };
        self.parser.advance();

        self.parser.consume(TokenType::GREATER);
        self.parser.consume(TokenType::EQ);

        let var_pos = self.get_struct_symbol_pos("List".to_string());

        if self.parser.cur.token_type == TokenType::IDENTIFIER {
            let value = self.parser.cur.value.iter().collect::<String>();

            let pos = self.parser.symbols
                .iter()
                .enumerate()
                .find(|(_, name)| *name.name == value && name.symbol_type != TokenType::KEYWORD(Keywords::STRUCT))
                .map(|(index, _)| index as i32)
                .unwrap_or(-1);

            if pos != -1 {
                self.parser.consume(TokenType::IDENTIFIER);
                let output_type = match self.parser.symbols[pos as usize].output_type {
                    TokenType::LIST(t) => {
                        t.convert()
                    },
                    _ => {
                        println!("CHECK THIS TYPE OF ERRORS line 624 in compiler.rs {:?}", self.parser.symbols[pos as usize]);
                        std::process::exit(1);                            
                    }
                };

                self.symbol_to_hold = pos as usize;
                self.parser.consume(TokenType::LEFT_PAREN);

                if output_type != list_type {
                    errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Mismatched types while assigning list instance, expected: {:?} found: {:?} {}:",
                        list_type,
                        output_type,
                        self.parser.line,
                    ));
                    std::process::exit(1);
                }
                
                self.fn_call();

                let list_type_value = self.get_list_type_value(list_type_token);
                self.get_cur_locals().push(Local{ name, local_type: TokenType::KEYWORD(Keywords::INSTANCE(var_pos)), is_special: SpecialType::List(list_type_value) });                

                return
            }

            self.expression();

            let value_type = match self.get_cur_chunk().get_last_value() {
                Value::List(val) => val,
                _ => {
                        println!("CHECK THIS TYPE OF ERRORS line 663 in compiler.rs {:?}", self.get_cur_chunk().get_last_value());
                        std::process::exit(1);
                }
            };

            if list_type != value_type {
                errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Mismatched types while declaring list, expected: \"{:?}\" found: {:?} {}:",
                    list_type,
                    value_type,
                    self.parser.line,
                ));
                std::process::exit(1);
            }

            let list_type_value = self.get_list_type_value(list_type_token);

            self.get_cur_locals().push(Local{ name: name.clone(), local_type: TokenType::KEYWORD(Keywords::INSTANCE(var_pos)), is_special: SpecialType::List(list_type_value) });

            return
        }

        let list_obj = StructInstance::new();

        let mut field_count = 0;

        self.parser.consume(TokenType::LEFT_BRACKET);        
        while self.parser.cur.token_type != TokenType::RIGHT_BRACKET {
            self.expression();

            if self.get_cur_chunk().get_last_value().convert() != list_type {
                let value_type = self.get_cur_chunk().get_last_value().convert();

                let list_type_error = match list_type {
                    TokenType::STRUCT(pos) => {
                        format!("STRUCT: {}", self.parser.symbols[pos].name.clone())  
                    },
                    val => val.to_string(),
                };

                errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR",
                format!("Expected to find {} but found {:?} {}:", 
                    list_type_error, 
                    value_type,
                    self.parser.line
                ));
                std::process::exit(1);
            }
            
            if self.parser.cur.token_type == TokenType::COMMA {
                self.parser.consume(TokenType::COMMA);
            }
            
            field_count += 1;
        }
        self.parser.consume(TokenType::RIGHT_BRACKET);       
        
        self.emit_byte(OpCode::INSTANCE_DEC(list_obj, field_count), self.parser.line);

        let list_type_value = self.get_list_type_value(list_type_token);

        self.get_cur_locals().push(Local{ name, local_type: TokenType::KEYWORD(Keywords::INSTANCE(var_pos)), is_special: SpecialType::List(list_type_value) });
    }
    
    pub fn identifier(&mut self) {
        if self.parser.cur.token_type == TokenType::EQ {
            self.var_assign();
            return
        }

        if self.parser.cur.token_type == TokenType::DOT {
            let pos = self.get_local_pos(self.parser.prev.value.iter().collect::<String>());

            let string_pos = self.get_struct_symbol_pos("String".to_string());
            let list_pos = self.get_struct_symbol_pos("List".to_string());
            match self.get_cur_locals()[pos].local_type {
                TokenType::KEYWORD(Keywords::INSTANCE(struct_pos)) => {
                    if string_pos == struct_pos {
                        self.get_cur_chunk().push_value(Value::String(String::new()));
                    } else if list_pos == struct_pos {
                        let list_type = match self.get_cur_locals()[pos].is_special.clone() {
                            SpecialType::List(v) => Value::List(v.convert()),
                            _ => {
                                panic!("This error should never happend; check if while declaring List the special type is set");
                            }
                        };

                        self.get_cur_chunk().push_value(list_type);
                    }
                    else {
                        self.get_cur_chunk().push_value(Value::InstanceRef(struct_pos));
                    }
                },
                _ => {                    
                    errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Cannot find root struct for instance {}:",
                        self.parser.line,
                    ));
                    std::process::exit(1);
                }
            }
                  
            self.emit_byte(OpCode::INC_RC(pos), self.parser.line);
            self.emit_byte(OpCode::GET_INSTANCE_RF(pos), self.parser.line);
            
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

        let pos = self.get_local_pos(var_name);

        if self.parser.cur.token_type == TokenType::STRING {
            self.expression();

            if !matches!(self.get_cur_chunk().get_last_value(), Value::String(_)) {
                errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Mismatched types while assigning var, expected: {:?} found: {:?} {}:",
                    TokenType::STRING,
                    self.get_cur_chunk().get_last_value().convert(),
                    self.parser.line,
                ));
                std::process::exit(1);
            }

            self.emit_byte(OpCode::DEC_RC(pos as usize), self.parser.line);
            self.emit_byte(OpCode::VAR_SET(pos as usize), self.parser.line);
            self.emit_byte(OpCode::POP, self.parser.line);

            let pos = self.get_cur_chunk().push_value(Value::Null);
            self.emit_byte(OpCode::CONSTANT_NULL(pos), self.parser.line);

            return;
        }else if let TokenType::KEYWORD(Keywords::INSTANCE(var_root_struct_pos)) = self.get_cur_locals()[pos].local_type {
            self.expression();

            let last_value = self.get_cur_chunk().get_last_value().convert();

            let string_pos = self.get_struct_symbol_pos("String".to_string());

            if last_value == TokenType::STRING && var_root_struct_pos != string_pos && TokenType::STRUCT(var_root_struct_pos) != last_value {
                let var_root_struct_name = self.parser.symbols[var_root_struct_pos].name.clone();

                let value_root_struct_name = match last_value {
                    TokenType::STRUCT(root_struct_pos) => {
                        self.parser.symbols[root_struct_pos].name.clone()
                    }
                    _ => panic!("Error never should be there"),
                };
                
                errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Mismatched types while assigning struct instance, expected: {:?} found: {:?} {}:",
                    var_root_struct_name,
                    value_root_struct_name,
                    self.parser.line,
                ));
                std::process::exit(1);
            }

            self.emit_byte(OpCode::DEC_RC(pos as usize), self.parser.line);

            self.emit_byte(OpCode::VAR_SET(pos as usize), self.parser.line);

            self.emit_byte(OpCode::POP, self.parser.line);
        
            let pos = self.get_cur_chunk().push_value(Value::Null);
            self.emit_byte(OpCode::CONSTANT_NULL(pos), self.parser.line);

            return;
        }
        self.expression();

        let value_type = self.get_cur_chunk().get_last_value().convert();
        let var_type = self.get_cur_locals()[pos as usize].local_type;
        if value_type != var_type {
            errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Mismatched types while assigning var, expected: {:?} found: {:?} {}:",
                var_type,
                value_type,
                self.parser.line,
            ));
            std::process::exit(1);
        }

        self.emit_byte(OpCode::VAR_SET(pos as usize), self.parser.line);
        self.emit_byte(OpCode::POP, self.parser.line);
        
        let pos = self.get_cur_chunk().push_value(Value::Null);
        self.emit_byte(OpCode::CONSTANT_NULL(pos), self.parser.line);
    }

    pub fn list_call(&mut self) {
        let list_type = self.get_cur_chunk().get_last_value();
       
        self.expression();

        if self.get_cur_chunk().get_last_value().convert() != TokenType::INT {
            let value_type = self.get_cur_chunk().get_last_value().convert();

            errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR",
                format!("Expected to find INT as index but found: {:?} {}:",  
                value_type,
                self.parser.line
            ));
            std::process::exit(1);
        }
    
        let list_type_token = match list_type.convert() {
            TokenType::LIST(val) => {
                val.convert()
            }
            _ => {
                errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR",
                    format!("Unable to convert list_type to TokenType {}:",  
                    self.parser.line
                ));
                std::process::exit(1);
            }
        };

        self.parser.consume(TokenType::RIGHT_BRACKET);

        if self.parser.cur.token_type == TokenType::EQ {
            self.parser.consume(TokenType::EQ);

            self.expression();

            if self.get_cur_chunk().get_last_value().convert() != list_type_token {
                let value_type = self.get_cur_chunk().get_last_value().convert();

                errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR",
                    format!("Expected to find {:?} but found: {:?} {}:", 
                    list_type_token, 
                    value_type,
                    self.parser.line
                ));
                std::process::exit(1);
            }
    
            self.emit_byte(OpCode::SET_LIST_FIELD, self.parser.line);

            let pos = self.get_cur_chunk().push_value(Value::Null);            
            self.emit_byte(OpCode::CONSTANT_NULL(pos), self.parser.line);
        
            return
        }
        
        self.emit_byte(OpCode::GET_LIST_FIELD, self.parser.line);
        match list_type_token {
                TokenType::INT => {
                    self.get_cur_chunk().push_value(Value::Int(0));
                },
                TokenType::FLOAT => {
                    self.get_cur_chunk().push_value(Value::Float(0.0));
                },
                TokenType::STRING => {
                    self.get_cur_chunk().push_value(Value::String(String::new()));
                },
                TokenType::LIST(val) => {
                    self.get_cur_chunk().push_value(Value::List(val.convert()));
                }
                TokenType::STRUCT(pos) => {
                    self.get_cur_chunk().push_value(Value::InstanceRef(pos));
                }
                TokenType::BOOL => {
                    self.get_cur_chunk().push_value(Value::Bool(true));
                },
                TokenType::NULL => {
                    self.get_cur_chunk().push_value(Value::Null);
                },
                _ => {},
        }
    }

    pub fn var_call(&mut self) {
        let var_name = self.parser.prev.value.iter().collect::<String>();

        let pos = self.get_local_pos(var_name);

        if self.get_cur_locals()[pos].is_special == SpecialType::String && !self.changing_fn {
            self.get_cur_chunk().push_value(Value::String(String::new()));

            self.emit_byte(OpCode::GET_INSTANCE_RF(pos as usize), self.parser.line);
            self.emit_byte(OpCode::INC_RC(pos as usize), self.parser.line);
        }else if matches!(self.get_cur_locals()[pos as usize].is_special, SpecialType::List(_)) && self.parser.cur.token_type == TokenType::LEFT_BRACKET {
            let list_type = match self.get_cur_locals()[pos as usize].is_special.clone() {
                SpecialType::List(val) => val,
                _ => {
                    errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR", format!("Unexpected special type while getting element: \"{:?}\" {}:",
                        self.get_cur_instances()[pos as usize].is_special.clone(),
                        self.parser.line,
                    ));
                    std::process::exit(1);
                }
            };

            self.get_cur_chunk().push_value(Value::List(list_type.convert()));

            self.emit_byte(OpCode::INC_RC(pos as usize), self.parser.line);
            self.emit_byte(OpCode::GET_INSTANCE_RF(pos as usize), self.parser.line);

            self.parser.consume(TokenType::LEFT_BRACKET);

            self.list_call();

            return
                        
        }  else if let TokenType::KEYWORD(Keywords::INSTANCE(struct_pos)) = self.get_cur_locals()[pos].local_type {
            let _ = match self.get_cur_locals()[pos].is_special.clone() {
                SpecialType::List(v) => self.get_cur_chunk().push_value(Value::List(v.convert())),
                SpecialType::String => self.get_cur_chunk().push_value(Value::String(String::new())),
                _ => self.get_cur_chunk().push_value(Value::InstanceRef(struct_pos)),
            };

            self.emit_byte(OpCode::INC_RC(pos as usize), self.parser.line);
            self.emit_byte(OpCode::GET_INSTANCE_RF(pos as usize), self.parser.line);

            return
        }

        if self.changing_fn && matches!(self.get_cur_locals()[pos].local_type, TokenType::KEYWORD(Keywords::INSTANCE(_))) {
            self.emit_byte(OpCode::INC_RC(pos as usize), self.parser.line);
        }
            
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
            TokenType::STRING => {
                self.get_cur_chunk().push_value(Value::String(String::new()));
            },
            TokenType::KEYWORD(Keywords::INSTANCE(_)) => return,
            local_type => {
                errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR", format!("Unexpected local type \"{:?}\" {}:", local_type, self.parser.line));
                std::process::exit(1);
            }
        };

        self.emit_byte(OpCode::VAR_CALL(pos as usize), self.parser.line);
    }

    pub fn var_declare(&mut self) {
        self.parser.consume(TokenType::IDENTIFIER);

        let var_name = self.parser.prev.value.iter().collect::<String>();
        if self.get_cur_locals().iter().any(| local | local.name == var_name ) {
            errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR", format!("Symbol: \"{}\" is already defined {}:", var_name, self.parser.line));
            std::process::exit(1);
        }

        self.parser.consume(TokenType::COLON);
        match self.parser.cur.token_type {
            TokenType::KEYWORD(Keywords::INT) |
            TokenType::KEYWORD(Keywords::FLOAT) |
            TokenType::KEYWORD(Keywords::BOOL) |
            TokenType::KEYWORD(Keywords::STRING) |
            TokenType::IDENTIFIER => {},
            _ => {
                errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR", format!("Expected var type after \":\" {}:", self.parser.line));
                std::process::exit(1);
            },
        };

        let var_type = match self.parser.cur.token_type {
            TokenType::IDENTIFIER | TokenType::KEYWORD(Keywords::STRING) => {
                let pos = self.get_struct_symbol_pos(self.parser.cur.value.iter().collect::<String>());

                TokenType::STRUCT(pos)
            }
            TokenType::KEYWORD(keyword) => keyword.convert(),
            _ => {
                errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR", format!("Expected var type after \":\" {}:", self.parser.line));
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
                errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Mismatched types while declaring var, expected: {:?} found: {:?} {}:",
                    var_type,
                    value_type,
                    self.parser.line,
                ));
                std::process::exit(1);
            }
        } else {
            let pos = self.get_cur_chunk().push_value(Value::Null);
            self.emit_byte(OpCode::CONSTANT_NULL(pos), self.parser.line);
        }
        
        self.get_cur_locals().push(Local { name: var_name, local_type: var_type, is_special: SpecialType::Null });
    }

    pub fn instance_call(&mut self) {
        let mut list_type = TokenType::NULL;
        let root_struct_pos = match self.get_cur_chunk().get_last_value() {
            Value::InstanceRef(struct_pos) => struct_pos,
            Value::String(_) => {
                self.get_struct_symbol_pos("String".to_string())
            },
            Value::List(token_type) => {
                list_type = token_type;
                self.get_struct_symbol_pos("List".to_string())
            },
            _ => {
                errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Type doesn't contains any fields: {:?} {}:",
                    self.get_cur_chunk().get_last_value(),
                    self.parser.line,
                ));
                std::process::exit(1);
            }
        };
        self.parser.consume(TokenType::IDENTIFIER);
        let field_name = self.parser.prev.value.iter().collect::<String>();

        let root_struct_name = self.parser.symbols[root_struct_pos].name.clone();
        if self.parser.cur.token_type == TokenType::LEFT_PAREN {
            match self.structs.get(&root_struct_name).unwrap().methods.get(&field_name) {
                Some(mth) => {
                    if matches!(list_type, TokenType::STRUCT(_)) || matches!(list_type, TokenType::STRING) && mth.name == "sort" {
                        errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Method: SORT is not declared for this type of list \"{}\" {}:",
                            list_type,
                            self.parser.line,
                        ));
                        std::process::exit(1);
                    }
                    
                    self.mth_call(mth.output_type, mth.arg_count, mth.arg_type.clone(), list_type);
                },
                None => {
                    errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Method: \"{}\" is not declared in struct \"{}\" {}:",
                        field_name,
                        root_struct_name,
                        self.parser.line,
                    ));
                    std::process::exit(1);
                },
            }
            
            match self.structs.get(&root_struct_name).unwrap().methods.get(&field_name) {
                Some(mth) => {
                    self.emit_byte(OpCode::METHOD_CALL(mth.clone()), self.parser.line);
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
            errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Field: \"{}\" is not declared in struct \"{}\" {}:",
                field_name,
                root_struct_name,
                self.parser.line,
            ));
            std::process::exit(1);
        }

        if self.parser.cur.token_type == TokenType::EQ {
            self.parser.consume(TokenType::EQ);

            self.expression();

            if self.get_cur_chunk().get_last_value().convert() != self.structs.get(&root_struct_name).unwrap().locals[field_index as usize].local_type {
                let value_type = self.get_cur_chunk().get_last_value().convert();

                errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR",
                format!("Expected to find {:?} but found: {:?} {}:", 
                    self.structs.get(&root_struct_name).unwrap().locals[field_index as usize].local_type, 
                    value_type,
                    self.parser.line
                ));
                std::process::exit(1);
            }

            self.emit_byte(OpCode::SET_INSTANCE_FIELD(field_index as usize), self.parser.line);

            let pos = self.get_cur_chunk().push_value(Value::Null);            
            self.emit_byte(OpCode::CONSTANT_NULL(pos), self.parser.line);
        }else{
            match self.structs.get(&root_struct_name).unwrap().locals[field_index as usize].local_type {
                TokenType::INT => {
                    self.get_cur_chunk().push_value(Value::Int(0));
                },
                TokenType::FLOAT => {
                    self.get_cur_chunk().push_value(Value::Float(0.0));
                },
                TokenType::STRING => {
                    self.get_cur_chunk().push_value(Value::String(String::new()));
                },
                TokenType::LIST(val) => {
                    self.get_cur_chunk().push_value(Value::List(val.convert()));
                }
                TokenType::STRUCT(pos) => {
                    self.get_cur_chunk().push_value(Value::InstanceRef(pos));
                }
                TokenType::BOOL => {
                    self.get_cur_chunk().push_value(Value::Bool(true));
                },
                TokenType::NULL => {
                    self.get_cur_chunk().push_value(Value::Null);
                },
                _ => {},
            }

            self.emit_byte(OpCode::GET_INSTANCE_FIELD(field_index as usize), self.parser.line);
        }
    }

    pub fn instance_declare(&mut self, var_pos: usize, name: String) {
        if self.parser.prev.value.iter().collect::<String>() == "List" {
            self.parser.consume(TokenType::LESS);
            self.list_dec(name);

            return
        }
        
        if self.parser.cur.token_type != TokenType::EQ {
            errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Struct cannot be left undeclared {}:",
                self.parser.line,
            ));
            std::process::exit(1);
        }
        self.parser.consume(TokenType::EQ);
        if self.parser.cur.token_type != TokenType::LEFT_BRACE {
            if self.parser.cur.token_type == TokenType::STRING {
                self.expression();
                
                if !matches!(self.get_cur_chunk().get_last_value(), Value::String(_)) {
                    errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Mismatched types while assigning var, expected: {:?} found: {:?} {}:",
                        TokenType::STRING,
                        self.get_cur_chunk().get_last_value().convert(),
                        self.parser.line,
                    ));
                    std::process::exit(1);
                }

                self.get_cur_locals().push(Local{ name, local_type: TokenType::KEYWORD(Keywords::INSTANCE(var_pos)), is_special: SpecialType::String });

                return
            }
            
            if self.parser.cur.token_type != TokenType::IDENTIFIER {
                errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Expected to find instance or function call {}:",
                    self.parser.line,
                ));
                std::process::exit(1);
            }            

            let value = self.parser.cur.value.iter().collect::<String>();

            let pos = self.parser.symbols
                .iter()
                .enumerate()
                .find(|(_, name)| *name.name == value && name.symbol_type != TokenType::KEYWORD(Keywords::STRUCT))
                .map(|(index, _)| index as i32)
                .unwrap_or(-1);

            if pos != -1 {
                self.parser.consume(TokenType::IDENTIFIER);
                let output_symbol_pos = match self.parser.symbols[pos as usize].output_type {
                    TokenType::STRUCT(root_pos) => root_pos,
                    TokenType::STRING => self.get_struct_symbol_pos("String".to_string()),
                    _ => {
                        println!("CHECK THIS TYPE OF ERRORS line 1117 in compiler.rs {:?}", self.parser.symbols[pos as usize]);
                        std::process::exit(1);                            
                    }
                };
                
                self.symbol_to_hold = pos as usize;
                self.parser.consume(TokenType::LEFT_PAREN);

                if output_symbol_pos != var_pos {
                    errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Mismatched types while assigning struct instance, expected: {:?} found: {:?} {}:",
                        self.parser.symbols[var_pos as usize].output_type,
                        self.parser.symbols[output_symbol_pos].name,
                        self.parser.line,
                    ));
                    std::process::exit(1);
                }
                
                self.fn_call();

                if value == "input" || self.parser.symbols[pos as usize].output_type == TokenType::STRING {
                    let pos = self.get_struct_symbol_pos("String".to_string());

                    self.get_cur_locals().push(Local{ name, local_type: TokenType::KEYWORD(Keywords::INSTANCE(pos)), is_special: SpecialType::String });

                    return
                }

                self.get_cur_locals().push(Local{ name, local_type: TokenType::KEYWORD(Keywords::INSTANCE(output_symbol_pos)), is_special: SpecialType::Null });                

                return
            }

            self.expression();

            let local_type = TokenType::KEYWORD(Keywords::INSTANCE(var_pos));

            let is_special = match &self.parser.symbols[var_pos].name as &str {
                "String" => SpecialType::String,
                _ => SpecialType::Null,
            };
        
            let value_pos = match self.get_cur_chunk().get_last_value().convert() {
                TokenType::STRING => {
                    self.get_struct_symbol_pos("String".to_string())
                },
                TokenType::STRUCT(pos) => pos,
                _ => {
                        println!("CHECK THIS TYPE OF ERRORS line 1181 in compiler.rs {:?}", self.get_cur_chunk().get_last_value());
                        std::process::exit(1);                            
                }
            };
            
            if var_pos != value_pos {
                errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Mismatched types while declaring instance, expected: {:?} found: {:?} {}:",
                    self.parser.symbols[var_pos].name.clone(),
                    self.get_cur_chunk().get_last_value().convert(),
                    self.parser.line,
                ));
                std::process::exit(1);
            }

           
            self.get_cur_locals().push(Local{ name: name.clone(), local_type, is_special });

            return
        }

        self.parser.consume(TokenType::LEFT_BRACE);
        let mut field_counts = 0;

        let root_struct_name = self.parser.symbols[var_pos].name.clone();
        while self.parser.cur.token_type != TokenType::RIGHT_BRACE {
            self.expression();

            if self.get_cur_chunk().get_last_value().convert() != self.structs.get(&root_struct_name).unwrap().locals[field_counts].local_type {
                let value_type = self.get_cur_chunk().get_last_value().convert();

                errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR",
                format!("Expected to find {:?} but found: {:?} {}:", 
                    self.structs.get(&root_struct_name).unwrap().locals[field_counts].local_type, 
                    value_type,
                    self.parser.line
                ));
                std::process::exit(1);

            }
          
            if self.parser.cur.token_type == TokenType::COMMA {
                self.parser.consume(TokenType::COMMA);
            }
            field_counts += 1;
        }
        self.parser.consume(TokenType::RIGHT_BRACE);

        let instance_obj = StructInstance::new();

        if field_counts != self.parser.symbols[var_pos].arg_count {
            errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR",
            format!("Expected to find {} fields but found: {} {}:", self.parser.symbols[var_pos].arg_count, field_counts, self.parser.line));
            std::process::exit(1);
        }
        self.emit_byte(OpCode::INSTANCE_DEC(instance_obj, field_counts), self.parser.line);

        self.get_cur_locals().push(Local{ name, local_type: TokenType::KEYWORD(Keywords::INSTANCE(var_pos)), is_special: SpecialType::Null });
    }

    pub fn struct_declare(&mut self) {
        self.parser.consume(TokenType::IDENTIFIER);

        let name = self.parser.prev.value.iter().collect::<String>();

        if self.scope_depth != 0 {
            errors::error_message(self.cur_filepath.clone(), "COMPILE ERROR", format!("Struct \"{}\" declaration inside bounds {}:", name, self.parser.line));
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
                TokenType::IDENTIFIER => {
                    let struct_name = self.parser.cur.value.iter().collect::<String>();

                    if struct_name == "List" {
                        self.parser.consume(TokenType::IDENTIFIER);
                        self.parser.consume(TokenType::LESS);

                        let list_type = self::Parser::list_type_value(&self.parser.symbols, self.parser.cur.clone(), struct_name.clone(), self.parser.line, self.cur_filepath.clone());

                        self.parser.advance();

                        TokenType::LIST(list_type)
                    } else {                    
                        let pos = self.parser.symbols
                            .iter()
                            .enumerate()
                            .find(|(_, name)| *name.name == struct_name && name.symbol_type == TokenType::KEYWORD(Keywords::STRUCT))
                            .map(|(index, _)| index as i32)
                            .unwrap_or(-1);

                        if pos == -1 {
                            errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR", format!("Expected field type after \":\" {}:", self.parser.line));
                            std::process::exit(1);
                        }

                        TokenType::STRUCT(pos as usize)
                    }
                },
                _ => {
                    errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR", format!("Expected field type after \":\" {}:", self.parser.line));
                    std::process::exit(1);
                },
            };
            self.parser.advance();

            self.parser.consume(TokenType::COMMA);

            struct_obj.locals.push(Local { name: field_name, local_type: field_type, is_special: SpecialType::Null });
        }

        // need to do that, because methods will not be compiled otherwise
        self.structs.insert(name.clone(), struct_obj.clone());

        if self.parser.cur.token_type == TokenType::KEYWORD(Keywords::METHODS) {
            self.parser.advance();
            self.mth_stmt(name.clone());
        }

        self.parser.consume(TokenType::RIGHT_BRACE);
        
        let locals_len = self.structs.get(&name.clone()).unwrap().locals.len();
        self.structs.get_mut(&(name.clone())).unwrap().field_count = locals_len; 

        let pos = self.get_struct_symbol_pos(name.clone());
        self.parser.symbols[pos].arg_count = locals_len;

        self.emit_byte(OpCode::STRUCT_DEC(self.structs.get(&name).unwrap().clone()), self.parser.line);
        
        self.scope_depth -= 1;
    }

    pub fn mth_call(&mut self, output_type: TokenType, mth_arg_count: usize, mth_arg_types: Vec<TokenType>, list_type: TokenType) {
        self.parser.consume(TokenType::LEFT_PAREN);

        let mut arg_count = 0;
        self.changing_fn = true;
        while self.parser.cur.token_type != TokenType::RIGHT_PAREN {
            arg_count += 1;
            
            self.expression();

            if mth_arg_types.len() > arg_count - 1 {
                let arg_type = if mth_arg_types[arg_count - 1] == TokenType::LIST_ELEMENT {
                    list_type
                } else {
                    mth_arg_types[arg_count - 1]
                };

                if arg_type != self.get_cur_chunk().get_last_value().convert() {
                    let mth_arg_error = match arg_type {
                        TokenType::STRUCT(pos) => {
                            format!("STRUCT: {}", self.parser.symbols[pos].name.clone())  
                        },
                        val => val.to_string(),
                    };

                    errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR",
                    format!("Expected to find {:?} as argument type but found: {:?} {}:", mth_arg_error, self.get_cur_chunk().get_last_value().convert(), self.parser.line));
                    std::process::exit(1);
                }
            }

            if self.parser.cur.token_type == TokenType::COMMA {
                self.parser.consume(TokenType::COMMA);
            }
        }
        self.parser.consume(TokenType::RIGHT_PAREN);
        self.changing_fn = false;

        if arg_count != mth_arg_count {
            errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR",
            format!("Expected to find {} arguments but found: {} {}:", mth_arg_count, arg_count, self.parser.line));
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
            TokenType::STRING => {
                self.get_cur_chunk().push_value(Value::String(String::new()));
            }
            TokenType::LIST_ELEMENT => {
                let list_type_value = match list_type {
                    TokenType::INT => Value::Int(0),
                    TokenType::FLOAT => Value::Float(0.0),
                    TokenType::STRING => Value::String(String::new()),
                    TokenType::BOOL =>  Value::Bool(false),
                    TokenType::STRUCT(val) => Value::InstanceRef(val),
                    _ => {
                        errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR",
                        format!("List of {:?} is not implemented yet {}:", 
                            list_type, 
                            self.parser.line
                        ));
                        std::process::exit(1);
                    }
                };

                self.get_cur_chunk().push_value(list_type_value);
            }
            output_type => {
                errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR", format!("Unexpected output type \"{:?}\" {}:", output_type, self.parser.line));
                std::process::exit(1);
            }
        };
    }

    pub fn mth_stmt(&mut self, struct_name: String) {
        self.parser.consume(TokenType::LEFT_BRACE);

        while self.parser.cur.token_type != TokenType::RIGHT_BRACE {
            let name = self.parser.cur.value.iter().collect::<String>();

            if self.structs.get(&struct_name).unwrap().methods.contains_key(&name) {
                errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR", format!("Method: \"{}\" is already defined for struct: \"{}\" {}:", name, struct_name, self.parser.line));
                std::process::exit(1);
            }

            let root_struct_pos = self.get_struct_symbol_pos(struct_name.clone());
            let mth = self.fn_declare(true, root_struct_pos);

            self.structs.get_mut(&struct_name.clone()).unwrap().methods.insert(name, mth);
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
            errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR",
            format!("Symbol: \"{}\" is not defined as function in this scope {}:", fn_name, self.parser.line));
            std::process::exit(1);
        }

        pos as usize
    }
    
    pub fn get_struct_symbol_pos(&self, struct_name: String) -> usize {
        let pos = self.parser.symbols
            .iter()
            .enumerate()
            .find(|(_, name)| *name.name == struct_name && name.symbol_type == TokenType::KEYWORD(Keywords::STRUCT))
            .map(|(index, _)| index as i32)
            .unwrap_or(-1);

        if pos == -1 {
            errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR",
            format!("Symbol: \"{}\" is not defined as struct in this scope {}:", struct_name, self.parser.line));
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
            errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR",
            format!("Symbol: \"{}\" is not defined as var in this scope {}:", name, self.parser.line));
            std::process::exit(1);
        }

        pos as usize
    }
    
    pub fn get_list_type_value(&self, list_token: Token) -> Value {
        let list_type_token = match list_token.token_type {
            TokenType::KEYWORD(keyword) => keyword.convert(),
            TokenType::IDENTIFIER => {
                let pos = self.get_struct_symbol_pos(list_token.value.iter().collect::<String>());

                TokenType::STRUCT(pos)
            },
            _ => list_token.token_type,
        };
   
        return match list_type_token {
            TokenType::INT => Value::Int(0),
            TokenType::FLOAT => Value::Float(0.0),
            TokenType::STRING => Value::String(String::new()),
            TokenType::BOOL =>  Value::Bool(false),
            TokenType::STRUCT(val) => Value::InstanceRef(val),
            _ => {
                errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR",
                format!("List of {:?} is not implemented yet {}:", 
                    list_token.token_type, 
                    self.parser.line
                ));
                std::process::exit(1);
            }
        };
    }

    pub fn fn_call(&mut self) {
        let mut arg_count: usize = 0;
        self.changing_fn = true;

        if self.parser.symbols[self.symbol_to_hold].symbol_type == TokenType::NATIVE_FN && !matches!(self.parser.symbols[self.symbol_to_hold].name.as_str(), "print" | "println" | "input")  {
            self.changing_fn = false;
        }

        let mut fn_arg_types: Vec<TokenType> = vec![];
        if self.changing_fn && !matches!(self.parser.symbols[self.symbol_to_hold].name.as_str(), "print" | "println" | "input") {
            let name = self.parser.symbols[self.symbol_to_hold].name.clone();
            fn_arg_types = self.functions[&name].arg_type.clone();
        }

        let symbol_to_hold_enclosing = self.symbol_to_hold;
        while self.parser.cur.token_type != TokenType::RIGHT_PAREN {
            arg_count += 1;

            self.expression();
            
            if fn_arg_types.len() > arg_count - 1 {
                if fn_arg_types[arg_count - 1] != self.get_cur_chunk().get_last_value().convert() {
                    let fn_arg_error = match fn_arg_types[arg_count - 1] {
                        TokenType::STRUCT(pos) => {
                            format!("STRUCT: {}", self.parser.symbols[pos].name.clone())  
                        },
                        val => val.to_string(),
                    };

                    errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR",
                    format!("Expected to find {:?} as argument type but found: {:?} {}:", fn_arg_error, self.get_cur_chunk().get_last_value().convert(), self.parser.line));
                    std::process::exit(1);
                }
            }
            
            if self.parser.cur.token_type == TokenType::COMMA {
                self.parser.consume(TokenType::COMMA);
            }
        }
        self.parser.consume(TokenType::RIGHT_PAREN);
        self.symbol_to_hold = symbol_to_hold_enclosing;

        self.changing_fn = false;
        if self.parser.symbols[self.symbol_to_hold].name == "print" || 
           self.parser.symbols[self.symbol_to_hold].name == "println" || 
           self.parser.symbols[self.symbol_to_hold].name == "input"
        {
            self.emit_byte(OpCode::IO_FN_CALL(self.symbol_to_hold, arg_count), self.parser.line);

            if self.parser.symbols[self.symbol_to_hold].name == "input" {
                self.get_cur_chunk().push_value(Value::String(String::new()));
            }
            
            return
        }

        if arg_count != self.parser.symbols[self.symbol_to_hold].arg_count {
            errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR",
            format!("Expected to find {} arguments but found: {} {}:", self.parser.symbols[self.symbol_to_hold].arg_count, arg_count, self.parser.line));
            std::process::exit(1);
        }

        if self.parser.symbols[self.symbol_to_hold].symbol_type == TokenType::NATIVE_FN {
            self.emit_byte(OpCode::NATIVE_FN_CALL(self.symbol_to_hold), self.parser.line);

            if self.parser.symbols[self.symbol_to_hold].name == "conv" {
                self.get_cur_chunk().push_value(Value::Int(0));
            }else if self.parser.symbols[self.symbol_to_hold].name == "convf" {
                self.get_cur_chunk().push_value(Value::Float(0.0));
            }else if self.parser.symbols[self.symbol_to_hold].name == "convstr" {
                self.get_cur_chunk().push_value(Value::String("".to_string()));
            }
        }else{
            self.emit_byte(OpCode::FUNCTION_CALL(self.symbol_to_hold), self.parser.line);
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
                TokenType::STRING => {
                    self.get_cur_chunk().push_value(Value::String(String::new()));
                },
                TokenType::STRUCT(val) => {
                    self.get_cur_chunk().push_value(Value::InstanceRef(val));  
                },
                TokenType::LIST(keyword) => {
                    let out = match keyword {
                        Keywords::INT => TokenType::INT,
                        Keywords::FLOAT => TokenType::FLOAT,
                        Keywords::BOOL => TokenType::BOOL,
                        Keywords::STRING => TokenType::STRING,
                        Keywords::INSTANCE(pos) => TokenType::STRUCT(pos),
                        t => {
                            errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR", format!("Unexpected output type \"{:?}\" {}:", t, self.parser.line));
                            std::process::exit(1);
                        }
                    };

                    self.get_cur_chunk().push_value(Value::List(out));                                       
                },
                output_type => {
                    errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR", format!("Unexpected output type \"{:?}\" {}:", output_type, self.parser.line));
                    std::process::exit(1);
                }
            };
        }
    }

    pub fn fn_declare(&mut self, is_mth: bool, root_struct_pos: usize) -> Function {
        let name = self.parser.cur.value.iter().collect::<String>();

        if (self.scope_depth != 0 && !is_mth) || (self.scope_depth == 0 && is_mth) {
            errors::error_message(self.cur_filepath.clone(), "COMPILE ERROR", format!("Function/Method \"{}\" declaration inside bounds {}:", name, self.parser.line));
            std::process::exit(1)
        }

        let mut function = Function::new(name.clone());
        if !is_mth {
            function = self.functions[&name.clone()].clone();
        }
        

        self.parser.advance();

        self.parser.consume(TokenType::LEFT_PAREN);

        while self.parser.cur.token_type != TokenType::RIGHT_PAREN {
            function.arg_count += 1;

            self.parser.consume(TokenType::IDENTIFIER);
            let arg_name = self.parser.prev.value.iter().collect::<String>();

            if arg_name == "self" && is_mth {
                if function.arg_count != 1 {
                    errors::error_message(self.cur_filepath.clone(), "COMPILE ERROR", format!("\"self\" keyword need to be first in argument list {}:", self.parser.line));
                    std::process::exit(1)
                }

                function.is_self_arg =  true;
                function.arg_count -= 1;

                if self.parser.cur.token_type == TokenType::COMMA {
                    self.parser.consume(TokenType::COMMA);
                }

                function.locals.push(Local { name: "self".to_string(), local_type: TokenType::KEYWORD(Keywords::INSTANCE(root_struct_pos)), is_special: SpecialType::Null });

                continue;
            }

            self.parser.consume(TokenType::COLON);
            let arg_type = match self.parser.cur.token_type {
                TokenType::IDENTIFIER | TokenType::KEYWORD(Keywords::STRING) => {
                    let value = self.parser.cur.value.iter().collect::<String>();
                    let pos = self.get_struct_symbol_pos(value);

                    TokenType::KEYWORD(Keywords::INSTANCE(pos))
                }
                TokenType::KEYWORD(keyword) => keyword.convert(),
                _ => {
                    errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR", format!("Expected arg type after \":\" {}:", self.parser.line));
                    std::process::exit(1);
                }
            };
            self.parser.advance();

            match arg_type {
                TokenType::KEYWORD(Keywords::INSTANCE(pos)) => {
                    if self.parser.symbols[pos].name == "String" {
                        if is_mth {                        
                            function.arg_type.push(TokenType::STRING);
                        }
                        
                        function.locals.push(Local { name: arg_name, local_type: arg_type, is_special: SpecialType::String });
                    }else if self.parser.symbols[pos].name == "List" {
                        self.parser.consume(TokenType::LESS);
                        
                        let list_type_value = self.get_list_type_value(self.parser.cur.clone());
                        self.parser.advance();

                        self.parser.consume(TokenType::GREATER);

                        if is_mth {                        
                            function.arg_type.push(TokenType::LIST(Keywords::INSTANCE(pos)));
                        }
                        function.locals.push(Local { name: arg_name, local_type: arg_type, is_special: SpecialType::List(list_type_value) });
                    }else {
                        if is_mth {                        
                            function.arg_type.push(TokenType::STRUCT(pos));
                        }
                        function.locals.push(Local { name: arg_name, local_type: arg_type, is_special: SpecialType::Null });
                    }
                },
                _ => {
                    if is_mth {                    
                        function.arg_type.push(arg_type);
                    }
                    function.locals.push(Local { name: arg_name, local_type: arg_type, is_special: SpecialType::Null });
                },
            };

            if self.parser.cur.token_type == TokenType::COMMA {
                self.parser.consume(TokenType::COMMA);
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

                if !is_mth {                
                    let pos = self.get_fn_symbol_pos(name.clone());
                    self.parser.symbols[pos].output_type = function.output_type;
                }
                        
                self.parser.consume(TokenType::KEYWORD(keyword))
            },
            TokenType::IDENTIFIER => {
                let val = self.parser.cur.value.iter().collect::<String>();

                if val == "List" {
                    self.parser.consume(TokenType::IDENTIFIER);
                    self.parser.consume(TokenType::LESS);

                    let list_type = self::Parser::list_type_value(&self.parser.symbols, self.parser.cur.clone(), val.clone(), self.parser.line, self.cur_filepath.clone());

                    function.output_type = TokenType::LIST(list_type);

                    self.parser.advance();

                    self.parser.consume(TokenType::GREATER);
                } else {
                    
                    if !self.structs.contains_key(&val) {
                        errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR", format!("Unexpected return type {:?} {}:", self.parser.cur.token_type, self.parser.line));
                        std::process::exit(1);
                    }
                    
                    let pos = self.get_struct_symbol_pos(val);
                    function.output_type = TokenType::STRUCT(pos);  
                    
                    self.parser.consume(TokenType::IDENTIFIER);
                }
            },
            _ => {
                function.output_type = TokenType::NULL;
            }
        };

        if !is_mth {
            // This if stmt is left, because of tests on reference counting, it should never panic
            let fn_pos = self.get_fn_symbol_pos(function.name.clone());
            if function.output_type != self.parser.symbols[fn_pos].output_type {
                println!("{:?} {:?}", function.output_type, self.parser.symbols[fn_pos].output_type);
                panic!()
            }
        }
        self.parser.consume(TokenType::LEFT_BRACE);

        self.scope_depth += 1;

        let enclosing = self.cur_function.clone();
        self.cur_function = function;

        self.block();

        let pos = self.get_cur_chunk().push_value(Value::Null);
        self.emit_byte(OpCode::CONSTANT_NULL(pos), self.parser.line);

        self.emit_byte(OpCode::RETURN, self.parser.line);
        for index in 0..self.get_cur_locals().len() {
            match self.get_cur_locals()[index].local_type {
                TokenType::KEYWORD(Keywords::INSTANCE(_)) => {
                    self.emit_byte(OpCode::DEC_RC(index), self.parser.line);
                },
                _ => {},
            }
        }

        self.emit_byte(OpCode::END_OF_FN, self.parser.line);

        if is_mth {
            let fun = self.cur_function.clone();
            self.cur_function = enclosing;
            self.scope_depth -= 1;

            return fun
        }

        let op_code = OpCode::FUNCTION_DEC(self.cur_function.clone());

        *self.functions.get_mut(&name.clone()).unwrap() = enclosing.clone();

        self.cur_function = enclosing;

        self.emit_byte(op_code, self.parser.line);

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
            _ => errors::error_unexpected(self.cur_filepath.clone(), self.parser.prev.clone(), "declare function"),
        }
    }

    pub fn return_stmt(&mut self) {
        self.expression();
        let var_type = match self.get_cur_chunk().get_last_instruction().op {
            OpCode::VAR_CALL(index) => {           
                self.get_cur_locals()[index].local_type
            },
            OpCode::GET_INSTANCE_RF(index) => {
                match self.get_cur_locals()[index].local_type {
                    TokenType::KEYWORD(Keywords::INSTANCE(pos)) => {
                        if let SpecialType::List(list_type) = self.get_cur_locals()[index].is_special.clone() {
                            let token = match list_type.convert() {
                                TokenType::INT => Keywords::INT,
                                TokenType::FLOAT => Keywords::FLOAT,
                                TokenType::BOOL => Keywords::BOOL,
                                TokenType::STRING => Keywords::STRING,
                                TokenType::STRUCT(pos) => Keywords::INSTANCE(pos),
                                _ => panic!("never should happen"),
                            };

                            TokenType::LIST(token)                            
                        } else {                        
                            TokenType::STRUCT(pos)
                        }
                        
                    }
                    _ => panic!("Unexpected token in return stmt for Instance Ref")
                }
            }
            _ => {
                self.get_cur_chunk().get_last_value().convert()   
            }
        };

        if var_type != self.cur_function.output_type {
            errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Mismatched types while returning function, expected: {:?} found: {:?} {}:",
                self.cur_function.output_type,
                var_type,
                self.parser.line,
            ));
            std::process::exit(1);
        }

        self.emit_byte(OpCode::RETURN, self.parser.line);
    }

    pub fn if_stmt(&mut self) {
        if self.parser.cur.token_type == TokenType::LEFT_BRACE {
            errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Expected to find expression after {} statement {}:",
                self.parser.prev.value.iter().collect::<String>().to_ascii_uppercase(),
                self.parser.line,
            ));
            std::process::exit(1);
        }
        
        self.expression();

        if self.parser.symbols.len() > 1 && 
        self.parser.symbols[self.symbol_to_hold].symbol_type == TokenType::KEYWORD(Keywords::FN) &&
        self.parser.symbols[self.symbol_to_hold].output_type != TokenType::BOOL {
            // println!("{:?}", self.parser.symbols[self.symbol_to_hold]);
            errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Expected to find BOOL but found {:?} {}:",
                self.parser.symbols[self.symbol_to_hold].output_type,
                self.parser.line,
            ));
            std::process::exit(1);
        }

        let index_jump_to_stmt = self.get_cur_chunk().code.len();
        self.emit_byte(OpCode::IF_STMT_OFFSET(0), self.parser.line);

        self.emit_byte(OpCode::POP, self.parser.line);

        self.parser.consume(TokenType::LEFT_BRACE);

        let local_counter = self.get_cur_locals().len();

        self.block();

        for i in (0..self.get_cur_locals().len() - local_counter).rev() {
            match self.get_cur_locals()[local_counter + i].local_type.clone() {
                TokenType::KEYWORD(Keywords::INSTANCE(_)) => {
                    self.emit_byte(OpCode::DEC_RC(local_counter + i), self.parser.line);
                },
                _ => {},
            }

            self.emit_byte(OpCode::POP, self.parser.line);
            self.get_cur_locals().pop();
        }

        let index_exit_if = self.get_cur_chunk().code.len();
        self.emit_byte(OpCode::JUMP(0), self.parser.line);

        let offset_stmt = (self.get_cur_chunk().code.len() - index_jump_to_stmt) - 1;
        self.get_cur_chunk().code[index_jump_to_stmt] = Instruction { op: OpCode::IF_STMT_OFFSET(offset_stmt), line: self.parser.line };

        self.emit_byte(OpCode::POP, self.parser.line);

        if self.parser.cur.token_type == TokenType::KEYWORD(Keywords::ELIF) || self.parser.cur.token_type == TokenType::KEYWORD(Keywords::ELSE) {
            self.compile_line();
        }

        let offset_exit_if = (self.get_cur_chunk().code.len() - index_exit_if) - 1; 
        self.get_cur_chunk().code[index_exit_if] = Instruction { op: OpCode::JUMP(offset_exit_if), line: self.parser.line };

        self.emit_byte(OpCode::RF_REMOVE, self.parser.line);
    }

    pub fn else_stmt(&mut self) {
        self.parser.consume(TokenType::LEFT_BRACE);
        self.block();
    }

    pub fn while_stmt(&mut self) {
        let loop_start_index = self.get_cur_chunk().code.len();

        if self.parser.cur.token_type == TokenType::LEFT_BRACE {
            errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Expected to find expression after {} statement {}:",
                self.parser.prev.value.iter().collect::<String>().to_ascii_uppercase(),
                self.parser.line,
            ));
            std::process::exit(1);
        }

        self.expression();

        if self.parser.symbols.len() > 1 && 
        self.parser.symbols[self.symbol_to_hold].symbol_type == TokenType::KEYWORD(Keywords::FN) &&
        self.parser.symbols[self.symbol_to_hold].output_type != TokenType::BOOL {
            errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Expected to find BOOL but found {:?} {}:",
                self.parser.symbols[self.symbol_to_hold].output_type,
                self.parser.line,
            ));
            std::process::exit(1);
        };

        let index_exit_stmt = self.get_cur_chunk().code.len();
        self.emit_byte(OpCode::IF_STMT_OFFSET(0), self.parser.line);
        self.emit_byte(OpCode::POP, self.parser.line);

        self.parser.consume(TokenType::LEFT_BRACE);

        let local_counter = self.get_cur_locals().len();
        self.scope_depth += 1;

        self.loop_info.loop_type = TokenType::KEYWORD(Keywords::WHILE);
        self.loop_info.locals_start = local_counter;
        self.loop_info.start = loop_start_index;

        self.block();

        self.loop_info.loop_type = TokenType::KEYWORD(Keywords::WHILE);
        self.loop_info.locals_start = local_counter;
        self.loop_info.start = loop_start_index;
        self.scope_depth -= 1;

        for i in (0..self.get_cur_locals().len() - local_counter).rev() {
            match self.get_cur_locals()[local_counter + i].local_type.clone() {
                TokenType::KEYWORD(Keywords::INSTANCE(_)) => {
                    self.emit_byte(OpCode::DEC_RC(local_counter + i), self.parser.line);
                },
                _ => {},
            }
            self.emit_byte(OpCode::POP, self.parser.line);
            self.get_cur_locals().pop();
        }

        let offset_loop = (self.get_cur_chunk().code.len() - loop_start_index) + 1;
        self.emit_byte(OpCode::LOOP(offset_loop), self.parser.line);

        let offset_stmt = (self.get_cur_chunk().code.len() - index_exit_stmt) - 1;
        self.get_cur_chunk().code[index_exit_stmt] = Instruction { op: OpCode::IF_STMT_OFFSET(offset_stmt), line: self.parser.line };

        self.emit_byte(OpCode::RF_REMOVE, self.parser.line);

        self.emit_byte(OpCode::POP, self.parser.line);
    }

    pub fn for_stmt(&mut self) {
        self.parser.consume(TokenType::IDENTIFIER);

        let identifier = self.parser.prev.value.iter().collect::<String>();
       
        if self.get_cur_locals().iter().any(| local | local.name == identifier ) {
            errors::error_message(self.cur_filepath.clone(), "COMPILER ERROR", format!("Symbol: \"{}\" is already defined {}:", identifier, self.parser.line));
            std::process::exit(1);
        }
        
        self.get_cur_locals().push(Local { name: identifier, local_type: TokenType::INT, is_special: SpecialType::Null });

        self.parser.consume(TokenType::KEYWORD(Keywords::IN));

        // in future there need to check if I got a range or vec list to iterate on.
        self.parser.consume(TokenType::LEFT_PAREN);
        
        self.expression();

        if !matches!(self.get_cur_chunk().get_last_value(), Value::Int(_)) { 
            errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("For statement: values in range must be INT type, found: {:?} {}:",
                self.get_cur_chunk().get_last_value().clone(),
                self.parser.line,
            ));
            std::process::exit(1);
        }

        self.parser.consume(TokenType::COMMA);

        self.expression();

        if !matches!(self.get_cur_chunk().get_last_value(), Value::Int(_)) { 
            errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("For statement: values in range must be INT type, found: {:?} {}:",
                self.get_cur_chunk().get_last_value().clone(),
                self.parser.line,
            ));
            std::process::exit(1);
        }
        
        self.get_cur_locals().push(Local { name: "".to_string(), local_type: TokenType::INT, is_special: SpecialType::Null });

        if self.parser.cur.token_type != TokenType::RIGHT_PAREN {
            self.parser.consume(TokenType::COMMA);

            self.expression();

            match self.get_cur_chunk().get_last_instruction().op {
                OpCode::FUNCTION_CALL(_) => {
                    errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Functions cannot be used as STEP BY argument {}:",
                        self.parser.line,
                    ));
                    std::process::exit(1);
                },
                _ => {},
            }
        }else {
            let pos = self.get_cur_chunk().push_value(Value::Int(1));
            self.emit_byte(OpCode::CONSTANT_INT(pos), self.parser.line);
        }

        self.get_cur_locals().push(Local { name: "".to_string(), local_type: TokenType::INT, is_special: SpecialType::Null });

        self.parser.consume(TokenType::RIGHT_PAREN);

        let loop_start_index = self.get_cur_chunk().code.len();

        // check if condition is still true
        let len_locals = self.get_cur_locals().len();

        self.emit_byte(OpCode::VAR_CALL(len_locals - 3), self.parser.line);
        self.emit_byte(OpCode::VAR_CALL(len_locals - 2), self.parser.line);

        self.emit_byte(OpCode::LESS_INT, self.parser.line);
        //

        let index_exit_stmt = self.get_cur_chunk().code.len();
        self.emit_byte(OpCode::IF_STMT_OFFSET(0), self.parser.line);
        self.emit_byte(OpCode::POP, self.parser.line);

        self.parser.consume(TokenType::LEFT_BRACE);

        let local_counter = self.get_cur_locals().len();
        self.scope_depth += 1;

        self.loop_info.loop_type = TokenType::KEYWORD(Keywords::FOR);
        self.loop_info.locals_start = local_counter;
        self.loop_info.start = loop_start_index;
        self.loop_info.end_stmt_pos = index_exit_stmt;

        self.block();

        self.loop_info.loop_type = TokenType::KEYWORD(Keywords::FOR);
        self.loop_info.start = loop_start_index;
        self.loop_info.locals_start = local_counter;
        self.loop_info.end_stmt_pos = index_exit_stmt;
        self.scope_depth -= 1;

        // adding
        self.emit_byte(OpCode::VAR_CALL(len_locals - 3), self.parser.line);

        self.emit_byte(OpCode::VAR_CALL(len_locals - 1), self.parser.line);

        self.emit_byte(OpCode::ADD_INT, self.parser.line);

        self.emit_byte(OpCode::VAR_SET(len_locals - 3), self.parser.line);
        self.emit_byte(OpCode::POP, self.parser.line);
        //

        for i in (0..self.get_cur_locals().len() - local_counter).rev() {
            match self.get_cur_locals()[local_counter + i].local_type.clone() {
                TokenType::KEYWORD(Keywords::INSTANCE(_)) => {
                    self.emit_byte(OpCode::DEC_RC(local_counter + i), self.parser.line);
                },
                _ => {},
            }
            self.emit_byte(OpCode::POP, self.parser.line);
            self.get_cur_locals().pop();
        }
        
        let offset_loop = (self.get_cur_chunk().code.len() - loop_start_index) + 1;
        self.emit_byte(OpCode::LOOP(offset_loop), self.parser.line);

        let offset_stmt = (self.get_cur_chunk().code.len() - index_exit_stmt) - 1;
        self.get_cur_chunk().code[index_exit_stmt] = Instruction { op: OpCode::IF_STMT_OFFSET(offset_stmt), line: self.parser.line };

        self.emit_byte(OpCode::RF_REMOVE, self.parser.line);

        for _ in 0..3 {
            self.emit_byte(OpCode::POP, self.parser.line);
            self.get_cur_locals().pop();
        }
        self.emit_byte(OpCode::POP, self.parser.line);
    }

    pub fn and_op(&mut self) {
        let index = self.get_cur_chunk().code.len();
        self.emit_byte(OpCode::IF_STMT_OFFSET(0), self.parser.line);

        if self.parser.cur.token_type == TokenType::LEFT_BRACE {
            errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Expected to find expression after {} statement {}:",
                self.parser.prev.value.iter().collect::<String>().to_ascii_uppercase(),
                self.parser.line,
            ));
            std::process::exit(1);
        };
        self.emit_byte(OpCode::POP, self.parser.line);
        self.parse(Precedence::AND);

        let offset = (self.get_cur_chunk().code.len() - index) - 1;
        self.get_cur_chunk().code[index] = Instruction { op: OpCode::IF_STMT_OFFSET(offset), line: self.parser.line };
    }

    pub fn or_op(&mut self) {
        let index = self.get_cur_chunk().code.len();

        self.emit_byte(OpCode::IF_STMT_OFFSET(0), self.parser.line);

        let index_or = self.get_cur_chunk().code.len();
        self.emit_byte(OpCode::JUMP(0), self.parser.line);

        if self.parser.cur.token_type == TokenType::LEFT_BRACE {
            errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("Expected to find expression after {} statement {}:",
                self.parser.prev.value.iter().collect::<String>().to_ascii_uppercase(),
                self.parser.line,
            ));
            std::process::exit(1);
        };
        let offset = (self.get_cur_chunk().code.len() - index) - 1;
        self.get_cur_chunk().code[index] = Instruction { op: OpCode::IF_STMT_OFFSET(offset), line: self.parser.line };

        self.emit_byte(OpCode::POP, self.parser.line);

        self.parse(Precedence::OR);

        let offset = (self.get_cur_chunk().code.len() - index_or) - 1;
        self.get_cur_chunk().code[index_or] = Instruction { op: OpCode::JUMP(offset), line: self.parser.line };
    }

    fn import_file(&mut self) {
        if self.parser.cur.token_type != TokenType::STRING {
            error_message(self.cur_filepath.clone(), "COMPILER ERROR", format!("Expected to find path to file after \"import\" keyword {}:", self.parser.line));
            std::process::exit(1);
        }
        self.parser.consume(TokenType::STRING);
        
        let mut file_path = self.parser.prev.value.iter().collect::<String>();
        file_path = self.parser.base_dir.clone() + &"/" + &file_path;

        if file_path == self.main_file {
            error_message(self.cur_filepath.clone(), "COMPILER ERROR", format!("Expected to find path to file after \"import\" keyword {}:", self.parser.line));
            std::process::exit(1);
        }

        if self.compiled_files.contains_key(&file_path) {
            return
        }
        self.compiled_files.insert(file_path.clone(), 1);
        
        let enclosing_tokens = self.parser.tokens.clone();
        let enclosing_cur = self.parser.cur.clone();
        let enclosing_prev = self.parser.prev.clone();
        let enclosing_line = self.parser.line;
        let enclosing_index = self.parser.index;

        self.parser.tokens = self.parser.imported_files.get(&file_path).unwrap().to_vec();
        self.parser.cur = Token { token_type: TokenType::ERROR, value: vec![], line: 0};
        self.parser.prev = Token { token_type: TokenType::ERROR, value: vec![], line: 0};
        self.parser.line = 0;
        self.parser.index = 0;
        
        self.compile_file();

        self.parser.tokens = enclosing_tokens;
        self.parser.cur = enclosing_cur;
        self.parser.prev = enclosing_prev;
        self.parser.line = enclosing_line;
        self.parser.index = enclosing_index;
     }

    fn compile_line(&mut self) {
        match self.parser.cur.token_type {
            TokenType::KEYWORD(Keywords::FN) | TokenType::KEYWORD(Keywords::VAR) | TokenType::KEYWORD(Keywords::LIST) => {
                self.parser.advance();
                self.declare();
            },
            TokenType::KEYWORD(Keywords::IMPORT) => {
                self.parser.advance();
                self.import_file();
            }
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
                    error_message(self.cur_filepath.clone(), "COMPILER ERROR", format!("Expected to find }} before ELIF statment {}:", self.parser.line));
                    std::process::exit(1);
                }
                self.parser.advance();
                self.if_stmt();
            },
            TokenType::KEYWORD(Keywords::ELSE) => {
                if self.parser.prev.token_type != TokenType::RIGHT_BRACE {
                    error_message(self.cur_filepath.clone(), "COMPILER ERROR", format!("Expected to find }} before ELSE statment {}:", self.parser.line));
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
                    errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("BREAK statment used out of loop {}:",
                        self.parser.line,
                    ));
                    std::process::exit(1);
                };

                self.emit_byte(OpCode::BREAK, self.parser.line);

                let local_counter = self.loop_info.locals_start;

                for i in (0..self.get_cur_locals().len() - local_counter).rev() {
                    match self.get_cur_locals()[local_counter + i].local_type.clone() {
                        TokenType::KEYWORD(Keywords::INSTANCE(_)) => {
                            self.emit_byte(OpCode::DEC_RC(local_counter + i), self.parser.line);
                        },
                        _ => {},
                    }
                    self.emit_byte(OpCode::POP, self.parser.line);
                }

                let offset = (self.get_cur_chunk().code.len() - self.loop_info.start) + 1;

                self.emit_byte(OpCode::LOOP(offset), self.parser.line);

            },
            TokenType::KEYWORD(Keywords::CONTINUE) => {
                self.parser.advance();

                if self.scope_depth <= 1 {
                    errors::error_message(self.cur_filepath.clone(), "COMPILING ERROR", format!("CONTINUE statment used out of loop {}:",
                        self.parser.line,
                    ));
                    std::process::exit(1);
                };

                if self.loop_info.loop_type == TokenType::KEYWORD(Keywords::FOR) {
                    self.emit_byte(OpCode::VAR_CALL(self.loop_info.locals_start - 3), self.parser.line);

                    self.emit_byte(OpCode::VAR_CALL(self.loop_info.locals_start - 1), self.parser.line);
        
                    self.emit_byte(OpCode::ADD_INT, self.parser.line);
        
                    self.emit_byte(OpCode::VAR_SET(self.loop_info.locals_start - 3), self.parser.line);

                    self.emit_byte(OpCode::POP, self.parser.line);
                }

                let local_counter = self.loop_info.locals_start;

                for i in (0..self.get_cur_locals().len() - local_counter).rev() {
                    match self.get_cur_locals()[local_counter + i].local_type.clone() {
                        TokenType::KEYWORD(Keywords::INSTANCE(_)) => {
                            self.emit_byte(OpCode::DEC_RC(local_counter + i), self.parser.line);
                        },
                        _ => {},
                    }
                    self.emit_byte(OpCode::POP, self.parser.line);
                }

                let offset = (self.get_cur_chunk().code.len() - self.loop_info.start) + 1;
                self.emit_byte(OpCode::LOOP(offset), self.parser.line);
            },
            _ => {
                self.expression();
                self.emit_byte(OpCode::POP_UNUSED, self.parser.line);
            }
        }
    }

    pub fn impl_native_types(&mut self) {
        // STRING

        // 19 natives builtin functions
        let string_type = StringObj::init(19);
        let list_type = ListObj::init(19 + string_type.methods.len());

        self.parser.get_builtin_symbols(string_type.methods.len());

        self.functions = self.parser.get_symbols_and_functions(true, self.main_file.clone());
        self.get_cur_chunk().push(Instruction { op: OpCode::STRUCT_DEC(string_type.clone()), line: 0 });
        self.structs.insert("String".to_string(), string_type);

        self.get_cur_chunk().push(Instruction { op: OpCode::STRUCT_DEC(list_type.clone()), line: 0 });
        self.structs.insert("List".to_string(), list_type);
    }

    pub fn compile(&mut self) -> Chunk {
        self.impl_native_types();
        self.compile_file();
        self.get_cur_chunk().clone()
    }

    fn compile_file(&mut self) {
        self.parser.advance();
        loop {
            self.parser.line = self.parser.cur.line;
            if self.parser.check_if_eof() {
                break;
            }
            self.compile_line();
            self.symbol_to_hold = 0;
            self.loop_info = LoopInfo::new();
        }
    }

    pub fn parse(&mut self, prec: Precedence) {
        self.parser.advance();

        if !self.parser.rules.contains_key(&self.parser.prev.token_type) {
            errors::error_message(self.cur_filepath.clone(), "PARSING ERROR", format!("Cannot get a parse rule for: {:?}: \"{}\", {}:",
                self.parser.prev.token_type,
                self.parser.prev.value.iter().collect::<String>(),
                self.parser.line,
            ));
            std::process::exit(1);
        }
        let rule = self.parser.get_rule(&self.parser.prev.token_type);

        match rule.prefix {
            Some(f) => f(self),
            _ => {
                errors::error_message(self.cur_filepath.clone(), "PARSING ERROR", format!("Expected prefix for: {:?}, {}:", self.parser.prev.token_type, self.parser.line));
                std::process::exit(1);
            },
        };

        while prec <= self.parser.get_rule(&self.parser.cur.token_type).prec {
            self.parser.advance();

            if !self.parser.rules.contains_key(&self.parser.prev.token_type) {
                errors::error_message(self.cur_filepath.clone(), "PARSING ERROR", format!("Cannot get a parse rule for: {:?}: \"{}\", {}:",
                    self.parser.prev.token_type,
                    self.parser.prev.value.iter().collect::<String>(),
                    self.parser.line,
                ));
                std::process::exit(1);
            }
            let rule = self.parser.get_rule(&self.parser.prev.token_type);
            match rule.infix {
                Some(f) => f(self),
                _ => {
                    errors::error_message(self.cur_filepath.clone(), "PARSING ERROR", format!("Expected infix for: {:?}, {}:", self.parser.prev.token_type, self.parser.line));
                    std::process::exit(1);
                },
            }
        }
    }

    pub fn emit_byte(&mut self, op: OpCode, line: u32) {
        if self.scope_depth == 0 {
            errors::error_message(self.cur_filepath.clone(), "PARSER ERROR", format!("Expression found outside of bounds {}:",self.parser.line));
            std::process::exit(1)
        }
        self.get_cur_chunk().push(Instruction{ op, line });
    }
}
