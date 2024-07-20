use std::env;
use std::str::FromStr;

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

    let level_filter = if let Ok(log_level) = env::var("LOG_LEVEL") {
        if let Ok(log_level) = LevelFilter::from_str(&log_level) {
            log_level
        } else {
            LevelFilter::Debug
        }
    } else {
        LevelFilter::Debug
    };

    TermLogger::init(
        level_filter,
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
