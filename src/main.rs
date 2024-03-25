use::std::env;

mod frontend;
mod vm;
mod compiler;
mod objects;
mod std;

#[cfg(feature = "debug_chunk")]
mod debug;

fn run(file_path: &String) {
    let source_code = frontend::lexer::get_file(file_path);

    let mut scanner = frontend::lexer::Scanner::init(&source_code);
    let tokens = scanner.get_tokens();

    #[cfg(feature = "debug_tokens")]
    {
        println!("{:?}", tokens);
    }

    let mut compiler = compiler::compiler::Compiler::new(tokens);
    compiler.parser.get_symbols();

    let main_chunk = compiler.compile();
    #[cfg(feature = "debug_chunk")]
    {
        debug::debug_chunk(&main_fn.chunk);
    }
    let mut vm = vm::vm::VM::new();
    let main_frame = vm.declare_all(main_chunk);

    vm.frames.push(main_frame);

    vm.run();
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        2 => run(&args[1]),
        _ => println!("Usage: shlang [file name]"),
    }
}
