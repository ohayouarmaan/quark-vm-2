use std::{fs, io::{Read, Write}};
use crate::machine::machine_types::*;

use super::bytecode::ByteCodeCompiler;
const MAX_STACK_SIZE: usize = 4096;


impl QuarkVM {
    pub fn new(byte_code_compiler: ByteCodeCompiler) -> Self {
        Self {
            stack: [0; MAX_STACK_SIZE],
            sp: -1,
            pc: 0,
            running: true,
            instructions: vec![
                DEFINE_PUSH(17),
                DEFINE_PUSH(20),
                DEFINE_ADD(),
                DEFINE_PUSH(37),
                DEFINE_SUB(),
                DEFINE_JMPZ(2),
                DEFINE_PUSH(4),
                DEFINE_PUSH(4),
            ],
            byte_code_file: Some(byte_code_compiler)
        }
    }

    pub fn pop_stack(&mut self) -> u16 {
        let popped_value = self.stack[self.sp as usize];
        self.sp -= 1;
        popped_value
    }

    pub fn push_stack(&mut self, value: u16) {
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
                let a = self.pop_stack();
                let b = self.pop_stack();
                self.push_stack(a + b);
                self.pc += 1;
            },
            InstructionType::INST_PUSH => {
                match &self.instructions[self.pc as usize].values {
                    Some(value) => {
                        self.push_stack(value[0]);
                        self.pc += 1;
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
                let a = self.pop_stack();
                let b = self.pop_stack();
                self.push_stack(a * b);
                self.pc += 1;
            },
            InstructionType::INST_DIV => {
                let a = self.pop_stack();
                let b = self.pop_stack();
                self.push_stack(a / b);
                self.pc += 1;
            },
            InstructionType::INST_SUB => {
                let a = self.pop_stack();
                let b = self.pop_stack();
                self.push_stack(b - a);
                self.pc += 1;
            },
            InstructionType::INST_AND => {
                let a = self.pop_stack();
                let b = self.pop_stack();
                self.push_stack(a & b);
            },
            InstructionType::INST_OR => {
                let a = self.pop_stack();
                let b = self.pop_stack();
                self.push_stack(a | b);
            },
            InstructionType::INST_XOR => {
                let a = self.pop_stack();
                let b = self.pop_stack();
                self.push_stack(a ^ b);
            },
            InstructionType::INST_NOT => {
                let a = self.pop_stack();
                self.push_stack(!a);
            },
            InstructionType::INST_SHL => {
                let a = self.pop_stack();
                let b = self.pop_stack();
                self.push_stack(b << a);
            },
            InstructionType::INST_SHR => {
                let a = self.pop_stack();
                let b = self.pop_stack();
                self.push_stack(b >> a);
            },
            InstructionType::INST_JMPZ => {
                if self.stack[self.sp as usize] == 0 {
                    match &self.instructions[self.pc as usize].values {
                        Some(value) => {
                            self.pc += value[0] as i16;
                        }
                        None => {
                            panic!("QUARMVM: does not have a value to push {:?}", self.instructions[self.pc as usize]);
                        }
                    }
                };
            },
            InstructionType::INST_JMPEQ => {
                if self.stack[self.sp as usize] == self.stack[(self.sp - 1) as usize] {
                    match &self.instructions[self.pc as usize].values {
                        Some(value) => {
                            self.pc += value[0] as i16;
                        }
                        None => {
                            panic!("QUARMVM: does not have a value to push {:?}", self.instructions[self.pc as usize]);
                        }
                    }
                };
            },
            InstructionType::INST_JMPNEQ => {
                if self.stack[self.sp as usize] != self.stack[(self.sp - 1) as usize] {
                    match &self.instructions[self.pc as usize].values {
                        Some(value) => {
                            self.pc += value[0] as i16;
                        }
                        None => {
                            panic!("QUARMVM: does not have a value to push {:?}", self.instructions[self.pc as usize]);
                        }
                    }
                };
            },
            InstructionType::INST_JMPNZ => {
                if self.stack[self.sp as usize] != 0 {
                    match &self.instructions[self.pc as usize].values {
                        Some(value) => {
                            self.pc += value[0] as i16;
                        }
                        None => {
                            panic!("QUARMVM: does not have a value to push {:?}", self.instructions[self.pc as usize]);
                        }
                    }
                };
            },
            InstructionType::INST_NOOP => {
                self.pc += 1;
            },
        }
    }

    pub fn debug_stack(&self) {
        println!("sp: {:?} stack: {:?}", self.sp, &self.stack[0..(self.sp as usize + 1)]);
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
