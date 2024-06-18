use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::cli::config::CliConfig;
use crate::cli::profile::CliProfile;
use crate::cli::run::RunCmds;
use crate::state::AppState;

mod config;
mod profile;
mod run;

#[derive(PartialEq, Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
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
    match cli.commands {
        Commands::Run(run) => {
            let app_state = AppState::initialize().await?;
            run::execute_run_cmd(run, &app_state).await?;
        }
        Commands::Profile(profile) => {
            let app_state = AppState::initialize().await?;
            profile::run_profile_command(profile, &app_state).await?
        }
        Commands::Config(cfg) => config::run_config_cmd(cfg).await?,
    }

    Ok(())
}
