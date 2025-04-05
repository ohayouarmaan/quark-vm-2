use std::collections::HashMap;
use proton::lib::machine_type::{Instruction, Word};

use super::parser::parser::ASTNode;

#[derive(Debug)]
pub enum CompilerError {
    UnexpectedArgument
}

#[derive(Debug)]
pub struct Symbol<'a> {
    name: &'a str,
}

#[derive(Debug)]
pub struct Compiler<'a> {
    pub symbol_table: Vec<HashMap<&'a str, Symbol<'a>>>,
    pub ASTnodes: Vec<ASTNode>,
    pub ic: usize
}

impl<'a> Compiler<'a> {
    pub fn new(ASTnodes: Vec<ASTNode>) -> Self {
        Self {
            symbol_table: vec![],
            ASTnodes,
            ic: 0
        }
    }

    pub fn advance(&mut self) {
        if self.ic < self.ASTnodes.len() {
            self.ic += 1;
        }
    }

    pub fn get_address_from_label(&mut self, name: &str) -> u16 {
        todo!("TODO: SymbolTable Implementation");
    }

    pub fn parse_arg(&mut self, arg: &ASTNode) -> Result<Vec<Word>, CompilerError> {
        match arg {
            ASTNode::Label(x) => {
                let address = self.get_address_from_label(&x[0..]);
                return Ok(vec![Word::U16(address)]);
            },
            ASTNode::StringLiteral(x) => {
                let mut args = vec![Word::from(x.len() as u16)];
                dbg!(&args);
                args.extend(x.chars().map(|t| Word::from(t)));
                return Ok(args);
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

    pub fn compile(&mut self) -> Result<Vec<Instruction>, CompilerError> {
        let mut instructions: Vec<Instruction> = vec![];
        while self.ic < self.ASTnodes.len() {
            let value = &self.ASTnodes[self.ic].clone();
            match value {
                ASTNode::Instruction(it, args) => {
                    dbg!(it, args);
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
                    dbg!("xd", it, &args);
                    instructions.push(Instruction {
                        tt: *it,
                        values: args
                    });
                    self.advance();
                },
                ASTNode::Label(_l) => {

                }
                _=> {

                }
            }
        }
        Ok(instructions)
    }
}
