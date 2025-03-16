use machine::bytecode::ByteCodeCompiler;
mod machine;

fn main() {
    let mut quark_machine = machine::machine_types::QuarkVM::new(ByteCodeCompiler::new("./test.qasm"));
    quark_machine.store_file();
    quark_machine.run();
    quark_machine.debug_stack();
}
