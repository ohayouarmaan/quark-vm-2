use crate::assembler::lexer::lexer::Lexer;
use std::fs;
use std::io::{self};

pub struct Assembler {
    source_code: String,
    lexer: Lexer<'static>,
}

impl Assembler {
    pub fn new(src: &str) -> io::Result<Self> {
        let source_code = fs::read_to_string(src)?;

        let source_code_ref: &'static str = Box::leak(source_code.clone().into_boxed_str());

        Ok(Self {
            source_code,
            lexer: Lexer::new(source_code_ref),
        })
    }

    pub fn compile(&mut self) {
        let _ = self.lexer.lex();
        println!("{:?}", self.lexer.tokens);
    }
}
