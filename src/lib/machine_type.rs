use super::bytecode::ByteCodeCompiler;
use core::{arch::asm, panic};
use half::f16;
use libffi::{self, high::Arg};
use libloading::{Library, Symbol};
use std::fs::File;
use std::rc::Rc;
use std::cell::RefCell;
use std::{
    collections::{HashMap, VecDeque},
    ops::Deref,
    os::fd::{AsRawFd, RawFd},
    process::exit,
};
use libc;

const MAX_STACK_SIZE: usize = 4096;

#[derive(Debug, Clone)]
pub enum Word {
    Char(char),
    U16(u16),
    F16(f16),
    I16(i16),
}

impl Word {
    pub fn to_be_bytes(&self) -> [u8; 2] {
        match self {
            Self::U16(x) => x.to_be_bytes(),
            Self::F16(x) => x.to_be_bytes(),
            Self::I16(x) => x.to_be_bytes(),
            Self::Char(c) => (*c as u16).to_be_bytes(),
        }
    }
}

impl From<u16> for Word {
    fn from(value: u16) -> Self {
        Self::U16(value)
    }
}

impl From<f16> for Word {
    fn from(value: f16) -> Self {
        Self::F16(value)
    }
}

impl From<char> for Word {
    fn from(value: char) -> Self {
        Self::Char(value)
    }
}

