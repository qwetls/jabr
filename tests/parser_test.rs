// Integration tests for the Jabr parser.
//
// These verify that the parser produces the correct AST for
// expressions and statements, and that precedence is handled
// correctly (e.g. `1 + 2 * 3` parses as `1 + (2 * 3)`).

use jabr::ast::*;
use jabr::lexer::Lexer;
use jabr::parser::Parser;

fn parse_expr(src: &str) -> Expr {
    let mut lx = Lexer::new(src);
    let tokens = lx.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    // Wrap in a print statement to parse a single expression
    let program = parser.parse_program().unwrap();
    match &program[0] {
        Stmt::Print(e) => e.clone(),
        Stmt::Expr(e) => e.clone(),
        _ => panic!("Expected expression statement, got {:?}", program[0]),
    }
}

fn parse_stmts(src: &str) -> Vec<Stmt> {
    let mut lx = Lexer::new(src);
    let tokens = lx.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    parser.parse_program().unwrap()
}

#[test]
fn number_literal() {
    assert_eq!(parse_expr("print 42;"), Expr::Number(42.0));
}

#[test]
fn string_literal() {
    assert_eq!(parse_expr("print \"hello\";"), Expr::String("hello".into()));
}

#[test]
fn bool_literals() {
    assert_eq!(parse_expr("print true;"), Expr::Bool(true));
    assert_eq!(parse_expr("print false;"), Expr::Bool(false));
}

#[test]
fn variable_reference() {
    assert_eq!(parse_expr("print x;"), Expr::Ident("x".into()));
}

#[test]
fn precedence_add_mul() {
    // 1 + 2 * 3  →  1 + (2 * 3)
    let ast = parse_expr("print 1 + 2 * 3;");
    match ast {
        Expr::BinOp(left, BinOpKind::Add, right) => {
            assert_eq!(*left, Expr::Number(1.0));
            match *right {
                Expr::BinOp(l2, BinOpKind::Mul, r2) => {
                    assert_eq!(*l2, Expr::Number(2.0));
                    assert_eq!(*r2, Expr::Number(3.0));
                }
                _ => panic!("Expected BinOp(Mul) on right side"),
            }
        }
        _ => panic!("Expected BinOp(Add) at root"),
    }
}

#[test]
fn precedence_paren_override() {
    // (1 + 2) * 3  →  (1 + 2) * 3
    let ast = parse_expr("print (1 + 2) * 3;");
    match ast {
        Expr::BinOp(left, BinOpKind::Mul, right) => {
            assert_eq!(*right, Expr::Number(3.0));
            match *left {
                Expr::BinOp(l2, BinOpKind::Add, r2) => {
                    assert_eq!(*l2, Expr::Number(1.0));
                    assert_eq!(*r2, Expr::Number(2.0));
                }
                _ => panic!("Expected BinOp(Add) in left"),
            }
        }
        _ => panic!("Expected BinOp(Mul) at root"),
    }
}

#[test]
fn left_associative_subtraction() {
    // 1 - 2 - 3  →  (1 - 2) - 3
    let ast = parse_expr("print 1 - 2 - 3;");
    match ast {
        Expr::BinOp(left, BinOpKind::Sub, right) => {
            assert_eq!(*right, Expr::Number(3.0));
            match *left {
                Expr::BinOp(l2, BinOpKind::Sub, r2) => {
                    assert_eq!(*l2, Expr::Number(1.0));
                    assert_eq!(*r2, Expr::Number(2.0));
                }
                _ => panic!("Expected nested BinOp(Sub)"),
            }
        }
        _ => panic!("Expected BinOp(Sub) at root"),
    }
}

#[test]
fn unary_negation() {
    let ast = parse_expr("print -5;");
    assert_eq!(ast, Expr::UnaryOp(UnaryOpKind::Neg, Box::new(Expr::Number(5.0))));
}

#[test]
fn unary_not() {
    let ast = parse_expr("print !true;");
    assert_eq!(ast, Expr::UnaryOp(UnaryOpKind::Not, Box::new(Expr::Bool(true))));
}

#[test]
fn let_statement() {
    let stmts = parse_stmts("let x = 10;");
    assert_eq!(stmts, vec![Stmt::Let("x".into(), Expr::Number(10.0))]);
}

#[test]
fn print_statement() {
    let stmts = parse_stmts("print 42;");
    assert_eq!(stmts, vec![Stmt::Print(Expr::Number(42.0))]);
}

#[test]
fn if_statement() {
    let stmts = parse_stmts("if true { print 1; } else { print 2; }");
    assert!(matches!(&stmts[0], Stmt::If(_, _, _)));
}

#[test]
fn while_statement() {
    let stmts = parse_stmts("while true { print 1; }");
    assert!(matches!(&stmts[0], Stmt::While(_, _)));
}

#[test]
fn function_definition() {
    let stmts = parse_stmts("fn add(a, b) { return a + b; }");
    match &stmts[0] {
        Stmt::FnDef(name, params, body) => {
            assert_eq!(name, "add");
            assert_eq!(params, &vec!["a".to_string(), "b".to_string()]);
            assert!(!body.is_empty());
        }
        _ => panic!("Expected FnDef"),
    }
}

#[test]
fn function_call() {
    let ast = parse_expr("print add(1, 2);");
    match ast {
        Expr::Call(name, args) => {
            assert_eq!(name, "add");
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected Call"),
    }
}

#[test]
fn comparison_operators() {
    let ast = parse_expr("print 1 < 2;");
    assert_eq!(ast, Expr::BinOp(
        Box::new(Expr::Number(1.0)),
        BinOpKind::Lt,
        Box::new(Expr::Number(2.0)),
    ));
}

#[test]
fn equality_operators() {
    let ast = parse_expr("print 1 == 1;");
    assert_eq!(ast, Expr::BinOp(
        Box::new(Expr::Number(1.0)),
        BinOpKind::Eq,
        Box::new(Expr::Number(1.0)),
    ));
}

#[test]
fn logical_operators() {
    let ast = parse_expr("print true and false or true;");
    // or is lowest precedence: (true and false) or true
    match ast {
        Expr::BinOp(left, BinOpKind::Or, right) => {
            assert_eq!(*right, Expr::Bool(true));
            match *left {
                Expr::BinOp(l2, BinOpKind::And, r2) => {
                    assert_eq!(*l2, Expr::Bool(true));
                    assert_eq!(*r2, Expr::Bool(false));
                }
                _ => panic!("Expected And inside"),
            }
        }
        _ => panic!("Expected Or at root"),
    }
}

#[test]
fn parse_error_on_unexpected_token() {
    let mut lx = Lexer::new("let = 5;");
    let tokens = lx.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    assert!(parser.parse_program().is_err());
}
