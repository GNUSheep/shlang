use std::env;

mod frontend;
mod vm;
mod debug;

fn run(file_path: &String) {
    let source_code = frontend::lexer::get_file(file_path);

    let mut scanner = frontend::lexer::Scanner::init(&source_code);
    scanner.get_tokens();

    let mut chunk = vm::bytecode::Chunk::new();
    chunk.push(vm::bytecode::Instruction{op: vm::bytecode::OpCode::RETURN, bytes: 0});
    
    debug::debug_chunk(&chunk);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        2 => run(&args[1]),
        _ => println!("Usage: shlang [file name]"),
    }
}
