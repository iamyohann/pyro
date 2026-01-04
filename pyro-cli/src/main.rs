use clap::{Parser, Subcommand};
use pyro_core::interpreter::Interpreter;
// use pyro_core::ast::Stmt;
use anyhow::{Context, Result};
use std::collections::HashSet;
// use std::fs;
use std::path::PathBuf;
// use pyro_core::lexer::Lexer;
// use pyro_core::parser::Parser as PyroParser;

mod cmd;
mod util;
mod manifest;

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
    /// Compile to binary
    Build {
        file: PathBuf,
        /// Optional output path
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Target output format (binary or rust)
        #[arg(short, long, default_value = "binary")]
        target: BuildTarget,
    },
    /// Install dependencies
    Install,
    /// Run the interactive shell
    Shell,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum BuildTarget {
    Binary,
    Rust,
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
            
            util::process_file(file.clone(), &mut loaded, &mut statements)?;

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
        Commands::Install => {
            cmd::installer::r#impl()?;
        }
        Commands::Build { file, output, target } => {
            cmd::build::r#impl(file.clone(), output.clone(), target.clone())?;
        }
        Commands::Shell => {
            cmd::shell::run()?;
        }
    }

    Ok(())
}
