const MAX_STACK_SIZE: usize = 4096;

#[derive(Debug)]
pub struct QuarkVM {
    pub stack: [u16; MAX_STACK_SIZE],
    pub sp: i16,
    pub pc: i16,
    pub running: bool,
    pub instructions: Vec<Instruction>
}

impl Default for QuarkVM {
    fn default() -> Self {
        Self {
            stack: [0; MAX_STACK_SIZE],
            sp: -1,
            pc: 0,
            running: false,
            instructions: vec![]
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
    INST_MUL,
    INST_DIV,
    INST_SUB
}

impl Default for InstructionType {
    fn default() -> Self {
        return Self::INST_NOOP;
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
