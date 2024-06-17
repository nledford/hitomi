use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
use chrono::{Local, Timelike};
use clap::Args;
use log::info;
use simplelog::error;
use tokio::time::sleep;

use crate::profiles::profile::Profile;
use crate::profiles::ProfileAction;
use crate::state::AppState;

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

pub async fn execute_run_cmd(cmd: RunCmds, app_state: &AppState) -> Result<()> {
    print_title(cmd.run_loop);

    perform_refresh(app_state, cmd.run_loop, false).await?;

    if cmd.run_loop {
        loop {
            sleep(Duration::from_secs(1)).await;

            if Local::now().second() == 0 {
                perform_refresh(app_state, cmd.run_loop, true).await?;
            }
        }
    }

    Ok(())
}

async fn perform_refresh(app_state: &AppState, run_loop: bool, ran_once: bool) -> Result<()> {
    let mut profiles = app_state.get_enabled_profiles();
    let mut refresh_failures = HashMap::new();

    for profile in profiles.iter_mut() {
        let playlist_id = profile.get_playlist_id().to_owned();
        refresh_failures.entry(playlist_id.clone()).or_insert(0);

        if !ran_once || Local::now().minute() == profile.get_current_refresh_minute() {
            match Profile::build_playlist(profile, app_state, ProfileAction::Update).await {
                Ok(_) => {
                    refresh_failures.entry(playlist_id.clone()).and_modify(|v| *v = 0);
                }
                Err(err) => {
                    refresh_failures.entry(playlist_id.clone()).and_modify(|v| *v += 1);
                    let failures = refresh_failures.get(&playlist_id).unwrap();

                    if *failures <= 3 {
                        error!("An error occurred while attempting to build the `{}` playlist: {err}", profile.get_title());
                        error!("Skipping building this playlist. {} build attempt(s) remaining...", 3 - *failures);
                    } else {
                        panic!("Failed to connect to Plex server more than three times.");
                    }
                }
            }

            if run_loop {
                profile.print_next_refresh();
            }
        }
    }

    Ok(())
}
