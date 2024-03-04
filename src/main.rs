use std::env;

mod frontend;
mod vm;
mod debug;

fn run(file_path: &String) {
    let source_code = frontend::lexer::get_file(file_path);

    let mut scanner = frontend::lexer::Scanner::init(&source_code);
    scanner.get_tokens();

    let mut chunk = vm::bytecode::Chunk::new();
    let costant_pos = chunk.push_value(vm::value::Value::Float(0.5));
    chunk.push(vm::bytecode::Instruction{op: vm::bytecode::OpCode::CONSTANT(costant_pos), line: 1});
    chunk.push(vm::bytecode::Instruction{op: vm::bytecode::OpCode::RETURN, line: 2});
    
    debug::debug_chunk(&chunk);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        2 => run(&args[1]),
        _ => println!("Usage: shlang [file name]"),
    }
}
