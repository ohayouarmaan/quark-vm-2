use proton::lib::machine_type::{ InstructionType };
use half::f16;

const IGNORE: [char; 3] = ['\n', '\t', ' '];

pub struct Lexer<'a> {
    pub source_code: &'a str,
    current_index: usize,
    pub tokens: Vec<Token>,
    line_number: usize,
    column: usize
}

#[derive(Debug)]
pub enum LexerError {
    EOFWithNoTokens,
    InvalidInstructionType,
    InvalidNumber,
    InvalidLabel
}

#[derive(Debug, Clone)]
pub enum TokenType {
    InstructionType(InstructionType),
    Label(usize, usize),
    String(String),
    Number(NumberType),
    Colon,
    Comma,
}

#[derive(Debug, Clone, Copy)]
pub enum NumberType {
    u16(u16),
    f16(f16)
}

#[derive(Debug, Clone)]
pub struct Token {
    pub tt: TokenType,
    pub line_number: usize,
    pub column: usize
}

impl<'a> Lexer<'a> {
    pub fn new(source_code: &'a str) -> Self {
        Self {
            source_code,
            current_index: 0,
            column: 0,
            tokens: Vec::new(),
            line_number: 0
        }
    }

    pub fn build_ident(&mut self) -> Result<InstructionType, LexerError> {
        let lexed_starting = self.current_index;
        while let Some(c) = self.source_code.chars().nth(self.current_index) {
            if c.is_ascii_alphabetic() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let lexed_ending = self.current_index;

        match &self.source_code[lexed_starting..lexed_ending] {
            "NOOP"      => Ok(InstructionType::INST_NOOP),
            "PUSH"      => Ok(InstructionType::INST_PUSH),
            "POP"       => Ok(InstructionType::INST_POP),
            "ADD"       => Ok(InstructionType::INST_ADD),
            "AND"       => Ok(InstructionType::INST_AND),
            "OR"        => Ok(InstructionType::INST_OR),
            "XOR"       => Ok(InstructionType::INST_XOR),
            "NOT"       => Ok(InstructionType::INST_NOT),
            "SHL"       => Ok(InstructionType::INST_SHL),
            "SHR"       => Ok(InstructionType::INST_SHR),
            "MUL"       => Ok(InstructionType::INST_MUL),
            "DIV"       => Ok(InstructionType::INST_DIV),
            "SUB"       => Ok(InstructionType::INST_SUB),
            "JMPZ"      => Ok(InstructionType::INST_JMPZ),
            "JMPEQ"     => Ok(InstructionType::INST_JMPEQ),
            "JMPNEQ"    => Ok(InstructionType::INST_JMPNEQ),
            "JMPNZ"     => Ok(InstructionType::INST_JMPNZ),
            "PUSH_STR"  => Ok(InstructionType::INST_PUSH_STR),
            "ALLOC"     => Ok(InstructionType::INST_ALLOC),
            "ALLOC_RAW" => Ok(InstructionType::INST_ALLOC_RAW),
            "SYSCALL"   => Ok(InstructionType::INST_SYSCALL),
            "DUP"       => Ok(InstructionType::INST_DUP),
            "INSWAP"    => Ok(InstructionType::INST_INSWAP),
            "PRINT"     => Ok(InstructionType::INST_PRINT),
            "LOAD"      => Ok(InstructionType::INST_LOAD),
            "STORE"     => Ok(InstructionType::INST_STORE),
            "DEREF"     => Ok(InstructionType::INST_DEREF),
            "REF"       => Ok(InstructionType::INST_REF),
            "DEBUG"     => Ok(InstructionType::INST_DEBUG),
            _ => Err(LexerError::InvalidInstructionType),
        }
    }

    pub fn build_label(&mut self) -> Result<(usize, usize), LexerError> {
        let lexed_starting = self.current_index;
        while let Some(c) = self.source_code.chars().nth(self.current_index) {
            if c.is_ascii_alphabetic() {
                self.advance();
            } else {
                break;
            }
        }
        let lexed_ending = self.current_index;

        Ok((lexed_starting, lexed_ending))
    }

    pub fn build_string(&mut self) -> Result<String, LexerError> {
        self.advance();
        let lexed_starting = self.current_index;
        while let Some(c) = self.source_code.chars().nth(self.current_index) {
            if c != '"' {
                self.advance();
            } else {
                break;
            }
        }
        self.advance();
        let lexed_ending = self.current_index - 1;
        Ok(self.source_code[lexed_starting..lexed_ending].to_string())
    }

    pub fn advance(&mut self) {
        if self.current_index < self.source_code.len() {
            self.current_index += 1;
            self.column += 1;
        }
    }

    pub fn build_number(&mut self) -> Result<NumberType, LexerError> {
        let lexed_string = self.current_index;
        let dot_count = 0;

        while let Some(c) = self.source_code.chars().nth(self.current_index) {
            if c.is_ascii_digit() || (c == '.' && dot_count < 1) {
                self.advance();
            } else {
                break;
            }
        }

        let lexed_ending = self.current_index;

        if dot_count == 1 {
            let float = &self.source_code[lexed_string..lexed_ending];
            if let Ok(parsed_float) = float.parse::<f16>() {
                Ok(NumberType::f16(parsed_float))
            } else {
                Err(LexerError::InvalidNumber)
            }
        } else {
            let num = &self.source_code[lexed_string..lexed_ending];
            if let Ok(parsed_number) = num.parse::<u16>() {
                Ok(NumberType::u16(parsed_number))
            } else {
                Err(LexerError::InvalidNumber)
            }
        }

    }

    pub fn lex(&mut self) -> Result<usize, LexerError> {
        while self.current_index < self.source_code.len() || self.current_index == 0 {
            if let Some(c) = self.source_code.chars().nth(self.current_index) {
                match c {
                    'a'..='z' | 'A'..='Z' => {
                        let column = self.current_index;
                        if let Ok(found_type) = self.build_ident() {
                            self.tokens.push(Token {
                                column,
                                tt: TokenType::InstructionType(found_type),
                                line_number: self.line_number
                            });
                        } else if let Ok((x, y)) = self.build_label() {
                            self.tokens.push(Token {
                                column,
                                tt: TokenType::Label(x, y),
                                line_number: self.line_number
                            });
                        } else {
                            return Err(LexerError::InvalidLabel)
                        }
                    },
                    '0'..='9' => {
                        let column = self.column;
                        let number = self.build_number()?;
                        self.tokens.push(Token {
                            column,
                            tt: TokenType::Number(number),
                            line_number: self.line_number
                        });
                    },
                    '"' => {
                        let column = self.column;
                        let parsed_string = self.build_string()?;
                        self.tokens.push(Token {
                            column,
                            tt: TokenType::String(parsed_string),
                            line_number: self.line_number
                        });
                    },
                    ';' => {},
                    ':' => {},
                    '\n' => {
                        self.line_number += 1;
                        self.column = 1;
                        self.advance();
                    },
                    '\0' => {
                        break;
                    },
                    other_chars => {
                        if IGNORE.iter().any(|&ch| ch == other_chars) {
                            self.advance();
                        }
                    }
                }
            }
        }
        if self.tokens.is_empty() {
            Err(LexerError::EOFWithNoTokens)
        } else {
            Ok(self.tokens.len())
        }
    }
}

