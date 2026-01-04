use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::fs;
use pyro_core::lexer::Lexer;
use pyro_core::parser::Parser as PyroParser;
use pyro_core::interpreter::Interpreter;
use pyro_core::ast::Stmt;
use anyhow::{Context, Result};
use std::collections::HashSet;

mod cmd;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a Pyro script
    Run {
        /// The file to run
        file: PathBuf,
    },
    /// Package management commands
    Mod {
        #[command(subcommand)]
        command: ModCommands,
    },
    /// Add a dependency
    Get {
        url: String,
    },
}

#[derive(Subcommand)]
enum ModCommands {
    /// Initialize a new module
    Init {
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { file } => {
            let mut statements = Vec::new();
            let mut loaded = HashSet::new();
            
            process_file(file.clone(), &mut loaded, &mut statements)?;

            // 3. Interpret
            let mut interpreter = Interpreter::new();
            interpreter.run(statements).map_err(|e| anyhow::anyhow!("Runtime error: {}", e))?;
        }
        Commands::Mod { command } => {
            match command {
                ModCommands::Init { name } => {
                    cmd::init::r#impl(name.clone())?;
                }
            }
        }
        Commands::Get { url } => {
            cmd::get::r#impl(url.clone())?;
        }
    }

    Ok(())
}

fn process_file(path: PathBuf, loaded: &mut HashSet<PathBuf>, statements: &mut Vec<Stmt>) -> Result<()> {
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
