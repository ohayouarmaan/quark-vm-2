use std::env;
use std::process;

use proton::lib::bytecode::ByteCodeCompiler;
use proton::lib::machine_type::QuarkVM;


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: machine <input_file>");
        process::exit(1);
    }

    let input_file = &args[1];

    let mut quark_machine = QuarkVM::new(ByteCodeCompiler::new(input_file));
    quark_machine.load_file();
    quark_machine.run();
}
