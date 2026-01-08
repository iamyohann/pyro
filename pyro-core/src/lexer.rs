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
    For,
    Break,
    Continue,
    In,
    Record,
    Enum,
    Match,
    Case,
    Import,
    Interface,
    Class,
    Type,
    Try,
    Except,
    Finally,
    Raise,
    From,
    Go,
    Chan,
    Extern,

    // Identifiers and Literals
    Identifier(String),
    Integer(i64),
    Float(f64),
    StringLiteral(String),
    Bool(bool),

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
    Dot,         // .
    Arrow,       // ->
    Pipe,        // |
    Comma,
    LParen,
    RParen,
    LBracket,    // [
    RBracket,    // ]
    LBrace,      // {
    RBrace,      // }

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
                '#' => {
                    // Skip to end of line
                    while let Some(&c) = self.input.peek() {
                         if c == '\n' {
                             break;
                         }
                         self.input.next();
                    }
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
                '/' => {
                    self.input.next();
                    if let Some(&'/') = self.input.peek() {
                        // Skip to end of line
                        while let Some(&c) = self.input.peek() {
                             if c == '\n' {
                                 break;
                             }
                             self.input.next();
                        }
                    } else {
                        tokens.push(Token::Slash);
                    }
                }
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
                    } else if let Some(&'-') = self.input.peek() {
                        // Check for ArrowLeft <-
                        // self.input.next();
                        // tokens.push(Token::ArrowLeft);
                        tokens.push(Token::Less); // Treat as just Less if <- is removed?
                        // Or just remove the branch if we don't support it anymore.
                        // Actually if we remove support, < followed by - is Less, Minus
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
                '.' => {
                    // Check if it's a digit next (float starting with .)
                     // Actually parser usually handles numbers starting with . differently or not at all.
                     // But here we have `input.peek()`
                     // If we want to support `.5`, we need to check next char.
                     // The `read_number` logic assumes it starts with digit.
                     // Python allows `.5`.
                     // Let's see if next is digit.
                     // Let's see if next is digit.
                     // We can't peek 2 ahead easily with Peekable<Chars>.
                     // Just emit Dot for now. A number starting with dot can be tricky without lookahead.
                     // In `read_number` we handle `.` if it follows digits.
                     // So `1.2` works. `.5` might be tokenized as Dot Integer(5)?
                     // For simplicity, let's treat `.` as Dot token unless we implement specific float parsing here.
                     // Users can write `0.5`.
                     self.input.next(); 
                     tokens.push(Token::Dot); 
                }
                '|' => { self.input.next(); tokens.push(Token::Pipe); }
                ',' => { self.input.next(); tokens.push(Token::Comma); }
                '(' => { self.input.next(); tokens.push(Token::LParen); }
                ')' => { self.input.next(); tokens.push(Token::RParen); }
                '[' => { self.input.next(); tokens.push(Token::LBracket); }
                ']' => { self.input.next(); tokens.push(Token::RBracket); }
                '{' => { self.input.next(); tokens.push(Token::LBrace); }
                '}' => { self.input.next(); tokens.push(Token::RBrace); }
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
            "for" => Token::For,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "in" => Token::In,
            "record" => Token::Record,
            "return" => Token::Return,


            "enum" => Token::Enum,
            "match" => Token::Match,
            "case" => Token::Case,
            "import" => Token::Import,
            "interface" => Token::Interface,
            "type" => Token::Type,
            "class" => Token::Class,
            "try" => Token::Try,
            "except" => Token::Except,
            "finally" => Token::Finally,
            "raise" => Token::Raise,
            "from" => Token::From,
            "go" => Token::Go,
            "chan" => Token::Chan,
            "extern" => Token::Extern,
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
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
                // We need to be careful here. If we have `1.method()`, is that float `1.` or integer `1` then `.`?
                // Usually `1.` is float. `1..` is range (not supported yet). `1.method()` is float method?
                // Most langs require `(1).method()` or `1.0.method()`.
                // Let's assume greedy matching for float. `1.2` is float.
                // If next char is not digit, then `.` should probably terminate number?
                // But `peek` just sees one char.
                // We can't see the char *after* dot here easily without looking ahead 2.
                // But wait, `read_number` is called when we see a digit.
                // We consume digits. Then we see `.`.
                // If we consume `.`, we commit to float.
                // The issue: `obj.0` isn't valid syntax usually. `arr.0` (tuple index) might be.
                // `1.foo()` -> `1.` is float? No, `1.` is valid float. `foo` is identifier? 
                // `1.foo` -> float `1.` then `foo`? 
                // Rust requires `1.method` to be `(1).method` or `1.0.method`.
                // Let's implement peek check: if '.' is followed by digit, consume it.
                // Otherwise stop.
                
                // This is hard with just `peek()`.
                // We can consume `.`, then check peek. If not digit, we sort of messed up if we wanted it to be a specific token?
                // Actually if we consume `.`, and next is not digit, then we produce `Token::Float` like `1.`
                // Then next token is identifier `foo`. So `1.foo` -> `Float(1.0)`, `Identifier(foo)`.
                // That parses as two tokens next to each other.
                // That's syntax error usually.
                // BUT `list.length`. `list` is Identifier. `.` is Dot.
                // So this `read_number` is only for when we started with digit.
                
                // Improved logic:
                // If `c` is `.`:
                //   If next char (peek) is digit, valid float (e.g. `1.2`).
                //   If next char is not digit, is it valid float `1.`? Yes.
                //   So `1. method` -> `Float(1.0)`, `Warning/Error` in parser?
                //   Or `1.method` -> `Integer(1)`, `Dot`, `Identifier`.
                // Rust tokenizes `1.foo` as `1.0` then `foo`.
                // We will stick to simple greedy float: if we see `.`, we take it.
                is_float = true;
                number_str.push(c);
                self.input.next();
            } else {
                break;
            }
        }

        if is_float {
            // Check if it ends with `.`. If so, it might be ambiguous but for now it's float 1.0
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
