use std::{fs::File, io::{Read, Write}};

use crate::machine::machine_types::InstructionType;

use super::machine_types::Instruction;

#[derive(Debug)]
pub struct ByteCodeCompiler {
    pub source_code: Vec<u8>,
    pub file_name: String,
    pub fp: File
}

impl ByteCodeCompiler {
    pub fn new(file_name: &str) -> Self {
        let mut fp = File::options().read(true).write(true).open(file_name).expect("QUARKVM: Error while opening the file");
        let mut source_code = vec![];
        fp.read_to_end(&mut source_code).expect("QUARKVM: Error while reading the file");
        Self {
            file_name: file_name.to_string(),
            source_code,
            fp
        }
    }

    pub fn store_file(&mut self, instructions: &Vec<Instruction>) {
        for instruction in instructions.iter() {
            self.fp.write_all(&instruction.to_bytes()).unwrap_or_else(|e| panic!("QUARMVM: Error while writing instruction {:?} ERROR: {:?}", instruction, e));
        }
    }

    pub fn load_file(&mut self) -> Vec<Instruction> {
        let mut buf: Vec<u8> = vec![]; 
        self.fp.read_to_end(&mut buf).expect("QUARMVM: Error while reading the file");
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
        ins
    }
}
