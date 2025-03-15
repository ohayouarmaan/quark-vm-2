mod machine;

fn main() {
    let mut quark_machine = machine::machine_types::QuarkVM::new();
    quark_machine.load_file("./test.qasm");
    quark_machine.run();
    println!("stack: {:?}", &quark_machine.stack[0..(quark_machine.sp as usize)]);
}
