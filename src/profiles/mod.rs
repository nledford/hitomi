use std::time::Duration;

use anyhow::Result;
use chrono::{Local, Timelike};
use clap::Subcommand;
use futures::future;
use serde::{Deserialize, Serialize};
use simplelog::error;
use strum::{Display, EnumString, FromRepr, VariantNames};
use tokio::time::sleep;

use crate::profiles::profile::Profile;
use crate::state::AppState;

pub mod profile;
mod profile_section;
pub mod types;
pub mod wizards;

/// Divisors of 60
static VALID_INTERVALS: [u32; 10] = [2, 3, 4, 5, 6, 10, 12, 15, 20, 30];

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Display,
    EnumString,
    PartialEq,
    Serialize,
    VariantNames,
)]
pub enum SectionType {
    /// Tracks that have never been played at least once
    #[default]
    #[strum(to_string = "Unplayed Tracks")]
    Unplayed,
    /// The least played tracks, (e.g., 1 or 2 plays)
    #[strum(to_string = "Least Played Tracks")]
    LeastPlayed,
    /// The tracks that have not been played in a long while
    /// (e.g., a track was last played six months ago)
    #[strum(to_string = "Oldest Tracks")]
    Oldest,
}

#[derive(
    Clone, Copy, Debug, Default, Deserialize, Display, FromRepr, PartialEq, Serialize, VariantNames,
)]
pub enum ProfileSource {
    #[default]
    Library,
    Collection,
    Playlist,
    #[strum(to_string = "Single Artist")]
    SingleArtist,
}

#[derive(Debug, PartialEq, Subcommand)]
pub enum ProfileAction {
    /// Create a new profile
    Create,
    /// Delete the playlist
    Delete,
    /// Edit an existing profile
    Edit,
    /// List existing profiles found on disk
    List,
    /// Display a sample of songs from the profile
    Preview,
    /// Update profile's playlist on the plex server
    Update,
    /// View profiles
    View,
}

pub async fn perform_refresh(app_state: &AppState, run_loop: bool) -> Result<()> {
    refresh_playlists_from_profiles(app_state, run_loop, false).await?;

    if run_loop {
        loop {
            sleep(Duration::from_secs(1)).await;
            let now = Local::now();

            if now.second() == 0 {
                refresh_playlists_from_profiles(app_state, run_loop, true).await?;
            }
        }
    }

    Ok(())
}

async fn refresh_playlists_from_profiles(
    app_state: &AppState,
    run_loop: bool,
    ran_once: bool,
) -> Result<()> {
    let mut profiles = app_state.get_enabled_profiles();

    let mut tasks: Vec<_> = vec![];
    for profile in profiles.iter_mut() {
        let is_current_minute = Local::now().minute() == profile.get_current_refresh_minute();

        if !ran_once || is_current_minute {
            let task = Profile::build_playlist(profile, app_state, ProfileAction::Update, None);
            tasks.push(task);
        }
    }

    match future::try_join_all(tasks).await {
        Ok(_) => {
            if run_loop {
                profiles
                    .iter()
                    .for_each(|profile| profile.print_next_refresh())
            }
        }
        Err(err) => {
            error!("Error occurred while attempting to refresh profiles: {err}")
        }
    }

    Ok(())
}
