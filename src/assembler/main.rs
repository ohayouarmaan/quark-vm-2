use std::env;
use std::process;
use crate::assembler::Assembler;
mod assembler;
mod lexer;
mod parser;
mod compiler;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: assembler <input_file> <output_file>");
        process::exit(1);
    }


    let input = &args[1];
    let output = &args[2];

    match Assembler::new(input, output) {
        Ok(mut assembler) => {
            assembler.compile()
        },
        Err(e) => {
            eprintln!("Failed to read source file: {}", e);
            process::exit(1);
        }
    }
}
