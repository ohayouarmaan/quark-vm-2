use crate::machine::machine_types::*;
use core::arch::asm;
use std::collections::HashMap;

use super::bytecode::ByteCodeCompiler;
const MAX_STACK_SIZE: usize = 4096;

impl QuarkVM {
    pub fn new(byte_code_compiler: ByteCodeCompiler) -> Self {
        Self {
            stack: [StackValues::U16(0); MAX_STACK_SIZE],
            memory: Vec::new(),
            heap: Vec::new(),
            constant_pools: [StackValues::U16(0); 4096],
            free_list: Vec::new(),
            allocated_memory: HashMap::new(),
            sp: -1,
            pc: 0,
            running: true,

            instructions: vec![
                DEFINE_PUSH(10),      // n (change this for different Fibonacci numbers)
                DEFINE_STORE(0),      // Store n in memory[0]
                DEFINE_PUSH(0),       // Base case: Fib(0) = 0
                DEFINE_STORE(1),      // Store Fib(0) in memory[1]
                DEFINE_LOAD(1),       // Load Fib(0)
                DEFINE_PRINT(),       // Print Fib(0)
                DEFINE_PUSH(1),       // Base case: Fib(1) = 1
                DEFINE_STORE(2),      // Store Fib(1) in memory[2]
                DEFINE_LOAD(2),       // Load Fib(1)
                DEFINE_PRINT(),       // Print Fib(1)
                DEFINE_LOAD(0),       // Load n
                DEFINE_PUSH(2),       // Push 2
                DEFINE_SUB(),         // n - 2
                DEFINE_JMPZ(20),      // If n <= 1, jump to END (index 20)
                DEFINE_LOAD(1),       // Load Fib(n-1)
                DEFINE_LOAD(2),       // Load Fib(n-2)
                DEFINE_ADD(),         // Fib(n) = Fib(n-1) + Fib(n-2)
                DEFINE_STORE(3),      // Store new Fib(n) in memory[3]
                DEFINE_LOAD(3),       // Load new Fib(n)
                DEFINE_PRINT(),       // Print Fib(n)
                DEFINE_LOAD(2),       // Move Fib(n-1) to Fib(n-2)
                DEFINE_STORE(1),      // Store in memory[1]
                DEFINE_LOAD(3),       // Move Fib(n) to Fib(n-1)
                DEFINE_STORE(2),      // Store in memory[2]
                DEFINE_LOAD(0),       // Load n
                DEFINE_PUSH(1),       // Push 1
                DEFINE_SUB(),         // n = n - 1
                DEFINE_STORE(0),      // Store new n
                DEFINE_LOAD(0),
                DEFINE_JMPNZ(10),      // If n > 1, jump back to LOOP (index 9)
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

    pub fn allocate(&mut self, size: u16) -> Result<*mut StackValues, ()> {
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
            self.heap.push(StackValues::U16(0));
        }
        println!("RETURNING NOT FROM FREELIST: {:?}", &self.memory[starting_index] as *const u16);
        self.allocated_memory.insert(&mut self.heap[starting_index] as *mut StackValues, size);
        Ok(&mut self.heap[starting_index] as *mut StackValues)
    }

    pub fn deallocate(&mut self, ptr: *mut StackValues) {
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
                }else if let StackValues::I16(a) = self.pop_stack() {
                    if let StackValues::I16(b) = self.pop_stack() {
                        self.push_stack(StackValues::I16(a + b));
                    }
                }
                self.pc += 1;
            },
            InstructionType::INST_PUSH => {
                match &self.instructions[self.pc as usize].values {
                    Some(value) => {
                        if let Word::U16(v) = value[0] {
                            self.push_stack(StackValues::U16(v));
                        } else if let Word::I16(v) = value[0] {
                            self.push_stack(StackValues::I16(v));
                        }
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
                if let StackValues::U16(a) = self.pop_stack() {
                    if let StackValues::U16(b) = self.pop_stack() {
                        self.push_stack(StackValues::U16(a * b));
                    }
                }else if let StackValues::I16(a) = self.pop_stack() {
                    if let StackValues::I16(b) = self.pop_stack() {
                        self.push_stack(StackValues::I16(a * b));
                    }
                }
                self.pc += 1;
            },
            InstructionType::INST_DIV => {
                if let StackValues::U16(a) = self.pop_stack() {
                    if let StackValues::U16(b) = self.pop_stack() {
                        self.push_stack(StackValues::U16(a / b));
                    }
                }else if let StackValues::I16(a) = self.pop_stack() {
                    if let StackValues::I16(b) = self.pop_stack() {
                        self.push_stack(StackValues::I16(a / b));
                    }
                }
                self.pc += 1;
            },
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
                }
                self.pc += 1;
            },
            InstructionType::INST_AND => {
                if let StackValues::U16(a) = self.pop_stack() {
                    if let StackValues::U16(b) = self.pop_stack() {
                        self.push_stack(StackValues::U16(a & b));
                    }
                }else if let StackValues::I16(a) = self.pop_stack() {
                    if let StackValues::I16(b) = self.pop_stack() {
                        self.push_stack(StackValues::I16(a & b));
                    }
                }
                self.pc += 1;
            },
            InstructionType::INST_OR => {
                if let StackValues::U16(a) = self.pop_stack() {
                    if let StackValues::U16(b) = self.pop_stack() {
                        self.push_stack(StackValues::U16(a | b));
                    }
                }else if let StackValues::I16(a) = self.pop_stack() {
                    if let StackValues::I16(b) = self.pop_stack() {
                        self.push_stack(StackValues::I16(a | b));
                    }
                }
                self.pc += 1;
            },
            InstructionType::INST_XOR => {
                if let StackValues::U16(a) = self.pop_stack() {
                    if let StackValues::U16(b) = self.pop_stack() {
                        self.push_stack(StackValues::U16(a ^ b));
                    }
                }else if let StackValues::I16(a) = self.pop_stack() {
                    if let StackValues::I16(b) = self.pop_stack() {
                        self.push_stack(StackValues::I16(a ^ b));
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
                }else if let StackValues::I16(a) = self.pop_stack() {
                    if let StackValues::I16(b) = self.pop_stack() {
                        self.push_stack(StackValues::I16(a << b));
                    }
                }
                self.pc += 1;
            },
            InstructionType::INST_SHR => {
                if let StackValues::U16(a) = self.pop_stack() {
                    if let StackValues::U16(b) = self.pop_stack() {
                        self.push_stack(StackValues::U16(a >> b));
                    }
                }else if let StackValues::I16(a) = self.pop_stack() {
                    if let StackValues::I16(b) = self.pop_stack() {
                        self.push_stack(StackValues::I16(a >> b));
                    }
                }
                self.pc += 1;
            },
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
                                panic!("QUARMVM: does not have a value to push {:?}", self.instructions[self.pc as usize]);
                            }
                        }
                    } else {
                        self.pc += 1;
                    }
                }
            },
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
                                    panic!("QUARMVM: does not have a value to push {:?}", self.instructions[self.pc as usize]);
                                }
                            }
                        } else {
                            self.pc += 1;
                        }
                    }
                }
            },
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
                                        self.pc = v as u16;
                                    } else if let Word::I16(v) = value[0] {
                                        self.pc = v as u16;
                                    } else {
                                        self.pc += 1;
                                    }

                                }
                                None => {
                                    panic!("QUARMVM: does not have a value to push {:?}", self.instructions[self.pc as usize]);
                                }
                            }
                        } else {
                            self.pc += 1;
                        }
                    }
                }
            },
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
                                panic!("QUARMVM: does not have a value to push {:?}", self.instructions[self.pc as usize]);
                            }
                        }
                    } else {
                        self.pc += 1;
                    }
                }
            },
            InstructionType::INST_PUSH_STR => {
                if let Some(args) = &self.instructions[self.pc as usize].values {
                    let str_len = match args[0] {
                        Word::U16(l) => l,
                        _ => panic!("QUARMVM: Expected a u16 as a string size")
                    };
                
                    let mut str_buffer: Vec<StackValues> = Vec::with_capacity(str_len as usize + 1);
                    for i in 0..str_len as usize {
                        if let Word::Char(c) = args.get(1 + i).expect("QUARMVM: Invalid string index") {
                            str_buffer.push(StackValues::U16(*c as u16));
                        } else {
                            panic!("QUARMVM: Expected a Char in string");
                        }
                    }

                    str_buffer.push(StackValues::U16('\0' as u16));
                    let starting_pointer = self.allocate(str_len + 1);
                    if let Ok(ptr) = starting_pointer {
                        for i in 0..(str_len + 1) {
                            unsafe {
                                let dest = ptr.add(i as usize);
                                *dest = str_buffer[i as usize];
                            }
                        }
                        self.push_stack(StackValues::Pointer(ptr));
                    }
                    panic!("QUARMVM: Error, not enough memory left.");
                    
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
            InstructionType::INST_DUP => {
                let value = self.pop_stack();
                self.push_stack(value);
                self.push_stack(value);
                self.pc += 1;
            }
            InstructionType::INST_INSWAP => {
                if let Some(value) = &self.instructions[self.pc as usize].values {
                    if let Word::U16(index) = value[0] {
                        self.stack.swap(self.sp as usize, index as usize);
                    }
                }
                self.pc += 1;
            }
            InstructionType::INST_PRINT => {
                println!("{:?}", self.stack[self.sp as usize]);
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
        }
    }

    pub fn debug_stack(&self) {
        println!("______________________________________________________________________");
        println!("SP: {:?} stack: {:?} pc: {:?}", self.sp, &self.stack[0..(self.sp as usize)], self.pc);
        println!("MEMORY: {:?}", self.memory);
        println!("HEAP: {:?}", self.heap);
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
