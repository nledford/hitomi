use std::collections::HashMap;
use std::time::Duration;

use anyhow::{anyhow, Result};
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, FromRepr, VariantNames};
use tokio::time::sleep;

use crate::plex::models::tracks::Track;
use crate::profiles::profile_section::ProfileSection;
use crate::state::{self, APP_STATE};
use crate::utils;

pub mod profile;
mod profile_section;
// mod sections;
pub mod types;
pub mod wizards;
pub mod manager;

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

/*
pub async fn perform_refresh(run_loop: bool) -> Result<()> {
    state::perform_refresh(run_loop, false).await?;

    if run_loop {
        loop {
            sleep(Duration::from_secs(1)).await;

            if utils::perform_refresh().await {
                state::perform_refresh(run_loop, true).await?;
            }
        }
    }

    Ok(())
}
 */
