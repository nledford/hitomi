use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::cli::config::CliConfig;
use crate::cli::profile::CliProfile;
use crate::cli::run::RunCmds;

mod config;
mod profile;
mod run;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Run(RunCmds),
    Profile(CliProfile),
    Config(CliConfig),
}

pub async fn run_cli_command(cli: Cli) -> Result<()> {
    match cli.commands {
        Commands::Run(run) => {
            run::execute_run_cmd(run).await?;
        }
        Commands::Profile(profile) => profile::run_profile_command(profile).await?,
        Commands::Config(cfg) => config::run_config_cmd(cfg).await?,
    }

    Ok(())
}
