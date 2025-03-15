use std::{fs, io::{Read, Write}};
use crate::machine::machine_types::*;
const MAX_STACK_SIZE: usize = 4096;


impl QuarkVM {
    pub fn new() -> Self {
        Self {
            stack: [0; MAX_STACK_SIZE],
            sp: -1,
            pc: 0,
            running: true,
            instructions: vec![
                DEFINE_PUSH(17),
                DEFINE_PUSH(27),
                DEFINE_ADD(),
                DEFINE_PUSH(37),
                DEFINE_ADD(),
                DEFINE_PUSH(27),
                DEFINE_PUSH(27),
                DEFINE_PUSH(27),
                DEFINE_PUSH(27),
                DEFINE_SUB()
            ]
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

    pub fn store_file(&mut self, file_name: &str) {
        let mut f = fs::File::create(file_name).expect("QUARMVM: Failed to create the file");
        for instruction in self.instructions.iter() {
            f.write_all(&instruction.to_bytes()).unwrap_or_else(|_| panic!("QUARMVM: Error while writing instruction {:?}", instruction));
        }
    }

    pub fn load_file(&mut self, file_name: &str) {
        let mut f = fs::File::open(file_name).expect("QUARMVM: Error while opening the file");
        let mut buf: Vec<u8> = vec![]; 
        f.read_to_end(&mut buf).expect("QUARMVM: Error while reading the file");
        let mut i = 0;
        println!("BUFFER: {:?}", buf);
        let mut ins = vec![];
        while i < buf.iter().len() {
            let instruction = buf[i];
            i += 1;
            let argument_length = buf[i];
            i += 1;
            let mut args: Vec<u16> = vec![];
            for x in 0..argument_length {
                let arg = u16::from_be_bytes([buf[i + x as usize], buf[i + x as usize + 1]]);
                i += 2;
                args.push(arg);
            }
            let instruction_type = InstructionType::try_from(instruction).expect("QUARMVM: Error while converting instruction to token type");
            let instruction = Instruction {
                tt: instruction_type,
                values: if argument_length > 0 {
                    Some(args)
                } else {
                    None
                }
            };
            ins.push(instruction);
        }
        self.instructions = ins;
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
