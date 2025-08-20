mod ast;
mod codegen;
mod lexer;
mod parser;

use codegen::JITCompiler;
use lexer::Lexer;
use parser::Parser;
use std::env;
use std::fs;

/// External function called by 'print' in the language.
#[no_mangle]
pub extern "C" fn print_i64(x: i64) -> i64 {
    println!("{}", x);
    x
}

fn main() {
    let file = env::args().nth(1).expect("Usage: frag-compiler <file>");
    let src = fs::read_to_string(&file).expect("Failed to read file");

    let lexer = Lexer::new(&src);
    let mut parser = Parser::new(lexer);

    match parser.parse_program() {
        Ok(prog) => {
            let mut jit = JITCompiler::new();
            let result = jit.compile_and_run(&prog);
            println!("Execution result: {}", result);
        }
        Err(e) => {
            eprintln!("Error parsing program: {:?}", e);
        }
    }
}
