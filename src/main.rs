use anyhow::Result;
use clap::Parser;

use hitomi::{cli, db, logger};

#[tokio::main]
async fn main() -> Result<()> {
    logger::initialize_logger()?;
    db::initialize_pool().await?;

    let cli = cli::Cli::parse();
    cli::run_cli_command(cli).await?;

    Ok(())
}
