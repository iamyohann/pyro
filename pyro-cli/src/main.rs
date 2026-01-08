use clap::{Parser, Subcommand};
// use pyro_core::ast::Stmt;
use anyhow::{Context, Result};
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
    /// Generate extern definitions for Rust dependencies
    Externs,
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
    let worker_threads = std::env::var("PYRO_WORKER_THREADS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| num_cpus::get());

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .enable_all()
        .build()
        .context("Failed to build tokio runtime")?;

    runtime.block_on(async_main())
}

async fn async_main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { file } => {
            cmd::run::r#impl(file.clone())?;
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
        Commands::Externs => {
            cmd::externs::run()?;
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
