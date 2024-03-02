#[derive(Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub value: Vec<char>,
    pub line: i32,
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum TokenType {
    LEFT_PAREN,
    RIGHT_PAREN,
    LEFT_BRACE,
    RIGHT_BRACE,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    STAR,
    SEMICOLON,
    SLASH,
    INTERJ,
    INTERJ_EQ,
    EQ,
    EQ_EQ,
    GREATER,
    GREATER_EQ,
    LESS,
    LESS_EQ,
    COMMENT,
    STRING,
    IDENTIFIER,
    KEYWORD,
    NUMBER,
    ERROR,
    EOF,
}

#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Keywords {
    VAR,
}

impl std::str::FromStr for Keywords {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "var" => Ok(Keywords::VAR),
            _ => Err(()),
        }
    }
}
