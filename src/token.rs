// Token definitions for the Jabr language.
//
// A token is the smallest unit of meaning produced by the lexer.
// Each token carries its lexical category and, where relevant, the
// literal text that produced it so error messages can point to source.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Number(f64),
    String(String),

    // Identifiers & keywords
    Ident(String),
    Let,
    Fn,
    Print,
    Return,
    True,
    False,
    If,
    Else,
    While,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Assign,
    EqEq,
    BangEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    Bang,
    And,
    Or,

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Semicolon,
    Colon,

    // End of input
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, col: usize) -> Self {
        Token { kind, line, col }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}
