use super::bytecode::ByteCodeCompiler;
use half::f16;


const MAX_STACK_SIZE: usize = 4096;

#[derive(Debug)]
pub enum Word {
    Char(char),
    U16(u16),
    F16(f16),
}

impl Word {
    pub fn to_be_bytes(&self) -> [u8; 2]  {
        match self {
            Self::U16(x) => x.to_be_bytes(),
            Self::F16(x) => x.to_be_bytes(),
            Self::Char(c) => (*c as u16).to_be_bytes()
        }
    }
}

impl From<u16> for Word {
    fn from(value: u16) -> Self {
        return Self::U16(value);
    }
}

impl From<f16> for Word {
    fn from(value: f16) -> Self {
        return Self::F16(value);
    }
}

impl From<char> for Word {
    fn from(value: char) -> Self {
        return Self::Char(value);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StackValues{
    U16(u16),
    Pointer(*const u8)
}

#[derive(Debug)]
pub struct QuarkVM {
    pub stack: [StackValues; MAX_STACK_SIZE],
    pub str_stack: Vec<u8>,
    pub sp: i16,
    pub pc: i16,
    pub running: bool,
    pub instructions: Vec<Instruction>,
    pub byte_code_file: Option<ByteCodeCompiler>
}

impl Default for QuarkVM {
    fn default() -> Self {
        Self {
            stack: [StackValues::U16(0); MAX_STACK_SIZE],
            str_stack: vec![],
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
    INST_PUSH_STR,
    INST_SYSCALL,
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
            x if x <= 19 => Ok(unsafe { std::mem::transmute(x) }),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct Instruction {
    pub tt: InstructionType,
    pub values: Option<Vec<Word>>
}

impl Instruction {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![];
        buffer.push(self.tt as u8);
        buffer.push(self.values.iter().len() as u8);
        if let Some(values) = &self.values {
            for value in values {
                match value {
                    Word::U16(_) => buffer.push(0x00),
                    Word::F16(_) => buffer.push(0x01),
                    Word::Char(_) => buffer.push(0x10),
                }
                buffer.extend_from_slice(&value.to_be_bytes());
            }
        }
        return buffer;
    }
}

pub fn DEFINE_PUSH(x: u16) -> Instruction {
    return Instruction {
        tt: InstructionType::INST_PUSH,
        values: Some(vec![Word::from(x)])
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
        values: Some(vec![Word::from(x)])
    }
}

pub fn DEFINE_JMPEQ(x: u16) -> Instruction {
    return Instruction {
        tt: InstructionType::INST_JMPEQ,
        values: Some(vec![Word::from(x)])
    }
}

pub fn DEFINE_JMPNZ(x: u16) -> Instruction {
    return Instruction {
        tt: InstructionType::INST_JMPNZ,
        values: Some(vec![Word::from(x)])
    }
}

pub fn DEFINE_PUSH_STR(x: &str) -> Instruction {
    let mut values = vec![Word::from(x.len() as u16)];
    values.extend(x.chars().map(|c| Word::from(c)));
    Instruction {
        tt: InstructionType::INST_PUSH_STR,
        values: Some(values)
    }
}

pub fn DEFINE_SYSCALL(x: u16) -> Instruction {
    Instruction {
        tt: InstructionType::INST_SYSCALL,
        values: Some(vec![Word::from(x)])
    }
}

