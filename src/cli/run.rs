use std::time::Duration;

use anyhow::Result;
use clap::Args;
use log::info;
use tokio::time::sleep;

use crate::profiles::profile::Profile;
use crate::profiles::ProfileAction;
use crate::state::APP_STATE;

#[derive(Args, Debug)]
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

    let mut profiles;
    {
        let lock = APP_STATE.lock().await;
        profiles = lock.get_profiles().to_vec();
    }

    for profile in profiles.iter_mut() {
        // profile.build_playlist(ProfileAction::Update).await?;
        Profile::build_playlist(profile, ProfileAction::Update).await?
    }

    if cmd.run_loop {
        loop {
            sleep(Duration::from_secs(1)).await;

            info!("Application isn't running anything yet. Kill the loop!")
        }
    }

    Ok(())
}
