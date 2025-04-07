use crate::lexer::lexer::{TokenType, NumberType, Token};
use proton::lib::machine_type::{ InstructionType };
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum ParserError {
    UnexpectedEOF,
    UnexpectedToken,
    InvalidInstructionFormat,
}

#[derive(Debug, Clone)]
pub enum ASTNode {
    Instruction(InstructionType, Vec<ASTNode>),
    Variable(String),
    Label(String, Vec<ASTNode>),
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

        map.insert("NOOP", 0);
        map.insert("PUSH", 1);
        map.insert("POP", 0);
        map.insert("ADD", 0);
        map.insert("SUB", 0);
        map.insert("MUL", 0);
        map.insert("DIV", 0);
        map.insert("AND", 0);
        map.insert("OR", 0);
        map.insert("XOR", 0);
        map.insert("NOT", 0);
        map.insert("SHL", 0);
        map.insert("SHR", 0);
        map.insert("JMPZ", 1);
        map.insert("JMPEQ", 1);
        map.insert("JMPNEQ", 1);
        map.insert("JMPNZ", 1);
        map.insert("PUSH_STR", 1); // Technically a string, so 1 logical arg
        map.insert("ALLOC", 1);
        map.insert("ALLOC_RAW", 1);
        map.insert("SYSCALL", 1);
        map.insert("DUP", 0);
        map.insert("INSWAP", 1);
        map.insert("PRINT", 0);
        map.insert("LOAD", 1); // Optional, default to 0?
        map.insert("STORE", 1);
        map.insert("DEREF", 0);
        map.insert("REF", 0);
        map.insert("DEBUG", 0);

        map
    }

    pub fn parse(&mut self, tokens: Vec<Token>) -> Result<Vec<ASTNode>, ParserError> {
        let mut nodes = Vec::new();
        self.tokens = tokens;

        while self.current_index < self.tokens.len() {
            // dbg!(&self.tokens[self.current_index]);
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
                            // dbg!(&label_name);
                            println!("LABEL: {:?}", label_name);
                            args.push(ASTNode::Variable(label_name));
                            self.advance();
                        }
                        TokenType::String(s) => {
                            let string_value = s;
                            args.push(ASTNode::StringLiteral(string_value.clone()));
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
                if let TokenType::Colon = self.tokens[self.current_index].tt {
                    self.advance();
                }
                let mut ins: Vec<ASTNode> = vec![];
                while !matches!(self.tokens[self.current_index].tt, TokenType::Label(_, _)) {
                    ins.push(self.parse_instruction()?);
                    self.current_index += 1;
                }
                Ok(ASTNode::Label(label_name, ins))
            }
            _ => {
                dbg!("WTF");
                Err(ParserError::UnexpectedToken)
            },
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
