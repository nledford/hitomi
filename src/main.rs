use anyhow::Result;
use log::*;
use simplelog::*;

use hitomi::db;

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

    db::initialize_pool().await?;

    let profiles = db::profiles::get_profiles().await?;
    for profile in profiles {
        println!("{:?}", &profile);

        for section in db::profiles::get_profile_sections(profile.profile_id).await? {
            println!("{:?}", &section);
        }
    }

    // config::delete_config_file().await;

    // let cli = cli::Cli::parse();
    // cli::run_cli_command(cli).await?;

    Ok(())
}
