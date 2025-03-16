use core::panic;
use std::{fs::File, io::{Read, Write}};
use half::f16;
use crate::machine::machine_types::{InstructionType, Word};

use super::machine_types::Instruction;

#[derive(Debug)]
pub struct ByteCodeCompiler {
    pub file_name: String,
}

impl ByteCodeCompiler {
    pub fn new(file_name: &str) -> Self {
        Self {
            file_name: file_name.to_string(),
        }
    }

    pub fn store_file(&mut self, instructions: &[Instruction]) {
        let mut file = File::create(&self.file_name).expect("QUARMVM: Error while creating file");
        for instruction in instructions.iter() {
            file.write_all(&instruction.to_bytes()).expect("QUARMVM: Error while writing instruction");
        }
    }

    pub fn load_file(&mut self) -> Vec<Instruction> {
        let mut file = File::open(&self.file_name).expect("QUARMVM: Error while opening the file");
        let mut buffer: Vec<u8> = vec![];
        file.read_to_end(&mut buffer).expect("QUARMVM: Error while reading the file");
        let mut i = 0;
        let mut ins = vec![];
        while i < buffer.iter().len() {
            let instruction = buffer[i];
            i += 1;
            let argument_length = buffer[i];
            i += 1;
            let mut args: Vec<Word> = vec![];
            for x in 0..argument_length {
                let arg_type = buffer[i];
                i += 1;
                let arg = u16::from_be_bytes([buffer[i + x as usize], buffer[i + x as usize + 1]]);
                i += 2;
                match arg_type {
                    0 => {
                        args.push(Word::U16(arg));
                    }
                    1 => {
                        args.push(Word::F16(f16::from_bits(arg)));
                    }
                    2 => {
                        args.push(Word::Char(char::from_u32(u16::from_be_bytes(arg.to_be_bytes()) as u32).expect("QUARMVM: Error while decoding character")));
                    }
                    _ => {
                        panic!("QUARMVM: Unknown argument type while parsing the qasm file");
                    }
                }
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
        ins
    }
}
