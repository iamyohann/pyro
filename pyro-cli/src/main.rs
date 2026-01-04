use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::fs;
use pyro_core::lexer::Lexer;
use pyro_core::parser::Parser as PyroParser;
use pyro_core::interpreter::Interpreter;
use anyhow::{Context, Result};

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
    /// Initialize a new package (TODO)
    Init {
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { file } => {
            let content = fs::read_to_string(file)
                .with_context(|| format!("Could not read file {:?}", file))?;
            
            // 1. Lex
            let mut lexer = Lexer::new(&content);
            let tokens = lexer.tokenize();
            
            // 2. Parse
            let mut parser = PyroParser::new(&tokens);
            let program = parser.parse().map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
            
            // 3. Interpret
            let mut interpreter = Interpreter::new();
            interpreter.run(program.statements).map_err(|e| anyhow::anyhow!("Runtime error: {}", e))?;
        }
        Commands::Init { name } => {
            println!("Initializing new Pyro project: {}", name);
            // TODO: Create folder structure
        }
    }

    Ok(())
}
