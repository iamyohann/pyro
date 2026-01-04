use crate::ast::{BinaryOp, Expr, Stmt, Type};

pub struct Transpiler {
    output: String,
}

impl Transpiler {
    pub fn new() -> Self {
        Self {
            output: String::new(),
        }
    }

    pub fn transpile(&mut self, statements: Vec<Stmt>) -> String {
        self.output.clear();
        // Add prelude/helper code if necessary
        // For now, minimal rust
        
        for stmt in statements {
            self.transpile_stmt(stmt, 0);
        }
        
        self.output.clone()
    }

    fn push_indent(&mut self, indent: usize) {
        for _ in 0..indent {
            self.output.push_str("    ");
        }
    }

    fn transpile_stmt(&mut self, stmt: Stmt, indent: usize) {
        self.push_indent(indent);
        match stmt {
            Stmt::VarDecl { name, typ, value, mutable: _ } => {
                self.output.push_str(&format!("let mut usr_{} = ", name));
                self.transpile_expr(value);
                self.output.push_str(";\n");
            }
            Stmt::Expr(expr) => {
                self.transpile_expr(expr);
                self.output.push_str(";\n");
            }
            Stmt::Assign { name, value } => {
                self.output.push_str(&format!("usr_{} = ", name));
                self.transpile_expr(value);
                self.output.push_str(";\n");
            }
            Stmt::If { cond, then_block, else_block } => {
                self.output.push_str("if ");
                self.transpile_expr(cond);
                self.output.push_str(" {\n");
                for s in then_block {
                    self.transpile_stmt(s, indent + 1);
                }
                self.push_indent(indent);
                self.output.push_str("}");
                if let Some(else_stmts) = else_block {
                    self.output.push_str(" else {\n");
                    for s in else_stmts {
                        self.transpile_stmt(s, indent + 1);
                    }
                    self.push_indent(indent);
                    self.output.push_str("}");
                }
                self.output.push_str("\n");
            }
            Stmt::While { cond, body } => {
                self.output.push_str("while ");
                self.transpile_expr(cond);
                self.output.push_str(" {\n");
                for s in body {
                    self.transpile_stmt(s, indent + 1);
                }
                self.push_indent(indent);
                self.output.push_str("}\n");
            }
            Stmt::FnDecl { name, generics: _, params, return_type, body } => {
                // Rust requires types for params. If we don't have them inferred/specified, we might have issues.
                // Assuming AST has types populated (Parser does rudimentary parsing)
                
                self.output.push_str(&format!("fn usr_{}(", name));
                for (i, (p_name, p_type)) in params.iter().enumerate() {
                    if i > 0 { self.output.push_str(", "); }
                    self.output.push_str(&format!("usr_{}: {}", p_name, self.map_type(p_type)));
                }
                self.output.push_str(") ");
                
                if return_type != Type::Void {
                    self.output.push_str(&format!("-> {} ", self.map_type(&return_type)));
                }

                self.output.push_str("{\n");
                for s in body {
                    self.transpile_stmt(s, indent + 1);
                }
                self.push_indent(indent);
                self.output.push_str("}\n");
            }
            Stmt::Return(expr_opt) => {
                self.output.push_str("return");
                if let Some(expr) = expr_opt {
                    self.output.push_str(" ");
                    self.transpile_expr(expr);
                }
                self.output.push_str(";\n");
            }
            Stmt::Import(_) => {
                 self.output.push_str("// import resolved \n");
            }
            Stmt::StructDef { .. } | Stmt::InterfaceDef { .. } | Stmt::TypeAlias { .. } => {
                 self.output.push_str("// type defs not yet supported in transpiler \n");
            }
        }
    }

