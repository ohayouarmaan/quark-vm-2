use std::collections::HashMap;
use proton::lib::machine_type::{Instruction, InstructionType, Word};
use super::parser::parser::ASTNode;

#[derive(Debug)]
pub enum CompilerError {
    UnexpectedArgument
}

#[derive(Debug)]
pub enum SymbolValue {
    Label(Vec<Instruction>),
    Variable(u16)
}

#[derive(Debug)]
pub struct Compiler {
    pub symbol_table: Vec<HashMap<String, SymbolValue>>,
    pub ASTnodes: Vec<ASTNode>,
    pub ic: usize,
    pub const_pool_index: usize
}

impl Compiler {
    pub fn new(ASTnodes: Vec<ASTNode>) -> Self {
        Self {
            symbol_table: vec![],
            ASTnodes,
            ic: 0,
            const_pool_index: 0
        }
    }

    pub fn advance(&mut self) {
        if self.ic < self.ASTnodes.len() {
            self.ic += 1;
        }
    }

pub fn get_or_allocate_variable_address(&mut self, name: &str) -> Result<u16, CompilerError> {
    for scope in self.symbol_table.iter().rev() {
        if let Some(SymbolValue::Variable(addr)) = scope.get(name) {
            return Ok(*addr);
        }
    }

    let addr = self.const_pool_index as u16;
    if let Some(scope) = self.symbol_table.last_mut() {
        scope.insert(name.to_string(), SymbolValue::Variable(addr));
    } else {
        let mut new_scope = HashMap::new();
        new_scope.insert(name.to_string(), SymbolValue::Variable(addr));
        self.symbol_table.push(new_scope);
    }

    self.const_pool_index += 1;
    Ok(addr)
}

    pub fn parse_arg(&mut self, arg: &ASTNode) -> Result<Vec<Word>, CompilerError> {
        match arg {
            ASTNode::Variable(x) => {
                let address = self.get_or_allocate_variable_address(&x[0..])?;
                Ok(vec![Word::U16(address)])
            },
            ASTNode::StringLiteral(x) => {
                let mut args = vec![Word::from(x.len() as u16)];
                dbg!(&args);
                args.extend(x.chars().map(Word::from));
                Ok(args)
            },
            ASTNode::Number(x) => {
                match x {
                    super::lexer::lexer::NumberType::u16(u) => Ok(vec![Word::from(*u)]),
                    super::lexer::lexer::NumberType::f16(f) => Ok(vec![Word::from(*f)])
                }
            }
            _ => {
                Err(CompilerError::UnexpectedArgument)
            }
        }
    }

    pub fn compile_instruction(&mut self, it: InstructionType, args: Vec<ASTNode>) -> Result<Instruction, CompilerError> {
        let mut args_flattened: Vec<Word> = vec![];
        for arg in &args[0..] { 
            for a in self.parse_arg(arg)? {
                args_flattened.push(a);
            }
        }
        let args: Vec<Word> = args.iter().flat_map(|arg| self.parse_arg(arg)).flatten().collect();
        let args: Option<Vec<Word>> = if !args.is_empty() {
            Some(args)
        } else {
            None
        };
        Ok(Instruction {
            tt: it,
            values: args
        })
    }

    pub fn compile(&mut self) -> Result<Vec<Instruction>, CompilerError> {
        let mut instructions: Vec<Instruction> = vec![];
        while self.ic < self.ASTnodes.len() {

            let value = self.ASTnodes[self.ic].clone();
            match value {
                ASTNode::Instruction(it, args) => {
                    instructions.push(self.compile_instruction(it, args)?);
                    self.advance();
                },
                ASTNode::Label(l, ast_nodes) => {
                    let last = self.symbol_table.len().checked_sub(1);
                    if let Some(last_index) = last {
                        let mut label_instructions = vec![]; 
                        for node in ast_nodes {
                            if let ASTNode::Instruction(it, args) = node {
                                if let Ok(ins) = self.compile_instruction(it, args) {
                                    label_instructions.push(ins);
                                }
                            }
                        }
                        self.symbol_table[last_index].insert(l.to_string(), SymbolValue::Label(label_instructions));
                    } else {
                        let mut d = HashMap::new();
                        let mut label_instructions = vec![];
                        for node in ast_nodes {
                            if let ASTNode::Instruction(it, args) = node {
                                let ins = self.compile_instruction(it, args)?;
                                label_instructions.push(ins);
                            }
                        }
                        d.insert(l.to_string(), SymbolValue::Label(label_instructions));
                        self.symbol_table.push(d);
                    }
                },
                _=> {

                }
            }
        }
        Ok(instructions)
    }
}
