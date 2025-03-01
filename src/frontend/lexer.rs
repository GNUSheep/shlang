use std::env;
use std::{fs::File, path::Path};
use std::io::Read;

use crate::{compiler::errors::error_message, frontend::tokens::{Keywords, Token, TokenType}};

pub struct Scanner {
    source_code: Vec<char>,
    start: usize,
    cur: usize,
    line: u32,
}

impl Scanner {
    pub fn init(source_code: &String) -> Self {
        Self {
            source_code: source_code.clone().chars().collect(),
            start: 0,
            cur: 0,
            line: 1,
        }
    }

    fn next(&mut self) -> char {
        let c = self.peek();
        self.cur += 1;

        if c == '\n' {
            self.line += 1;
        }

        c
    }

    fn next_while(&mut self, f: fn(&char) -> bool) {
        let mut c = self.peek();
        while c != '\0' {
            if f(&c) {
                self.next();
            } else {
                break;
            }
            c = self.peek();
        }
    }

    fn peek(&self) -> char {
        *self.source_code.get(self.cur).unwrap_or(&'\0')
    }

    fn peek_next(&self) -> char {
        *self.source_code.get(self.cur + 1).unwrap_or(&'\0')
    }

    fn remove_whitespace(&mut self) {
        loop {
            if !self.peek().is_whitespace() {
                break;
            }
            self.next();
        }
    }

    fn string(&mut self) -> Token {
        self.next_while(|&c| c != '"' && c != '\0');

        if self.peek() == '\0' {
            return Token {
                token_type: TokenType::ERROR,
                value: format!("Missing \" at the end of string {}:{}", self.line, self.cur + 1).chars().collect(),
                line: self.line,
            };
        }

        self.next();
        
        let mut token_value: String = String::new();
        let mut esc_seq = false; 
        for c in self.source_code[self.start..self.cur].to_vec() {
            if c == '\\' {
                esc_seq = true;
                continue;
            }

            if esc_seq {
                match c {
                    'n' => token_value.push('\n'),
                    'r' => token_value.push('\r'),
                    't' => token_value.push('\t'),
                    _ => {
                        token_value.push('\\');
                        token_value.push(c);
                    }
                }
                esc_seq = false;

                continue;
            }

            token_value.push(c);
        };

        return Token {
            token_type: TokenType::STRING,
            value: token_value.trim_matches('"').chars().collect(),
            line: self.line,
        };
    }

    fn identifier(&mut self) -> Token {
        self.next_while(|&c| {
            (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c == '_') || c.is_digit(10)
        });

        let token_type = self.source_code[self.start..self.cur]
            .iter()
            .collect::<String>()
            .parse::<Keywords>()
            .map(|keyword| TokenType::KEYWORD(keyword))
            .unwrap_or(TokenType::IDENTIFIER);

        return Token {
            token_type: token_type,
            value: self.source_code[self.start..self.cur].to_vec(),
            line: self.line,
        };
    }

    fn number(&mut self) -> Token {
        self.next_while(|&c| c.is_digit(10));

        let mut token_type = TokenType::INT;
        if self.peek() == '.' && self.peek_next().is_digit(10) {
            self.next();
            self.next_while(|&c| c.is_digit(10));
            token_type = TokenType::FLOAT;
        }

        return Token {
            token_type: token_type,
            value: self.source_code[self.start..self.cur].to_vec(),
            line: self.line,
        };
    }
    
    pub fn get_tokens(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = vec![];
        
        loop {
            let token = self.scan_token();
            if token.token_type == TokenType::EOF {
                break;
            }
            if token.token_type != TokenType::COMMENT {
                tokens.push(token);
            }
        }
        tokens.push(Token {
            token_type: TokenType::EOF,
            value: vec!['E', 'O', 'F'],
            line: self.line,
        });
        return tokens
    }

    pub fn scan_token(&mut self) -> Token {
        self.remove_whitespace();

        self.start = self.cur;

        if self.peek() == '\0' {
            return Token {
                token_type: TokenType::EOF,
                value: vec!['E', 'O', 'F'],
                line: self.line,
            };
        }

        let c = self.next();
        let token_type = match c {
            '(' => TokenType::LEFT_PAREN,
            ')' => TokenType::RIGHT_PAREN,
            '{' => TokenType::LEFT_BRACE,
            '}' => TokenType::RIGHT_BRACE,
            '[' => TokenType::LEFT_BRACKET,
            ']' => TokenType::RIGHT_BRACKET,
            ',' => TokenType::COMMA,
            '.' => TokenType::DOT,
            '+' => TokenType::PLUS,
            '-' => TokenType::MINUS,
            '*' => TokenType::STAR,
            ':' => TokenType::COLON,
            '/' => TokenType::SLASH,
            '%' => TokenType::MOD,
            '!' => {
                if self.peek() == '=' {
                    self.next();
                    TokenType::INTERJ_EQ
                } else {
                    TokenType::INTERJ
                }
            }
            '=' => {
                if self.peek() == '=' {
                    self.next();
                    TokenType::EQ_EQ
                } else {
                    TokenType::EQ
                }
            }
            '>' => {
                if self.peek() == '=' {
                    self.next();
                    TokenType::GREATER_EQ
                } else {
                    TokenType::GREATER
                }
            }
            '<' => {
                if self.peek() == '=' {
                    self.next();
                    TokenType::LESS_EQ
                } else {
                    TokenType::LESS
                }
            }
            '#' => {
                self.next_while(|&c| c != '\n');
                self.next();
                TokenType::COMMENT
            }
            '"' => return self.string(),
            'a'..='z' | 'A'..='Z' | '_' => return self.identifier(),
            _ if c.is_digit(10) => {
                return self.number();
            }
            _ => {
                return Token {
                    token_type: TokenType::ERROR,
                    value: format!("Invalid char ({}) {}:{}", c, self.line, self.cur + 1).chars().collect(),
                    line: self.line,
                }
            }
        };

        return Token {
            token_type: token_type,
            value: self.source_code[self.start..self.cur].to_vec(),
            line: self.line,
        };
    }
}

pub fn get_file(file_path: &String) -> (String, String) {
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => {
            error_message("FILE OPEN", format!("Error while trying to open a file: {:?}", file_path));
            std::process::exit(1);
        },
    };

    let mut buffer: String = String::new();
    match file.read_to_string(&mut buffer) {
        Ok(_) => {}
        Err(_) => {
            error_message("FILE OPEN", format!("Error while trying to read a file: {:?}", file_path));
            std::process::exit(1);
        }
    };

    let base_dir = if let Some(directory) = Path::new(file_path).parent() {
        let mut dir = directory.to_string_lossy().to_string();
        if dir.is_empty() {
            dir = ".".to_string();
        }
        dir
    } else {
        env::current_dir().map(| path | path.to_string_lossy().to_string() ).unwrap_or(".".to_string())
    };
    (buffer, base_dir)
}
