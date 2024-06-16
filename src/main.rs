use anyhow::Result;
use clap::Parser;
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, TermLogger, TerminalMode, ConfigBuilder};

use chidori::cli;

#[tokio::main]
async fn main() -> Result<()> {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Trace,
        ConfigBuilder::default()
            .set_time_offset_to_local()
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])?;

    // config::delete_config_file().await;

    let cli = cli::Cli::parse();
    cli::run_cli_command(cli).await?;

    Ok(())
}
