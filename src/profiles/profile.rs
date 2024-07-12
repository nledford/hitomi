use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Result;
use chrono::{DateTime, Local, TimeDelta, Timelike};
use derive_builder::Builder;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use serde::{Deserialize, Serialize};
use simplelog::{debug, error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::plex::models::tracks::Track;
use crate::plex::types::PlexId;
use crate::profiles::types::{ProfileSourceId, RefreshInterval};
use crate::profiles::{ProfileAction, ProfileSource};
use crate::profiles::profile_section::ProfileSection;
use crate::state::APP_STATE;
use crate::types::Title;
use crate::utils;

// PROFILE ####################################################################

#[derive(Builder, Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[builder(default)]
pub struct Profile {
    /// The plex ID for the playlist
    playlist_id: PlexId,
    /// The name of the profile and the resulting playlist
    title: Title,
    /// The summary for the profile and the resulting playlist
    summary: String,
    /// Indicates whether to use the profile. If false, the application will skip this profile when
    /// refreshing playlists
    #[builder(default = "true")]
    enabled: bool,
    /// The location from which the profile fetches tracks
    profile_source: ProfileSource,
    profile_source_id: Option<ProfileSourceId>,
    /// How often in minutes the profile should refresh in an hour
    refresh_interval: RefreshInterval,
    /// The time limit in hours of the playlist.
    time_limit: u32,
    /// The track limit of the playlist
    track_limit: u32,
    /// Profile sections
    sections: Vec<ProfileSection>,
}

impl Profile {
    pub fn get_title(&self) -> &str {
        &self.title
    }

    // pub fn get_enabled(&self) -> bool {
    //     self.enabled
    // }

    fn file_name(&self) -> String {
        format!("{}.json", self.title)
    }

    pub async fn get_profile_path(&self) -> PathBuf {
        let app_state = APP_STATE.get().read().await;

        PathBuf::new()
            .join(app_state.get_profiles_directory().unwrap())
            .join(self.file_name())
    }
}

/*impl Profile {
    fn set_playlist_id(&mut self, playlist_id: &PlexId) {
        playlist_id.clone_into(&mut self.playlist_id)
    }



    pub fn get_playlist_id(&self) -> &str {
        &self.playlist_id
    }

    pub fn get_profile_source(&self) -> &ProfileSource {
        &self.profile_source
    }

    pub fn get_profile_source_id(&self) -> Option<&ProfileSourceId> {
        self.profile_source_id.as_ref()
    }



    pub fn get_summary(&self) -> &str {
        &self.summary
    }

    pub fn get_sections(&self) -> &Sections {
        &self.sections
    }



    fn refresh_interval_str(&self) -> String {
        format!(
            "Every {} minutes ({} refreshes per hour)",
            self.refresh_interval,
            self.refreshes_per_hour()
        )
    }

    fn refreshes_per_hour(&self) -> i32 {
        60 / *self.refresh_interval.as_ref() as i32
    }

    fn time_limit_str(&self) -> String {
        if self.time_limit == 0 {
            "No Limit".to_string()
        } else {
            format!("{} hours", self.time_limit)
        }
    }

    pub fn get_section_time_limit(&self) -> f64 {
        self.time_limit as f64 / self.sections.num_enabled() as f64
    }

    fn get_track_limit_str(&self) -> String {
        if self.track_limit == 0 {
            "No Limit".to_string()
        } else {
            format!("{} tracks", self.track_limit)
        }
    }

    pub fn check_for_refresh(&self, force_refresh: bool) -> bool {
        if force_refresh {
            return true;
        }

        let current_minute = Local::now().minute();
        let matches_top_of_the_hour = current_minute == 0;
        let matches_refresh_minute = current_minute == self.get_current_refresh_time().minute();

        matches_top_of_the_hour || matches_refresh_minute
    }

    fn get_current_refresh_time(&self) -> DateTime<Local> {
        let current_minute = utils::build_refresh_minutes(&self.refresh_interval)
            .into_iter()
            .find(|x| *x >= Local::now().minute())
            .unwrap_or(0);

        Local::now()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .add(TimeDelta::minutes(current_minute as i64))
    }

    pub fn get_next_refresh_time(&self) -> DateTime<Local> {
        let next_minute = utils::build_refresh_minutes(&self.refresh_interval)
            .into_iter()
            .find(|x| *x > Local::now().minute())
            .unwrap_or(0);

        Local::now()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .add(TimeDelta::minutes(next_minute as i64))
    }

    pub fn get_next_refresh_hour_minute(&self) -> String {
        self.get_next_refresh_time().format("%R").to_string()
    }

    fn get_next_refresh_str(&self) -> String {
        let next_refresh_time = self.get_next_refresh_time();
        format!(
            "LAST UPDATE: {}\nNEXT UPDATE: {}",
            Local::now().format("%F %T"),
            next_refresh_time.format("%R")
        )
    }

    fn has_unplayed_tracks(&self) -> bool {
        self.sections.has_unplayed_section()
    }

    fn get_unplayed_track(&self, index: usize) -> Option<&Track> {
        if let Some(tracks) = self.sections.get_unplayed_tracks() {
            tracks.get(index)
        } else {
            None
        }
    }

    fn has_least_played_tracks(&self) -> bool {
        self.sections.has_least_played_section()
    }

    fn get_least_played_track(&self, index: usize) -> Option<&Track> {
        if let Some(tracks) = self.sections.get_least_played_tracks() {
            tracks.get(index)
        } else {
            None
        }
    }

    fn has_oldest_tracks(&self) -> bool {
        self.sections.has_oldest_section()
    }

    fn get_oldest_track(&self, index: usize) -> Option<&Track> {
        if let Some(tracks) = self.sections.get_oldest_tracks() {
            tracks.get(index)
        } else {
            None
        }
    }

    fn get_largest_section_length(&self) -> usize {
        *[
            self.sections.num_unplayed_tracks(),
            self.sections.num_least_played_tracks(),
            self.sections.num_oldest_tracks(),
        ]
        .iter()
        .max()
        .unwrap_or(&0)
    }
}

/// Plex functions
impl Profile {
    pub async fn build_playlist(
        profile: Profile,
        action: ProfileAction,
        limit: Option<i32>,
    ) -> Result<Profile> {
        let mut profile = profile.clone();
        info!("Building `{}` playlist...", profile.get_title());

        info!("Fetching tracks for section(s)...");
        let tracks = profile
            .sections
            .fetch_tracks(profile.get_title(), limit)
            .await?;

        profile
            .sections
            .set_tracks(tracks, profile.get_section_time_limit())
            .await;

        info!("Combining sections into single playlist...");
        let combined = profile.combine_sections()?;
        let items = &combined
            .iter()
            .map(|track| track.id())
            .collect::<Vec<&str>>();

        let app_state = APP_STATE.get().read().await;
        let plex_client = app_state.get_plex_client()?;
        match action {
            ProfileAction::Create => {
                let save = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Would you like to save this profile?")
                    .default(true)
                    .interact()?;

                if save {
                    info!("Creating playlist in plex...");
                    let playlist_id = plex_client.create_playlist(&profile).await?;
                    let playlist_id = PlexId::try_new(playlist_id)?;
                    profile.set_playlist_id(&playlist_id);

                    info!("Adding tracks to newly created playlist...");
                    plex_client
                        .add_items_to_playlist(&playlist_id, items)
                        .await?;
                } else {
                    info!("Playlist not saved");
                }
            }
            ProfileAction::Preview => {
                for (i, track) in combined.iter().take(25).enumerate() {
                    println!("{:2} {}", i + 1, track)
                }
            }
            ProfileAction::Update => {
                info!("Wiping destination playlist...");
                plex_client.clear_playlist(&profile.playlist_id).await?;

                info!("Updating destination playlist...");
                plex_client
                    .add_items_to_playlist(&profile.playlist_id, items)
                    .await?;

                let summary = format!("{}\n{}", profile.get_next_refresh_str(), profile.summary);
                plex_client
                    .update_summary(&profile.playlist_id, &summary)
                    .await?;
            }
            // Other actions are not relevant to this function and are ignored
            _ => {}
        };

        if action != ProfileAction::Preview {
            show_results(&combined, profile.get_title(), action);
        }

        Ok(profile)
    }

    fn combine_sections(&self) -> Result<Vec<Track>> {
        info!("Combing {} sections...", self.sections.num_enabled());
        let mut combined = vec![];

        for i in 0..self.get_largest_section_length() {
            if let Some(track) = self.get_unplayed_track(i) {
                combined.push(track.clone())
            }

            if let Some(track) = self.get_least_played_track(i) {
                combined.push(track.clone())
            }

            if let Some(track) = self.get_oldest_track(i) {
                combined.push(track.clone())
            }
        }

        Ok(combined)
    }
}

fn show_results(tracks: &[Track], title: &str, action: ProfileAction) {
    let size = tracks.len();

    let duration: i64 = tracks.iter().map(|t| t.duration()).sum();
    let duration = Duration::from_millis(duration as u64);
    let duration = humantime::format_duration(duration).to_string();

    let action = if action == ProfileAction::Create {
        "created"
    } else {
        "updated"
    };

    log::info!(
        "Successfully {} `{}` playlist!\n\tFinal size: {}\n\tFinal duration: {}",
        action,
        title,
        size,
        duration
    );
}

impl Display for Profile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut str = format!("\n{}", self.title);
        str += &format!("\n{}", self.summary);
        str += &format!("\nEnabled:          {}", self.enabled);
        str += &format!("\nSource:           {}", self.profile_source);
        str += &format!("\nRefresh Interval: {}", self.refresh_interval_str());
        str += &format!("\nTime Limit:       {}", self.time_limit_str());
        str += &format!("\nTrack Limit:      {}", self.get_track_limit_str());

        str += "\n\nSections:";
        if self.has_unplayed_tracks() {
            str += &format!("\n{}", self.sections.get_unplayed_section().unwrap())
        }

        if self.has_least_played_tracks() {
            str += &format!("\n{}", self.sections.get_least_played_section().unwrap())
        }

        if self.has_oldest_tracks() {
            str += &format!("\n{}", self.sections.get_oldest_section().unwrap())
        }

        write!(f, "{str}")
    }
}
*/