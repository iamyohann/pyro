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
            Some(Token::If) => self.parse_if(),
            Some(Token::While) => self.parse_while(),
            _ => {
                let expr = self.parse_expression()?;
                
                if let Some(Token::Equal) = self.tokens.peek() {
                    // Assignment: left side must be identifier
                    if let Expr::Identifier(name) = expr {
                        self.tokens.next(); // consume '='
                        let value = self.parse_expression()?;
                        if let Some(Token::Newline) = self.tokens.peek() {
                            self.tokens.next();
                        }
                        return Ok(Stmt::Assign { name, value });
                    } else {
                        return Err("Invalid assignment target".to_string());
                    }
                }

                // Consume optional newline after expression statement
                if let Some(Token::Newline) = self.tokens.peek() {
                    self.tokens.next();
                }
                Ok(Stmt::Expr(expr))
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
        match self.tokens.next() {
            Some(Token::Identifier(s)) => match s.as_str() {
                "int" => Ok(Type::Int),
                "float" => Ok(Type::Float),
                "bool" => Ok(Type::Bool),
                "string" => Ok(Type::String),
                "void" => Ok(Type::Void),
                _ => Ok(Type::UserDefined(s.clone())),
            },
            _ => Err("Expected type identifier".to_string()),
        }
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
        let mut left = self.parse_primary()?;

        while let Some(&token) = self.tokens.peek() {
            let op = match token {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                _ => break,
            };
            self.tokens.next();
            let right = self.parse_primary()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
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
                if let Some(Token::LParen) = self.tokens.peek() {
                    // Function call
                    self.tokens.next();
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
                        function: Box::new(Expr::Identifier(name)),
                        args,
                    })
                } else {
                    Ok(Expr::Identifier(name))
                }
            }
            Some(Token::LParen) => {
                self.tokens.next();
                let expr = self.parse_expression()?;
                if let Some(Token::RParen) = self.tokens.next() {
                    Ok(expr)
                } else {
                    Err("Expected ')'".to_string())
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
                if let Some(Token::Colon) = self.tokens.next() {} else { return Err("Expected ':'".to_string()); }
                let param_type = self.parse_type()?;
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

        Ok(Stmt::FnDecl { name, params, return_type, body })
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
}
