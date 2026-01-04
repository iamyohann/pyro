use crate::ast::{BinaryOp, Expr, Stmt, Type};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    // Managed by RC
    String(Rc<String>), 
    Function {
        params: Vec<(String, Type)>,
        body: Rc<Vec<Stmt>>,
    },
    List(Rc<RefCell<Vec<Value>>>),
    Void,
}

pub struct Interpreter {
    // Nested scopes: push hashmap on entry, pop on exit
    // optimizing to single scope for now for simplicity
    globals: HashMap<String, Value>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
        }
    }

    pub fn run(&mut self, statements: Vec<Stmt>) -> Result<Option<Value>, String> {
        for stmt in statements {
            if let Some(v) = self.execute_stmt(stmt)? {
                return Ok(Some(v));
            }
        }
        Ok(None)
    }

    fn execute_stmt(&mut self, stmt: Stmt) -> Result<Option<Value>, String> {
        match stmt {
            Stmt::VarDecl { name, value, .. } => {
                let val = self.evaluate(value)?;
                self.globals.insert(name, val);
            }
            Stmt::Expr(expr) => {
                self.evaluate(expr)?;
            }
            Stmt::Return(expr) => {
                let val = if let Some(e) = expr {
                    self.evaluate(e)?
                } else {
                    Value::Void
                };
                return Ok(Some(val));
            }
            Stmt::If { cond, then_block, else_block } => {
                let cond_val = self.evaluate(cond)?;
                let truthy = match cond_val {
                    Value::Bool(b) => b,
                    _ => return Err("Condition must be boolean".to_string()),
                };

                if truthy {
                    if let Some(v) = self.run(then_block)? {
                        return Ok(Some(v));
                    }
                } else if let Some(else_stmts) = else_block {
                    if let Some(v) = self.run(else_stmts)? {
                        return Ok(Some(v));
                    }
                }
            }
            Stmt::While { cond, body } => {
                while let Value::Bool(true) = self.evaluate(cond.clone())? {
                    if let Some(v) = self.run(body.clone())? {
                        return Ok(Some(v));
                    }
                }
            }
            Stmt::Assign { name, value } => {
                if !self.globals.contains_key(&name) {
                    return Err(format!("Undefined variable '{}' in assignment", name));
                }
                // TODO: Check mutability
                let val = self.evaluate(value)?;
                self.globals.insert(name, val);
            }
            Stmt::FnDecl { name, params, body, .. } => {
                self.globals.insert(name, Value::Function { params, body: Rc::new(body) });
            }
        }
        Ok(None)
    }

    fn evaluate(&mut self, expr: Expr) -> Result<Value, String> {
        match expr {
            Expr::LiteralInt(i) => Ok(Value::Int(i)),
            Expr::LiteralFloat(f) => Ok(Value::Float(f)),
            Expr::LiteralBool(b) => Ok(Value::Bool(b)),
            Expr::LiteralString(s) => Ok(Value::String(Rc::new(s))),
            Expr::List(elements) => {
                let mut vals = Vec::new();
                for e in elements {
                    vals.push(self.evaluate(e)?);
                }
                Ok(Value::List(Rc::new(RefCell::new(vals))))
            }
            Expr::Identifier(name) => {
                if name == "print" {
                    // special hack for print
                   return Ok(Value::String(Rc::new("print".to_string()))); 
                }
                
                self.globals.get(&name).cloned().ok_or_else(|| format!("Undefined variable: {}", name))
            }
            Expr::Binary { left, op, right } => {
                let l = self.evaluate(*left)?;
                let r = self.evaluate(*right)?;
                
                match (l, op, r) {
                    (Value::Int(a), BinaryOp::Add, Value::Int(b)) => Ok(Value::Int(a + b)),
                    (Value::Int(a), BinaryOp::Sub, Value::Int(b)) => Ok(Value::Int(a - b)),
                    (Value::Int(a), BinaryOp::Mul, Value::Int(b)) => Ok(Value::Int(a * b)),
                    (Value::Int(a), BinaryOp::Div, Value::Int(b)) => Ok(Value::Int(a / b)),
                    (Value::Int(a), BinaryOp::Gt, Value::Int(b)) => Ok(Value::Bool(a > b)),
                    (Value::Int(a), BinaryOp::Lt, Value::Int(b)) => Ok(Value::Bool(a < b)),
                    // Add more ops
                    _ => Err("Unsupported operation".to_string()),
                }
            }
            Expr::Call { function, args } => {
                let func_val = self.evaluate(*function)?;
                
                // Hacky print built-in
                if let Value::String(s) = &func_val {
                    if s.as_str() == "print" {
                        for arg in args {
                             let v = self.evaluate(arg)?;
                             println!("{:?}", v);
                        }
                        return Ok(Value::Void);
                    }
                }
                
                match func_val {
                    Value::Function { params, body } => {
                        // TODO: Implement proper stack frames
                        // For now just setting globals (WRONG but works for simple script)
                        for (i, (param_name, _)) in params.iter().enumerate() {
                            let arg_val = self.evaluate(args[i].clone())?;
                            self.globals.insert(param_name.clone(), arg_val);
                        }
                        // Clone Rc pointer
                        let result = self.run((*body).clone())?;
                        Ok(result.unwrap_or(Value::Void))
                    }
                    _ => Err("Not a function".to_string()),
                }
            }
        }
    }
}
