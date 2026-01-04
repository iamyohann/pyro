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
        generics: Vec<String>,
        params: Vec<(String, Type)>,
        body: Rc<Vec<Stmt>>,
    },
    List(Rc<Vec<Value>>), // Immutable
    Tuple(Rc<Vec<Value>>),
    Set(Rc<Vec<Value>>),
    Dict(Rc<Vec<(Value, Value)>>),
    
    // Mutable
    ListMutable(Rc<RefCell<Vec<Value>>>),
    TupleMutable(Rc<RefCell<Vec<Value>>>),
    SetMutable(Rc<RefCell<Vec<Value>>>),
    DictMutable(Rc<RefCell<Vec<(Value, Value)>>>),
    
    BuiltinMethod {
        object: Box<Value>,
        name: String,
    },

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
            Stmt::FnDecl { name, generics, params, body, .. } => {
                self.globals.insert(name, Value::Function { generics, params, body: Rc::new(body) });
            }
            Stmt::Import(path) => {
                println!("Importing module: {}", path);
                // Implementation will come with module resolution
            }
            Stmt::StructDef { .. } | Stmt::InterfaceDef { .. } | Stmt::TypeAlias { .. } => {
                // Not yet supported in interpreter
            }
        }
        Ok(None)
    }

    pub fn evaluate(&mut self, expr: Expr) -> Result<Value, String> {
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
                Ok(Value::List(Rc::new(vals)))
            }
            Expr::Tuple(elements) => {
                let mut vals = Vec::new();
                for e in elements {
                    vals.push(self.evaluate(e)?);
                }
                Ok(Value::Tuple(Rc::new(vals)))
            }
            Expr::Set(elements) => {
                let mut vals = Vec::new();
                for e in elements {
                    vals.push(self.evaluate(e)?);
                }
                Ok(Value::Set(Rc::new(vals)))
            }
            Expr::Dict(elements) => {
                let mut vals = Vec::new();
                for (k, v) in elements {
                    let key = self.evaluate(k)?;
                    let val = self.evaluate(v)?;
                    vals.push((key, val));
                }
                Ok(Value::Dict(Rc::new(vals)))
            }
            Expr::Identifier(name) => {
                if name == "print" 
                   || name == "ListMutable" 
                   || name == "TupleMutable" 
                   || name == "SetMutable" 
                   || name == "DictMutable" {
                    // special hack for built-ins
                   return Ok(Value::String(Rc::new(name))); 
                }
                
                self.globals.get(&name).cloned().ok_or_else(|| format!("Undefined variable: {}", name))
            }
            Expr::Get { object, name } => {
                let obj_val = self.evaluate(*object)?;
                Ok(Value::BuiltinMethod {
                    object: Box::new(obj_val),
                    name,
                })
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
                
                // Hacky built-ins
                if let Value::String(s) = &func_val {
                    let name = s.as_str();
                    if name == "print" {
                        for arg in args {
                             let v = self.evaluate(arg)?;
                             println!("{:?}", v);
                        }
                        return Ok(Value::Void);
                    }
                     if name == "ListMutable" {
                         // Expect 1 arg: List
                         if args.len() != 1 { return Err("ListMutable takes 1 argument".to_string()); }
                         let v = self.evaluate(args[0].clone())?;
                         match v {
                             Value::List(l) => return Ok(Value::ListMutable(Rc::new(RefCell::new((*l).clone())))),
                             _ => return Err("ListMutable expects a List".to_string()),
                         }
                    }
                    if name == "TupleMutable" {
                         if args.len() != 1 { return Err("TupleMutable takes 1 argument".to_string()); }
                         let v = self.evaluate(args[0].clone())?;
                         match v {
                             Value::Tuple(l) => return Ok(Value::TupleMutable(Rc::new(RefCell::new((*l).clone())))),
                             _ => return Err("TupleMutable expects a Tuple".to_string()),
                         }
                    }
                    if name == "SetMutable" {
                         if args.len() != 1 { return Err("SetMutable takes 1 argument".to_string()); }
                         let v = self.evaluate(args[0].clone())?;
                         match v {
                             Value::Set(l) => return Ok(Value::SetMutable(Rc::new(RefCell::new((*l).clone())))),
                             _ => return Err("SetMutable expects a Set".to_string()),
                         }
                    }
                    if name == "DictMutable" {
                         if args.len() != 1 { return Err("DictMutable takes 1 argument".to_string()); }
                         let v = self.evaluate(args[0].clone())?;
                         match v {
                             Value::Dict(l) => return Ok(Value::DictMutable(Rc::new(RefCell::new((*l).clone())))),
                             _ => return Err("DictMutable expects a Dict".to_string()),
                         }
                    }
                }
                
                match func_val {
                    Value::Function { generics: _, params, body } => {
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
                    Value::BuiltinMethod { object, name } => {
                        let mut evaluated_args = Vec::new();
                        for arg in args {
                            evaluated_args.push(self.evaluate(arg)?);
                        }
                        self.call_method(*object, &name, evaluated_args)
                    }
                    _ => Err("Not a function".to_string()),
                }
            }
        }
    }

    fn call_method(&mut self, object: Value, name: &str, args: Vec<Value>) -> Result<Value, String> {
        match object {
            Value::ListMutable(list_rc) => {
                let mut list = list_rc.borrow_mut();
                match name {
                    "push" => {
                        if args.len() != 1 { return Err("push expects 1 argument".to_string()); }
                        list.push(args[0].clone());
                        Ok(Value::Void)
                    }
                    "pop" => {
                        if !args.is_empty() { return Err("pop expects 0 arguments".to_string()); }
                        Ok(list.pop().unwrap_or(Value::Void))
                    }
                    "len" => Ok(Value::Int(list.len() as i64)),
                    "clear" => {
                        list.clear();
                        Ok(Value::Void)
                    }
                    "insert" => {
                        if args.len() != 2 { return Err("insert expects 2 arguments: index, value".to_string()); }
                        match args[0] {
                            Value::Int(idx) => {
                                let idx = idx as usize;
                                if idx > list.len() { return Err("Index out of bounds".to_string()); }
                                list.insert(idx, args[1].clone());
                                Ok(Value::Void)
                            }
                            _ => Err("insert index must be an integer".to_string()),
                        }
                    }
                    "remove" => {
                        if args.len() != 1 { return Err("remove expects 1 argument".to_string()); }
                         if let Some(pos) = list.iter().position(|x| *x == args[0]) {
                             list.remove(pos);
                         }
                         Ok(Value::Void)
                    }
                    "reverse" => {
                        list.reverse();
                        Ok(Value::Void)
                    }
                    _ => Err(format!("Method '{}' not found on ListMutable", name)),
                }
            }
            Value::List(list_rc) => {
                match name {
                    "len" => Ok(Value::Int(list_rc.len() as i64)),
                     "push" | "pop" | "clear" | "insert" | "remove" | "reverse" => {
                        Err(format!("Cannot call '{}' on immutable List. Use ListMutable if modifications are needed.", name))
                    }
                    _ => Err(format!("Method '{}' not found on List", name)),
                }
            }
            Value::DictMutable(dict_rc) => {
                let mut dict = dict_rc.borrow_mut();
                match name {
                    "keys" => {
                        let keys: Vec<Value> = dict.iter().map(|(k, _)| k.clone()).collect();
                        Ok(Value::List(Rc::new(keys)))
                    }
                    "values" => {
                         let vals: Vec<Value> = dict.iter().map(|(_, v)| v.clone()).collect();
                         Ok(Value::List(Rc::new(vals)))
                    }
                    "items" => {
                         let items: Vec<Value> = dict.iter().map(|(k, v)| {
                             Value::Tuple(Rc::new(vec![k.clone(), v.clone()]))
                         }).collect();
                         Ok(Value::List(Rc::new(items)))
                    }
                    "len" => Ok(Value::Int(dict.len() as i64)),
                    "clear" => {
                        dict.clear();
                        Ok(Value::Void)
                    }
                    "remove" => {
                        if args.len() != 1 { return Err("remove expects 1 argument (key)".to_string()); }
                        let key = &args[0];
                         if let Some(pos) = dict.iter().position(|(k, _)| k == key) {
                             dict.remove(pos);
                         }
                         Ok(Value::Void)
                    }
                    "get" => {
                        if args.len() != 1 { return Err("get expects 1 argument (key)".to_string()); }
                        let key = &args[0];
                        for (k, v) in dict.iter() {
                            if k == key {
                                return Ok(v.clone());
                            }
                        }
                        Ok(Value::Void)
                    }
                    _ => Err(format!("Method '{}' not found on DictMutable", name)),
                }
            }
             Value::Dict(dict_rc) => {
                 match name {
                    "keys" => {
                        let keys: Vec<Value> = dict_rc.iter().map(|(k, _)| k.clone()).collect();
                        Ok(Value::List(Rc::new(keys)))
                    }
                    "values" => {
                         let vals: Vec<Value> = dict_rc.iter().map(|(_, v)| v.clone()).collect();
                         Ok(Value::List(Rc::new(vals)))
                    }
                    "items" => {
                         let items: Vec<Value> = dict_rc.iter().map(|(k, v)| {
                             Value::Tuple(Rc::new(vec![k.clone(), v.clone()]))
                         }).collect();
                         Ok(Value::List(Rc::new(items)))
                    }
                    "len" => Ok(Value::Int(dict_rc.len() as i64)),
                    "get" => {
                         if args.len() != 1 { return Err("get expects 1 argument (key)".to_string()); }
                        let key = &args[0];
                        for (k, v) in dict_rc.iter() {
                            if k == key {
                                return Ok(v.clone());
                            }
                        }
                        Ok(Value::Void)
                    }
                    _ => Err(format!("Method '{}' not found on Dict", name)),
                }
            }
            Value::SetMutable(set_rc) => {
                let mut set = set_rc.borrow_mut();
                match name {
                    "add" => {
                         if args.len() != 1 { return Err("add expects 1 argument".to_string()); }
                         if !set.contains(&args[0]) {
                             set.push(args[0].clone());
                         }
                         Ok(Value::Void)
                    }
                    "remove" => {
                        if args.len() != 1 { return Err("remove expects 1 argument".to_string()); }
                        if let Some(pos) = set.iter().position(|x| *x == args[0]) {
                             set.remove(pos);
                        }
                        Ok(Value::Void)
                    }
                    "contains" => {
                         if args.len() != 1 { return Err("contains expects 1 argument".to_string()); }
                         Ok(Value::Bool(set.contains(&args[0])))
                    }
                    "len" => Ok(Value::Int(set.len() as i64)),
                    _ => Err(format!("Method '{}' not found on SetMutable", name)),
                }
            }
            Value::Set(set_rc) => {
                match name {
                    "contains" => {
                         if args.len() != 1 { return Err("contains expects 1 argument".to_string()); }
                         Ok(Value::Bool(set_rc.contains(&args[0])))
                    }
                    "len" => Ok(Value::Int(set_rc.len() as i64)),
                    _ => Err(format!("Method '{}' not found on Set", name)),
                }
            }
            Value::String(s) => {
                match name {
                    "len" => Ok(Value::Int(s.len() as i64)),
                    "upper" => Ok(Value::String(Rc::new(s.to_uppercase()))),
                    "lower" => Ok(Value::String(Rc::new(s.to_lowercase()))),
                    "split" => {
                         if args.len() != 1 { return Err("split expects 1 argument (delimiter)".to_string()); }
                         match &args[0] {
                             Value::String(delim) => {
                                 let parts: Vec<Value> = s.split(delim.as_str())
                                     .map(|p| Value::String(Rc::new(p.to_string())))
                                     .collect();
                                 Ok(Value::List(Rc::new(parts)))
                             }
                             _ => Err("split expects a string delimiter".to_string()),
                         }
                    }
                    "contains" => {
                         if args.len() != 1 { return Err("contains expects 1 argument".to_string()); }
                         match &args[0] {
                             Value::String(sub) => Ok(Value::Bool(s.contains(sub.as_str()))),
                             _ => Err("contains argument must be a string".to_string()),
                         }
                    }
                    _ => Err(format!("Method '{}' not found on String", name)),
                }
            }
             _ => Err(format!("Type does not support method '{}'", name)),
        }
    }
}
