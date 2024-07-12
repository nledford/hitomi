use anyhow::Result;
use clap::Parser;
use log::*;
use simplelog::*;

use hitomi::cli;

use hitomi::state;
use hitomi::state::APP_STATE;

#[tokio::main]
async fn main() -> Result<()> {
    let logger_config = ConfigBuilder::new()
        .set_time_level(LevelFilter::Off)
        .build();

    TermLogger::init(
        LevelFilter::Trace,
        logger_config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;

    state::initialize_app_state().await?;

    let app_state = APP_STATE.get().read().await;
    let profiles = app_state.get_profiles();
    for profile in profiles {
        println!("{:?}", &profile)
    }

    // config::delete_config_file().await;

    // let cli = cli::Cli::parse();
    // cli::run_cli_command(cli).await?;

    Ok(())
}
