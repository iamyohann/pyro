use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use pyro_core::interpreter::{Interpreter, Value};
use pyro_core::parser::Parser;
use pyro_core::lexer::{Lexer, Token};
use pyro_core::ast::Stmt;
use std::collections::HashSet;
use std::path::PathBuf;
use crate::util;

pub fn run() -> Result<()> {
    // 1. Initialize Interpreter
    let mut interpreter = Interpreter::new();
    let mut loaded_files = HashSet::new();

    // 2. Initialize Rustyline Editor
    let mut rl = DefaultEditor::new()?;
    if let Ok(home) = std::env::var("HOME") {
         let _ = rl.load_history(&format!("{}/.pyro_history", home));
    }

    println!("Pyro Shell v0.1.0");
    println!("Type 'exit' or Ctrl-D to exit");

    let mut buffer = String::new();

    loop {
        let prompt = if buffer.is_empty() { ">> " } else { ".. " };
        let readline = rl.readline(prompt);
        match readline {
            Ok(line) => {
                let input_part = line.as_str();
                
                if buffer.is_empty() {
                    if input_part.trim() == "exit" {
                        break;
                    }
                    if input_part.trim().is_empty() {
                         continue;
                    }
                }

                buffer.push_str(input_part);
                buffer.push('\n');

                if is_input_complete(&buffer) {
                    let input = buffer.trim();
                    if !input.is_empty() {
                        let _ = rl.add_history_entry(input);
                        
                        // Parse the line
                        let mut lexer = Lexer::new(input);
                        let tokens = lexer.tokenize();
                        
                        // Check for lexer errors (like unclosed strings) if we want?
                        // But parser will handle it.

                        let mut parser = Parser::new(&tokens);
                        
                        match parser.parse() {
                            Ok(program) => {
                                for stmt in program.statements {
                                    match stmt {
                                        Stmt::Import(path) => {
                                            if interpreter.has_native_module(&path) {
                                                if let Err(e) = interpreter.run(vec![Stmt::Import(path.clone())]) {
                                                    println!("Runtime Error: {:?}", e);
                                                }
                                                continue;
                                            }

                                            // Resolve and process file
                                            let mut statements = Vec::new();
                                            // Quick hack for resolution relative to CWD
                                            let import_path = PathBuf::from(&path);
                                            let resolved_path = if import_path.exists() {
                                                    if import_path.is_absolute() {
                                                        import_path
                                                    } else {
                                                        if let Ok(cwd) = std::env::current_dir() {
                                                            cwd.join(import_path)
                                                        } else {
                                                            import_path
                                                        }
                                                    }
                                            } else {
                                                 if let Ok(home) = std::env::var("HOME") {
                                                    let pkg_path = PathBuf::from(home).join(".pyro/pkg").join(&path);
                                                    if pkg_path.exists() {
                                                        pkg_path
                                                    } else {
                                                         println!("Error: Could not resolve import '{}'", path);
                                                         continue;
                                                    }
                                                } else {
                                                     println!("Error: Could not resolve import '{}'", path);
                                                     continue;
                                                }
                                            };
                                            
                                            if let Err(e) = util::process_file(resolved_path, &mut loaded_files, &mut statements) {
                                                 println!("Error importing file: {}", e);
                                            } else {
                                                if let Err(e) = interpreter.run(statements) {
                                                    println!("Runtime Error: {:?}", e);
                                                }
                                            }
                                        }
                                        Stmt::Expr(expr) => {
                                            match interpreter.evaluate(expr) {
                                                Ok(val) => {
                                                    match val {
                                                        Value::Void => (),
                                                        _ => println!("{:?}", val),
                                                    }
                                                }
                                                Err(e) => println!("Runtime Error: {:?}", e),
                                            }
                                        }
                                        _ => {
                                            if let Err(e) = interpreter.run(vec![stmt]) {
                                                println!("Runtime Error: {:?}", e);
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => println!("Parse Error: {}", e),
                        }
                    }
                    buffer.clear();
                }
                // else continue loop to get more input
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                if !buffer.is_empty() {
                    buffer.clear();
                    println!("Input cancelled.");
                } else {
                    break;
                }
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    
    if let Ok(home) = std::env::var("HOME") {
         let _ = rl.save_history(&format!("{}/.pyro_history", home));
    }

    Ok(())
}

fn is_input_complete(input: &str) -> bool {
    // Quick checks
    if input.trim().is_empty() {
        return true;
    }
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    
    let mut parens = 0;
    let mut brackets = 0;
    let mut braces = 0;

    for token in &tokens {
        match token {
            Token::LParen => parens += 1,
            Token::RParen => parens -= 1,
            Token::LBracket => brackets += 1,
            Token::RBracket => brackets -= 1,
            Token::LBrace => braces += 1,
            Token::RBrace => braces -= 1,
            _ => {}
        }
    }

    // If unbalanced delimiters, definitely incomplete
    if parens > 0 || brackets > 0 || braces > 0 {
        return false;
    }
    
    // If indentation level > 0, we need an empty line to signal completion
    // The tokenizer emits Dedent tokens at EOF to balance the stack, 
    // BUT Lexer::tokenize() adds Dedents at the end automatically!
    // So indent_level will always be 0 after full tokenization if we rely on the implementation I saw earlier:
    // "while self.indent_stack.len() > 1 { ... tokens.push(Token::Dedent); }"
    // So looking at the *tokens* won't tell us if we are "currently" indented in the mental model of the user 
    // unless we look at the structure *before* the automatic EOF dedenting.
    // However, the lexer implementation I read (file snapshot) does exactly that:
    // `tokens.push(Token::EOF);` after popping indent stack.
    
    // So counting Indent/Dedent from the *output* of `tokenize()` will always result in 0 net change.
    
    // We need a different heuristic or modify how we check.
    // We can check if the input ends with a double newline if we suspect we are in a block.
    // Or we scan the tokens excluding the final automatic Dedents.
    
    // Let's filter out the EOF-generated Dedents?
    // The `Lexer` doesn't mark them as special.
    // But we know that for every block starter (Colon usually followed by Newline+Indent), there is an Indent.
    // If we simply check the text for ending with empty line?
    
    // Heuristic:
    // If we have "def foo():" -> parens balanced.
    // Lexer will output: Def, Identifier, LParen, RParen, Colon, EOF. (If no newline)
    // If "def foo():\n" -> ... Colon, Newline, Indent (if spaces), ...
    
    // Wait, the lexer handles indentation by looking at spaces after Newline.
    // If I type `def foo():\n  return 1`, the lexer sees:
    // Def ... Colon, Newline, Indent, Return, Integer.
    // At end of string, it adds Dedent, EOF.
    
    // If I type `def foo():\n`, trailing string is `\n`.
    // Lexer: ... Colon, Newline.
    // No Indent yet because no next char to peek spaces?
    // Actually `handle_indentation` peeks. If EOF follows \n, it returns.
    // So `def foo():\n` produces NO Indent token.
    
    // If I want to support blocks, I need to know if the last statement started a block.
    // `Colon` at end of line usually starts a block.
    
    let last_significant_token = tokens.iter().rev()
        .find(|t| !matches!(t, Token::Newline | Token::EOF | Token::Indent | Token::Dedent));
        
    if let Some(Token::Colon) = last_significant_token {
        return false; // Expecting more input after colon
    }
    
    // If we are deep in brackets, handled above.
    
    // What if we are inside a block?
    // `def foo():\n  print(1)`
    // We hit enter. Input is `def foo():\n  print(1)\n`.
    // Lexer: ... Indent, Print, LParen ... RParen, Newline.
    // Then auto-dedent.
    
    // If we are in a block (how do we know? Indented line exists?), we validly expect more lines OR end of block.
    // Standard REPL behavior:
    // If previous line caused indentation, continue.
    // If currently indented, continue until empty line.
    
    // How to detect "currently indented" logic without exposed Lexer state?
    // We can count the Indents manually from the tokens, IGNORING the ones that appear *only* because of EOF?
    // No, all Dedents appear at EOF if the file ends.
    
    // Let's try checking specific tokens at end.
    // Also, raw string check for double newline `\n\n` or `\n\s*\n` is a good signal to stop.
    // If parens are closed, and we hit double newline, we are probably done.
    // If parens are closed, and we have NO double newline, but we have `def ...`, do we wait?
    // Yes.
    
    // Refined logic:
    // 1. Check brackets/braces/parens balance. If unbalanced -> false.
    // 2. If line ends with `\`, continuation -> false.
    // 3. If last significant token is an operator that requires RHS (e.g. `+`, `-`, equal, etc) -> false.
    // 4. (The hard part) Blocks.
    //    If the code contains tokens that start blocks (`def`, `if`, `while`, `for`...), 
    //    WE REQUIRE an empty line to finish, UNLESS it's a simple one-liner (which Python supports `if x: y`).
    //    But one-liner `if x: y` ends with newline. Input complete.
    //    Multi-line `if x:\n  y` needs to know when `y` is done.
    
    // So: if we suspect block structure (indentation used), we require double newline to terminate.
    // How to detect if indentation is "active"?
    // We can infer it: if `Indent` tokens exist in the stream, we are in "multi-line block mode".
    // In that mode, we terminate only on double newline or if the input is closed (balanced) and somehow we know it's done?
    // Safest REPL approach for blocks: require empty line.
    
    let has_indent_token = tokens.iter().any(|t| matches!(t, Token::Indent));
    
    if has_indent_token {
        // We are likely in a block. Require double newline at end of input.
        // Input buffer usually has `\n` at end because we push it in the loop.
        // So checking for `\n\n` at tail.
        if input.ends_with("\n\n") || input.ends_with("\r\n\r\n") {
            return true;
        }
        
        return input.ends_with("\n\n");
    }
    
    // If no indentation tokens, we might still be STARTING a block `def foo():`.
    // In that case `last_significant_token` is Colon. We returned `false`. Correct.
    
    match last_significant_token {
        Some(Token::Plus) | Some(Token::Minus) | Some(Token::Star) | Some(Token::Slash) | 
        Some(Token::Equal) | Some(Token::EqualEqual) | Some(Token::BangEqual) |
        Some(Token::Less) | Some(Token::LessEqual) | Some(Token::Greater) | Some(Token::GreaterEqual) |
        Some(Token::Pipe) | Some(Token::Comma) | Some(Token::Dot) | Some(Token::Arrow) => {
             return false;
        }
        _ => {}
    }
    
    true
}

