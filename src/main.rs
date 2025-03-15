mod machine;

fn main() {
    let mut quark_machine = machine::machine_types::QuarkVM::new();
    quark_machine.load_file("./test.qasm");
    quark_machine.run();
    quark_machine.debug_stack();
}
