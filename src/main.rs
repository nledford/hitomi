use anyhow::Result;
use clap::Parser;
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, TermLogger, TerminalMode};

use chidori::cli;
use chidori::state::{AppState, APP_STATE};

#[tokio::main]
async fn main() -> Result<()> {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Trace,
        simplelog::Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])?;

    // config::delete_config_file().await;

    let cli = cli::Cli::parse();

    {
        let mut lock = APP_STATE.lock().await;
        *lock = AppState::initialize().await?;
    }

    cli::run_cli_command(cli).await?;

    Ok(())
}
