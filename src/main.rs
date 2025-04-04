use machine::bytecode::ByteCodeCompiler;
use assembler::assembler::Assembler;
use std::{env, io};
mod assembler;
mod machine;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file.qasm>", args[0]);
        std::process::exit(1);
    }

    let file_name = &args[1];

    // Create the assembler from the file
    let mut assembler = Assembler::new(file_name)?;
    assembler.compile();
    // let mut quark_machine = machine::machine_types::QuarkVM::new(ByteCodeCompiler::new("./test.out"));
    // quark_machine.store_file();
    // quark_machine.run();
    Ok(())
}
