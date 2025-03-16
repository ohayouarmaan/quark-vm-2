use crate::machine::machine_types::*;
use core::arch::asm;

use super::bytecode::ByteCodeCompiler;
const MAX_STACK_SIZE: usize = 4096;

impl QuarkVM {
    pub fn new(byte_code_compiler: ByteCodeCompiler) -> Self {
        Self {
            stack: [StackValues::U16(0); MAX_STACK_SIZE],
            str_stack: Vec::new(),
            sp: -1,
            pc: 0,
            running: true,
            instructions: vec![
                DEFINE_PUSH(14),
                DEFINE_PUSH_STR("Hello, World\n"),
                DEFINE_PUSH(1),
                DEFINE_PUSH(1),
                DEFINE_SYSCALL(3),
            ],
            byte_code_file: Some(byte_code_compiler)
        }
    }

    pub fn pop_stack(&mut self) -> StackValues {
        let popped_value = self.stack[self.sp as usize];
        self.sp -= 1;
        popped_value
    }

    pub fn push_stack(&mut self, value: StackValues) {
        self.sp += 1;
        self.stack[self.sp as usize] = value;
    }

    pub fn store_file(&mut self) {
        match &mut self.byte_code_file {
            Some(bc) => {
                bc.store_file(&self.instructions)
            },
            None => {
                panic!("QUARMVM: Error while storing to file, bytecode compiler not provided.")
            }
        }
    }

    pub fn load_file(&mut self) {
        match &mut self.byte_code_file {
            Some(bc) => {
                self.instructions = bc.load_file()
            },
            None => {
                panic!("QUARMVM: Error while storing to file, bytecode compiler not provided.")
            }
        }
    }

