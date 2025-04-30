use std::collections::HashMap;
use proton::lib::machine_type::{Instruction, InstructionType, Word};
use super::parser::parser::ASTNode;

#[derive(Debug)]
pub enum CompilerError {
    UnexpectedArgument
}

#[derive(Debug)]
pub enum SymbolValue {
    Label(u16),
    Variable(u16)
}

#[derive(Debug)]
pub struct Compiler {
    pub symbol_table: Vec<HashMap<String, SymbolValue>>,
    pub ASTnodes: Vec<ASTNode>,
    pub ic: usize,
    pub instruction_index: usize,
    pub const_pool_index: usize
}

impl Compiler {
    pub fn new(ASTnodes: Vec<ASTNode>) -> Self {
        Self {
            symbol_table: vec![],
            ASTnodes,
            ic: 0,
            instruction_index: 1,
            const_pool_index: 0
        }
    }

    pub fn advance(&mut self) {
        if self.ic < self.ASTnodes.len() {
            self.ic += 1;
        }
    }

    pub fn generate_label_table(&mut self) {
        while self.ic < self.ASTnodes.len() {
            match &self.ASTnodes[self.ic] {
                ASTNode::Instruction(_, _) => {
                    self.instruction_index += 1;
                },
                ASTNode::Label(label_name) => {
                    dbg!(label_name, &self.instruction_index);
                    if let Some(ctx) = self.symbol_table.last_mut() {
                        ctx.insert(label_name.to_string(), SymbolValue::Label(self.instruction_index as u16));
                    } else {
                        let mut d = HashMap::new();
                        d.insert(label_name.to_string(), SymbolValue::Label(self.instruction_index as u16));
                        self.symbol_table.push(d);
                    }
                },
                _ => {}
            }
            self.advance();
        }
        self.ic = 0;
        self.instruction_index = 0;
        dbg!(&self.symbol_table);
    }

    pub fn get_or_allocate_variable_address(&mut self, name: &str) -> Result<u16, CompilerError> {
        for scope in self.symbol_table.iter().rev() {
            if let Some(SymbolValue::Variable(addr)) = scope.get(name) {
                return Ok(*addr);
            } else if let Some(SymbolValue::Label(addr)) = scope.get(name) {
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
            ASTNode::Label(x) => {
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
        self.generate_label_table();
        while self.ic < self.ASTnodes.len() {
            let value = self.ASTnodes[self.ic].clone();
            match value {
                ASTNode::Instruction(it, args) => {
                    instructions.push(self.compile_instruction(it, args)?);
                    self.advance();
                },
                ASTNode::Label(_) => {
                    self.advance();
                },
                _=> {

                }
            }
        }
        if let Some(SymbolValue::Label(index)) = self.symbol_table.last().expect("NO CONTEXT").get("main") {
            instructions.insert(0, Instruction {
                tt: InstructionType::INST_CALL,
                values: Some(vec![Word::from(*index)])
            });
        }
        Ok(instructions)
    }
}
