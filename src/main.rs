use std::env;

mod frontend;
mod vm;
mod debug;

fn run(file_path: &String) {
    let source_code = frontend::lexer::get_file(file_path);

    let mut scanner = frontend::lexer::Scanner::init(&source_code);
    let tokens = scanner.get_tokens();

    let mut compiler = vm::compiler::Compiler::new(tokens);
    let chunk = compiler.compile();

    let mut vm = vm::vm::VM::new(chunk);
    vm.run();
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        2 => run(&args[1]),
        _ => println!("Usage: shlang [file name]"),
    }
}
