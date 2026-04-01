mod cli;
mod commands;
mod config;
mod error;
mod init;
mod worktree;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    let config = config::Config::load()?;

    match cli.command {
        cli::Commands::New {
            feature_name,
            repos,
        } => {
            commands::new::run(&feature_name, &repos, &config)?;
        }
        cli::Commands::List => {
            commands::list::run(&config)?;
        }
        cli::Commands::Remove { feature_name, yes } => {
            commands::remove::run(&feature_name, yes, &config)?;
        }
        cli::Commands::Init { shell } => {
            init::run(&shell);
        }
    }

    Ok(())
}
