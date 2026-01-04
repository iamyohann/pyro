use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Let,
    Mut,
    Def,
    Return,
    If,
    Else,
    While,
    Struct,
    Enum,
    Match,
    Case,
    Import,

    // Identifiers and Literals
    Identifier(String),
    Integer(i64),
    Float(f64),
    StringLiteral(String),

    // Symbols
    Plus,
    Minus,
    Star,
    Slash,
    Equal,       // =
    EqualEqual,  // ==
    BangEqual,   // !=
    Less,        // <
    LessEqual,   // <=
    Greater,     // >
    GreaterEqual,// >=
    Colon,
    Arrow,       // ->
    Comma,
    LParen,
    RParen,
    LBracket,    // [
    RBracket,    // ]

    // Significant Whitespace
    Indent,
    Dedent,
    Newline,

    EOF,
}

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    indent_stack: Vec<usize>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.chars().peekable(),
            indent_stack: vec![0],
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        
        while let Some(&c) = self.input.peek() {
            match c {
                ' ' | '\t' => {
                    // Skip whitespace inside lines, indentation handled by Newline logic
                    self.input.next(); 
                }
                '\n' => {
                    self.input.next();
                    tokens.push(Token::Newline);
                    self.handle_indentation(&mut tokens);
                }
                '+' => { self.input.next(); tokens.push(Token::Plus); }
                '-' => {
                    self.input.next();
                    if let Some(&'>') = self.input.peek() {
                        self.input.next();
                        tokens.push(Token::Arrow);
                    } else {
                        tokens.push(Token::Minus);
                    }
                }
                '*' => { self.input.next(); tokens.push(Token::Star); }
                '/' => { self.input.next(); tokens.push(Token::Slash); }
                '=' => {
                    self.input.next();
                    if let Some(&'=') = self.input.peek() {
                        self.input.next();
                        tokens.push(Token::EqualEqual);
                    } else {
                        tokens.push(Token::Equal);
                    }
                }
                '!' => {
                    self.input.next();
                    if let Some(&'=') = self.input.peek() {
                        self.input.next();
                        tokens.push(Token::BangEqual);
                    } else {
                        // For now panic or error, purely ! not supported yet
                    }
                }
                '<' => {
                    self.input.next();
                    if let Some(&'=') = self.input.peek() {
                        self.input.next();
                        tokens.push(Token::LessEqual);
                    } else {
                        tokens.push(Token::Less);
                    }
                }
                '>' => {
                    self.input.next();
                    if let Some(&'=') = self.input.peek() {
                        self.input.next();
                        tokens.push(Token::GreaterEqual);
                    } else {
                        tokens.push(Token::Greater);
                    }
                }
                ':' => { self.input.next(); tokens.push(Token::Colon); }
                ',' => { self.input.next(); tokens.push(Token::Comma); }
                '(' => { self.input.next(); tokens.push(Token::LParen); }
                ')' => { self.input.next(); tokens.push(Token::RParen); }
                '[' => { self.input.next(); tokens.push(Token::LBracket); }
                ']' => { self.input.next(); tokens.push(Token::RBracket); }
                '"' => {
                    tokens.push(self.read_string());
                }
                c if c.is_alphabetic() || c == '_' => {
                    tokens.push(self.read_identifier());
                }
                c if c.is_digit(10) => {
                    tokens.push(self.read_number());
                }
                _ => {
                    // Unexpected char, skip for now
                    self.input.next();
                }
            }
        }
        
        // Handle remaining dedents at EOF
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            tokens.push(Token::Dedent);
        }
        
        tokens.push(Token::EOF);
        tokens
    }

    fn handle_indentation(&mut self, tokens: &mut Vec<Token>) {
        let mut spaces = 0;
        while let Some(&c) = self.input.peek() {
            if c == ' ' {
                spaces += 1;
                self.input.next();
            } else {
                break;
            }
        }
        
        // Check if line is empty/comment only (TODO: handle comments)
        if let Some(&'\n') = self.input.peek() {
            // Empty line, ignore indentation
            return;
        }

        let current_indent = *self.indent_stack.last().unwrap();
        if spaces > current_indent {
            self.indent_stack.push(spaces);
            tokens.push(Token::Indent);
        } else if spaces < current_indent {
            while spaces < *self.indent_stack.last().unwrap() {
                self.indent_stack.pop();
                tokens.push(Token::Dedent);
            }
            if spaces != *self.indent_stack.last().unwrap() {
                // Indentation error
                eprintln!("Indentation Error");
            }
        }
    }

    fn read_identifier(&mut self) -> Token {
        let mut ident = String::new();
        while let Some(&c) = self.input.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.input.next();
            } else {
                break;
            }
        }

        match ident.as_str() {
            "let" => Token::Let,
            "mut" => Token::Mut,
            "def" => Token::Def,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "return" => Token::Return,
            "struct" => Token::Struct,
            "enum" => Token::Enum,
            "match" => Token::Match,
            "case" => Token::Case,
            "import" => Token::Import,
            _ => Token::Identifier(ident),
        }
    }

    fn read_number(&mut self) -> Token {
        let mut number_str = String::new();
        let mut is_float = false;
        
        while let Some(&c) = self.input.peek() {
            if c.is_digit(10) {
                number_str.push(c);
                self.input.next();
            } else if c == '.' && !is_float {
                is_float = true;
                number_str.push(c);
                self.input.next();
            } else {
                break;
            }
        }

        if is_float {
            Token::Float(number_str.parse().unwrap())
        } else {
            Token::Integer(number_str.parse().unwrap())
        }
    }

    fn read_string(&mut self) -> Token {
        self.input.next(); // skip opening "
        let mut s = String::new();
        while let Some(&c) = self.input.peek() {
            if c == '"' {
                self.input.next();
                return Token::StringLiteral(s);
            }
            s.push(c);
            self.input.next();
        }
        Token::StringLiteral(s) // EOF or unterminated
    }
}
