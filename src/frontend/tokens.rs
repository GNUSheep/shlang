#[derive(Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub value: Vec<char>,
    pub line: i32,
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
    EOF,
    NUMBER,
    IDENTIFIER,
}
