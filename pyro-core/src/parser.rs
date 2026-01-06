use crate::ast::{BinaryOp, Expr, Stmt, Type, Program};
use crate::lexer::Token;
use std::iter::Peekable;
use std::slice::Iter;

pub struct Parser<'a> {
    tokens: Peekable<Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens: tokens.iter().peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();
        while let Some(token) = self.tokens.peek() {
            if **token == Token::EOF {
                break;
            }
            if **token == Token::Newline {
                self.tokens.next();
                continue;
            }
            statements.push(self.parse_statement()?);
        }
        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Stmt, String> {
        match self.tokens.peek() {
            Some(Token::Let) => self.parse_var_decl(false),
            Some(Token::Mut) => self.parse_var_decl(true),
            Some(Token::Def) => self.parse_fn_decl(),
            Some(Token::Return) => self.parse_return(),
            Some(Token::Break) => {
                self.tokens.next();
                if let Some(Token::Newline) = self.tokens.peek() {
                    self.tokens.next();
                }
                Ok(Stmt::Break)
            },
            Some(Token::Continue) => {
                self.tokens.next();
                if let Some(Token::Newline) = self.tokens.peek() {
                    self.tokens.next();
                }
                Ok(Stmt::Continue)
            },
            Some(Token::If) => self.parse_if(),

            Some(Token::While) => self.parse_while(),
            Some(Token::For) => self.parse_for(),
            Some(Token::Import) => self.parse_import(),
            Some(Token::Record) => self.parse_record_decl(),
            Some(Token::Class) => self.parse_class_decl(),
            Some(Token::Interface) => self.parse_interface_decl(),
            Some(Token::Type) => self.parse_type_alias(),
            Some(Token::Try) => self.parse_try(),
            Some(Token::Raise) => self.parse_raise(),
            Some(Token::Go) => self.parse_go(),
            _ => {
                let expr = self.parse_expression()?;
                
                if let Some(Token::Equal) = self.tokens.peek() {
                    self.tokens.next(); // consume '='
                    let value = self.parse_expression()?;
                    if let Some(Token::Newline) = self.tokens.peek() {
                        self.tokens.next();
                    }
                    match expr {
                        Expr::Identifier(name) => Ok(Stmt::Assign { name, value }),
                        Expr::Get { object, name } => Ok(Stmt::Set { object: *object, name, value }),
                        _ => Err("Invalid assignment target".to_string()),
                    }
                } else {
                    // Consume optional newline after expression statement
                    if let Some(Token::Newline) = self.tokens.peek() {
                       self.tokens.next();
                    }
                    Ok(Stmt::Expr(expr))
                }
            }
        }
    }

    // let x: int = 10
    fn parse_var_decl(&mut self, is_mut: bool) -> Result<Stmt, String> {
        self.tokens.next(); // consume let/mut
        
        let name = match self.tokens.next() {
            Some(Token::Identifier(s)) => s.clone(),
            _ => return Err("Expected identifier".to_string()),
        };

        let mut typ = None;
        if let Some(Token::Colon) = self.tokens.peek() {
            self.tokens.next();
            typ = Some(self.parse_type()?);
        }

        if let Some(Token::Equal) = self.tokens.peek() {
            self.tokens.next();
        } else {
            return Err("Expected '=' in variable declaration".to_string());
        }

        let value = self.parse_expression()?;

        if let Some(Token::Newline) = self.tokens.peek() {
            self.tokens.next();
        }

        Ok(Stmt::VarDecl {
            name,
            typ,
            value,
            mutable: is_mut,
        })
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        let first_type = self.parse_single_type()?;
        
        // Check for Union |
        if let Some(Token::Pipe) = self.tokens.peek() {
            let mut types = vec![first_type];
            while let Some(Token::Pipe) = self.tokens.peek() {
                self.tokens.next(); // consume |
                types.push(self.parse_single_type()?);
            }
            Ok(Type::Union(types))
        } else {
            Ok(first_type)
        }
    }

    fn parse_single_type(&mut self) -> Result<Type, String> {
        match self.tokens.next() {
            Some(Token::Identifier(s)) => {
                let name = s.clone();
                match name.as_str() {
                    "int" => Ok(Type::Int),
                    "float" => Ok(Type::Float),
                    "bool" => Ok(Type::Bool),
                    "string" => Ok(Type::String),
                    "void" => Ok(Type::Void),
                    "list" => Ok(Type::List),
                    "tuple" => Ok(Type::Tuple),
                    "set" => Ok(Type::Set),
                    "dict" => Ok(Type::Dict),
                    "list_mut" => Ok(Type::ListMutable),
                    "tuple_mut" => Ok(Type::TupleMutable),
                    "set_mut" => Ok(Type::SetMutable),
                    "dict_mut" => Ok(Type::DictMutable),
                    _ => {
                        // Check for generic arguments <T, U>
                        let mut generics = Vec::new();
                        if let Some(Token::Less) = self.tokens.peek() {
                            self.tokens.next(); // consume <
                            loop {
                                generics.push(self.parse_type()?);
                                match self.tokens.peek() {
                                    Some(Token::Comma) => { self.tokens.next(); }
                                    Some(Token::Greater) => {
                                        self.tokens.next();
                                        break;
                                    }
                                    _ => return Err("Expected ',' or '>' in generic type args".to_string()),
                                }
                            }
                        }
                        Ok(Type::UserDefined(name, generics))
                    },
                }
            }
            _ => Err("Expected type identifier".to_string()),
        }
    }
    
    // Parse generic parameters definition: <T, U>
    fn parse_generic_params(&mut self) -> Result<Vec<String>, String> {
        let mut params = Vec::new();
        if let Some(Token::Less) = self.tokens.peek() {
            self.tokens.next(); // consume <
            loop {
                match self.tokens.next() {
                    Some(Token::Identifier(s)) => params.push(s.clone()),
                    _ => return Err("Expected generic parameter name".to_string()),
                }
                
                match self.tokens.peek() {
                    Some(Token::Comma) => { self.tokens.next(); }
                    Some(Token::Greater) => {
                        self.tokens.next();
                        break;
                    }
                    _ => return Err("Expected ',' or '>' in generic parameters".to_string()),
                }
            }
        }
        Ok(params)
    }

    fn parse_expression(&mut self) -> Result<Expr, String> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;

        while let Some(&token) = self.tokens.peek() {
            let op = match token {
                Token::EqualEqual => BinaryOp::Eq,
                Token::BangEqual => BinaryOp::Neq,
                _ => break,
            };
            self.tokens.next();
            let right = self.parse_comparison()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_term()?;

        while let Some(&token) = self.tokens.peek() {
            let op = match token {
                Token::Less => BinaryOp::Lt,
                Token::LessEqual => BinaryOp::Lte,
                Token::Greater => BinaryOp::Gt,
                Token::GreaterEqual => BinaryOp::Gte,
                _ => break,
            };
            self.tokens.next();
            let right = self.parse_term()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_factor()?;

        while let Some(&token) = self.tokens.peek() {
            let op = match token {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.tokens.next();
            let right = self.parse_factor()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;

        while let Some(&token) = self.tokens.peek() {
            let op = match token {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                _ => break,
            };
            self.tokens.next();
            let right = self.parse_unary()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_atom()?;

        loop {
            if let Some(Token::LParen) = self.tokens.peek() {
                self.tokens.next(); // consume (
                let mut args = Vec::new();
                if let Some(Token::RParen) = self.tokens.peek() {
                    self.tokens.next();
                } else {
                    loop {
                        let arg = self.parse_expression()?;
                        args.push(arg);
                        // println!("Parsed arg: {:?}, Next token: {:?}", args.last(), self.tokens.peek());
                        match self.tokens.peek() {
                            Some(Token::Comma) => { 
                                self.tokens.next(); 
                                if let Some(Token::RParen) = self.tokens.peek() {
                                    self.tokens.next();
                                    break;
                                }
                            }
                            Some(Token::RParen) => {
                                self.tokens.next();
                                break;
                            }
                            Some(Token::RParen) => {
                                self.tokens.next();
                                break;
                            }
                            _ => {
                                println!("TOKEN FAIL: {:?}", self.tokens.peek());
                                return Err("Expected ',' or ')' in argument list".to_string());
                            }
                        }
                    }
                }
                expr = Expr::Call {
                    function: Box::new(expr),
                    generics: Vec::new(),
                    args,
                };
            } else if let Some(Token::Dot) = self.tokens.peek() {
                self.tokens.next(); // consume .
                let name = match self.tokens.next() {
                    Some(Token::Identifier(s)) => s.clone(),
                    _ => return Err("Expected property name after '.'".to_string()),
                };
                expr = Expr::Get {
                    object: Box::new(expr),
                    name,
                };
            } else if let Some(Token::LBracket) = self.tokens.peek() {
                self.tokens.next(); // consume [
                let index = self.parse_expression()?;
                if let Some(Token::RBracket) = self.tokens.next() {} else {
                    return Err("Expected ']' after index".to_string());
                }
                expr = Expr::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_atom(&mut self) -> Result<Expr, String> {
        match self.tokens.peek() {
            Some(Token::Integer(i)) => {
                let val = *i;
                self.tokens.next();
                Ok(Expr::LiteralInt(val))
            }
            Some(Token::Float(f)) => {
                let val = *f;
                self.tokens.next();
                Ok(Expr::LiteralFloat(val))
            }
            Some(Token::StringLiteral(s)) => {
                let val = s.clone();
                self.tokens.next();
                Ok(Expr::LiteralString(val))
            }
            Some(Token::Identifier(s)) => {
                let name = s.clone();
                self.tokens.next();
                Ok(Expr::Identifier(name))
            }
            Some(Token::Bool(b)) => {
                let val = *b;
                self.tokens.next();
                Ok(Expr::LiteralBool(val))
            }
            Some(Token::Chan) => {
                self.tokens.next(); // consume chan
                
                // Parse optional generics <T>
                let mut generics = Vec::new();
                if let Some(Token::Less) = self.tokens.peek() {
                    self.tokens.next(); // <
                    loop {
                        generics.push(self.parse_type()?);
                        match self.tokens.peek() {
                            Some(Token::Comma) => { self.tokens.next(); }
                            Some(Token::Greater) => {
                                self.tokens.next();
                                break;
                            }
                            _ => return Err("Expected ',' or '>' in generic type args".to_string()),
                        }
                    }
                }

                // Expect (args)
                if let Some(Token::LParen) = self.tokens.peek() {
                    self.tokens.next(); // (
                    let mut args = Vec::new();
                    if let Some(Token::RParen) = self.tokens.peek() {
                        self.tokens.next(); 
                    } else {
                        loop {
                            args.push(self.parse_expression()?);
                             match self.tokens.peek() {
                                Some(Token::Comma) => { self.tokens.next(); }
                                Some(Token::RParen) => {
                                    self.tokens.next();
                                    break;
                                }
                                _ => return Err("Expected ',' or ')' in argument list".to_string()),
                            }
                        }
                    }
                     Ok(Expr::Call {
                        function: Box::new(Expr::Identifier("chan".to_string())),
                        generics,
                        args,
                    })
                } else {
                     // treating `chan` as identifier if not followed by parens? 
                     // But we consumed `chan`. 
                     // If just `let c = chan`, that's an identifier usage (function pointer)?
                     // We return Identifier("chan") but that might confuse if we wanted generics.
                     // But `chan<int>` without parens is a type? No, `chan` is function.
                     // For now assume `chan` is always call or identifier.
                     Ok(Expr::Identifier("chan".to_string()))
                }
            }
            Some(Token::LParen) => {
                self.tokens.next(); // (
                
                // Check for empty tuple ()
                if let Some(Token::RParen) = self.tokens.peek() {
                    self.tokens.next();
                    return Ok(Expr::Tuple(Vec::new()));
                }

                let expr = self.parse_expression()?;
                
                if let Some(Token::Comma) = self.tokens.peek() {
                    // It's a tuple (expr, ...)
                    self.tokens.next(); // consume comma
                    let mut elements = vec![expr];
                    if let Some(Token::RParen) = self.tokens.peek() {
                        // (expr,)
                        self.tokens.next();
                        Ok(Expr::Tuple(elements))
                    } else {
                        // (expr, expr2, ...)
                        loop {
                            elements.push(self.parse_expression()?);
                            match self.tokens.peek() {
                                Some(Token::Comma) => { self.tokens.next(); }
                                Some(Token::RParen) => {
                                    self.tokens.next();
                                    break;
                                }
                                _ => return Err("Expected ',' or ')' in tuple".to_string()),
                            }
                        }
                        Ok(Expr::Tuple(elements))
                    }
                } else if let Some(Token::RParen) = self.tokens.peek() {
                    // It's a group (expr)
                    self.tokens.next();
                    Ok(expr)
                } else {
                    Err("Expected ')' or ','".to_string())
                }
            }
            Some(Token::Minus) => {
                self.tokens.next(); // -
                // Treat as Unary Minus or just parse literal if followed by number?
                // AST doesn't have UnaryOp yet?
                // Or just Expr::LiteralInt(-val).
                match self.tokens.peek() {
                    Some(Token::Integer(i)) => {
                        let val = *i;
                        self.tokens.next();
                        Ok(Expr::LiteralInt(-val))
                    }
                     Some(Token::Float(f)) => {
                        let val = *f;
                        self.tokens.next();
                        Ok(Expr::LiteralFloat(-val))
                    }
                    _ => Err("Unary minus only supported for literals currently".to_string()),
                }
            }
            Some(Token::LBrace) => {
                self.tokens.next(); // {
                
                if let Some(Token::RBrace) = self.tokens.peek() {
                    self.tokens.next();
                    // Empty brace is empty Dict
                    return Ok(Expr::Dict(Vec::new()));
                }

                let first = self.parse_expression()?;

                if let Some(Token::Colon) = self.tokens.peek() {
                    // It's a Dict
                    self.tokens.next(); // consume :
                    let val = self.parse_expression()?;
                    let mut entries = vec![(first, val)];

                    loop {
                        if let Some(Token::RBrace) = self.tokens.peek() {
                            self.tokens.next();
                            break;
                        }
                        if let Some(Token::Comma) = self.tokens.peek() {
                            self.tokens.next();
                        } else {
                            // Only check for RBrace again if no comma (for trailing comma support or end)
                            // Actually if no comma, must be RBrace
                             if let Some(Token::RBrace) = self.tokens.peek() {
                                self.tokens.next();
                                break;
                            }
                             return Err("Expected ',' or '}' in dict".to_string());
                        }

                        // Check if we hit RBrace after comma (trailing comma)
                        if let Some(Token::RBrace) = self.tokens.peek() {
                             self.tokens.next();
                             break;
                        }

                        let k = self.parse_expression()?;
                        if let Some(Token::Colon) = self.tokens.next() {} else {
                            return Err("Expected ':' in dict entry".to_string());
                        }
                        let v = self.parse_expression()?;
                        entries.push((k, v));
                    }
                    Ok(Expr::Dict(entries))
                } else {
                    // It's a Set
                    let mut elements = vec![first];
                     loop {
                        if let Some(Token::RBrace) = self.tokens.peek() {
                            self.tokens.next();
                            break;
                        }
                         if let Some(Token::Comma) = self.tokens.peek() {
                            self.tokens.next();
                        } else {
                             if let Some(Token::RBrace) = self.tokens.peek() {
                                self.tokens.next();
                                break;
                            }
                            return Err("Expected ',' or '}' in set".to_string());
                        }

                         if let Some(Token::RBrace) = self.tokens.peek() {
                             self.tokens.next();
                             break;
                        }

                        elements.push(self.parse_expression()?);
                    }
                    Ok(Expr::Set(elements))
                }
            }
            Some(Token::LBracket) => {
                self.tokens.next(); // [
                let mut elements = Vec::new();
                if let Some(Token::RBracket) = self.tokens.peek() {
                    self.tokens.next();
                } else {
                    loop {
                        elements.push(self.parse_expression()?);
                        match self.tokens.peek() {
                            Some(Token::Comma) => { self.tokens.next(); }
                            Some(Token::RBracket) => {
                                self.tokens.next();
                                break;
                            }
                            _ => return Err("Expected ',' or ']' in list".to_string()),
                        }
                    }
                }
                Ok(Expr::List(elements))
            }
            t => Err(format!("Unexpected token in expression: {:?}", t)),
        }
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.tokens.next(); // if
        let cond = self.parse_expression()?;
        if let Some(Token::Colon) = self.tokens.peek() {
            self.tokens.next();
        } else {
            return Err("Expected ':' after if condition".to_string());
        }
        
        let _ = self.tokens.next_if(|t| matches!(t, Token::Newline));

        let then_block = self.parse_block()?;
        let mut else_block = None;

        if let Some(Token::Else) = self.tokens.peek() {
            self.tokens.next();
            if let Some(Token::Colon) = self.tokens.peek() {
                self.tokens.next();
            } else {
                return Err("Expected ':' after else".to_string());
            }
             let _ = self.tokens.next_if(|t| matches!(t, Token::Newline));
            else_block = Some(self.parse_block()?);
        }

        Ok(Stmt::If { cond, then_block, else_block })
    }

    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.tokens.next(); // consume for
        
        let item_name = match self.tokens.next() {
            Some(Token::Identifier(s)) => s.clone(),
            _ => return Err("Expected identifier after 'for'".to_string()),
        };

        if let Some(Token::In) = self.tokens.next() {} else {
            return Err("Expected 'in' after loop variable".to_string());
        }

        let iterable = self.parse_expression()?;

        if let Some(Token::Colon) = self.tokens.peek() {
            self.tokens.next();
        } else {
            return Err("Expected ':' after for loop iterable".to_string());
        }

        let _ = self.tokens.next_if(|t| matches!(t, Token::Newline));

        let body = self.parse_block()?;

        Ok(Stmt::For { item_name, iterable, body })
    }

    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.tokens.next(); // while
        let cond = self.parse_expression()?;
        if let Some(Token::Colon) = self.tokens.peek() {
            self.tokens.next();
        } else {
            return Err("Expected ':' after while condition".to_string());
        }
        let _ = self.tokens.next_if(|t| matches!(t, Token::Newline));
        let body = self.parse_block()?;
        Ok(Stmt::While { cond, body })
    }

    fn parse_fn_decl(&mut self) -> Result<Stmt, String> {
        self.tokens.next(); // def
        let name = match self.tokens.next() {
            Some(Token::Identifier(s)) => s.clone(),
            _ => return Err("Expected function name".to_string()),
        };

        let generics = self.parse_generic_params()?;

        if let Some(Token::LParen) = self.tokens.next() {} else {
             return Err("Expected '('".to_string());
        }
        
        // Parse params
        let mut params = Vec::new();
        if let Some(Token::RParen) = self.tokens.peek() {
            self.tokens.next();
        } else {
            loop {
                let param_name = match self.tokens.next() {
                    Some(Token::Identifier(s)) => s.clone(),
                    _ => return Err("Expected parameter name".to_string()),
                };
                let param_type = if param_name == "self" {
                    if let Some(Token::Colon) = self.tokens.peek() {
                        self.tokens.next();
                        self.parse_type()?
                    } else {
                        // Implicit Self type
                        Type::UserDefined("Self".to_string(), Vec::new())
                    }
                } else {
                    if let Some(Token::Colon) = self.tokens.peek() {
                        self.tokens.next();
                        self.parse_type()?
                    } else {
                         Type::UserDefined("Any".to_string(), Vec::new())
                    }
                };
                params.push((param_name, param_type));

                match self.tokens.peek() {
                    Some(Token::Comma) => { self.tokens.next(); }
                    Some(Token::RParen) => { self.tokens.next(); break; }
                    _ => return Err("Expected ',' or ')'".to_string()),
                }
            }
        }

        let mut return_type = Type::Void; // default
        if let Some(Token::Arrow) = self.tokens.peek() {
            self.tokens.next();
            return_type = self.parse_type()?;
        }

        if let Some(Token::Colon) = self.tokens.next() {} else { return Err("Expected ':'".to_string()); }
         let _ = self.tokens.next_if(|t| matches!(t, Token::Newline));

        let body = self.parse_block()?;

        Ok(Stmt::FnDecl { name, generics, params, return_type, body })
    }

    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.tokens.next(); // consume return
        
        let expr = if let Some(Token::Newline) | Some(Token::EOF) | Some(Token::Dedent) = self.tokens.peek() {
            None
        } else {
            Some(self.parse_expression()?)
        };

        // Consume optional newline
        if let Some(Token::Newline) = self.tokens.peek() {
            self.tokens.next();
        }

        Ok(Stmt::Return(expr))
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        if let Some(Token::Indent) = self.tokens.next() {} else {
            return Err("Expected indentation".to_string());
        }

        let mut stmts = Vec::new();
        while let Some(token) = self.tokens.peek() {
            match token {
                Token::Dedent => {
                    self.tokens.next();
                    break;
                }
                Token::EOF => break,
                Token::Newline => { self.tokens.next(); continue; }
                _ => {
                    stmts.push(self.parse_statement()?);
                }
            }
        }
        Ok(stmts)
    }
    fn parse_import(&mut self) -> Result<Stmt, String> {
        self.tokens.next(); // consume import
        
        let mut path = String::new();
        
        if let Some(Token::StringLiteral(s)) = self.tokens.peek() {
             path = s.clone();
             self.tokens.next();
        } else {
            // Parse dotted identifier: std.math
            loop {
                if let Some(Token::Identifier(s)) = self.tokens.next() {
                    if !path.is_empty() {
                        path.push('.');
                    }
                    path.push_str(s);
                } else {
                    return Err("Expected identifier in import path".to_string());
                }

                if let Some(Token::Dot) = self.tokens.peek() {
                    self.tokens.next();
                } else {
                    break;
                }
            }
        }

        if let Some(Token::Newline) = self.tokens.peek() {
            self.tokens.next();
        }

        Ok(Stmt::Import(path))
    }


    fn parse_record_decl(&mut self) -> Result<Stmt, String> {
        self.tokens.next(); // consume record
        let name = match self.tokens.next() {
            Some(Token::Identifier(s)) => s.clone(),
            _ => return Err("Expected record name".to_string()),
        };

        let generics = self.parse_generic_params()?;

        if let Some(Token::LParen) = self.tokens.next() {} else {
            return Err("Expected '('".to_string());
        }

        let mut fields = Vec::new();
        if let Some(Token::RParen) = self.tokens.peek() {
            self.tokens.next();
        } else {
            loop {
                 let field_name = match self.tokens.next() {
                     Some(Token::Identifier(s)) => s.clone(),
                     _ => return Err("Expected field name".to_string()),
                 };

                 if let Some(Token::Colon) = self.tokens.next() {} else {
                     return Err("Expected ':'".to_string());
                 }

                 let field_type = self.parse_type()?;
                 fields.push((field_name, field_type));
                 
                 match self.tokens.peek() {
                     Some(Token::Comma) => { self.tokens.next(); }
                     Some(Token::RParen) => {
                         self.tokens.next();
                         break;
                     }
                     _ => return Err("Expected ',' or ')'".to_string()),
                 }
            }
        }

        let mut methods = Vec::new();
        if let Some(Token::Colon) = self.tokens.peek() {
            self.tokens.next(); // consume ':'
            if let Some(Token::Newline) = self.tokens.peek() {
                 self.tokens.next();
            } else {
                 return Err("Expected newline after ':'".to_string());
            }
             methods = self.parse_block()?;
        } else {
             // Optional newline if no body
             if let Some(Token::Newline) = self.tokens.peek() {
                 self.tokens.next();
             }
        }

        Ok(Stmt::RecordDef { name, generics, fields, methods })
    }

    fn parse_interface_decl(&mut self) -> Result<Stmt, String> {
        self.tokens.next(); // consume interface
        let name =match self.tokens.next() {
            Some(Token::Identifier(s)) => s.clone(),
            _ => return Err("Expected interface name".to_string()),
        };

        let generics = self.parse_generic_params()?;

        if let Some(Token::LBrace) = self.tokens.next() {} else {
            return Err("Expected '{'".to_string());
        }

        let mut methods = Vec::new();
        while let Some(token) = self.tokens.peek() {
             if **token == Token::RBrace {
                 self.tokens.next();
                 break;
             }
             if **token == Token::Newline || **token == Token::Indent || **token == Token::Dedent {
                 self.tokens.next();
                 continue;
             }

             if let Some(Token::Def) = self.tokens.next() {} else {
                 return Err("Expected 'def' for interface method".to_string());
             }

             let method_name = match self.tokens.next() {
                 Some(Token::Identifier(s)) => s.clone(),
                 _ => return Err("Expected method name".to_string()),
             };

             if let Some(Token::LParen) = self.tokens.next() {} else {
                 return Err("Expected '('".to_string());
             }

             let mut params = Vec::new();
             if let Some(Token::RParen) = self.tokens.peek() {
                 self.tokens.next();
             } else {
                 loop {
                     let pname = match self.tokens.next() {
                         Some(Token::Identifier(s)) => s.clone(),
                         _ => return Err("Expected param name".to_string()),
                     };
                     if let Some(Token::Colon) = self.tokens.next() {} else {
                         return Err("Expected ':'".to_string());
                     }
                     let ptype = self.parse_type()?;
                     params.push((pname, ptype));

                     match self.tokens.peek() {
                         Some(Token::Comma) => { self.tokens.next(); }
                         Some(Token::RParen) => { self.tokens.next(); break; }
                         _ => return Err("Expected ',' or ')'".to_string()),
                     }
                 }
             }

             let mut ret_type = Type::Void;
             if let Some(Token::Arrow) = self.tokens.peek() {
                 self.tokens.next();
                 ret_type = self.parse_type()?;
             }

             methods.push((method_name, params, ret_type));

             if let Some(Token::Newline) = self.tokens.peek() {
                 self.tokens.next();
             }
        }

        if let Some(Token::Newline) = self.tokens.peek() {
            self.tokens.next();
        }

        Ok(Stmt::InterfaceDef { name, generics, methods })
    }

    fn parse_type_alias(&mut self) -> Result<Stmt, String> {
        self.tokens.next(); // consume type
        let name = match self.tokens.next() {
            Some(Token::Identifier(s)) => s.clone(),
            _ => return Err("Expected alias name".to_string()),
        };

        let generics = self.parse_generic_params()?;

        if let Some(Token::Equal) = self.tokens.next() {} else {
            return Err("Expected '=' in type alias".to_string());
        }

        let alias = self.parse_type()?;

        if let Some(Token::Newline) = self.tokens.peek() {
            self.tokens.next();
        }

        Ok(Stmt::TypeAlias { name, generics, alias })
    }

    fn parse_class_decl(&mut self) -> Result<Stmt, String> {
        self.tokens.next(); // consume class
        let name = match self.tokens.next() {
            Some(Token::Identifier(s)) => s.clone(),
            _ => return Err("Expected class name".to_string()),
        };

        let mut parent = None;
        if let Some(Token::LParen) = self.tokens.peek() {
             self.tokens.next(); // consume (
             match self.tokens.next() {
                 Some(Token::Identifier(s)) => parent = Some(s.clone()),
                 _ => return Err("Expected parent class name".to_string()),
             }
             if let Some(Token::RParen) = self.tokens.next() {} else {
                 return Err("Expected ')' after parent class name".to_string());
             }
        }

        if let Some(Token::Colon) = self.tokens.next() {} else {
             println!("Debug: Failed to find colon. Next token: {:?}", self.tokens.peek());
             return Err("Expected ':' after class declaration".to_string());
        }

        let _ = self.tokens.next_if(|t| matches!(t, Token::Newline));

        if let Some(Token::Indent) = self.tokens.next() {} else {
             return Err("Expected indentation for class body".to_string());
        }

        let mut methods = Vec::new();
        loop {
             let token = if let Some(t) = self.tokens.peek() {
                 (*t).clone()
             } else {
                 break;
             };

             if token == Token::Dedent {
                 self.tokens.next();
                 break;
             }
             if token == Token::EOF { break; }
             if token == Token::Newline { 
                 self.tokens.next(); 
                 continue; 
             }

             if token == Token::Def {
                 methods.push(self.parse_fn_decl()?);
             } else {
                 return Err(format!("Unexpected token in class body: {:?}. Only methods supported currently.", token));
             }
        }
        
        Ok(Stmt::ClassDecl { name, parent, methods })
    }
    fn parse_try(&mut self) -> Result<Stmt, String> {
        self.tokens.next(); // consume try
        
        if let Some(Token::Colon) = self.tokens.peek() {
            self.tokens.next();
        } else {
            return Err("Expected ':' after try".to_string());
        }

        let _ = self.tokens.next_if(|t| matches!(t, Token::Newline));
        let body = self.parse_block()?;

        let mut catch_var = None;
        let mut catch_body = None;

        if let Some(Token::Except) = self.tokens.peek() {
            self.tokens.next(); // consume except
            
            // Check for optional variable: except e:
            if let Some(Token::Identifier(s)) = self.tokens.peek() {
                catch_var = Some(s.clone());
                self.tokens.next();
            }

            if let Some(Token::Colon) = self.tokens.peek() {
                self.tokens.next();
            } else {
                return Err("Expected ':' after except".to_string());
            }

            let _ = self.tokens.next_if(|t| matches!(t, Token::Newline));
            catch_body = Some(self.parse_block()?);
        }

        let mut finally_body = None;
        if let Some(Token::Finally) = self.tokens.peek() {
            self.tokens.next(); // consume finally
            
            if let Some(Token::Colon) = self.tokens.peek() {
                self.tokens.next();
            } else {
                return Err("Expected ':' after finally".to_string());
            }

             let _ = self.tokens.next_if(|t| matches!(t, Token::Newline));
            finally_body = Some(self.parse_block()?);
        }

        if catch_body.is_none() && finally_body.is_none() {
            return Err("Try block must be followed by except or finally".to_string());
        }

        Ok(Stmt::Try {
            body,
            catch_var,
            catch_body,
            finally_body,
        })
    }

    fn parse_raise(&mut self) -> Result<Stmt, String> {
        self.tokens.next(); // consume raise
        let error = self.parse_expression()?;
        let mut cause = None;

        if let Some(Token::From) = self.tokens.peek() {
            self.tokens.next();
            cause = Some(self.parse_expression()?);
        }
        
        if let Some(Token::Newline) = self.tokens.peek() {
            self.tokens.next();
        }
        
        Ok(Stmt::Raise { error, cause })
    }

    fn parse_go(&mut self) -> Result<Stmt, String> {
        self.tokens.next(); // consume go
        
        let expr = self.parse_expression()?;
        
        // Ensure the expression is a function call
        match expr {
            Expr::Call { .. } => {},
             _ => return Err("Expected function call after 'go'".to_string()),
        }

        if let Some(Token::Newline) = self.tokens.peek() {
            self.tokens.next();
        }

        Ok(Stmt::Go(Box::new(expr)))
    }
}
