use crate::ast::{BinaryOp, Expr, Stmt, Type};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio;

#[derive(Clone)]
pub struct NativeClosure(pub Arc<dyn Fn(Vec<Value>) -> Result<Value, Value> + Send + Sync>);

impl std::fmt::Debug for NativeClosure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}

impl PartialEq for NativeClosure {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    // Managed by Arc
    String(Arc<String>), 
    Function {
        generics: Vec<String>,
        params: Vec<(String, Type)>,
        body: Arc<Vec<Stmt>>,
        partial_args: Vec<Value>, // For currying
    },
    List(Arc<Vec<Value>>), // Immutable
    Tuple(Arc<Vec<Value>>),
    Set(Arc<Vec<Value>>),
    Dict(Arc<Vec<(Value, Value)>>),
    
    Class {
        name: String,
        parent: Option<String>,
        methods: Arc<HashMap<String, Value>>, // methods are Function values
    },
    Instance {
        class_name: String,
        fields: Arc<RwLock<HashMap<String, Value>>>,
        methods: Arc<HashMap<String, Value>>, // shared from Class
    },
    BoundMethod {
        object: Box<Value>, // Instance
        method: Box<Value>, // Function
    },
    // Records
    Record {
        name: String,
        fields: Arc<Vec<String>>, 
        values: Arc<Vec<Value>>,
        methods: Arc<HashMap<String, Value>>,
    },
    RecordConstructor {
        name: String,
        fields: Vec<String>, // Field names
        methods: Arc<HashMap<String, Value>>,
        partial_args: Vec<Value>, // For currying
    },

    // Mutable
    ListMutable(Arc<RwLock<Vec<Value>>>),
    TupleMutable(Arc<RwLock<Vec<Value>>>),
    SetMutable(Arc<RwLock<Vec<Value>>>),
    DictMutable(Arc<RwLock<Vec<(Value, Value)>>>),
    
    BuiltinMethod {
        object: Box<Value>,
        name: String,
    },

    NativeFunction {
        name: String,
        func: NativeClosure,
    },
    NativeModule(Arc<HashMap<String, Value>>),

    Void,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Function { .. }, Value::Function { .. }) => false, // Functions not equitable
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Tuple(a), Value::Tuple(b)) => a == b,
            (Value::Set(a), Value::Set(b)) => a == b,
            (Value::Dict(a), Value::Dict(b)) => a == b, // Arc<Vec> equality
            
            (Value::Class { name: n1, .. }, Value::Class { name: n2, .. }) => n1 == n2,
            
            (Value::Instance { fields: f1, .. }, Value::Instance { fields: f2, .. }) => {
                 if Arc::ptr_eq(f1, f2) { return true; }
                 
                 let map1 = f1.read().unwrap();
                 let map2 = f2.read().unwrap();
                 if map1.len() != map2.len() { return false; }
                 for (k, v) in map1.iter() {
                     match map2.get(k) {
                         Some(v2) => if v != v2 { return false; },
                         None => return false,
                     }
                 }
                 true
             },
             
             (Value::ListMutable(l1), Value::ListMutable(l2)) => {
                 if Arc::ptr_eq(l1, l2) { return true; }
                 let v1 = l1.read().unwrap();
                 let v2 = l2.read().unwrap();
                 *v1 == *v2
             },
             (Value::TupleMutable(l1), Value::TupleMutable(l2)) => {
                 if Arc::ptr_eq(l1, l2) { return true; }
                 let v1 = l1.read().unwrap();
                 let v2 = l2.read().unwrap();
                 *v1 == *v2
             },
             (Value::SetMutable(l1), Value::SetMutable(l2)) => {
                 if Arc::ptr_eq(l1, l2) { return true; }
                 let v1 = l1.read().unwrap();
                 let v2 = l2.read().unwrap();
                 *v1 == *v2
             },
             (Value::DictMutable(l1), Value::DictMutable(l2)) => {
                 if Arc::ptr_eq(l1, l2) { return true; }
                 let v1 = l1.read().unwrap();
                 let v2 = l2.read().unwrap();
                 *v1 == *v2
             },
             
             (Value::Void, Value::Void) => true,
             
             _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Flow {
    Return(Value),
    Break,
    Continue,
    None,
}

#[derive(Clone)]
pub struct Interpreter {
    // Nested scopes: push hashmap on entry, pop on exit
    // optimizing to single scope for now for simplicity
    globals: HashMap<String, Value>,
    native_modules: HashMap<String, Value>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut globals = HashMap::new();

