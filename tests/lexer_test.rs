// Integration tests for the Jabr lexer.
//
// These verify that the tokenizer produces the correct token
// stream for various inputs, including edge cases like nested
// comments, string escapes, and multi-character operators.

use jabr::lexer::Lexer;
use jabr::token::{Token, TokenKind};

fn lex(src: &str) -> Vec<TokenKind> {
    let mut lx = Lexer::new(src);
    lx.tokenize().unwrap().into_iter().map(|t| t.kind).collect()
}

#[test]
fn basic_arithmetic() {
    let tokens = lex("1 + 2 * 3");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Number(1.0),
            TokenKind::Plus,
            TokenKind::Number(2.0),
            TokenKind::Star,
            TokenKind::Number(3.0),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn decimal_numbers() {
    let tokens = lex("3.14 + 0.5");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Number(3.14),
            TokenKind::Plus,
            TokenKind::Number(0.5),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn string_literal() {
    let tokens = lex("\"hello world\"");
    assert_eq!(
        tokens,
        vec![
            TokenKind::String("hello world".into()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn string_escapes() {
    let tokens = lex("\"line1\\nline2\\t\\\"quoted\\\"\"");
    assert_eq!(
        tokens,
        vec![
            TokenKind::String("line1\nline2\t\"quoted\"".into()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn keywords() {
    let tokens = lex("let fn print return true false if else while and or");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Let,
            TokenKind::Fn,
            TokenKind::Print,
            TokenKind::Return,
            TokenKind::True,
            TokenKind::False,
            TokenKind::If,
            TokenKind::Else,
            TokenKind::While,
            TokenKind::And,
            TokenKind::Or,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn identifiers() {
    let tokens = lex("foo _bar baz123");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Ident("foo".into()),
            TokenKind::Ident("_bar".into()),
            TokenKind::Ident("baz123".into()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn multi_char_operators() {
    let tokens = lex("a == b != c <= d >= e");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Ident("a".into()),
            TokenKind::EqEq,
            TokenKind::Ident("b".into()),
            TokenKind::BangEq,
            TokenKind::Ident("c".into()),
            TokenKind::LtEq,
            TokenKind::Ident("d".into()),
            TokenKind::GtEq,
            TokenKind::Ident("e".into()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn single_char_operators() {
    let tokens = lex("a + b - c * d / e % f");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Ident("a".into()),
            TokenKind::Plus,
            TokenKind::Ident("b".into()),
            TokenKind::Minus,
            TokenKind::Ident("c".into()),
            TokenKind::Star,
            TokenKind::Ident("d".into()),
            TokenKind::Slash,
            TokenKind::Ident("e".into()),
            TokenKind::Percent,
            TokenKind::Ident("f".into()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn delimiters() {
    let tokens = lex("fn add(a, b) { return a + b; }");
    assert!(tokens.contains(&TokenKind::LParen));
    assert!(tokens.contains(&TokenKind::RParen));
    assert!(tokens.contains(&TokenKind::LBrace));
    assert!(tokens.contains(&TokenKind::RBrace));
    assert!(tokens.contains(&TokenKind::Comma));
    assert!(tokens.contains(&TokenKind::Semicolon));
}

#[test]
fn line_comments_are_skipped() {
    let tokens = lex("1 // this is a comment\n+ 2");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Number(1.0),
            TokenKind::Plus,
            TokenKind::Number(2.0),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn unterminated_string_errors() {
    let mut lx = Lexer::new("\"unterminated");
    assert!(lx.tokenize().is_err());
}

#[test]
fn unexpected_char_errors() {
    let mut lx = Lexer::new("1 @ 2");
    assert!(lx.tokenize().is_err());
}

#[test]
fn empty_input() {
    let tokens = lex("");
    assert_eq!(tokens, vec![TokenKind::Eof]);
}

#[test]
fn only_whitespace() {
    let tokens = lex("   \n\t  \n");
    assert_eq!(tokens, vec![TokenKind::Eof]);
}

#[test]
fn line_and_col_tracking() {
    let mut lx = Lexer::new("1\n+ 2");
    let tokens = lx.tokenize().unwrap();
    assert_eq!(tokens[0].line, 1);
    assert_eq!(tokens[0].col, 1);
    assert_eq!(tokens[1].line, 2);
    assert_eq!(tokens[1].col, 1);
    assert_eq!(tokens[2].line, 2);
    assert_eq!(tokens[2].col, 3);
}
