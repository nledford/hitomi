use anyhow::Result;
use clap::Parser;
use log::*;
use simplelog::*;

use hitomi::{cli, db};

#[tokio::main]
async fn main() -> Result<()> {
    let logger_config = ConfigBuilder::new()
        .set_time_level(LevelFilter::Off)
        .build();

    TermLogger::init(
        LevelFilter::Debug,
        logger_config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;

    db::initialize_pool().await?;

    // config::delete_config_file().await;

    let cli = cli::Cli::parse();
    cli::run_cli_command(cli).await?;

    Ok(())
}