        // Define built-in Error class
        // class Error:
        //     def __init__(self, message):
        //         self.message = message
        let init_body = vec![
            Stmt::Set {
                object: Expr::Identifier("self".to_string()),
                name: "message".to_string(),
                value: Expr::Identifier("message".to_string()),
            }
        ];

        let init_func = Value::Function {
            generics: Vec::new(),
            params: vec![("self".to_string(), Type::Void), ("message".to_string(), Type::String)],
            body: Arc::new(init_body),
            partial_args: Vec::new(),
        };

        let mut error_methods = HashMap::new();
        error_methods.insert("__init__".to_string(), init_func);

        globals.insert("Error".to_string(), Value::Class {
            name: "Error".to_string(),
            parent: None,
            methods: Arc::new(error_methods),
        });

        let mut interpreter = Self {
            globals,
            native_modules: HashMap::new(),
        };
        crate::stdlib::register_std_libs(&mut interpreter);
        interpreter
    }

    pub fn register_native_module(&mut self, name: &str, module: Value) {
        self.native_modules.insert(name.to_string(), module);
    }

    pub fn has_native_module(&self, name: &str) -> bool {
        self.native_modules.contains_key(name)
    }
    
    fn make_error(&self, msg: &str) -> Value {
        // Construct an instance of Error
        let mut fields = HashMap::new();
        fields.insert("message".to_string(), Value::String(Arc::new(msg.to_string())));
        
        let methods = if let Some(Value::Class { methods, .. }) = self.globals.get("Error") {
            methods.clone()
        } else {
             Arc::new(HashMap::new())
        };

        Value::Instance {
            class_name: "Error".to_string(),
            fields: Arc::new(RwLock::new(fields)),
            methods,
        }
    }


    pub fn run(&mut self, statements: Vec<Stmt>) -> Result<Flow, Value> {
        for stmt in statements {
            let flow = self.execute_stmt(stmt)?;
            match flow {
                Flow::None => continue,
                _ => return Ok(flow),
            }
        }
        Ok(Flow::None)
    }

    fn execute_stmt(&mut self, stmt: Stmt) -> Result<Flow, Value> {
        match stmt {
            Stmt::Try { body, catch_var, catch_body, finally_body } => {
                let result = self.run(body);
                
                let mut flow_result = Ok(Flow::None); // default

                if let Err(e) = result {
                    // Exception occurred
                    if let Some(catch_block) = catch_body {
                         // Enter implicit scope (simplified for now)
                         let mut old_globals = self.globals.clone(); // inefficient but works for now as scope push
                         
                         if let Some(var_name) = catch_var {
                             self.globals.insert(var_name, e);
                         }

                         let catch_res = self.run(catch_block);
                         
                         // Restore scope
                         self.globals = old_globals;

                         if let Err(new_e) = catch_res {
                             flow_result = Err(new_e);
                         } else {
                             flow_result = catch_res;
                         }

                    } else {
                        // No catch block, propagate error (delayed until finally runs)
                        flow_result = Err(e);
                    }
                } else {
                    flow_result = result;
                }

                // Finally block
                if let Some(finally_block) = finally_body {
                     // Run finally, if it errors/returns/breaks it overrides previous result
                     let fin_res = self.run(finally_block);
                     match fin_res {
                         Ok(Flow::None) => {
                             // Finally finished normally, return previous result
                             return flow_result;
                         }
                         _ => {
                             // Finally returned/broke/raised, override previous result
                             return fin_res;
                         }
                     }
                }

                return flow_result;
            }
            Stmt::Raise { error, cause } => {
                let err_val = self.evaluate(error)?;
                // Ignoring cause for now or wrap it
                return Err(err_val);
            }
            Stmt::Go(expr) => {
                // We need to clone the interpreter to move it into the thread
                // Since everything is Arc<RwLock>, this is a shallow clone of references
                let mut new_interpreter = self.clone();
                let expr_clone = expr.clone();
                
                tokio::spawn(async move {
                     // In a real implementation, we might want to handle the error
                     // Or provide a way to join the handle.
                     // For 'go' fire and forget style:
                     if let Err(e) = new_interpreter.evaluate(*expr_clone) {
                         eprintln!("Error in go routine: {:?}", e);
                     }
                });
            }
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
                return Ok(Flow::Return(val));
            }
            Stmt::Break => return Ok(Flow::Break),
            Stmt::Continue => return Ok(Flow::Continue),
            Stmt::If { cond, then_block, else_block } => {
                let cond_val = self.evaluate(cond)?;
                let truthy = match cond_val {
                    Value::Bool(b) => b,
                    _ => return Err(self.make_error("Condition must be boolean")),
                };

                if truthy {
                    let flow = self.run(then_block)?;
                    if flow != Flow::None { return Ok(flow); }
                } else if let Some(else_stmts) = else_block {
                    let flow = self.run(else_stmts)?;
                    if flow != Flow::None { return Ok(flow); }
                }
            }
            Stmt::While { cond, body } => {
                while let Value::Bool(true) = self.evaluate(cond.clone())? {
                    let flow = self.run(body.clone())?;
                    match flow {
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Break => break,
                        Flow::Continue => continue,
                        Flow::None => {},
                    }
                }
            }
            Stmt::Assign { name, value } => {
                if !self.globals.contains_key(&name) {
                    return Err(self.make_error(&format!("Undefined variable '{}' in assignment", name)));
                }
                let val = self.evaluate(value)?;
                self.globals.insert(name, val);
            }
            Stmt::Set { object, name, value } => {
                let obj_val = self.evaluate(object)?;
                let val = self.evaluate(value)?;
                
                match obj_val {
                    Value::Instance { fields, .. } => {
                        fields.write().unwrap().insert(name, val);
                    }
                    _ => return Err(self.make_error("Only instances have fields")),
                }
            }
            Stmt::FnDecl { name, generics, params, body, .. } => {
                self.globals.insert(name, Value::Function { generics, params, body: Arc::new(body), partial_args: Vec::new() });
            }
            Stmt::Import(path) => {
                if let Some(module) = self.native_modules.get(&path) {
                    // Simple binding strategy: use the last segment as the name
                    // e.g. "std.math" -> "math"
                    let name = path.split('.').last().unwrap_or(&path).to_string();
                    self.globals.insert(name, module.clone());
                } else {
                     println!("Warning: Module '{}' not found", path);
                }
            }
            Stmt::RecordDef { name, generics: _, fields, methods } => {
                let mut field_names = Vec::new();
                for (n, _) in fields {
                    field_names.push(n);
                }
                
                let mut method_map = HashMap::new();
                for method in methods {
                    if let Stmt::FnDecl { name, generics, params, return_type: _, body } = method {
                         method_map.insert(name, Value::Function { generics, params, body: Arc::new(body), partial_args: Vec::new() });
                    }
                }

                self.globals.insert(name.clone(), Value::RecordConstructor { 
                    name, 
                    fields: field_names, 
                    methods: Arc::new(method_map),
                    partial_args: Vec::new() 
                });
            }
            Stmt::InterfaceDef { .. } | Stmt::TypeAlias { .. } => {
                // Not yet supported
            }
            Stmt::For { item_name, iterable, body } => {
                let iterable_val = self.evaluate(iterable)?;
                let items = match iterable_val {
                    Value::List(items) => items,
                    Value::ListMutable(items) => items.read().unwrap().clone().into(),
                    Value::Tuple(items) => items,
                    Value::Set(items) => items,
                    _ => return Err(self.make_error("For loop expects iterable")),
                };

                for item in items.iter() {
                    self.globals.insert(item_name.clone(), item.clone());
                    let flow = self.run(body.clone())?;
                    match flow {
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Break => break,
                        Flow::Continue => continue,
                        Flow::None => {},
                    }
                }
            }
            Stmt::ClassDecl { name, parent, methods } => {
                let mut method_map = HashMap::new();
                
                if let Some(parent_name) = &parent {
                     if let Some(Value::Class { methods: parent_methods, .. }) = self.globals.get(parent_name) {
                         for (k, v) in parent_methods.iter() {
                             method_map.insert(k.clone(), v.clone());
                         }
                     } else {
                         return Err(self.make_error(&format!("Parent class '{}' not found", parent_name)));
                     }
                }

                for method in methods {
                    if let Stmt::FnDecl { name, generics, params, body, .. } = method {
                        method_map.insert(name.clone(), Value::Function { generics, params, body: Arc::new(body), partial_args: Vec::new() });
                    }
                }
                self.globals.insert(name.clone(), Value::Class { name, parent, methods: Arc::new(method_map) });
            }
        }
        Ok(Flow::None)
    }

    pub fn evaluate(&mut self, expr: Expr) -> Result<Value, Value> {
        match expr {
            Expr::LiteralInt(i) => Ok(Value::Int(i)),
            Expr::LiteralFloat(f) => Ok(Value::Float(f)),
            Expr::LiteralBool(b) => Ok(Value::Bool(b)),
            Expr::LiteralString(s) => Ok(Value::String(Arc::new(s))),
            Expr::List(elements) => {
                let mut vals = Vec::new();
                for e in elements {
                    vals.push(self.evaluate(e)?);
                }
                Ok(Value::List(Arc::new(vals)))
            }
            Expr::Tuple(elements) => {
                let mut vals = Vec::new();
                for e in elements {
                    vals.push(self.evaluate(e)?);
                }
                Ok(Value::Tuple(Arc::new(vals)))
            }
            Expr::Set(elements) => {
                let mut vals = Vec::new();
                for e in elements {
                    vals.push(self.evaluate(e)?);
                }
                Ok(Value::Set(Arc::new(vals)))
            }
            Expr::Dict(elements) => {
                let mut vals = Vec::new();
                for (k, v) in elements {
                    let key = self.evaluate(k)?;
                    let val = self.evaluate(v)?;
                    vals.push((key, val));
                }
                Ok(Value::Dict(Arc::new(vals)))
            }
            Expr::Identifier(name) => {
                if name == "print" 
                   || name == "range"
                   || name == "ListMutable" 
                   || name == "TupleMutable" 
                   || name == "SetMutable" 
                   || name == "DictMutable" {
                    // special hack for built-ins
                   return Ok(Value::String(Arc::new(name))); 
                }
                
                self.globals.get(&name).cloned().ok_or_else(|| self.make_error(&format!("Undefined variable: {}", name)))
            }
            Expr::Get { object, name } => {
                let obj_val = self.evaluate(*object)?;
                match obj_val {
                    Value::Instance { ref fields, ref methods, class_name: _ } => {
                        // Check fields first
                        if let Some(val) = fields.read().unwrap().get(&name) {
                            return Ok(val.clone());
                        }
                        // Check methods
                        if let Some(method) = methods.get(&name) {
                            return Ok(Value::BoundMethod {
                                object: Box::new(Value::Instance { 
                                    class_name: "".to_string(), 
                                    fields: fields.clone(), 
                                    methods: methods.clone() 
                                }), 
                                method: Box::new(method.clone()),
                            });
                        }
                    }
                    Value::Record { name: ref rec_name, fields, values, methods } => {
                        // Check fields
                         if let Some(pos) = fields.iter().position(|f| f == &name) {
                             return Ok(values[pos].clone());
                         }
                         // Check methods
                         if let Some(func) = methods.get(&name) {
                             return Ok(Value::BoundMethod {
                                    object: Box::new(Value::Record { name: rec_name.clone(), fields: fields.clone(), values: values.clone(), methods: methods.clone() }),
                                    method: Box::new(func.clone()),
                             });
                        }
                        return Err(self.make_error(&format!("Field or method '{}' not found on Record", name)));
                    }
                    _ => {}
                }

                if let Value::NativeModule(exports) = &obj_val {
                    if let Some(val) = exports.get(&name) {
                        return Ok(val.clone());
                    }
                    return Err(self.make_error(&format!("Export '{}' not found in module", name)));
                }
                
                // Fallback for built-in method hack (str.len, list.push) 
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
                    (Value::Int(a), BinaryOp::Div, Value::Int(b)) => {
                        if b == 0 {
                            return Err(self.make_error("Division by zero"));
                        }
                        Ok(Value::Int(a / b))
                    },
                    (Value::Int(a), BinaryOp::Gt, Value::Int(b)) => Ok(Value::Bool(a > b)),
                    (Value::Int(a), BinaryOp::Lt, Value::Int(b)) => Ok(Value::Bool(a < b)),
                    (Value::Int(a), BinaryOp::Eq, Value::Int(b)) => Ok(Value::Bool(a == b)),
                    (Value::Int(a), BinaryOp::Neq, Value::Int(b)) => Ok(Value::Bool(a != b)),
                    (Value::Int(a), BinaryOp::Gte, Value::Int(b)) => Ok(Value::Bool(a >= b)),
                    (Value::Int(a), BinaryOp::Lte, Value::Int(b)) => Ok(Value::Bool(a <= b)),
                    (Value::String(a), BinaryOp::Add, Value::String(b)) => Ok(Value::String(Arc::new(format!("{}{}", a, b)))),
                    (Value::String(a), BinaryOp::Eq, Value::String(b)) => Ok(Value::Bool(a == b)),
                    (Value::String(a), BinaryOp::Neq, Value::String(b)) => Ok(Value::Bool(a != b)),
                    
                    // Float ops
                    (Value::Float(a), BinaryOp::Add, Value::Float(b)) => Ok(Value::Float(a + b)),
                    (Value::Float(a), BinaryOp::Sub, Value::Float(b)) => Ok(Value::Float(a - b)),
                    (Value::Float(a), BinaryOp::Mul, Value::Float(b)) => Ok(Value::Float(a * b)),
                    (Value::Float(a), BinaryOp::Div, Value::Float(b)) => Ok(Value::Float(a / b)),
                    (Value::Float(a), BinaryOp::Gt, Value::Float(b)) => Ok(Value::Bool(a > b)),
                    (Value::Float(a), BinaryOp::Lt, Value::Float(b)) => Ok(Value::Bool(a < b)),
                    (Value::Float(a), BinaryOp::Eq, Value::Float(b)) => Ok(Value::Bool(a == b)),
                    (Value::Float(a), BinaryOp::Neq, Value::Float(b)) => Ok(Value::Bool(a != b)),
                    (Value::Float(a), BinaryOp::Gte, Value::Float(b)) => Ok(Value::Bool(a >= b)),
                    (Value::Float(a), BinaryOp::Lte, Value::Float(b)) => Ok(Value::Bool(a <= b)),

                    // Bool ops
                    (Value::Bool(a), BinaryOp::Eq, Value::Bool(b)) => Ok(Value::Bool(a == b)),
                    (Value::Bool(a), BinaryOp::Neq, Value::Bool(b)) => Ok(Value::Bool(a != b)),

                    _ => Err(self.make_error("Unsupported operation")),
                }
            }
            Expr::Index { object, index } => {
                let obj_val = self.evaluate(*object)?;
                let idx_val = self.evaluate(*index)?;

                match obj_val {
                    Value::List(l) => {
                        if let Value::Int(i) = idx_val {
                             let idx = i as usize;
                             if i < 0 || idx >= l.len() { return Err(self.make_error("Index out of bounds")); }
                             Ok(l[idx].clone())
                        } else { Err(self.make_error("List index must be integer")) }
                    }
                    Value::ListMutable(l) => {
                         let list = l.read().unwrap();
                         if let Value::Int(i) = idx_val {
                            let idx = i as usize;
                            if i < 0 || idx >= list.len() { return Err(self.make_error("Index out of bounds")); }
                            Ok(list[idx].clone())
                        } else { Err(self.make_error("List index must be integer")) }
                    }
                    Value::Tuple(l) => {
                         if let Value::Int(i) = idx_val {
                             let idx = i as usize;
                             if i < 0 || idx >= l.len() { return Err(self.make_error("Index out of bounds")); }
                             Ok(l[idx].clone())
                        } else { Err(self.make_error("Tuple index must be integer")) }
                    }
                    Value::TupleMutable(l) => {
                         let list = l.read().unwrap();
                         if let Value::Int(i) = idx_val {
                            let idx = i as usize;
                            if i < 0 || idx >= list.len() { return Err(self.make_error("Index out of bounds")); }
                            Ok(list[idx].clone())
                        } else { Err(self.make_error("Tuple index must be integer")) }
                    }
                    Value::String(s) => {
                         if let Value::Int(i) = idx_val {
                             let idx = i as usize;
                             if i < 0 || idx >= s.len() { return Err(self.make_error("Index out of bounds")); }
                             if let Some(c) = s.chars().nth(idx) {
                                 Ok(Value::String(Arc::new(c.to_string())))
                             } else {
                                  Err(self.make_error("Index out of bounds"))
                             }
                        } else { Err(self.make_error("String index must be integer")) }
                    }
                    Value::Dict(d) => {
                         for (k, v) in d.iter() {
                            if k == &idx_val {
                                return Ok(v.clone());
                            }
                        }
                        Err(self.make_error("Key error"))
                    }
                    Value::DictMutable(d) => {
                        let dict = d.read().unwrap();
                         for (k, v) in dict.iter() {
                            if k == &idx_val {
                                return Ok(v.clone());
                            }
                        }
                        Err(self.make_error("Key error"))
                    }
                    _ => Err(self.make_error("Type is not indexable")),
                }
            }
            Expr::Call { function, args } => {
                let func_val = self.evaluate(*function)?;
                
                let mut evaluated_args = Vec::new();
                for arg_expr in args {
                     evaluated_args.push(self.evaluate(arg_expr)?);
                }
                
                return self.apply(func_val, evaluated_args);


            }
        }
    }

    fn call_method(&mut self, object: Value, name: &str, args: Vec<Value>) -> Result<Value, Value> {
        match object {
            Value::ListMutable(list_rc) => {
                let mut list = list_rc.write().unwrap();
                match name {
                    "push" => {
                        if args.len() != 1 { return Err(self.make_error("push expects 1 argument")); }
                        list.push(args[0].clone());
                        Ok(Value::Void)
                    }
                    "pop" => {
                        if !args.is_empty() { return Err(self.make_error("pop expects 0 arguments")); }
                        Ok(list.pop().unwrap_or(Value::Void))
                    }
                    "len" => Ok(Value::Int(list.len() as i64)),
                    "clear" => {
                        list.clear();
                        Ok(Value::Void)
                    }
                    "insert" => {
                        if args.len() != 2 { return Err(self.make_error("insert expects 2 arguments: index, value")); }
                        match args[0] {
                            Value::Int(idx) => {
                                let idx = idx as usize;
                                if idx > list.len() { return Err(self.make_error("Index out of bounds")); }
                                list.insert(idx, args[1].clone());
                                Ok(Value::Void)
                            }
                            _ => Err(self.make_error("insert index must be an integer")),
                        }
                    }
                    "remove" => {
                        if args.len() != 1 { return Err(self.make_error("remove expects 1 argument")); }
                         if let Some(pos) = list.iter().position(|x| *x == args[0]) {
                             list.remove(pos);
                         }
                         Ok(Value::Void)
                    }
                    "reverse" => {
                        list.reverse();
                        Ok(Value::Void)
                    }
                    _ => Err(self.make_error(&format!("Method '{}' not found on ListMutable", name))),
                }
            }
            Value::List(list_rc) => {
                match name {
                    "len" => Ok(Value::Int(list_rc.len() as i64)),
                     "push" | "pop" | "clear" | "insert" | "remove" | "reverse" => {
                        Err(self.make_error(&format!("Cannot call '{}' on immutable List. Use ListMutable if modifications are needed.", name)))
                    }
                    _ => Err(self.make_error(&format!("Method '{}' not found on List", name))),
                }
            }
            Value::DictMutable(dict_rc) => {
                let mut dict = dict_rc.write().unwrap();
                match name {
                    "keys" => {
                        let keys: Vec<Value> = dict.iter().map(|(k, _)| k.clone()).collect();
                        Ok(Value::List(Arc::new(keys)))
                    }
                    "values" => {
                         let vals: Vec<Value> = dict.iter().map(|(_, v)| v.clone()).collect();
                         Ok(Value::List(Arc::new(vals)))
                    }
                    "items" => {
                         let items: Vec<Value> = dict.iter().map(|(k, v)| {
                             Value::Tuple(Arc::new(vec![k.clone(), v.clone()]))
                         }).collect();
                         Ok(Value::List(Arc::new(items)))
                    }
                    "len" => Ok(Value::Int(dict.len() as i64)),
                    "clear" => {
                        dict.clear();
                        Ok(Value::Void)
                    }
                    "remove" => {
                        if args.len() != 1 { return Err(self.make_error("remove expects 1 argument (key)")); }
                        let key = &args[0];
                         if let Some(pos) = dict.iter().position(|(k, _)| k == key) {
                             dict.remove(pos);
                         }
                         Ok(Value::Void)
                    }
                    "get" => {
                        if args.len() != 1 { return Err(self.make_error("get expects 1 argument (key)")); }
                        let key = &args[0];
                        for (k, v) in dict.iter() {
                            if k == key {
                                return Ok(v.clone());
                            }
                        }
                        Ok(Value::Void)
                    }
                    _ => Err(self.make_error(&format!("Method '{}' not found on DictMutable", name))),
                }
            }
             Value::Dict(dict_rc) => {
                 match name {
                    "keys" => {
                        let keys: Vec<Value> = dict_rc.iter().map(|(k, _)| k.clone()).collect();
                        Ok(Value::List(Arc::new(keys)))
                    }
                    "values" => {
                         let vals: Vec<Value> = dict_rc.iter().map(|(_, v)| v.clone()).collect();
                         Ok(Value::List(Arc::new(vals)))
                    }
                    "items" => {
                         let items: Vec<Value> = dict_rc.iter().map(|(k, v)| {
                             Value::Tuple(Arc::new(vec![k.clone(), v.clone()]))
                         }).collect();
                         Ok(Value::List(Arc::new(items)))
                    }
                    "len" => Ok(Value::Int(dict_rc.len() as i64)),
                    "get" => {
                         if args.len() != 1 { return Err(self.make_error("get expects 1 argument (key)")); }
                        let key = &args[0];
                        for (k, v) in dict_rc.iter() {
                            if k == key {
                                return Ok(v.clone());
                            }
                        }
                        Ok(Value::Void)
                    }
                    _ => Err(self.make_error(&format!("Method '{}' not found on Dict", name))),
                }
            }
            Value::SetMutable(set_rc) => {
                let mut set = set_rc.write().unwrap();
                match name {
                    "add" => {
                         if args.len() != 1 { return Err(self.make_error("add expects 1 argument")); }
                         if !set.contains(&args[0]) {
                             set.push(args[0].clone());
                         }
                         Ok(Value::Void)
                    }
                    "remove" => {
                        if args.len() != 1 { return Err(self.make_error("remove expects 1 argument")); }
                        if let Some(pos) = set.iter().position(|x| *x == args[0]) {
                             set.remove(pos);
                        }
                        Ok(Value::Void)
                    }
                    "contains" => {
                         if args.len() != 1 { return Err(self.make_error("contains expects 1 argument")); }
                         Ok(Value::Bool(set.contains(&args[0])))
                    }
                    "len" => Ok(Value::Int(set.len() as i64)),
                    _ => Err(self.make_error(&format!("Method '{}' not found on SetMutable", name))),
                }
            }
            Value::Set(set_rc) => {
                match name {
                    "contains" => {
                         if args.len() != 1 { return Err(self.make_error("contains expects 1 argument")); }
                         Ok(Value::Bool(set_rc.contains(&args[0])))
                    }
                    "len" => Ok(Value::Int(set_rc.len() as i64)),
                    _ => Err(self.make_error(&format!("Method '{}' not found on Set", name))),
                }
            }
            Value::String(s) => {
                match name {
                    "len" => Ok(Value::Int(s.len() as i64)),
                    "upper" => Ok(Value::String(Arc::new(s.to_uppercase()))),
                    "lower" => Ok(Value::String(Arc::new(s.to_lowercase()))),
                    "split" => {
                         if args.len() != 1 { return Err(self.make_error("split expects 1 argument (delimiter)")); }
                         match &args[0] {
                             Value::String(delim) => {
                                 let parts: Vec<Value> = s.split(delim.as_str())
                                     .map(|p| Value::String(Arc::new(p.to_string())))
                                     .collect();
                                 Ok(Value::List(Arc::new(parts)))
                             }
                             _ => Err(self.make_error("split expects a string delimiter")),
                         }
                    }
                    "contains" => {
                         if args.len() != 1 { return Err(self.make_error("contains expects 1 argument")); }
                         match &args[0] {
                             Value::String(sub) => Ok(Value::Bool(s.contains(sub.as_str()))),
                             _ => Err(self.make_error("contains argument must be a string")),
                         }
                    }
                    _ => Err(self.make_error(&format!("Method '{}' not found on String", name))),
                }
            }
             _ => Err(self.make_error(&format!("Type does not support method '{}'", name))),
        }
    }

    // Helper for applying arguments with currying support
    fn apply(&mut self, func: Value, args: Vec<Value>) -> Result<Value, Value> {
        match func {
            Value::Function { generics, params, body, partial_args } => {
                let mut all_args = partial_args.clone();
                all_args.extend(args);

                if all_args.len() < params.len() {
                    // Partial application
                    return Ok(Value::Function {
                        generics,
                        params,
                        body,
                        partial_args: all_args,
                    });
                } else if all_args.len() == params.len() {
                    // Full execution
                    let mut new_env = self.globals.clone();
                    for (i, val) in all_args.iter().enumerate() {
                        new_env.insert(params[i].0.clone(), val.clone());
                    }
                    
                    let old_globals = self.globals.clone();
                    self.globals = new_env;
                    let result = self.run(body.to_vec());
                    self.globals = old_globals;
                    
                    match result {
                        Ok(Flow::Return(v)) => Ok(v),
                        Ok(Flow::None) => Ok(Value::Void),
                        Ok(Flow::Break) => Err(self.make_error("Unexpected 'break' outside of loop")),
                        Ok(Flow::Continue) => Err(self.make_error("Unexpected 'continue' outside of loop")),
                        Err(e) => Err(e),
                    }
                } else {
                    // Over-application
                    let (needed, remaining) = all_args.split_at(params.len());
                    let result = self.apply(Value::Function {
                        generics: generics.clone(),
                        params: params.clone(),
                        body: body.clone(),
                        partial_args: needed.to_vec(),
                    }, Vec::new())?;
                    
                    self.apply(result, remaining.to_vec())
                }
            }
            Value::RecordConstructor { name, fields, methods, partial_args } => {
                let mut all_args = partial_args.clone();
                all_args.extend(args);
                
                if all_args.len() < fields.len() {
                    return Ok(Value::RecordConstructor {
                        name,
                        fields,
                        methods,
                        partial_args: all_args,
                    });
                } else if all_args.len() == fields.len() {
                    return Ok(Value::Record {
                        name,
                        fields: Arc::new(fields),
                        values: Arc::new(all_args),
                        methods,
                    });
                } else {
                     // Over-application
                    let (needed, remaining) = all_args.split_at(fields.len());
                    let result = self.apply(Value::RecordConstructor { 
                        name: name.clone(), 
                        fields: fields.clone(), 
                        methods: methods.clone(),
                        partial_args: needed.to_vec() 
                    }, Vec::new())?;
                     self.apply(result, remaining.to_vec())
                }
            }
            Value::Class { name, methods, .. } => {
                 let instance = Value::Instance {
                     class_name: name.clone(),
                     fields: Arc::new(RwLock::new(HashMap::new())),
                     methods: methods.clone(),
                 };
                 if let Some(init_method) = methods.get("__init__") {
                     if let Value::Function { generics, params, body, partial_args } = init_method {
                         let mut init_args = vec![instance.clone()];
                         init_args.extend(args);
                         
                         self.apply(Value::Function {
                             generics: generics.clone(),
                             params: params.clone(),
                             body: body.clone(),
                             partial_args: partial_args.clone(),
                         }, init_args)?;
                     }
                 }
                 Ok(instance)
            }
            Value::BoundMethod { object, method } => {
                let call_args = args;
                if let Value::Function { generics: ref generics, params: ref params, body: ref body, partial_args: ref partial_args } = *method {
                     if partial_args.is_empty() && !params.is_empty() {
                         let mut new_partial = vec![*object.clone()];
                         new_partial.extend(partial_args.clone()); 
                         return self.apply(Value::Function {
                             generics: generics.clone(), params: params.clone(), body: body.clone(), partial_args: new_partial
                         }, call_args);
                     } else {
                         return self.apply(*method, call_args);
                     }
                }
                Err(self.make_error("BoundMethod expects Function"))
            }
            Value::BuiltinMethod { object, name } => {
                 self.call_method(*object, &name, args)
            }
            Value::NativeFunction { name: _, func } => {
                (func.0)(args).map_err(|e| {
                    if let Value::String(s) = &e {
                        return self.make_error(s);
                    }
                    e
                })
            }
            Value::String(s) => {
                 let name = s.as_str();
                 if name == "print" {
                    for arg in args {
                        println!("{:?}", arg);
                    }
                    Ok(Value::Void)
                 } else if name == "range" {
                     if args.len() < 1 || args.len() > 3 { return Err(self.make_error("range expects 1 to 3 arguments")); }
                        let start = if args.len() == 1 { 0 } else { match args[0] { Value::Int(i) => i, _ => return Err(self.make_error("start int")) } };
                        let end = if args.len() == 1 { match args[0] { Value::Int(i) => i, _ => return Err(self.make_error("end int")) } } else { match args[1] { Value::Int(i) => i, _ => return Err(self.make_error("end int")) } };
                        let step = if args.len() == 3 { match args[2] { Value::Int(i) => i, _ => return Err(self.make_error("step int")) } } else { 1 };
                        
                        let mut vals = Vec::new();
                        let mut current = start;
                        if step > 0 { while current < end { vals.push(Value::Int(current)); current += step; } }
                        else { while current > end { vals.push(Value::Int(current)); current += step; } }
                        Ok(Value::List(Arc::new(vals)))
                 } else if name == "ListMutable" {
                     if args.len() != 1 { return Err(self.make_error("ListMutable takes 1 arg")); }
                     match &args[0] { Value::List(l) => Ok(Value::ListMutable(Arc::new(RwLock::new((**l).clone())))), _ => Err(self.make_error("Expects List")) }
                 } else if name == "TupleMutable" {
                     if args.len() != 1 { return Err(self.make_error("TupleMutable takes 1 arg")); }
                     match &args[0] { Value::Tuple(l) => Ok(Value::TupleMutable(Arc::new(RwLock::new((**l).clone())))), _ => Err(self.make_error("Expects Tuple")) }
                 } else if name == "SetMutable" {
                     if args.len() != 1 { return Err(self.make_error("SetMutable takes 1 arg")); }
                     match &args[0] { Value::Set(l) => Ok(Value::SetMutable(Arc::new(RwLock::new((**l).clone())))), _ => Err(self.make_error("Expects Set")) }
                 } else if name == "DictMutable" {
                     if args.len() != 1 { return Err(self.make_error("DictMutable takes 1 arg")); }
                     match &args[0] { Value::Dict(l) => Ok(Value::DictMutable(Arc::new(RwLock::new((**l).clone())))), _ => Err(self.make_error("Expects Dict")) }
                 } else {
                     Err(self.make_error(&format!("Unknown builtin function: {}", name)))
                 }
            }
            _ => Err(self.make_error(&format!("Not callable: {:?}", func))),
        }
    }
}
