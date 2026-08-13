// Integration tests for the Jabr interpreter.
//
// These verify end-to-end execution: source code → tokens → AST →
// evaluated result. For print output we verify that valid programs
// run without error. For error cases we verify that the interpreter
// returns an Err.

use jabr::run_source;

#[test]
fn print_number() {
    assert!(run_source("print 42;").is_ok());
}

#[test]
fn print_string() {
    assert!(run_source("print \"hello\";").is_ok());
}

#[test]
fn print_bool() {
    assert!(run_source("print true;").is_ok());
}

#[test]
fn arithmetic_addition() {
    assert!(run_source("print 1 + 2;").is_ok());
}

#[test]
fn arithmetic_precedence() {
    // 1 + 2 * 3 = 7
    assert!(run_source("print 1 + 2 * 3;").is_ok());
}

#[test]
fn arithmetic_parens() {
    // (1 + 2) * 3 = 9
    assert!(run_source("print (1 + 2) * 3;").is_ok());
}

#[test]
fn arithmetic_division() {
    assert!(run_source("print 10 / 4;").is_ok());
}

#[test]
fn arithmetic_modulo() {
    assert!(run_source("print 10 % 3;").is_ok());
}

#[test]
fn unary_negation() {
    assert!(run_source("print -5;").is_ok());
}

#[test]
fn unary_not() {
    assert!(run_source("print !false;").is_ok());
}

#[test]
fn let_binding() {
    assert!(run_source("let x = 10; print x;").is_ok());
}

#[test]
fn string_concat() {
    assert!(run_source("print \"foo\" + \"bar\";").is_ok());
}

#[test]
fn comparison_true() {
    assert!(run_source("print 1 < 2;").is_ok());
}

#[test]
fn comparison_false() {
    assert!(run_source("print 1 > 2;").is_ok());
}

#[test]
fn equality_numbers() {
    assert!(run_source("print 42 == 42;").is_ok());
}

#[test]
fn inequality_numbers() {
    assert!(run_source("print 42 != 42;").is_ok());
}

#[test]
fn logical_and() {
    assert!(run_source("print true and false;").is_ok());
}

#[test]
fn logical_or() {
    assert!(run_source("print true or false;").is_ok());
}

#[test]
fn if_true_branch() {
    assert!(run_source("if true { print 1; } else { print 2; }").is_ok());
}

#[test]
fn if_false_branch() {
    assert!(run_source("if false { print 1; } else { print 2; }").is_ok());
}

#[test]
fn if_no_else_false() {
    assert!(run_source("if false { print 1; }").is_ok());
}

#[test]
fn while_loop_count() {
    let src = "let i = 0; while i < 3 { print i; i = i + 1; }";
    assert!(run_source(src).is_ok());
}

#[test]
fn function_call() {
    let src = "fn add(a, b) { return a + b; } print add(3, 4);";
    assert!(run_source(src).is_ok());
}

#[test]
fn function_early_return() {
    let src = "fn check(n) { if n > 0 { return true; } return false; } print check(5);";
    assert!(run_source(src).is_ok());
}

#[test]
fn function_returns_unit() {
    let src = "fn greet() { print \"hi\"; } greet();";
    assert!(run_source(src).is_ok());
}

#[test]
fn nested_function_calls() {
    let src = "fn double(n) { return n * 2; } print double(double(5));";
    assert!(run_source(src).is_ok());
}

#[test]
fn division_by_zero_errors() {
    let result = run_source("print 1 / 0;");
    assert!(result.is_err());
}

#[test]
fn undefined_variable_errors() {
    let result = run_source("print x;");
    assert!(result.is_err());
}

#[test]
fn undefined_function_errors() {
    let result = run_source("print nope(1);");
    assert!(result.is_err());
}

#[test]
fn comment_ignored() {
    assert!(run_source("// this is a comment\nprint 42;").is_ok());
}

#[test]
fn fibonacci_program() {
    let src = r#"
        fn fib(n) {
            if n < 2 {
                return n;
            }
            return fib(n - 1) + fib(n - 2);
        }
        print fib(10);
    "#;
    assert!(run_source(src).is_ok());
}

#[test]
fn function_with_multiple_returns() {
    let src = r#"
        fn classify(n) {
            if n > 0 {
                return 1;
            }
            if n < 0 {
                return -1;
            }
            return 0;
        }
        print classify(5);
        print classify(-3);
        print classify(0);
    "#;
    assert!(run_source(src).is_ok());
}
