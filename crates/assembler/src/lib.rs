extern crate num_traits;
extern crate num_derive;
extern crate anyhow;

use std::path::Path;
use anyhow::{Error, Result};
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term;
use codespan_reporting::term::{Config};
use termcolor::{ColorChoice, StandardStream};
use crate::errors::AsDiagnostic;

// Tokenizer and parser
pub mod parser;
pub mod lexer;
pub mod opcode;

// Error handling and diagnostics
pub mod macros;
pub mod errors;
pub mod messages;

// Intermediate Representation
pub mod astnode;
pub mod dynsym;

// ELF header, program, section
pub mod header;
pub mod program;
pub mod section;

// Debug info
pub mod debuginfo;

#[cfg(test)]
mod tests;

// Type aliases for error handling
pub type ParserError = String;
pub type ProgramError = String;
pub type TokenizerError = String;

pub use self::{
    parser::Parser,
    program::Program,
    lexer::tokenize,
};

pub fn assemble(src: &str, deploy: &str) -> Result<()> {
    let source_code = std::fs::read_to_string(src)?;
    let file = SimpleFile::new(src.to_string(), source_code.clone());

    // TODO: ideally we should have only collect errors and then print them with parsers
    // errors all at once
    let tokens = match tokenize(&source_code) {
        Ok(tokens) => tokens,
        Err(errors) => {
            for error in errors {
                let writer = StandardStream::stderr(ColorChoice::Auto);
                let config = Config::default();
                let diagnostic = error.to_diagnostic();
                term::emit(&mut writer.lock(), &config, &file, &diagnostic)?;
            }
            return Err(Error::msg("Compilation failed"));
        }
    };
    let mut parser = Parser::new(tokens, &file);
    let parse_result = match parser.parse() {
        Ok(program) => program,
        Err(errors) => {
            for error in errors {
                let writer = StandardStream::stderr(ColorChoice::Auto);
                let config = Config::default();
                let diagnostic = error.to_diagnostic();
                term::emit(&mut writer.lock(), &config, &file, &diagnostic)?;
            }
            return Err(Error::msg("Compilation failed"));
        }
    };

    let program = Program::from_parse_result(parse_result);

    let bytecode = program.emit_bytecode();

    let output_path = Path::new(deploy)
        .join(Path::new(src)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .replace(".s", ".so"));

    std::fs::write(output_path, bytecode)?;
    Ok(())
}
