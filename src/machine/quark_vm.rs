use crate::machine::machine_types::*;
use core::arch::asm;
use std::{collections::HashMap, error::Error};

use super::bytecode::ByteCodeCompiler;
const MAX_STACK_SIZE: usize = 4096;

impl QuarkVM {
    pub fn new(byte_code_compiler: ByteCodeCompiler) -> Self {
        Self {
            stack: [StackValues::U16(0); MAX_STACK_SIZE],
            str_stack: Vec::new(),
            memory: Vec::new(),
            free_list: Vec::new(),
            allocated_memory: HashMap::new(),
            sp: -1,
            pc: 0,
            running: true,

            instructions: vec![
                DEFINE_ALLOC(4),
                DEFINE_PUSH(201),
                DEFINE_SYSCALL(1)
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

    pub fn allocate(&mut self, size: u16) -> Result<*const u16, ()> {
        for (i, (ptr, free_size)) in self.free_list.iter_mut().enumerate() {
            if *free_size >= size {
                let allocated_start = *ptr;
                *ptr = ptr.wrapping_add(size.into());
                *free_size -= size;
                if *free_size == 0 {
                    self.free_list.remove(i);
                }
                self.allocated_memory.insert(allocated_start, size);
                println!("RETURNING FROM FREELIST");
                return Ok(allocated_start);
            }
        }
        let starting_index = self.memory.len();
        for _ in 0..size {
            self.memory.push(0);
        }
        println!("RETURNING NOT FROM FREELIST: {:?}", &self.memory[starting_index] as *const u16);
        self.allocated_memory.insert(&self.memory[starting_index] as *const u16, size);
        Ok(&self.memory[starting_index] as *const u16)
    }

    pub fn deallocate(&mut self, ptr: *const u16) {
        let removed_value = self.allocated_memory.remove(&ptr);
        if let Some(freed_size) = removed_value {
            self.free_list.push((ptr, freed_size));
        }
        
        let mut i = 0;
        while i < self.free_list.len() - 1 {
            let (start1, size1) = self.free_list[i];
            let (start2, size2) = self.free_list[i + 1];

            if start1.wrapping_add(size1.into()) == start2 {
                // Merge blocks
                self.free_list[i] = (start1, size1 + size2);
                self.free_list.remove(i + 1);
            } else {
                i += 1;
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

                    let mut str_buffer: Vec<u16> = Vec::with_capacity(str_len as usize + 1);

                    for i in 0..str_len as usize {
                        if let Word::Char(c) = args.get(1 + i).expect("QUARMVM: Invalid string index") {
                            str_buffer.push(*c as u16);
                        } else {
                            panic!("QUARMVM: Expected a Char in string");
                        }
                    }

                    str_buffer.push(0);

                    self.str_stack.extend_from_slice(&str_buffer);
                    let string_pointer = &self.str_stack[starting_index] as *const u16;

                    println!("PUSHING STRING AT INDEX: {:?}", string_pointer);

                    self.push_stack(StackValues::Pointer(string_pointer));
                }

                self.pc += 1;
            }
            InstructionType::INST_NOOP => {
                self.pc += 1;
            },
            InstructionType::INST_SYSCALL => {
                if let StackValues::U16(syscall_num) = self.pop_stack() {
                    let mut args: [usize; 6] = [0; 6];

                    if let Some(t_values) = &self.instructions[self.pc as usize].values {
                        if let Word::U16(len) = t_values[0] {
                            (0..(len as usize)).for_each(|i| {
                                match self.pop_stack() {
                                    StackValues::U16(v) => args[i] = v as usize,
                                    StackValues::Pointer(v) => args[i] = v as usize,
                                    _ => {}
                                }
                            });
                        }
                    }

                    let result: isize;
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

                    println!("Syscall result: {}", result);

                    self.push_stack(StackValues::U16(result as u16));
                }
                self.pc += 1;
            }
            InstructionType::INST_ALLOC => {
                if let Some(values) = &self.instructions[self.pc as usize].values {
                    if let Word::U16(size) = values[0] {
                        if let Ok(ptr) = self.allocate(size) {
                            self.push_stack(StackValues::Pointer(ptr));
                        }
                    }
                }
                self.pc += 1;
            }
        }
    }

    pub fn debug_stack(&self) {
        println!("sp: {:?} stack: {:?}", self.sp, &self.stack[0..(self.sp as usize + 1)]);
        println!("STRING STACK: {:?}", self.str_stack);
        println!("MEMORY: {:?}", self.memory);
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
