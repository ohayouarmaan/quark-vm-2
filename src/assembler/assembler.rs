use crate::lexer::lexer::Lexer;
use crate::parser::parser::Parser;
use crate::compiler;
use proton::lib::bytecode::ByteCodeCompiler;
use std::fs;
use std::io::{self};

pub struct Assembler<'a> {
    source_code: String,
    output_name: &'a str,
    lexer: Lexer<'a>,
    parser: Parser<'a>
}

impl<'a> Assembler<'a> {
    pub fn new(src: &str, output_name: &'a str) -> io::Result<Self> {
        let source_code = fs::read_to_string(src)?;

        let source_code_ref: &'static str = Box::leak(source_code.clone().into_boxed_str());

        Ok(Self {
            source_code,
            output_name,
            lexer: Lexer::new(source_code_ref),
            parser: Parser::new(source_code_ref)
        })
    }

    pub fn compile(&mut self) {
        let size = self.lexer.lex();
        let parsed = self.parser.parse(self.lexer.tokens.clone());
        if let Ok(parse_result) = parsed  {
            let mut compiled = compiler::Compiler::new(parse_result);
            if let Ok(compiled_instructions) = compiled.compile() {
                let mut b = ByteCodeCompiler::new(self.output_name);
                b.store_file(&compiled_instructions[0..]);
            }
        } 
    }
}
