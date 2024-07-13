use std::time::Duration;

use anyhow::Result;
use clap::Args;
use simplelog::info;
use tokio::time::sleep;

use crate::profiles::manager::PROFILE_MANAGER;

#[derive(Args, Debug, PartialEq)]
pub struct RunCmds {
    /// Run the application indefinitely, refreshing based on the interval provided in each profile
    #[arg(short = 'l', long, default_value_t = false)]
    pub run_loop: bool,
}

fn print_title(looping: bool) {
    let version = env!("CARGO_PKG_VERSION");

    info!("Plex Playlists v{}", version);

    if looping {
        info!("Application is running in loop mode")
    }
}

pub async fn execute_run_cmd(cmd: RunCmds) -> Result<()> {
    print_title(cmd.run_loop);

    let manager = PROFILE_MANAGER.get().unwrap().read().await;
    let manager = manager.clone();

    manager
        .refresh_playlists_from_profiles(cmd.run_loop, false)
        .await?;

    if cmd.run_loop {
        loop {
            sleep(Duration::from_secs(1)).await;

            if manager.get_any_profile_refresh() {
                manager
                    .refresh_playlists_from_profiles(cmd.run_loop, true)
                    .await?;
            }
        }
    }

    Ok(())
}
