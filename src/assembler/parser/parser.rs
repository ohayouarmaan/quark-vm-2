use crate::assembler::lexer::lexer::{TokenType, NumberType, Token};
use crate::machine::machine_types::{Instruction, InstructionType};
use std::collections::HashMap;

#[derive(Debug)]
pub enum ParserError {
    UnexpectedEOF,
    UnexpectedToken,
    InvalidInstructionFormat,
}

#[derive(Debug)]
pub enum ASTNode {
    Instruction(InstructionType, Vec<ASTNode>),
    Label(String),
    Number(NumberType),
    StringLiteral(String),
}

pub struct Parser<'a> {
    tokens: Vec<Token>,
    current_index: usize,
    source_code: &'a str,
    instruction_arg_count: HashMap<&'static str, usize>,
}

impl<'a> Parser<'a> {
    pub fn new(source_code: &'a str) -> Self {
        let instruction_arg_count = Self::build_instruction_arg_map();

        Self {
            tokens: vec![],
            current_index: 0,
            source_code,
            instruction_arg_count,
        }
    }

    fn build_instruction_arg_map() -> HashMap<&'static str, usize> {
        let mut map = HashMap::new();

        map.insert("PUSH", 1);
        map.insert("LOAD", 1);
        map.insert("STORE", 1);
        map.insert("ADD", 0);
        map.insert("SUB", 0);
        map.insert("MUL", 0);
        map.insert("DIV", 0);
        map.insert("JMP", 1);
        map.insert("JZ", 1);
        map.insert("CALL", 1);
        map.insert("RET", 0);

        map
    }

    pub fn parse(&mut self, tokens: Vec<Token>) -> Result<Vec<ASTNode>, ParserError> {
        let mut nodes = Vec::new();
        self.tokens = tokens;

        while self.current_index < self.tokens.len() {
            match self.parse_instruction() {
                Ok(node) => nodes.push(node),
                Err(ParserError::UnexpectedEOF) => break,
                Err(err) => return Err(err),
            }
        }

        Ok(nodes)
    }

    fn parse_instruction(&mut self) -> Result<ASTNode, ParserError> {
        if self.current_index >= self.tokens.len() {
            return Err(ParserError::UnexpectedEOF);
        }

        let token = &self.tokens[self.current_index];

        match token.tt {
            TokenType::InstructionType(inst_type) => {
                let inst_str = format!("{:?}", inst_type);
                let inst_str = inst_str.split_at(5).1;
                let expected_args = self
                    .instruction_arg_count
                    .get(inst_str)
                    .copied()
                    .unwrap_or(0);

                dbg!(inst_str, expected_args);

                self.advance();

                let mut args = Vec::new();
                for _ in 0..expected_args {
                    if self.current_index >= self.tokens.len() {
                        return Err(ParserError::UnexpectedEOF);
                    }

                    match &self.tokens[self.current_index].tt {
                        TokenType::Number(num) => {
                            args.push(ASTNode::Number(*num));
                            self.advance();
                        }
                        TokenType::Label(start, end) => {
                            let label_name =
                                self.extract_label_name(*start, *end).ok_or(ParserError::UnexpectedToken)?;
                            println!("LABEL: {:?}", label_name);
                            args.push(ASTNode::Label(label_name));
                            self.advance();
                        }
                        TokenType::String => {
                            let string_value = self.extract_string();
                            args.push(ASTNode::StringLiteral(string_value));
                            self.advance();
                        }
                        _ => return Err(ParserError::InvalidInstructionFormat),
                    }
                }

                Ok(ASTNode::Instruction(inst_type, args))
            }
            TokenType::Label(start, end) => {
                let label_name = self.extract_label_name(start, end).ok_or(ParserError::UnexpectedToken)?;
                self.advance();
                Ok(ASTNode::Label(label_name))
            }
            _ => Err(ParserError::UnexpectedToken),
        }
    }

    fn advance(&mut self) {
        if self.current_index < self.tokens.len() {
            self.current_index += 1;
        }
    }

    fn extract_label_name(&self, start: usize, end: usize) -> Option<String> {
        let label_name = &self.source_code[start..end];

        if label_name.is_empty() {
            None
        } else {
            Some(label_name.to_string())
        }
    }

    fn extract_string(&self) -> String {
        "STRING_VALUE".to_string()
    }
}