    fn transpile_expr(&mut self, expr: Expr) {
        match expr {
            Expr::LiteralInt(i) => self.output.push_str(&i.to_string()),
            Expr::LiteralFloat(f) => self.output.push_str(&format!("{:?}", f)), // Debug format to keep decimal?
            Expr::LiteralBool(b) => self.output.push_str(&b.to_string()),
            Expr::LiteralString(s) => self.output.push_str(&format!("\"{}\".to_string()", s)), // String heap allocation
            Expr::Identifier(s) => {
                if s == "print" {
                    // This creates a special case where 'print' as an identifier (not call) is not mogrified
                    // passed as argument?
                     self.output.push_str("print");
                } else {
                     self.output.push_str(&format!("usr_{}", s));
                }
            }
            Expr::Binary { left, op, right } => {
                self.output.push_str("(");
                self.transpile_expr(*left);
                self.output.push_str(match op {
                    BinaryOp::Add => " + ",
                    BinaryOp::Sub => " - ",
                    BinaryOp::Mul => " * ",
                    BinaryOp::Div => " / ",
                    BinaryOp::Eq => " == ",
                    BinaryOp::Neq => " != ",
                    BinaryOp::Lt => " < ",
                    BinaryOp::Gt => " > ",
                    BinaryOp::Lte => " <= ",
                    BinaryOp::Gte => " >= ",
                });
                self.transpile_expr(*right);
                self.output.push_str(")");
            }
            Expr::Call { function, args } => {
                if let Expr::Identifier(name) = *function.clone() {
                    if name == "print" {
                        self.output.push_str("println!(\"{:?}\", ");
                         if args.is_empty() {
                            self.output.push_str("");
                        } else {
                            // Only handling one arg for print for now based on this simple logic
                             self.transpile_expr(args[0].clone());
                        }
                        self.output.push_str(")");
                        return;
                    }
                }
                
                self.transpile_expr(*function);
                self.output.push_str("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { self.output.push_str(", "); }
                    self.transpile_expr(arg.clone());
                }
                self.output.push_str(")");
            }
            Expr::List(elements) => {
                 self.output.push_str("vec![");
                 for (i, e) in elements.iter().enumerate() {
                     if i > 0 { self.output.push_str(", "); }
                     self.transpile_expr(e.clone());
                 }
                 self.output.push_str("]");
            }
            Expr::Tuple(elements) => {
                 self.output.push_str("(");
                 for (i, e) in elements.iter().enumerate() {
                     if i > 0 { self.output.push_str(", "); }
                     self.transpile_expr(e.clone());
                 }
                 // If single element tuple, need trailing comma? Rust doesn't strictly require it for (x,) if types imply it, but (x) is parens.
                 // (x,) syntax in Rust is valid.
                 if elements.len() == 1 {
                     self.output.push_str(",");
                 }
                 self.output.push_str(")");
            }
            Expr::Set(elements) => {
                 self.output.push_str("std::collections::HashSet::from([");
                 for (i, e) in elements.iter().enumerate() {
                     if i > 0 { self.output.push_str(", "); }
                     self.transpile_expr(e.clone());
                 }
                 self.output.push_str("])");
            }
            Expr::Dict(elements) => {
                 self.output.push_str("std::collections::HashMap::from([");
                 for (i, (k, v)) in elements.iter().enumerate() {
                     if i > 0 { self.output.push_str(", "); }
                     self.output.push_str("(");
                     self.transpile_expr(k.clone());
                     self.output.push_str(", ");
                     self.transpile_expr(v.clone());
                     self.output.push_str(")");
                 }
                 self.output.push_str("])");
            }
        }
    }

    fn map_type(&self, t: &Type) -> String {
        match t {
            Type::Int => "i64".to_string(),
            Type::Float => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "String".to_string(),
            Type::Void => "()".to_string(),
            Type::List => "Vec<Box<dyn std::any::Any>>".to_string(), // Placeholder
            Type::Tuple => "Box<dyn std::any::Any>".to_string(), // Placeholder
            Type::Set => "std::collections::HashSet<Box<dyn std::any::Any>>".to_string(), // Placeholder
            Type::Dict => "std::collections::HashMap<String, Box<dyn std::any::Any>>".to_string(), // Placeholder
            Type::ListMutable => "std::sync::Arc<std::sync::Mutex<Vec<Box<dyn std::any::Any>>>>".to_string(),
            Type::TupleMutable => "std::sync::Arc<std::sync::Mutex<Box<dyn std::any::Any>>>".to_string(),
            Type::SetMutable => "std::sync::Arc<std::sync::Mutex<std::collections::HashSet<Box<dyn std::any::Any>>>>".to_string(),
            Type::DictMutable => "std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, Box<dyn std::any::Any>>>>".to_string(),
            Type::UserDefined(s, _generics) => format!("usr_{}", s), // TODO: generics
            Type::Union(_types) => "Box<dyn std::any::Any>".to_string(),
        }
    }
}
