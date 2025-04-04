use crate::assembler::lexer::lexer::Lexer;
use crate::assembler::parser::parser::Parser;
use std::fs;
use std::io::{self};

pub struct Assembler<'a> {
    source_code: String,
    lexer: Lexer<'a>,
    parser: Parser<'a>
}

impl<'a> Assembler<'a> {
    pub fn new(src: &str) -> io::Result<Self> {
        let source_code = fs::read_to_string(src)?;

        let source_code_ref: &'static str = Box::leak(source_code.clone().into_boxed_str());

        Ok(Self {
            source_code,
            lexer: Lexer::new(source_code_ref),
            parser: Parser::new(source_code_ref)
        })
    }

    pub fn compile(&mut self) {
        let _ = self.lexer.lex();
        // dbg!(&self.lexer.tokens);
        let parsed = self.parser.parse(self.lexer.tokens.clone());
        if let Ok(parse_result) = parsed {
            println!("{:?}", parse_result);
        } else {
            dbg!(parsed);
        }
        // println!("{:?}", self.lexer.tokens);
    }
}
