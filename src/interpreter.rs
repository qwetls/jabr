// Tree-walking interpreter: executes the AST directly.
//
// This is the simplest execution model — no bytecode, no VM, no
// compilation. The interpreter holds a global environment (variable
// bindings) and a function table, then walks the statement list.
//
// Control flow (return) uses Rust's Result<Flow, Error> as a
// signal — Ok(Flow::Normal) continues, Ok(Flow::Return(value))
// unwinds the call stack.

use crate::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Unit,
}

#[derive(Debug, Clone)]
enum Flow {
    Normal,
    Return(Value),
}

pub struct Interpreter {
    globals: HashMap<String, Value>,
    functions: HashMap<String, (Vec<String>, Vec<Stmt>)>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            globals: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn run(&mut self, program: &[Stmt]) -> Result<(), String> {
        for stmt in program {
            self.exec_stmt(stmt, &mut self.globals.clone())?;
        }
        Ok(())
    }

    fn exec_stmt(&mut self, stmt: &Stmt, env: &mut HashMap<String, Value>) -> Result<Flow, String> {
        match stmt {
            Stmt::Let(name, expr) => {
                let val = self.eval_expr(expr, env)?;
                env.insert(name.clone(), val);
                Ok(Flow::Normal)
            }
            Stmt::Print(expr) => {
                let val = self.eval_expr(expr, env)?;
                println!("{}", self.format_value(&val));
                Ok(Flow::Normal)
            }
            Stmt::Expr(expr) => {
                self.eval_expr(expr, env)?;
                Ok(Flow::Normal)
            }
            Stmt::If(cond, then_body, else_body) => {
                let c = self.eval_expr(cond, env)?;
                if self.is_truthy(&c) {
                    for s in then_body {
                        match self.exec_stmt(s, env)? {
                            Flow::Return(v) => return Ok(Flow::Return(v)),
                            Flow::Normal => {}
                        }
                    }
                } else if let Some(elses) = else_body {
                    for s in elses {
                        match self.exec_stmt(s, env)? {
                            Flow::Return(v) => return Ok(Flow::Return(v)),
                            Flow::Normal => {}
                        }
                    }
                }
                Ok(Flow::Normal)
            }
            Stmt::While(cond, body) => {
                loop {
                    let c = self.eval_expr(cond, env)?;
                    if !self.is_truthy(&c) {
                        break;
                    }
                    for s in body {
                        match self.exec_stmt(s, env)? {
                            Flow::Return(v) => return Ok(Flow::Return(v)),
                            Flow::Normal => {}
                        }
                    }
                }
                Ok(Flow::Normal)
            }
            Stmt::FnDef(name, params, body) => {
                self.functions.insert(name.clone(), (params.clone(), body.clone()));
                Ok(Flow::Normal)
            }
            Stmt::Return(opt_expr) => {
                let val = match opt_expr {
                    Some(e) => self.eval_expr(e, env)?,
                    None => Value::Unit,
                };
                Ok(Flow::Return(val))
            }
        }
    }

