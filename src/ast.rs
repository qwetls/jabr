// AST: the tree structure produced by the parser.
//
// Every node is either a literal, a variable reference, a binary
// operation, a unary operation, a print statement, a let binding,
// an if/else, a while loop, a function definition, a return, or
// a function call. The interpreter walks this tree directly.

use crate::token::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    String(String),
    Bool(bool),
    Ident(String),
    BinOp(Box<Expr>, BinOpKind, Box<Expr>),
    UnaryOp(UnaryOpKind, Box<Expr>),
    Call(String, Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOpKind {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOpKind {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let(String, Expr),
    Print(Expr),
    Expr(Expr),
    If(Expr, Vec<Stmt>, Option<Vec<Stmt>>),
    While(Expr, Vec<Stmt>),
    FnDef(String, Vec<String>, Vec<Stmt>),
    Return(Option<Expr>),
}

/// Convert a token kind into the matching binary operator.
/// Returns None for tokens that are not binary operators.
pub fn binop_from_token(tok: &TokenKind) -> Option<BinOpKind> {
    match tok {
        TokenKind::Plus => Some(BinOpKind::Add),
        TokenKind::Minus => Some(BinOpKind::Sub),
        TokenKind::Star => Some(BinOpKind::Mul),
        TokenKind::Slash => Some(BinOpKind::Div),
        TokenKind::Percent => Some(BinOpKind::Mod),
        TokenKind::EqEq => Some(BinOpKind::Eq),
        TokenKind::BangEq => Some(BinOpKind::Neq),
        TokenKind::Lt => Some(BinOpKind::Lt),
        TokenKind::Gt => Some(BinOpKind::Gt),
        TokenKind::LtEq => Some(BinOpKind::LtEq),
        TokenKind::GtEq => Some(BinOpKind::GtEq),
        TokenKind::And => Some(BinOpKind::And),
        TokenKind::Or => Some(BinOpKind::Or),
        _ => None,
    }
}
