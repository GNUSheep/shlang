use crate::frontend::tokens::{Token, TokenType, Keywords};

pub fn error_message(file_path: String, title: &str, msg: String) {
    eprintln!("==== {} ====", title);
    eprintln!("At {}", file_path);
    eprintln!("{}", msg);
}

pub fn conversion_error(file_path: String, from: &str, to: &str) {
    error_message(file_path, "CONVERSION ERROR", format!("ERROR: Unable to convert {} to {}; exit code: 1", from, to));
}

pub fn token_error(file_path: String, token: Token) {
    error_message(file_path, "TOKEN ERROR", token.value.iter().collect::<String>());
    std::process::exit(1);
}

pub fn error_unexpected(file_path: String, token: Token, place: &str) {
    error_message(file_path, "COMPILER ERROR", format!("Unexpected token ({:?}) in {} {}:", token.token_type, place.to_ascii_uppercase(), token.line));
}

pub fn error_unexpected_token_type(file_path: String, token_type: TokenType, line: u32, place: &str) {
    error_message(file_path, "COMPILER ERROR", format!("Unexpected type ({:?}) in {} {}:", token_type, place.to_ascii_uppercase(), line));
}

pub fn error_unexpected_keyword(file_path: String, keyword: Keywords, line: u32, place: &str) {
    error_message(file_path, "COMPILER ERROR", format!("Unexpected keyword ({:?}) in {} {}:", keyword, place.to_ascii_uppercase(), line));
}
