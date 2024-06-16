use anyhow::Result;
use clap::Parser;
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, TermLogger, TerminalMode};

use chidori::cli;

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
    cli::run_cli_command(cli).await?;

    Ok(())
}
