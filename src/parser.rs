// Parser: converts a flat token stream into an AST.
//
// Hand-written recursive descent parser — no parser combinator
// library, no code generation. Grammar precedence is encoded
// directly in the call chain:
//
//   expression  -> or
//   or          -> and ( "or" and )*
//   and         -> equality ( "and" equality )*
//   equality    -> comparison ( (==|!=) comparison )*
//   comparison  -> term ( (<|>|<=|>=) term )*
//   term        -> factor ( (+|-) factor )*
//   factor      -> unary ( (*|/|%) unary )*
//   unary       -> (-|!) unary | primary
//   primary     -> number | string | bool | ident | call | "(" expression ")"

use crate::ast::*;
use crate::token::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    fn peek_at(&self, offset: usize) -> &TokenKind {
        &self.tokens[self.pos + offset].kind
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens[self.pos].clone();
        if !matches!(tok.kind, TokenKind::Eof) {
            self.pos += 1;
        }
        tok
    }

    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(self.peek()) == std::mem::discriminant(kind)
    }

    fn match_tok(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: &TokenKind, msg: &str) -> Result<Token, String> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(format!(
                "{} — found {:?} at line {}",
                msg,
                self.peek(),
                self.tokens[self.pos].line
            }))
        }
    }

    // ── Top-level program: a sequence of statements ──────────────

    pub fn parse_program(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::Eof) {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    // ── Statements ───────────────────────────────────────────────

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.peek() {
            TokenKind::Let => self.parse_let(),
            TokenKind::Print => self.parse_print(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::Fn => self.parse_fn(),
            TokenKind::Return => self.parse_return(),
            TokenKind::LBrace => self.parse_block_stmt(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_let(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'let'
        let name = match self.advance().kind {
            TokenKind::Ident(s) => s,
            ref k => return Err(format!(
                "Expected identifier after 'let' — found {:?} at line {}",
                k,
                self.tokens[self.pos - 1].line
            )),
        };
        self.expect(&TokenKind::Assign, "Expected '=' after let name")?;
        let expr = self.parse_expr()?;
        self.match_tok(&TokenKind::Semicolon);
        Ok(Stmt::Let(name, expr))
    }

    fn parse_print(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'print'
        let expr = self.parse_expr()?;
        self.match_tok(&TokenKind::Semicolon);
        Ok(Stmt::Print(expr))
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'if'
        let cond = self.parse_expr()?;
        let then_body = self.parse_block()?;
        let else_body = if self.match_tok(&TokenKind::Else) {
            if self.check(&TokenKind::If) {
                // else if → desugar to nested if
                vec![self.parse_if()?]
            } else {
                self.parse_block()?
            }
        } else {
            Vec::new()
        };
        Ok(Stmt::If(
            cond,
            then_body,
            if else_body.is_empty() { None } else { Some(else_body) },
        ))
    }

    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'while'
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::While(cond, body))
    }

    fn parse_fn(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'fn'
        let name = match self.advance().kind {
            TokenKind::Ident(s) => s,
            ref k => return Err(format!(
                "Expected function name — found {:?} at line {}",
                k,
                self.tokens[self.pos - 1].line
            )),
        };
        self.expect(&TokenKind::LParen, "Expected '(' after function name")?;
        let mut params = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                match self.advance().kind {
                    TokenKind::Ident(s) => params.push(s),
                    ref k => return Err(format!(
                        "Expected parameter name — found {:?} at line {}",
                        k,
                        self.tokens[self.pos - 1].line
                    )),
                }
                if !self.match_tok(&TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(&TokenKind::RParen, "Expected ')' after parameters")?;
        let body = self.parse_block()?;
        Ok(Stmt::FnDef(name, params, body))
    }

    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'return'
        // bare `return;` with no expression
        if self.check(&TokenKind::Semicolon) || self.check(&TokenKind::RBrace) {
            self.match_tok(&TokenKind::Semicolon);
            return Ok(Stmt::Return(None));
        }
        let expr = self.parse_expr()?;
        self.match_tok(&TokenKind::Semicolon);
        Ok(Stmt::Return(Some(expr)))
    }

    fn parse_block_stmt(&mut self) -> Result<Stmt, String> {
        let block = self.parse_block()?;
        // A bare block is executed as a sequence — we wrap the first
        // statement. For v1 simplicity, blocks are inlined by the caller.
        if block.is_empty() {
            Ok(Stmt::Expr(Expr::Number(0.0)))
        } else {
            Ok(Stmt::Expr(self.parse_expr()?))
        }
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        self.expect(&TokenKind::LBrace, "Expected '{' to start block")?;
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(&TokenKind::RBrace, "Expected '}' to end block")?;
        Ok(stmts)
    }

    fn parse_expr_stmt(&mut self) -> Result<Stmt, String> {
        let expr = self.parse_expr()?;
        self.match_tok(&TokenKind::Semicolon);
        Ok(Stmt::Expr(expr))
    }

    // ── Expression precedence climbing ──────────────────────────

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        while self.match_tok(&TokenKind::Or) {
            let right = self.parse_and()?;
            left = Expr::BinOp(Box::new(left), BinOpKind::Or, Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_equality()?;
        while self.match_tok(&TokenKind::And) {
            let right = self.parse_equality()?;
            left = Expr::BinOp(Box::new(left), BinOpKind::And, Box::new(right));
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;
        loop {
            let op = match self.peek() {
                TokenKind::EqEq => BinOpKind::Eq,
                TokenKind::BangEq => BinOpKind::Neq,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::BinOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_term()?;
        loop {
            let op = match self.peek() {
                TokenKind::Lt => BinOpKind::Lt,
                TokenKind::Gt => BinOpKind::Gt,
                TokenKind::LtEq => BinOpKind::LtEq,
                TokenKind::GtEq => BinOpKind::GtEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_term()?;
            left = Expr::BinOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_factor()?;
        loop {
            let op = match self.peek() {
                TokenKind::Plus => BinOpKind::Add,
                TokenKind::Minus => BinOpKind::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_factor()?;
            left = Expr::BinOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                TokenKind::Star => BinOpKind::Mul,
                TokenKind::Slash => BinOpKind::Div,
                TokenKind::Percent => BinOpKind::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        match self.peek() {
            TokenKind::Minus => {
                self.advance();
                Ok(Expr::UnaryOp(
                    UnaryOpKind::Neg,
                    Box::new(self.parse_unary()?),
                ))
            }
            TokenKind::Bang => {
                self.advance();
                Ok(Expr::UnaryOp(
                    UnaryOpKind::Not,
                    Box::new(self.parse_unary()?),
                ))
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::Number(n) => Ok(Expr::Number(n)),
            TokenKind::String(s) => Ok(Expr::String(s)),
            TokenKind::True => Ok(Expr::Bool(true)),
            TokenKind::False => Ok(Expr::Bool(false)),
            TokenKind::Ident(name) => {
                if self.check(&TokenKind::LParen) {
                    self.advance(); // '('
                    let mut args = Vec::new();
                    if !self.check(&TokenKind::RParen) {
                        loop {
                            args.push(self.parse_expr()?);
                            if !self.match_tok(&TokenKind::Comma) {
                                break;
                            }
                        }
                    }
                    self.expect(&TokenKind::RParen, "Expected ')' after arguments")?;
                    Ok(Expr::Call(name, args))
                } else {
                    Ok(Expr::Ident(name))
                }
            }
            TokenKind::LParen => {
                let expr = self.parse_expr()?;
                self.expect(&TokenKind::RParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            ref k => Err(format!(
                "Unexpected token {:?} at line {} while parsing expression",
                k,
                tok.line
            )),
        }
    }
}
