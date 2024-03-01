use std::fs::File;
use std::io::Read;

use crate::frontend::tokens::{Token, TokenType};

pub struct Scanner {
    source_code: Vec<char>,
    start: usize,
    cur: usize,
    line: i32,
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

    fn number(&mut self) -> Token {
        while self.peek().is_digit(10) {
            self.next();
        }

        if self.peek() == '.' && self.peek_next().is_digit(10) {
            while self.peek().is_digit(10) {
                self.next();
            }
        }

        return Token {
            token_type: TokenType::NUMBER,
            value: self.source_code[self.start..self.cur].to_vec(),
            line: self.line,
        };
    }

    fn identifier(&mut self) -> Token {
        while ((self.peek() >= 'a' && self.peek() <= 'z')
            || (self.peek() >= 'A' && self.peek() <= 'Z')
            || self.peek() == '_')
            || self.peek().is_digit(10)
        {
            self.next();
        }

        return Token {
            token_type: TokenType::IDENTIFIER,
            value: self.source_code[self.start..self.cur].to_vec(),
            line: self.line,
        };
    }

    pub fn get_tokens(&mut self) {
        loop {
            let token = self.scan_token();
            if token.token_type == TokenType::EOF {
                break;
            }
            println!("{:?}", token);
        }
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
        match c {
            'a'..='z' | 'A'..='Z' | '_' => return self.identifier(),
            _ if c.is_digit(10) => {
                return self.number();
            }
            _ => {}
        }

        return Token {
            token_type: TokenType::EOF,
            value: vec!['E', 'O', 'F'],
            line: self.line,
        };
    }
}

pub fn get_file(file_path: &String) -> String {
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => panic!("Error while opening a file: {:?}", e),
    };

    let mut buffer: String = String::new();
    match file.read_to_string(&mut buffer) {
        Ok(_) => {}
        Err(e) => panic!("Error while reading a file: {:?}", e),
    };

    buffer
}
