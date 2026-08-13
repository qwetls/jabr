// Jabr — a programming language inspired by the Islamic Golden Age.
//
// Library entry point: re-exports all public modules so that
// integration tests and the CLI binary share the same surface.

pub mod ast;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod token;

pub fn run_source(source: &str) -> Result<(), String> {
    let mut lx = lexer::Lexer::new(source);
    let tokens = lx.tokenize()?;
    let mut parser = parser::Parser::new(tokens);
    let program = parser.parse_program()?;
    let mut interp = interpreter::Interpreter::new();
    interp.run(&program)
}
