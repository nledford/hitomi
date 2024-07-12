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

/*
async fn fetch_section_tracks(
    section: Option<&ProfileSection>,
    profile_title: &str,
    limit: Option<i32>,
) -> Result<Vec<Track>> {
    let section = if let Some(section) = section {
        if !section.is_enabled() {
            return Ok(vec![]);
        }
        section
    } else {
        return Ok(vec![]);
    };

    let app_state = APP_STATE.get().read().await;
    let plex = app_state.get_plex_client()?;

    let profile = app_state
        .get_profile_by_title(profile_title)
        .ok_or(anyhow!("Profile `{profile_title}` not found"))?;
    let profile_source = profile.get_profile_source();
    let profile_source_id = profile.get_profile_source_id();

    let mut filters = HashMap::new();
    if section.get_minimum_track_rating() != 0 {
        filters.insert(
            "userRating>>".to_string(),
            section.get_minimum_track_rating().to_string(),
        );
    }

    if section.is_unplayed() {
        filters.insert("viewCount".to_string(), "0".to_string());
    } else {
        filters.insert("viewCount>>".to_string(), "0".to_string());
    }

    match profile_source {
        // Nothing special needs to be done for a library source, so this branch is left blank
        ProfileSource::Library => {}
        ProfileSource::Collection => {
            let collection = plex.fetch_collection(profile_source_id.unwrap()).await?;

            let artists = plex.fetch_artists_from_collection(&collection).await?;
            let artists = artists.join(",");

            filters.insert("artist.id".to_string(), artists);
        }
        ProfileSource::Playlist => {
            todo!("Playlist option not yet implemented")
        }
        ProfileSource::SingleArtist => {
            todo!("Single artist option not yet implemented")
        }
    }

    let tracks = plex
        .fetch_music(filters, section.get_sorting(), limit)
        .await?;

    Ok(tracks)
}
 */
