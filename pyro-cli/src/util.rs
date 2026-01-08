use anyhow::{Context, Result};
use pyro_core::ast::Stmt;
use pyro_core::lexer::Lexer;
use pyro_core::parser::Parser as PyroParser;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

pub fn process_file(path: PathBuf, loaded: &mut HashSet<PathBuf>, statements: &mut Vec<Stmt>) -> Result<()> {
    // Canonicalize path to handle relative paths correctly and deduplicate
    let canonical_path = if path.exists() {
        fs::canonicalize(&path)?
    } else {
        path.clone()
    };

    if loaded.contains(&canonical_path) {
        return Ok(());
    }
    loaded.insert(canonical_path);

    let content = fs::read_to_string(&path)
        .with_context(|| format!("Could not read file {:?}", path))?;
    
    // 1. Lex
    let mut lexer = Lexer::new(&content);
    let tokens = lexer.tokenize();
    
    // 2. Parse
    let mut parser = PyroParser::new(&tokens);
    let program = parser.parse().map_err(|e| anyhow::anyhow!("Parse error in {:?}: {}", path, e))?;

    for stmt in program.statements {
        if let Stmt::Import(import_path) = &stmt {
            if import_path.starts_with("std.") {
                statements.push(stmt.clone());
                continue;
            }
            let mut dep_path = PathBuf::from(import_path);
            
            // Resume resolution logic:
            // 1. Check relative to current file
            let relative = path.parent().unwrap().join(import_path);
            if relative.exists() {
                dep_path = relative;
            } else {
                // 2. Check ~/.pyro/pkg
                if let Ok(home) = std::env::var("HOME") {
                    let pkg_path = PathBuf::from(home).join(".pyro/pkg").join(import_path);
                    if pkg_path.exists() {
                        dep_path = pkg_path;
                    }
                }
                
                // 3. Check .externs relative to possible pyro.mod locations
                // This is a bit tricky as we don't know where pyro.mod is easily without searching up.
                // But for now, let's assume it's in the same dir as the file, or parent.
                // A better approach is to search up for .externs
                if !dep_path.exists() {
                    let mut current = path.parent().unwrap().to_path_buf();
                    loop {
                        let externs_path = current.join(".externs").join(import_path);
                         // Check for .pyro extension if not present? The import_path usually implies .pyro or is bare.
                         // The parser usually passes "foo.pyro" if it was `import "foo.pyro"`, or "foo" if `import foo`.
                         // If "foo", we need to append .pyro
                        let target = if externs_path.to_string_lossy().ends_with(".pyro") {
                             externs_path
                        } else {
                             let mut p = externs_path.clone().into_os_string();
                             p.push(".pyro");
                             PathBuf::from(p)
                        };

                        if target.exists() {
                            dep_path = target;
                            break;
                        }
                        if !current.pop() { break; }
                    }
                }
            }
            
            process_file(dep_path, loaded, statements)?;
        } else {
            statements.push(stmt);
        }
    }
    Ok(())
}
