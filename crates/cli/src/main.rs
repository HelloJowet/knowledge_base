mod commands;

use clap::Parser;
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

const KNOWLEDGE_BASE_PATH: &str = "KNOWLEDGE_BASE_PATH";

#[derive(Debug, Parser)]
#[command(name = "knowledge-base", version, about)]
struct Cli {
    #[command(subcommand)]
    command: commands::Command,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let root = if cli.command.requires_knowledge_base() {
        match knowledge_base_path() {
            Ok(root) => Some(root),
            Err(message) => {
                eprintln!("{message}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        None
    };

    match commands::execute(cli.command, root.as_deref()) {
        Ok(exit_code) => exit_code,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn knowledge_base_path() -> Result<PathBuf, &'static str> {
    match env::var_os(KNOWLEDGE_BASE_PATH) {
        Some(value) if !value.is_empty() => Ok(value.into()),
        _ => Err("KNOWLEDGE_BASE_PATH must be set to the knowledge-base root directory"),
    }
}
