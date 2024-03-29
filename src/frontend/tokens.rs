use crate::vm::value::Convert;
use crate::compiler::errors;

#[derive(Debug, PartialEq)]
#[derive(Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: Vec<char>,
    pub line: u32,
}

#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
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
    COLON,
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
    KEYWORD(Keywords),
    NATIVE_FN,
    INT,
    FLOAT,
    BOOL,
    ERROR,
    EOF,
}

#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum Keywords {
    VAR,
    INT,
    FLOAT,
    BOOL,
    TRUE,
    FALSE,
    NULL,
    IF,
    ELIF,
    ELSE,
    FN,
    RETURN,
}

impl std::str::FromStr for Keywords {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "var" => Ok(Keywords::VAR),
            "int" => Ok(Keywords::INT),
            "bool" => Ok(Keywords::BOOL),
            "float" => Ok(Keywords::FLOAT),
            "true" => Ok(Keywords::TRUE),
            "false" => Ok(Keywords::FALSE),
            "null" => Ok(Keywords::NULL),
            "if" => Ok(Keywords::IF),
            "elif" => Ok(Keywords::ELIF),
            "else" => Ok(Keywords::ELSE),
            "fn" => Ok(Keywords::FN),
            "return" => Ok(Keywords::RETURN),
            _ => Err(()),
        }
    }
}

impl Convert for Keywords {
    fn convert(&self) -> TokenType {
        match self {
            Keywords::INT => TokenType::INT,
            Keywords::FLOAT => TokenType::FLOAT,
            Keywords::TRUE | Keywords::FALSE | Keywords::BOOL => TokenType::BOOL,
            _ => {
                errors::conversion_error("Enum Keyword<_>", "TokenType");
                std::process::exit(1);
            },
        }
    }
}