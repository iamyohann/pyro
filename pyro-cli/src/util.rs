use anyhow::{Context, Result};
use pyro_core::ast::Stmt;
use pyro_core::lexer::Lexer;
use pyro_core::parser::Parser as PyroParser;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

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
            }
            
            process_file(dep_path, loaded, statements)?;
        } else {
            statements.push(stmt);
        }
    }
    Ok(())
}
