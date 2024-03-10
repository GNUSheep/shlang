use crate::frontend::tokens::{Token, TokenType};

pub fn error_message(title: &str, msg: String) {
    println!("==== {} ====", title);
    println!("{}", msg);
}

pub fn conversion_error(from: &str, to: &str) {
    error_message("CONVERSION ERROR", format!("ERROR: Unable to convert {} to {}; exit code: 1", from, to));
}

pub fn token_error(token: Token) {
    error_message("TOKEN ERROR", token.value.iter().collect::<String>());
    std::process::exit(1);
}

pub fn error_unexpected(token: Token, place: &str) {
    error_message("COMPILER ERROR", format!("Unexpected token ({:?}) in {} {}:", token.token_type, place.to_ascii_uppercase(), token.line));
}

pub fn error_unexpected_token_type(token_type: TokenType, line: u32, place: &str) {
    error_message("COMPILER ERROR", format!("Unexpected type ({:?}) in {} {}:", token_type, place.to_ascii_uppercase(), line));
}