impl From<i16> for Word {
    fn from(value: i16) -> Self {
        Self::I16(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StackValues {
    U16(u16),
    I16(i16),
    Pointer(*mut ()),
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum PointerType {
    RawPointer,
    StackValuesPointer,
}

#[derive(Debug)]
pub struct QuarkVM {
    pub stack: [StackValues; MAX_STACK_SIZE],
    pub memory: Vec<u8>,
    pub heap: Vec<StackValues>,
    pub constant_pools: [StackValues; 4096],
    pub call_stack: Vec<u16>,
    pub free_list: Vec<(*mut (), (u16, PointerType))>,
    pub allocated_memory: HashMap<*mut (), (u16, PointerType)>,
    pub sp: i16,
    pub pc: u16,
    pub running: bool,
    pub instructions: Vec<Rc<Instruction>>,
    pub byte_code_file: Option<ByteCodeCompiler>,
    pub fd_table: HashMap<u16, i32>,
    pub dlls: Rc<RefCell<Vec<Library>>>,
}

impl Default for QuarkVM {
    fn default() -> Self {
        Self {
            stack: [StackValues::U16(0); MAX_STACK_SIZE],
            memory: vec![],
            heap: vec![],
            constant_pools: [StackValues::U16(0); 4096],
            call_stack: vec![],
            free_list: vec![],
            allocated_memory: HashMap::new(),
            sp: -1,
            pc: 0,
            running: false,
            instructions: vec![],
            byte_code_file: None,
            fd_table: HashMap::new(),
            dlls: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl QuarkVM {
    pub fn new(byte_code_compiler: ByteCodeCompiler) -> Self {
        Self {
            stack: [StackValues::U16(0); MAX_STACK_SIZE],
            memory: Vec::new(),
            heap: Vec::new(),
            constant_pools: [StackValues::U16(0); 4096],
            call_stack: Vec::new(),
            free_list: Vec::new(),
            allocated_memory: HashMap::new(),
            sp: -1,
            pc: 0,
            running: true,
            fd_table: HashMap::new(),
            dlls: Rc::new(RefCell::new(Vec::new())),
            instructions: vec![],
            byte_code_file: Some(byte_code_compiler),
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
                let cloned: Vec<Instruction> =
                    self.instructions.iter().map(|rc| (**rc).clone()).collect();
                bc.store_file(&cloned)
            }
            None => {
                panic!("QUARMVM: Error while storing to file, bytecode compiler not provided.")
            }
        }
    }

    pub fn load_file(&mut self) {
        match &mut self.byte_code_file {
            Some(bc) => {
                self.instructions = bc.load_file().into_iter().map(|ins| Rc::new(ins)).collect();
            }
            None => {
                panic!("QUARMVM: Error while storing to file, bytecode compiler not provided.")
            }
        }
    }

    pub fn allocate(&mut self, size: u16, pointer_type: PointerType) -> Result<*mut (), ()> {
        for (i, (ptr, (free_size, _))) in self.free_list.iter_mut().enumerate() {
            if *free_size >= size {
                let allocated_start = *ptr;
                *ptr = ptr.wrapping_add(size.into());
                *free_size -= size;
                if *free_size == 0 {
                    self.free_list.remove(i);
                }
                self.allocated_memory
                    .insert(allocated_start, (size, pointer_type));
                println!("RETURNING FROM FREELIST");
                return Ok(allocated_start);
            }
        }

        let starting_index = match pointer_type {
            PointerType::RawPointer => self.memory.len(),
            PointerType::StackValuesPointer => self.heap.len(),
        };

        if pointer_type == PointerType::StackValuesPointer {
            for _ in 0..size {
                self.heap.push(StackValues::U16(0));
            }
            self.allocated_memory.insert(
                &mut self.heap[starting_index] as *mut StackValues as *mut (),
                (size, pointer_type),
            );
            Ok(&mut self.heap[starting_index] as *mut StackValues as *mut ())
        } else {
            for _ in 0..size {
                self.memory.push(0);
            }
            self.allocated_memory.insert(
                &mut self.memory[starting_index] as *mut u8 as *mut (),
                (size, pointer_type),
            );
            Ok(&mut self.memory[starting_index] as *mut u8 as *mut ())
        }
    }

    pub fn deallocate(&mut self, ptr: *mut ()) {
        let removed_value = self.allocated_memory.remove(&ptr);
        if let Some(freed_size) = removed_value {
            self.free_list
                .push((ptr, (freed_size.0, PointerType::RawPointer)));
        }

        self.free_list.sort_by_key(|&(ptr, _)| ptr as usize);

        let mut new_free_list: Vec<(*mut (), (u16, PointerType))> = Vec::new();
        let mut i = 0;

        while i < self.free_list.len() {
            let (start1, (size1, ptr1type)) = &self.free_list[i];

            if i + 1 < self.free_list.len() {
                let (start2, (size2, ptr2type)) = &self.free_list[i + 1];

                if ptr1type == ptr2type {
                    let can_merge = match ptr1type {
                        PointerType::StackValuesPointer => {
                            (*start1 as *mut StackValues).wrapping_add(*size1 as usize)
                                == (*start2 as *mut StackValues)
                        }
                        PointerType::RawPointer => {
                            (*start1 as *mut u8).wrapping_add(*size1 as usize)
                                == (*start2 as *mut u8)
                        }
                    };

                    if can_merge {
                        new_free_list.push((*start1, (*size1 + *size2, *ptr1type)));
                        i += 2;
                        continue;
                    }
                }
            }

            new_free_list.push((*start1, (*size1, *ptr1type)));
            i += 1;
        }

        self.free_list = new_free_list;
    }

    pub fn print(s: StackValues) {
        if let StackValues::Pointer(v) = s {
            unsafe { println!("{:?}", *v) };
        } else {
            println!("{:?}", s);
        }
    }

    pub fn determine_function(&mut self) {
        match self.instructions[self.pc as usize].tt {
            InstructionType::INST_ADD => {
                let t = self.pop_stack();
                if let StackValues::U16(a) = t {
                    let y = self.pop_stack();
                    if let StackValues::U16(b) = y {
                        self.push_stack(StackValues::U16(a + b));
                    } else if let StackValues::Pointer(b) = y {
                        for (&ptr, &(size, ptr_type)) in self.allocated_memory.clone().iter() {
                            unsafe {
                                let adjusted_ptr =
                                    (b as *mut u8).wrapping_add(a as usize) as *mut ();
                                if ptr_type == PointerType::RawPointer {
                                    if (adjusted_ptr as *mut u8) >= (ptr as *mut u8)
                                        && (adjusted_ptr as *mut u8)
                                            <= (ptr as *mut u8).add(size as usize)
                                    {
                                        self.push_stack(StackValues::Pointer(adjusted_ptr));
                                        break;
                                    }
                                } else if ptr_type == PointerType::StackValuesPointer
                                    && (b as *mut StackValues) >= (ptr as *mut StackValues)
                                    && (b as *mut StackValues)
                                        < (ptr as *mut StackValues).add(size as usize)
                                {
                                    self.push_stack(StackValues::Pointer(
                                        (b as *mut StackValues).wrapping_add(a as usize) as *mut (),
                                    ));
                                    break;
                                }
                            }
                        }
                    }
                } else if let StackValues::I16(a) = t {
                    if let StackValues::I16(b) = self.pop_stack() {
                        self.push_stack(StackValues::I16(a + b));
                    }
                } else if let StackValues::Pointer(a) = t {
                    if let StackValues::U16(b) = self.pop_stack() {
                        for (&ptr, &(size, ptr_type)) in self.allocated_memory.iter() {
                            unsafe {
                                let adjusted_ptr =
                                    (a as *mut u8).wrapping_add(b as usize) as *mut ();

                                if ptr_type == PointerType::RawPointer {
                                    if (adjusted_ptr as *mut u8) >= (ptr as *mut u8)
                                        && (adjusted_ptr as *mut u8)
                                            < (ptr as *mut u8).add(size as usize)
                                    {
                                        self.push_stack(StackValues::Pointer(adjusted_ptr));
                                        return;
                                    }
                                } else if ptr_type == PointerType::StackValuesPointer
                                    && (adjusted_ptr as *mut StackValues)
                                        >= (ptr as *mut StackValues)
                                    && (adjusted_ptr as *mut StackValues)
                                        < (ptr as *mut StackValues).add(size as usize)
                                {
                                    self.push_stack(StackValues::Pointer(adjusted_ptr));
                                    return;
                                }
                            }
                        }
                    }
                } else {
                    println!("WTF IS A : {:?}", t);
                }
                self.pc += 1;
            }
            InstructionType::INST_PUSH => match &self.instructions[self.pc as usize].values {
                Some(value) => {
                    if let Word::U16(v) = value[0] {
                        self.push_stack(StackValues::U16(v));
                    } else if let Word::I16(v) = value[0] {
                        self.push_stack(StackValues::I16(v));
                    }
                    self.pc += 1;
                }
                None => {
                    panic!(
                        "QUARMVM: does not have a value to push {:?}",
                        self.instructions[self.pc as usize]
                    );
                }
            },
            InstructionType::INST_POP => {
                self.pop_stack();
                self.pc += 1;
            }
            InstructionType::INST_MUL => {
                if let StackValues::U16(a) = self.pop_stack() {
                    if let StackValues::U16(b) = self.pop_stack() {
                        self.push_stack(StackValues::U16(a * b));
                    }
                } else if let StackValues::I16(a) = self.pop_stack() {
                    if let StackValues::I16(b) = self.pop_stack() {
                        self.push_stack(StackValues::I16(a * b));
                    }
                }
                self.pc += 1;
            }
            InstructionType::INST_DIV => {
                if let StackValues::U16(a) = self.pop_stack() {
                    if let StackValues::U16(b) = self.pop_stack() {
                        self.push_stack(StackValues::U16(a / b));
                    }
                } else if let StackValues::I16(a) = self.pop_stack() {
                    if let StackValues::I16(b) = self.pop_stack() {
                        self.push_stack(StackValues::I16(a / b));
                    }
                }
                self.pc += 1;
            }
            InstructionType::INST_SUB => {
                if let StackValues::U16(a) = self.pop_stack() {
                    if let StackValues::U16(b) = self.pop_stack() {
                        if (b as i16 - a as i16) < 0 {
                            self.push_stack(StackValues::U16(0));
                        } else {
                            self.push_stack(StackValues::U16(b - a));
                        }
                    }
                } else if let StackValues::I16(a) = self.pop_stack() {
                    if let StackValues::I16(b) = self.pop_stack() {
                        self.push_stack(StackValues::I16(b - a));
                    }
                } else if let StackValues::Pointer(a) = self.pop_stack() {
                    if let StackValues::U16(b) = self.pop_stack() {
                        if self.allocated_memory[&a].1 == PointerType::StackValuesPointer {
                            self.push_stack(StackValues::Pointer(
                                (a as *mut StackValues).wrapping_sub(b as usize) as *mut (),
                            ));
                        } else {
                            self.push_stack(StackValues::Pointer(
                                (a as *mut u8).wrapping_sub(b as usize) as *mut (),
                            ));
                        }
                    }
                }
                self.pc += 1;
            }
            InstructionType::INST_AND => {
                if let StackValues::U16(a) = self.pop_stack() {
                    if let StackValues::U16(b) = self.pop_stack() {
                        self.push_stack(StackValues::U16(a & b));
                    }
                } else if let StackValues::I16(a) = self.pop_stack() {
                    if let StackValues::I16(b) = self.pop_stack() {
                        self.push_stack(StackValues::I16(a & b));
                    }
                }
                self.pc += 1;
            }
            InstructionType::INST_OR => {
                if let StackValues::U16(a) = self.pop_stack() {
                    if let StackValues::U16(b) = self.pop_stack() {
                        self.push_stack(StackValues::U16(a | b));
                    }
                } else if let StackValues::I16(a) = self.pop_stack() {
                    if let StackValues::I16(b) = self.pop_stack() {
                        self.push_stack(StackValues::I16(a | b));
                    }
                }
                self.pc += 1;
            }
            InstructionType::INST_XOR => {
                if let StackValues::U16(a) = self.pop_stack() {
                    if let StackValues::U16(b) = self.pop_stack() {
                        self.push_stack(StackValues::U16(a ^ b));
                    }
                } else if let StackValues::I16(a) = self.pop_stack() {
                    if let StackValues::I16(b) = self.pop_stack() {
                        self.push_stack(StackValues::I16(a ^ b));
                    }
                }
                self.pc += 1;
            }
            InstructionType::INST_NOT => {
                if let StackValues::U16(a) = self.pop_stack() {
                    self.push_stack(StackValues::U16(!a));
                }
                self.pc += 1;
            }
            InstructionType::INST_SHL => {
                if let StackValues::U16(a) = self.pop_stack() {
                    if let StackValues::U16(b) = self.pop_stack() {
                        self.push_stack(StackValues::U16(a << b));
                    }
                } else if let StackValues::I16(a) = self.pop_stack() {
                    if let StackValues::I16(b) = self.pop_stack() {
                        self.push_stack(StackValues::I16(a << b));
                    }
                }
                self.pc += 1;
            }
            InstructionType::INST_SHR => {
                if let StackValues::U16(a) = self.pop_stack() {
                    if let StackValues::U16(b) = self.pop_stack() {
                        self.push_stack(StackValues::U16(a >> b));
                    }
                } else if let StackValues::I16(a) = self.pop_stack() {
                    if let StackValues::I16(b) = self.pop_stack() {
                        self.push_stack(StackValues::I16(a >> b));
                    }
                }
                self.pc += 1;
            }
            InstructionType::INST_JMPZ => {
                if let Some(t) = match self.stack[self.sp as usize] {
                    StackValues::U16(v) => Some(v as i16),
                    StackValues::I16(v) => Some(v),
                    _ => None,
                } {
                    if t == 0 {
                        match &self.instructions[self.pc as usize].values {
                            Some(value) => {
                                if let Word::U16(v) = value[0] {
                                    self.pc = v;
                                } else if let Word::I16(v) = value[0] {
                                    self.pc = v as u16;
                                } else {
                                    self.pc += 1;
                                }
                            }
                            None => {
                                panic!(
                                    "QUARMVM: does not have a value to push {:?}",
                                    self.instructions[self.pc as usize]
                                );
                            }
                        }
                    } else {
                        self.pc += 1;
                    }
                }
            }
            InstructionType::INST_JMPEQ => {
                if let Some(x) = match self.stack[self.sp as usize] {
                    StackValues::U16(x) => Some(x as i16),
                    StackValues::I16(x) => Some(x),
                    _ => None,
                } {
                    if let Some(y) = match self.stack[self.sp as usize - 1] {
                        StackValues::U16(y) => Some(y as i16),
                        StackValues::I16(y) => Some(y),
                        _ => None,
                    } {
                        if x == y {
                            match &self.instructions[self.pc as usize].values {
                                Some(value) => {
                                    if let Word::U16(v) = value[0] {
                                        self.pc = v;
                                    } else if let Word::I16(v) = value[0] {
                                        self.pc = v as u16;
                                    } else {
                                        self.pc += 1;
                                    }
                                }
                                None => {
                                    panic!(
                                        "QUARMVM: does not have a value to push {:?}",
                                        self.instructions[self.pc as usize]
                                    );
                                }
                            }
                        } else {
                            self.pc += 1;
                        }
                    }
                }
            }
            InstructionType::INST_JMPNEQ => {
                if let Some(x) = match self.stack[self.sp as usize] {
                    StackValues::U16(x) => Some(x as i16),
                    StackValues::I16(x) => Some(x),
                    _ => None,
                } {
                    if let Some(y) = match self.stack[self.sp as usize - 1] {
                        StackValues::U16(y) => Some(y as i16),
                        StackValues::I16(y) => Some(y),
                        _ => None,
                    } {
                        if x != y {
                            match &self.instructions[self.pc as usize].values {
                                Some(value) => {
                                    if let Word::U16(v) = value[0] {
                                        self.pc = v;
                                    } else if let Word::I16(v) = value[0] {
                                        self.pc = v as u16;
                                    } else {
                                        self.pc += 1;
                                    }
                                }
                                None => {
                                    panic!(
                                        "QUARMVM: does not have a value to push {:?}",
                                        self.instructions[self.pc as usize]
                                    );
                                }
                            }
                        } else {
                            self.pc += 1;
                        }
                    }
                }
            }
            InstructionType::INST_JMPNZ => {
                if let Some(x) = match self.stack[self.sp as usize] {
                    StackValues::U16(x) => Some(x as i16),
                    StackValues::I16(x) => Some(x),
                    _ => None,
                } {
                    if x != 0 {
                        match &self.instructions[self.pc as usize].values {
                            Some(value) => {
                                if let Word::U16(v) = value[0] {
                                    self.pc = v;
                                } else if let Word::I16(v) = value[0] {
                                    self.pc = v as u16;
                                } else {
                                    self.pc += 1;
                                }
                            }
                            None => {
                                panic!(
                                    "QUARMVM: does not have a value to push {:?}",
                                    self.instructions[self.pc as usize]
                                );
                            }
                        }
                    } else {
                        self.pc += 1;
                    }
                }
            }
            InstructionType::INST_PUSH_STR => {
                if let Some(args) = &self.instructions[self.pc as usize].values {
                    let str_len = match args[0] {
                        Word::U16(l) => l,
                        _ => panic!("QUARMVM: Expected a u16 as a string size"),
                    };

                    let mut str_buffer: Vec<StackValues> = Vec::with_capacity(str_len as usize + 1);
                    for i in 0..str_len as usize {
                        if let Word::Char(c) =
                            args.get(1 + i).expect("QUARMVM: Invalid string index")
                        {
                            str_buffer.push(StackValues::U16(*c as u16));
                        } else {
                            panic!("QUARMVM: Expected a Char in string");
                        }
                    }

                    str_buffer.push(StackValues::U16('\0' as u16));
                    let starting_pointer =
                        self.allocate(str_len + 1, PointerType::StackValuesPointer);
                    if let Ok(ptr) = starting_pointer {
                        for i in 0..(str_len + 1) {
                            unsafe {
                                let dest = (ptr as *mut StackValues).add(i as usize);
                                *dest.cast::<StackValues>() = str_buffer[i as usize];
                            }
                        }
                        self.push_stack(StackValues::Pointer(ptr));
                    }
                }

                self.pc += 1;
            }

            InstructionType::INST_NOOP => {
                self.pc += 1;
            }

            InstructionType::INST_SYSCALL => {
                if let StackValues::U16(syscall_num) = self.pop_stack() {
                    let mut args: [usize; 6] = [0; 6];

                    if let Some(t_values) = &self.instructions[self.pc as usize].values {
                        if let Word::U16(len) = t_values[0] {
                            (0..(len as usize)).for_each(|i| match self.pop_stack() {
                                StackValues::U16(v) => args[i] = v as usize,
                                StackValues::Pointer(v) => args[i] = v as usize,
                                _ => {}
                            });
                        }
                    }

                    let result: *mut u8;
                    unsafe {
                        asm!(
                            "syscall",
                            in("rax") syscall_num as usize,
                            in("rdi") args[0],
                            in("rsi") args[1],
                            in("rdx") args[2],
                            in("r10") args[3],
                            in("r8")  args[4],
                            in("r9")  args[5],
                            lateout("rax") result,
                        );
                    }
                    self.push_stack(StackValues::Pointer(result as *mut ()));
                }
                self.pc += 1;
            }

            InstructionType::INST_ALLOC => {
                if let Some(values) = &self.instructions[self.pc as usize].values {
                    if let Word::U16(size) = values[0] {
                        if let Ok(ptr) = self.allocate(size, PointerType::StackValuesPointer) {
                            self.push_stack(StackValues::Pointer(ptr));
                        }
                    }
                }
                self.pc += 1;
            }

            InstructionType::INST_ALLOC_RAW => {
                if let Some(values) = &self.instructions[self.pc as usize].values {
                    if let Word::U16(size) = values[0] {
                        if let Ok(ptr) = self.allocate(size, PointerType::RawPointer) {
                            self.push_stack(StackValues::Pointer(ptr));
                        }
                    }
                }
                self.pc += 1;
            }

            InstructionType::INST_DUP => {
                let value = self.pop_stack();
                self.push_stack(value);
                self.push_stack(value);
                self.pc += 1;
            }

            InstructionType::INST_INSWAP => {
                if let Some(value) = &self.instructions[self.pc as usize].values {
                    if let Word::U16(index) = value[0] {
                        self.stack.swap(
                            self.sp as usize,
                            self.sp.wrapping_sub(index as i16) as usize,
                        );
                    }
                }
                self.pc += 1;
            }

            InstructionType::INST_PRINT => {
                Self::print(self.stack[self.sp as usize]);
                self.pc += 1;
            }

            InstructionType::INST_STORE => {
                if let Some(value) = &self.instructions[self.pc as usize].values {
                    if let Word::U16(index) = value[0] {
                        self.constant_pools[index as usize] = self.pop_stack();
                    }
                }
                self.pc += 1;
            }

            InstructionType::INST_LOAD => {
                if let Some(value) = &self.instructions[self.pc as usize].values {
                    if let Word::U16(index) = value[0] {
                        self.push_stack(self.constant_pools[index as usize]);
                    }
                }
                self.pc += 1;
            }

            InstructionType::INST_DEREF => {
                let stack_ptr = self.pop_stack();
                if let StackValues::Pointer(x) = stack_ptr {
                    for (&ptr, &(size, ptr_type)) in self.allocated_memory.clone().iter() {
                        unsafe {
                            if ptr_type == PointerType::RawPointer
                                && (x as *mut u8) >= (ptr as *mut u8)
                                && (x as *mut u8) < (ptr as *mut u8).add(size as usize)
                            {
                                if let Ok(all_ptr) =
                                    self.allocate(size, PointerType::StackValuesPointer)
                                {
                                    for i in 0..size {
                                        let raw_byte = *((ptr as *mut u8).wrapping_add(i as usize));
                                        *((all_ptr as *mut StackValues).wrapping_add(i as usize)) =
                                            StackValues::U16(raw_byte as u16);
                                    }
                                    self.push_stack(StackValues::Pointer(all_ptr));
                                }
                                break;
                            } else if ptr_type == PointerType::StackValuesPointer
                                && (x as *mut StackValues) >= (ptr as *mut StackValues)
                                && (x as *mut StackValues)
                                    < (ptr as *mut StackValues).add(size as usize)
                            {
                                self.push_stack(*(x as *mut StackValues));
                                break;
                            }
                        }
                    }
                }
                self.pc += 1;
            }

            InstructionType::INST_REF => {
                let mut value = self.pop_stack();
                let _s = std::mem::size_of::<StackValues>();
                let v = StackValues::Pointer(&mut value as *mut _ as *mut ());
                self.push_stack(value);
                self.push_stack(v);
                self.pc += 1;
            }

            InstructionType::INST_DEBUG => {
                self.debug_stack();
                self.pc += 1;
            }

            InstructionType::INST_CALL => {
                if let Some(value) = &self.instructions[self.pc as usize].values {
                    if let Word::U16(index) = value[0] {
                        self.call_stack.push(self.pc + 1);
                        self.pc = index;
                    }
                }
            }

            InstructionType::INST_RET => {
                if let Some(to_return) = self.call_stack.pop() {
                    self.pc = to_return;
                }
            }

            InstructionType::INST_PUT => {
                if let StackValues::Pointer(ptr) = self.pop_stack() {
                    unsafe {
                        println!("ptr: {:?}", ptr);
                        let (_, ptr_type) = self
                            .allocated_memory
                            .get(&ptr)
                            .expect("QUARMVM: Error while getting info for pointer.");
                        if *ptr_type == PointerType::StackValuesPointer {
                            *(ptr as *mut StackValues) = self.pop_stack();
                        } else if let StackValues::U16(v) = self.pop_stack() {
                            let lsb = (v & 0xFF) as u8;
                            let msb = (v >> 8) as u8;
                            *(ptr as *mut u8) = lsb;
                            if msb != 0 {
                                *((ptr as *mut u8).wrapping_add(1)) = lsb;
                            }
                        }
                    }
                }
            }

            InstructionType::INST_STD_SYSCALL => {
                if let StackValues::U16(syscall_num) = self.pop_stack() {
                    let mut args: VecDeque<StackValues> = VecDeque::new();

                    if let Some(t_values) = &self.instructions[self.pc as usize].values {
                        if let Word::U16(len) = t_values[0] {
                            (0..(len as usize)).for_each(|i| {
                                self.debug_stack();
                                args.push_back(self.pop_stack());
                            });
                        }
                    }

                    self.std_syscall_match(syscall_num, args);
                    self.pc += 1;
                } else {
                    panic!("QUARKVM: Expected a u16");
                }
            }

            InstructionType::INST_DLL_LOAD => {
                if let StackValues::Pointer(str_ptr) = self.pop_stack() {
                    let dll_path = self.get_str_from_ptr(str_ptr);
                    unsafe {
                        let lib = Library::new(dll_path.as_str());
                        if let Ok(loaded_lib) = lib {
                            self.dlls.borrow_mut().push(loaded_lib);
                            self.push_stack(StackValues::U16((self.dlls.clone().borrow().len() - 1) as u16));
                        } else {
                            dbg!(&lib);
                            panic!("{:?}", lib);
                        }
                    }
                }
                self.pc += 1;
            }

            InstructionType::INST_DLL_CALL => {
                let maybe_value = self.instructions[self.pc as usize].values.clone();
                let dlls = self.dlls.clone();
                let mut mut_ref_dlls = dlls.borrow_mut();

                if let Some(value) = maybe_value {
                    if let StackValues::U16(stack_value_holding_dll) = self.pop_stack() {
                        let dll = mut_ref_dlls.get_mut(stack_value_holding_dll as usize)
                            .expect("QUARKVM: No such DLL Registered");

                        if let Word::U16(number_of_args) = value.first().expect("QUARKVM: No method name passed") {
                            if let StackValues::Pointer(method_name) = self.pop_stack() {
                                let method_name_string = self.get_str_from_ptr(method_name);

                                let mut arguments: Vec<libffi::middle::Arg> = vec![];
                                let mut argument_types: Vec<libffi::middle::Type> = vec![];
                                let mut arg_storage: Vec<Box<dyn std::any::Any>> = vec![];

                                for _ in 0..(*number_of_args) {
                                    let arg_type = match self.pop_stack() {
                                        StackValues::U16(u) => {
                                            let u = Box::new(u);
                                            arguments.push(libffi::middle::Arg::new(&*u));
                                            arg_storage.push(u);
                                            libffi::middle::Type::c_uint()
                                        }
                                        StackValues::I16(u) => {
                                            let u = Box::new(u);
                                            arguments.push(libffi::middle::Arg::new(&*u));
                                            arg_storage.push(u);
                                            libffi::middle::Type::c_int()
                                        }
                                        StackValues::Pointer(u) => {
                                            let u = Box::new(u);
                                            arguments.push(libffi::middle::Arg::new(&*u));
                                            arg_storage.push(u);
                                            libffi::middle::Type::pointer()
                                        }
                                    };
                                    argument_types.push(arg_type);
                                }
                                let method_handle = unsafe { dll.get(method_name_string.as_bytes()) }
                                    .expect("QUARKVM: Can not load the method");

                                let cif = libffi::middle::Cif::new(argument_types, libffi::middle::Type::pointer());
                                let cp = libffi::middle::CodePtr::from_fun(*method_handle);
                                unsafe {
                                    let output: *mut () = cif.call(cp, &arguments);
                                    self.push_stack(StackValues::Pointer(output));
                                    self.pc += 1;
                                }
                            }
                        }
                    }
                }
            },
            InstructionType::INST_DEREF_FOREIGN => {}
        }
    }

    pub fn get_str_from_ptr(&self, str_ptr: *mut ()) -> String {
        for (&ptr, &(size, ptr_type)) in self.allocated_memory.clone().iter() {
            unsafe {
                if ptr_type == PointerType::RawPointer
                    && (str_ptr as *mut u8) >= (ptr as *mut u8)
                    && (str_ptr as *mut u8) < (ptr as *mut u8).add(size as usize)
                {
                    let mut dup_str_ptr = str_ptr as *mut u8;
                    let mut s = String::new();
                    while *dup_str_ptr as char != '\0' {
                        s.push(*dup_str_ptr as char);
                        dup_str_ptr = dup_str_ptr.wrapping_add(1);
                    }
                    return s;
                } else if ptr_type == PointerType::StackValuesPointer
                    && (str_ptr as *mut StackValues) >= (ptr as *mut StackValues)
                    && (str_ptr as *mut StackValues) < (ptr as *mut StackValues).add(size as usize)
                {
                    let mut dup_str_ptr = str_ptr as *mut StackValues;
                    if let StackValues::U16(mut x) = *dup_str_ptr {
                        let mut s = String::new();
                        while char::from_u32(x.into())
                            .expect("QUARKVM: Error can not convert to char.")
                            != '\0'
                        {
                            s.push(char::from_u32(x.into()).expect("UNREACHABLE"));
                            dup_str_ptr = dup_str_ptr.wrapping_add(1);
                            if let StackValues::U16(x_new) = *dup_str_ptr {
                                x = x_new;
                            }
                        }
                        return s;
                    }
                }
            }
        }

        panic!("QUARKVM: Can't find string");
    }

    pub fn std_syscall_match(&mut self, id: u16, mut args: VecDeque<StackValues>) {
        match id {
            0 => {
                // SYSCALL to Exit.
                if let StackValues::U16(exit_code) = self.pop_stack() {
                    exit(exit_code.into());
                }
            }
            1 => {
                // Get FILE fd
                if let StackValues::Pointer(mut file_name) =
                    args.pop_front().expect("QUARMVM: Expected a file name")
                {
                    let mut name = String::new();
                    unsafe {
                        if let StackValues::U16(mut current_char) = *(file_name as *mut StackValues)
                        {
                            while char::from_u32(current_char.into())
                                .expect("QUARKVM: Not a character while reading a file path")
                                != '\0'
                            {
                                name.push(
                                    char::from_u32(current_char.into()).expect("UNREACHABLE"),
                                );
                                file_name =
                                    (file_name as *mut StackValues).wrapping_add(1) as *mut ();
                                if let StackValues::U16(next_char) =
                                    *(file_name as *mut StackValues)
                                {
                                    current_char = next_char;
                                }
                            }
                        }
                    }
                    let f = File::open(name);
                    if let Ok(ff) = f {
                        dbg!(&ff);
                        self.push_stack(StackValues::I16(ff.as_raw_fd() as i16));
                    } else {
                        panic!("QUARMVM: Error while opening the file: {:?}", f)
                    }
                }
            }
            _ => {}
        }
    }

    pub fn debug_stack(&self) {
        println!("______________________________________________________________________");
        println!(
            "SP: {:?} stack: {:?} pc: {:?}",
            self.sp,
            &self.stack[0..(self.sp.max(0) as usize + 1)],
            self.pc
        );
        println!("MEMORY: {:?}", self.memory);
        println!("HEAP: {:?}", self.heap);
        println!("ALLOCATIONS: {:?}", self.allocated_memory);
        println!("DLLs: {:?}", self.dlls);
        println!("______________________________________________________________________");
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

#[derive(Debug, Copy, Clone)]
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
    INST_ALLOC,
    INST_ALLOC_RAW,
    INST_SYSCALL,
    INST_DUP,
    INST_INSWAP,
    INST_PRINT,
    INST_LOAD,
    INST_STORE,
    INST_DEREF,
    INST_DEREF_FOREIGN,
    INST_REF,
    INST_DEBUG,
    INST_CALL,
    INST_RET,
    INST_PUT,
    INST_STD_SYSCALL,
    INST_DLL_LOAD,
    INST_DLL_CALL,
}

impl Default for InstructionType {
    fn default() -> Self {
        Self::INST_NOOP
    }
}

impl TryFrom<u8> for InstructionType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x <= 35 => Ok(unsafe { std::mem::transmute(x) }),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub tt: InstructionType,
    pub values: Option<Vec<Word>>,
}

impl Instruction {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![];
        buffer.push(self.tt as u8);
        match &self.values {
            Some(value) => {
                buffer.push(value.iter().len() as u8);
            }
            None => {
                buffer.push(0);
            }
        }
        if let Some(values) = &self.values {
            for value in values {
                println!("{:?}", value);
                match value {
                    Word::U16(_) => buffer.push(0),
                    Word::F16(_) => buffer.push(1),
                    Word::Char(_) => buffer.push(2),
                    Word::I16(_) => buffer.push(3),
                }
                buffer.extend_from_slice(&value.to_be_bytes());
            }
        }
        buffer
    }
}

pub fn DEFINE_PUSH(x: u16) -> Instruction {
    Instruction {
        tt: InstructionType::INST_PUSH,
        values: Some(vec![Word::from(x)]),
    }
}

pub fn DEFINE_POP() -> Instruction {
    Instruction {
        tt: InstructionType::INST_POP,
        values: None,
    }
}

pub fn DEFINE_ADD() -> Instruction {
    Instruction {
        tt: InstructionType::INST_ADD,
        values: None,
    }
}

pub fn DEFINE_MUL() -> Instruction {
    Instruction {
        tt: InstructionType::INST_MUL,
        values: None,
    }
}

pub fn DEFINE_DIV() -> Instruction {
    Instruction {
        tt: InstructionType::INST_DIV,
        values: None,
    }
}

pub fn DEFINE_SUB() -> Instruction {
    Instruction {
        tt: InstructionType::INST_SUB,
        values: None,
    }
}

pub fn DEFINE_JMPZ(x: i16) -> Instruction {
    Instruction {
        tt: InstructionType::INST_JMPZ,
        values: Some(vec![Word::from(x)]),
    }
}

pub fn DEFINE_JMPEQ(x: i16) -> Instruction {
    Instruction {
        tt: InstructionType::INST_JMPEQ,
        values: Some(vec![Word::from(x)]),
    }
}

pub fn DEFINE_JMPNZ(x: i16) -> Instruction {
    Instruction {
        tt: InstructionType::INST_JMPNZ,
        values: Some(vec![Word::from(x)]),
    }
}

pub fn DEFINE_PUSH_STR(x: &str) -> Instruction {
    let mut values = vec![Word::from(x.len() as u16)];
    values.extend(x.chars().map(|c| Word::from(c)));
    Instruction {
        tt: InstructionType::INST_PUSH_STR,
        values: Some(values),
    }
}

pub fn DEFINE_SYSCALL(x: u16) -> Instruction {
    Instruction {
        tt: InstructionType::INST_SYSCALL,
        values: Some(vec![Word::from(x)]),
    }
}

pub fn DEFINE_ALLOC(x: u16) -> Instruction {
    Instruction {
        tt: InstructionType::INST_ALLOC,
        values: Some(vec![Word::from(x)]),
    }
}

pub fn DEFINE_ALLOC_RAW(x: u16) -> Instruction {
    Instruction {
        tt: InstructionType::INST_ALLOC_RAW,
        values: Some(vec![Word::from(x)]),
    }
}

pub fn DEFINE_DUP() -> Instruction {
    Instruction {
        tt: InstructionType::INST_DUP,
        values: None,
    }
}

pub fn DEFINE_INSWAP(x: u16) -> Instruction {
    Instruction {
        tt: InstructionType::INST_INSWAP,
        values: Some(vec![Word::from(x)]),
    }
}

pub fn DEFINE_PRINT() -> Instruction {
    Instruction {
        tt: InstructionType::INST_PRINT,
        values: None,
    }
}

pub fn DEFINE_LOAD(x: Option<u16>) -> Instruction {
    match x {
        Some(x) => Instruction {
            tt: InstructionType::INST_LOAD,
            values: Some(vec![Word::from(x)]),
        },
        None => Instruction {
            tt: InstructionType::INST_LOAD,
            values: None,
        },
    }
}

pub fn DEFINE_STORE(x: u16) -> Instruction {
    Instruction {
        tt: InstructionType::INST_STORE,
        values: Some(vec![Word::from(x)]),
    }
}

pub fn DEFINE_DEREF() -> Instruction {
    Instruction {
        tt: InstructionType::INST_DEREF,
        values: None,
    }
}

pub fn DEFINE_DEREF_FOREIGN(x: u16) -> Instruction {
    Instruction {
        tt: InstructionType::INST_DEREF,
        values: Some(vec![Word::from(x)]),
    }
}

pub fn DEFINE_REF() -> Instruction {
    Instruction {
        tt: InstructionType::INST_REF,
        values: None,
    }
}

pub fn DEBUG() -> Instruction {
    Instruction {
        tt: InstructionType::INST_DEBUG,
        values: None,
    }
}

pub fn DEFINE_CALL(x: u16) -> Instruction {
    Instruction {
        tt: InstructionType::INST_CALL,
        values: Some(vec![Word::from(x)]),
    }
}

pub fn DEFINE_RET() -> Instruction {
    Instruction {
        tt: InstructionType::INST_RET,
        values: None,
    }
}

pub fn DEFINE_PUT() -> Instruction {
    Instruction {
        tt: InstructionType::INST_PUT,
        values: None,
    }
}

pub fn DEFINE_STD_SYSCALL(x: u16) -> Instruction {
    Instruction {
        tt: InstructionType::INST_STD_SYSCALL,
        values: Some(vec![Word::from(x)]),
    }
}

pub fn DEFINE_DLL_LOAD() -> Instruction {
    Instruction {
        tt: InstructionType::INST_DLL_LOAD,
        values: None,
    }
}

pub fn DEFINE_DLL_CALL(x: &str) -> Instruction {
    let mut values = vec![Word::from(x.len() as u16)];
    values.extend(x.chars().map(|c| Word::from(c)));
    Instruction {
        tt: InstructionType::INST_DLL_CALL,
        values: Some(values),
    }
}
