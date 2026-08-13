// Lexer: converts Jabr source text into a flat stream of tokens.
//
// This is a hand-written single-pass scanner — no regex, no external
// crates. It walks the source character by character and emits tokens
// with line/column tracking for error reporting.

use crate::token::{Token, TokenKind};

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.source.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek();
        if let Some(c) = ch {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        ch
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(' ') | Some('\t') | Some('\r') | Some('\n') => {
                    self.advance();
                }
                Some('/') if self.peek_next() == Some('/') => {
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn read_number(&mut self) -> TokenKind {
        let start = self.pos;
        let mut has_dot = false;

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.advance();
            } else if c == '.' && !has_dot && self.peek_next().map_or(false, |n| n.is_ascii_digit()) {
                has_dot = true;
                self.advance();
            } else {
                break;
            }
        }

        let text: String = self.source[start..self.pos].iter().collect();
        TokenKind::Number(text.parse::<f64>().unwrap_or(0.0))
    }

    fn read_string(&mut self) -> Result<TokenKind, String> {
        self.advance(); // opening quote
        let mut chars = Vec::new();

        loop {
            match self.peek() {
                None => return Err("Unterminated string literal".into()),
                Some('"') => {
                    self.advance(); // closing quote
                    break;
                }
                Some('\\') => {
                    self.advance();
                    match self.peek() {
                        Some('n') => chars.push('\n'),
                        Some('t') => chars.push('\t'),
                        Some('\\') => chars.push('\\'),
                        Some('"') => chars.push('"'),
                        Some(c) => chars.push(c),
                        None => return Err("Unterminated string escape".into()),
                    }
                    self.advance();
                }
                Some(c) => {
                    chars.push(c);
                    self.advance();
                }
            }
        }

        Ok(TokenKind::String(chars.into_iter().collect()))
    }

    fn read_ident(&mut self) -> TokenKind {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let text: String = self.source[start..self.pos].iter().collect();

        match text.as_str() {
            "let" => TokenKind::Let,
            "fn" => TokenKind::Fn,
            "print" => TokenKind::Print,
            "return" => TokenKind::Return,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            _ => TokenKind::Ident(text),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace_and_comments();

            let line = self.line;
            let col = self.col;

            match self.peek() {
                None => {
                    tokens.push(Token::new(TokenKind::Eof, line, col));
                    break;
                }
                Some(c) if c.is_ascii_digit() => {
                    tokens.push(Token::new(self.read_number(), line, col));
                }
                Some('"') => {
                    tokens.push(Token::new(self.read_string()?, line, col));
                }
                Some(c) if c.is_alphabetic() || c == '_' => {
                    tokens.push(Token::new(self.read_ident(), line, col));
                }
                Some(c) => {
                    self.advance();
                    let kind = match c {
                        '+' => TokenKind::Plus,
                        '-' => TokenKind::Minus,
                        '*' => TokenKind::Star,
                        '/' => TokenKind::Slash,
                        '%' => TokenKind::Percent,
                        '(' => TokenKind::LParen,
                        ')' => TokenKind::RParen,
                        '{' => TokenKind::LBrace,
                        '}' => TokenKind::RBrace,
                        ',' => TokenKind::Comma,
                        ';' => TokenKind::Semicolon,
                        ':' => TokenKind::Colon,
                        '=' => {
                            if self.peek() == Some('=') {
                                self.advance();
                                TokenKind::EqEq
                            } else {
                                TokenKind::Assign
                            }
                        }
                        '!' => {
                            if self.peek() == Some('=') {
                                self.advance();
                                TokenKind::BangEq
                            } else {
                                TokenKind::Bang
                            }
                        }
                        '<' => {
                            if self.peek() == Some('=') {
                                self.advance();
                                TokenKind::LtEq
                            } else {
                                TokenKind::Lt
                            }
                        }
                        '>' => {
                            if self.peek() == Some('=') {
                                self.advance();
                                TokenKind::GtEq
                            } else {
                                TokenKind::Gt
                            }
                        }
                        _ => return Err(format!("Unexpected character '{}' at {}:{}",
                                                c, line, col)),
                    };
                    tokens.push(Token::new(kind, line, col));
                }
            }
        }

        Ok(tokens)
    }
}
