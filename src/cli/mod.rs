use crate::cli::config::CliConfig;
use crate::cli::profile::CliProfile;
use crate::cli::run::RunCmds;
use crate::db;
use crate::profiles::manager::ProfileManager;
use anyhow::Result;
use clap::{Parser, Subcommand};
use log::Level;

mod config;
mod profile;
mod run;

#[derive(PartialEq, Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Use the database file at this location, if `DATABASE_URL` is not set
    #[arg(long = "db")]
    pub database_url: Option<String>,
    /// Set logging level, e.g. Debug, Info, Error.
    #[arg(long)]
    pub log_level: Option<Level>,
    /// hitomi commands
    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(PartialEq, Subcommand)]
pub enum Commands {
    Run(RunCmds),
    Profile(CliProfile),
    Config(CliConfig),
}

pub async fn run_cli_command(cli: Cli) -> Result<()> {
    db::initialize_pool(cli.database_url.as_deref()).await?;
    match cli.commands {
        Commands::Run(run) => {
            run::execute_run_cmd(run).await?;
        }
        Commands::Profile(profile) => {
            let manager = ProfileManager::new().await?;
            profile::run_profile_command(profile, manager).await?
        }
        Commands::Config(cfg) => config::run_config_cmd(cfg).await?,
    }

    Ok(())
}
