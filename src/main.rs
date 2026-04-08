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
        cli::Commands::New { feature_name } => {
            commands::new::run(&feature_name, &config)?;
        }
        cli::Commands::List { path } => {
            commands::list::run(&config, path)?;
        }
        cli::Commands::Remove { feature_name, yes } => {
            commands::remove::run(&feature_name, yes, &config)?;
        }
        cli::Commands::Sync { feature_name, from } => {
            commands::sync::run(&feature_name, &config, from.as_deref())?;
        }
        cli::Commands::Add {
            feature_name,
            repos,
            from,
            branch,
        } => {
            commands::add::run(
                &feature_name,
                &repos,
                from.as_deref(),
                branch.as_deref(),
                &config,
            )?;
        }
        cli::Commands::Init { shell } => {
            init::run(&shell);
        }
    }

    Ok(())
}
