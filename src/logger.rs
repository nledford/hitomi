use std::env;
use std::str::FromStr;

use anyhow::Result;
use log::LevelFilter;
use simplelog::*;

pub fn initialize_logger() -> Result<()> {
    let logger_config = ConfigBuilder::new()
        .set_time_level(LevelFilter::Off)
        .build();

    let level_filter = get_log_level();

    TermLogger::init(
        level_filter,
        logger_config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;

    Ok(())
}

fn get_log_level() -> LevelFilter {
    if let Ok(log_level) = env::var("LOG_LEVEL") {
        if let Ok(log_level) = LevelFilter::from_str(&log_level) {
            log_level
        } else {
            LevelFilter::Debug
        }
    } else {
        LevelFilter::Debug
    }
}
