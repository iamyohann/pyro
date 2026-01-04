use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use pyro_core::interpreter::{Interpreter, Value};
use pyro_core::parser::Parser;
use pyro_core::lexer::Lexer;
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

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                let input = line.trim();
                if input.is_empty() {
                    continue;
                }
                if input == "exit" {
                    break;
                }

                // Parse the line
                let mut lexer = Lexer::new(input);
                let tokens = lexer.tokenize();
                let mut parser = Parser::new(&tokens);
                
                match parser.parse() {
                    Ok(program) => {
                        for stmt in program.statements {
                            match stmt {
                                Stmt::Import(path) => {
                                    // Handle imports - reuse util logic partially but we don't need to pass statements vector
                                    // We need to execute the file side-effects into the interpreter directly.
                                    // But Interpreter::run takes a Vec<Stmt>. 
                                    // We can reuse util::process_file to get all statements including transitive imports
                                    
                                    let mut statements = Vec::new();
                                    // Hack: create a dummy path for resolution relative to CWD
                                    let _dummy_path = std::env::current_dir()?.join("shell"); 
                                    // But we want to resolve 'path'
                                    // Let's resolve 'path' manually first
                                    let import_path = PathBuf::from(&path);
                                    let resolved_path = if import_path.exists() {
                                            if import_path.is_absolute() {
                                                import_path
                                            } else {
                                                std::env::current_dir()?.join(import_path)
                                            }
                                    } else {
                                        // Try pkg path
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
                                            println!("Runtime Error: {}", e);
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
                                        Err(e) => println!("Runtime Error: {}", e),
                                    }
                                }
                                _ => {
                                    if let Err(e) = interpreter.run(vec![stmt]) {
                                        println!("Runtime Error: {}", e);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => println!("Parse Error: {}", e),
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
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
