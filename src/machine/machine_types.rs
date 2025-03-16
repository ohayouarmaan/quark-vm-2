use super::bytecode::ByteCodeCompiler;


const MAX_STACK_SIZE: usize = 4096;

#[derive(Debug)]
pub struct QuarkVM {
    pub stack: [u16; MAX_STACK_SIZE],
    pub sp: i16,
    pub pc: i16,
    pub running: bool,
    pub instructions: Vec<Instruction>,
    pub byte_code_file: Option<ByteCodeCompiler>
}

impl Default for QuarkVM {
    fn default() -> Self {
        Self {
            stack: [0; MAX_STACK_SIZE],
            sp: -1,
            pc: 0,
            running: false,
            instructions: vec![],
            byte_code_file: None
        }
    }
}

#[derive(Debug,Copy,Clone)]
#[repr(u8)]
pub enum InstructionType {
    INST_NOOP = 0,
    INST_PUSH,
    INST_POP,
    INST_ADD,
    INST_AND,
    INST_OR,
    INST_XOR,
    INST_NOT,
    INST_SHL,
    INST_SHR,
    INST_MUL,
    INST_DIV,
    INST_SUB,
    INST_JMPZ,
    INST_JMPEQ,
    INST_JMPNEQ,
    INST_JMPNZ,
}

impl Default for InstructionType {
    fn default() -> Self {
        return Self::INST_NOOP;
    }
}

impl TryFrom<u8> for InstructionType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x <= 17 => Ok(unsafe { std::mem::transmute(x) }),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct Instruction {
    pub tt: InstructionType,
    pub values: Option<Vec<u16>>
}

impl Instruction {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![];
        buffer.push(self.tt as u8);
        buffer.push(self.values.iter().len() as u8);
        if let Some(values) = &self.values {
            for value in values {
                buffer.extend_from_slice(&value.to_be_bytes());
            }
        }
        return buffer;
    }
}

pub fn DEFINE_PUSH(x: u16) -> Instruction {
    return Instruction {
        tt: InstructionType::INST_PUSH,
        values: Some(vec![x])
    }
}

pub fn DEFINE_POP() -> Instruction {
    return Instruction {
        tt: InstructionType::INST_POP,
        values: None
    }
}

pub fn DEFINE_ADD() -> Instruction {
    return Instruction {
        tt: InstructionType::INST_ADD,
        values: None
    }
}

pub fn DEFINE_MUL() -> Instruction {
    return Instruction {
        tt: InstructionType::INST_MUL,
        values: None
    }
}

pub fn DEFINE_DIV() -> Instruction {
    return Instruction {
        tt: InstructionType::INST_DIV,
        values: None
    }
}

pub fn DEFINE_SUB() -> Instruction {
    return Instruction {
        tt: InstructionType::INST_SUB,
        values: None
    }
}

pub fn DEFINE_JMPZ(x: u16) -> Instruction {
    return Instruction {
        tt: InstructionType::INST_JMPZ,
        values: Some(vec![x])
    }
}

pub fn DEFINE_JMPEQ(x: u16) -> Instruction {
    return Instruction {
        tt: InstructionType::INST_JMPEQ,
        values: Some(vec![x])
    }
}

pub fn DEFINE_JMPNZ(x: u16) -> Instruction {
    return Instruction {
        tt: InstructionType::INST_JMPNZ,
        values: Some(vec![x])
    }
}
