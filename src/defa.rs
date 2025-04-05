use lib::bytecode::ByteCodeCompiler;
use assembler::assembler::Assembler;
use std::{env, io};
mod assembler;
mod machine;
mod lib;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <file.qasm> <output.out>", args[0]);
        std::process::exit(1);
    }

    let file_name = &args[1];
    let output_name = &args[2];

    // let mut assembler = Assembler::new(file_name, output_name)?;
    // assembler.compile();

    let mut quark_machine = lib::machine_type::QuarkVM::new(ByteCodeCompiler::new("./test.out"));
    quark_machine.load_file();
    quark_machine.run();
    Ok(())
}