    fn eval_expr(&mut self, expr: &Expr, env: &mut HashMap<String, Value>) -> Result<Value, String> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Ident(name) => {
                env.get(name)
                    .cloned()
                    .or_else(|| self.globals.get(name).cloned())
                    .ok_or_else(|| format!("Undefined variable '{}'", name))
            }
            Expr::UnaryOp(op, inner) => {
                let val = self.eval_expr(inner, env)?;
                match (op, val) {
                    (UnaryOpKind::Neg, Value::Number(n)) => Ok(Value::Number(-n)),
                    (UnaryOpKind::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                    (UnaryOpKind::Neg, v) => Err(format!("Cannot negate {:?}", v)),
                    (UnaryOpKind::Not, v) => Err(format!("Cannot logically negate {:?}", v)),
                }
            }
            Expr::BinOp(left, op, right) => {
                // Short-circuit for and/or
                match op {
                    BinOpKind::And => {
                        let l = self.eval_expr(left, env)?;
                        if !self.is_truthy(&l) {
                            return Ok(Value::Bool(false));
                        }
                        let r = self.eval_expr(right, env)?;
                        Ok(Value::Bool(self.is_truthy(&r)))
                    }
                    BinOpKind::Or => {
                        let l = self.eval_expr(left, env)?;
                        if self.is_truthy(&l) {
                            return Ok(Value::Bool(true));
                        }
                        let r = self.eval_expr(right, env)?;
                        Ok(Value::Bool(self.is_truthy(&r)))
                    }
                    _ => {
                        let l = self.eval_expr(left, env)?;
                        let r = self.eval_expr(right, env)?;
                        self.eval_binop(*op, l, r)
                    }
                }
            }
            Expr::Call(name, args) => {
                let (params, body) = match self.functions.get(name) {
                    Some(f) => f.clone(),
                    None => return Err(format!("Undefined function '{}'", name)),
                };

                if args.len() != params.len() {
                    return Err(format!("Function '{}' expects {} args, got {}",
                                      name, params.len(), args.len()));
                }

                let mut call_env = self.globals.clone();
                for (param, arg_expr) in params.iter().zip(args.iter()) {
                    let arg_val = self.eval_expr(arg_expr, env)?;
                    call_env.insert(param.clone(), arg_val);
                }

                for s in &body {
                    match self.exec_stmt(s, &mut call_env)? {
                        Flow::Return(v) => return Ok(v),
                        Flow::Normal => {}
                    }
                }
                Ok(Value::Unit)
            }
        }
    }

    fn eval_binop(&self, op: BinOpKind, l: Value, r: Value) -> Result<Value, String> {
        match (&l, &r) {
            (Value::Number(a), Value::Number(b)) => {
                let result = match op {
                    BinOpKind::Add => a + b,
                    BinOpKind::Sub => a - b,
                    BinOpKind::Mul => a * b,
                    BinOpKind::Div => {
                        if *b == 0.0 {
                            return Err("Division by zero".into());
                        }
                        a / b
                    }
                    BinOpKind::Mod => {
                        if *b == 0.0 {
                            return Err("Modulo by zero".into());
                        }
                        a % b
                    }
                    BinOpKind::Eq => return Ok(Value::Bool(a == b)),
                    BinOpKind::Neq => return Ok(Value::Bool(a != b)),
                    BinOpKind::Lt => return Ok(Value::Bool(a < b)),
                    BinOpKind::Gt => return Ok(Value::Bool(a > b)),
                    BinOpKind::LtEq => return Ok(Value::Bool(a <= b)),
                    BinOpKind::GtEq => return Ok(Value::Bool(a >= b)),
                    BinOpKind::And | BinOpKind::Or => unreachable!(),
                };
                Ok(Value::Number(result))
            }
            (Value::String(a), Value::String(b)) => {
                match op {
                    BinOpKind::Add => Ok(Value::String(format!("{}{}", a, b))),
                    BinOpKind::Eq => Ok(Value::Bool(a == b)),
                    BinOpKind::Neq => Ok(Value::Bool(a != b)),
                    _ => Err(format!("Cannot {:?} two strings", op)),
                }
            }
            (Value::Bool(a), Value::Bool(b)) => {
                match op {
                    BinOpKind::Eq => Ok(Value::Bool(a == b)),
                    BinOpKind::Neq => Ok(Value::Bool(a != b)),
                    _ => Err(format!("Cannot {:?} two booleans", op)),
                }
            }
            _ => Err(format!("Type mismatch: {:?} {:?} {:?}", l, op, r)),
        }
    }

    fn is_truthy(&self, val: &Value) -> bool {
        match val {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Unit => false,
        }
    }

    fn format_value(&self, val: &Value) -> String {
        match val {
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Value::String(s) => s.clone(),
            Value::Bool(b) => format!("{}", b),
            Value::Unit => "unit".into(),
        }
    }
}