    pub fn determine_function(&mut self) {
        match self.instructions[self.pc as usize].tt {
            InstructionType::INST_ADD => {
                if let StackValues::U16(a) = self.pop_stack() {
                    if let StackValues::U16(b) = self.pop_stack() {
                        self.push_stack(StackValues::U16(a + b));
                    }
                }
                self.pc += 1;
            },
            InstructionType::INST_PUSH => {
                match &self.instructions[self.pc as usize].values {
                        Some(value) => {
                            if let Word::U16(v) = value[0] {
                                self.push_stack(StackValues::U16(v));
                                self.pc += 1;
                            }
                        }
                    None => {
                        panic!("QUARMVM: does not have a value to push {:?}", self.instructions[self.pc as usize]);
                    }
                }
            },
            InstructionType::INST_POP => {
                self.pop_stack();
                self.pc += 1;
            },
            InstructionType::INST_MUL => {
                if let StackValues::U16(a) = self.pop_stack() {
                    if let StackValues::U16(b) = self.pop_stack() {
                        self.push_stack(StackValues::U16(a * b));
                    }
                }
                self.pc += 1;
            },
            InstructionType::INST_DIV => {
                if let StackValues::U16(a) = self.pop_stack() {
                    if let StackValues::U16(b) = self.pop_stack() {
                        self.push_stack(StackValues::U16(a / b));
                    }
                }
                self.pc += 1;
            },
            InstructionType::INST_SUB => {
                if let StackValues::U16(a) = self.pop_stack() {
                    if let StackValues::U16(b) = self.pop_stack() {
                        self.push_stack(StackValues::U16(a - b));
                    }
                }
                self.pc += 1;
            },
            InstructionType::INST_AND => {
                if let StackValues::U16(a) = self.pop_stack() {
                    if let StackValues::U16(b) = self.pop_stack() {
                        self.push_stack(StackValues::U16(a & b));
                    }
                }
                self.pc += 1;
            },
            InstructionType::INST_OR => {
                if let StackValues::U16(a) = self.pop_stack() {
                    if let StackValues::U16(b) = self.pop_stack() {
                        self.push_stack(StackValues::U16(a | b));
                    }
                }
                self.pc += 1;
            },
            InstructionType::INST_XOR => {
                if let StackValues::U16(a) = self.pop_stack() {
                    if let StackValues::U16(b) = self.pop_stack() {
                        self.push_stack(StackValues::U16(a ^ b));
                    }
                }
                self.pc += 1;
            },
            InstructionType::INST_NOT => {
                if let StackValues::U16(a) = self.pop_stack() {
                    self.push_stack(StackValues::U16(!a));
                }
            },
            InstructionType::INST_SHL => {
                if let StackValues::U16(a) = self.pop_stack() {
                    if let StackValues::U16(b) = self.pop_stack() {
                        self.push_stack(StackValues::U16(a << b));
                    }
                }
                self.pc += 1;
            },
            InstructionType::INST_SHR => {
                if let StackValues::U16(a) = self.pop_stack() {
                    if let StackValues::U16(b) = self.pop_stack() {
                        self.push_stack(StackValues::U16(a >> b));
                    }
                }
                self.pc += 1;
            },
            InstructionType::INST_JMPZ => {
                if let StackValues::U16(t) = self.stack[self.sp as usize] {
                    if t == 0 {
                        match &self.instructions[self.pc as usize].values {
                            Some(value) => {
                                if let Word::U16(v) = value[0] {
                                    self.pc += v as i16;
                                }
                            }
                            None => {
                                panic!("QUARMVM: does not have a value to push {:?}", self.instructions[self.pc as usize]);
                            }
                        }
                    };

                }
            },
            InstructionType::INST_JMPEQ => {
                if let StackValues::U16(x) = self.stack[self.sp as usize] {
                    if let StackValues::U16(y) = self.stack[(self.sp - 1) as usize] {
                        if x == y {
                            match &self.instructions[self.pc as usize].values {
                                Some(value) => {
                                    if let Word::U16(v) = value[0] {
                                        self.pc += v as i16;
                                    }
                                }
                                None => {
                                    panic!("QUARMVM: does not have a value to push {:?}", self.instructions[self.pc as usize]);
                                }
                            }
                        };

                    }
                }
            },
            InstructionType::INST_JMPNEQ => {
                if let StackValues::U16(x) = self.stack[self.sp as usize] {
                    if let StackValues::U16(y) = self.stack[(self.sp - 1) as usize] {
                        if x != y {
                            match &self.instructions[self.pc as usize].values {
                                Some(value) => {
                                    if let Word::U16(v) = value[0] {
                                        self.pc += v as i16;
                                    }
                                }
                                None => {
                                    panic!("QUARMVM: does not have a value to push {:?}", self.instructions[self.pc as usize]);
                                }
                            }
                        };

                    }
                }
            },
            InstructionType::INST_JMPNZ => {
                if let StackValues::U16(x) = self.stack[self.sp as usize] {
                    if x != 0 {
                        match &self.instructions[self.pc as usize].values {
                            Some(value) => {
                                if let Word::U16(v) = value[0] {
                                    self.pc += v as i16;
                                }
                            }
                            None => {
                                panic!("QUARMVM: does not have a value to push {:?}", self.instructions[self.pc as usize]);
                            }
                        }
                    };

                }
            },
            InstructionType::INST_PUSH_STR => {
                if let Some(args) = &self.instructions[self.pc as usize].values {
                    let starting_index = self.str_stack.len();
                    let str_len = match args[0] {
                        Word::U16(l) => l,
                        _ => panic!("QUARMVM: Expected a u16 as string size"),
                    };

                    let mut str_buffer: Vec<u8> = Vec::with_capacity(str_len as usize + 1);

                    for i in 0..str_len as usize {
                        if let Word::Char(c) = args.get(1 + i).expect("QUARMVM: Invalid string index") {
                            str_buffer.push(*c as u8);
                        } else {
                            panic!("QUARMVM: Expected a Char in string");
                        }
                    }

                    str_buffer.push(0);

                    self.str_stack.extend_from_slice(&str_buffer);
                    let string_pointer = &self.str_stack[starting_index] as *const u8;

                    println!("PUSHING STRING AT INDEX: {:?}", string_pointer);

                    self.push_stack(StackValues::Pointer(string_pointer as *const u8));
                }

                self.pc += 1;
            }
            InstructionType::INST_NOOP => {
                self.pc += 1;
            },
            InstructionType::INST_SYSCALL => {
                if let StackValues::U16(syscall_num) = self.pop_stack() {
                    let mut args: [usize; 6] = [0; 6];

                    // Read syscall arguments from stack
                    if let Some(t_values) = &self.instructions[self.pc as usize].values {
                        if let Word::U16(len) = t_values[0] {
                            for i in 0..(len as usize).min(6) {
                                match self.pop_stack() {
                                    StackValues::U16(v) => args[i] = v as usize,
                                    StackValues::Pointer(v) => args[i] = v as usize,
                                    _ => {}
                                }
                            }
                        }
                    }

                    let result: isize;
                    unsafe {
                        asm!(
                            "syscall",
                            in("rax") syscall_num as usize,  // Syscall number
                            in("rdi") args[0],  // File descriptor (1 = stdout)
                            in("rsi") args[1],  // Pointer to buffer
                            in("rdx") args[2],  // Length of buffer
                            lateout("rax") result, // Return value
                            options(nostack, preserves_flags)
                        );
                    }

                    println!("Syscall result: {}", result);

                    // Push syscall return value
                    self.push_stack(StackValues::U16(result as u16));
                }
                self.pc += 1;
            }
        }
    }

    pub fn debug_stack(&self) {
        println!("sp: {:?} stack: {:?}", self.sp, &self.stack[0..(self.sp as usize + 1)]);
        println!("STRING STACK: {:?}", self.str_stack);
    }

    pub fn run(&mut self) {
        while self.running {
            self.determine_function();
            if (self.pc as usize) >= self.instructions.len() {
                self.running = false;
            }
        }
    }
}
