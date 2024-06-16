use anyhow::Result;
use chrono::Local;
use clap::Parser;
use log::*;
use simplelog::*;
use time::UtcOffset;

use chidori::cli;

#[tokio::main]
async fn main() -> Result<()> {
    let logger_config = ConfigBuilder::new()
        .set_time_offset(UtcOffset::from_whole_seconds(Local::now().offset().local_minus_utc()).unwrap())
        .build();

    TermLogger::init(
        LevelFilter::Trace,
        logger_config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;

    // config::delete_config_file().await;

    let cli = cli::Cli::parse();
    cli::run_cli_command(cli).await?;

    Ok(())
}
