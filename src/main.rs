use anyhow::Result;
use clap::Parser;

use hitomi::{cli, logger};

#[tokio::main]
async fn main() -> Result<()> {
    logger::initialize_logger()?;

    let cli = cli::Cli::parse();
    cli::run_cli_command(cli).await?;

    Ok(())
}
