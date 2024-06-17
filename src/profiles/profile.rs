use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Result;
use chrono::{DateTime, Local, TimeDelta, Timelike};
use default_struct_builder::DefaultBuilder;
use dialoguer::Confirm;
use dialoguer::theme::ColorfulTheme;
use futures_lite::future;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use simplelog::{debug, error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::plex::models::Track;
use crate::profiles::{ProfileAction, ProfileSource};
use crate::profiles::profile_section::Sections;
use crate::state::AppState;

// PROFILE ####################################################################

#[derive(Clone, Debug, DefaultBuilder, Default, Deserialize, Serialize, PartialEq)]
pub struct Profile {
    /// The plex ID for the playlist
    playlist_id: String,
    /// The name of the profile and the resulting playlist
    title: String,
    /// The summary for the profile and the resulting playlist
    summary: String,
    /// Indicates whether to use the profile. If false, the application will skip this profile when
    /// refreshing playlists
    enabled: bool,
    /// The location from which the profile fetches tracks
    profile_source: ProfileSource,
    profile_source_id: Option<String>,
    /// How often in minutes the profile should refresh in an hour
    refresh_interval: u32,
    /// The time limit in hours of the playlist. Maximum value is 72 hours.
    time_limit: u32,
    track_limit: i32,
    sections: Sections,
}

impl Profile {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_title(title: &str) -> Self {
        Self::default().title(title.to_string())
    }

    fn set_playlist_id(&mut self, playlist_id: &str) {
        self.playlist_id = playlist_id.to_string()
    }

    pub fn get_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_playlist_id(&self) -> &str {
        &self.playlist_id
    }

    pub fn get_profile_source(&self) -> ProfileSource {
        self.profile_source
    }

    pub fn get_profile_source_id(&self) -> Option<String> {
        self.profile_source_id.clone()
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn get_summary(&self) -> &str {
        &self.summary
    }

    pub fn get_sections(&self) -> &Sections {
        &self.sections
    }

    fn file_name(&self) -> String {
        format!("{}.json", self.title)
    }

    fn profile_path(&self, profiles_directory: &str) -> PathBuf {
        future::block_on(async {
            PathBuf::new()
                .join(profiles_directory)
                .join(self.file_name())
        })
    }

    // fn refresh_interval_str(&self) -> String {
    //     format!(
    //         "{} minutes ({} refreshes per hour)",
    //         self.refresh_interval,
    //         self.refreshes_per_hour()
    //     )
    // }

    // fn refreshes_per_hour(&self) -> i32 {
    //     60 / self.refresh_interval as i32
    // }

    // fn time_limit_str(&self) -> String {
    //     format!("{} hours", self.time_limit)
    // }

    pub fn get_section_time_limit(&self) -> f64 {
        self.time_limit as f64 / self.sections.num_enabled() as f64
    }

    // fn get_track_limit(&self) -> Option<i32> {
    //     if self.track_limit == 0 {
    //         Some(1111)
    //     } else {
    //         Some(self.track_limit)
    //     }
    // }

    pub fn get_current_refresh_minute(&self) -> u32 {
        *build_refresh_minutes(self.refresh_interval)
            .iter()
            .find(|x| *x >= &Local::now().minute())
            .unwrap_or(&0)
    }

    pub fn get_next_refresh_minute(&self) -> u32 {
        *build_refresh_minutes(self.refresh_interval)
            .iter()
            .find(|x| *x > &Local::now().minute())
            .unwrap_or(&0)
    }

    pub fn get_next_refresh_time(&self) -> DateTime<Local> {
        let next_minute = self.get_next_refresh_minute();

        Local::now()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .add(TimeDelta::minutes(next_minute as i64))
    }

    pub fn get_next_refresh_str(&self) -> String {
        let next_refresh_time = self.get_next_refresh_time();
        format!(
            "LAST UPDATE: {}\nNEXT UPDATE: {}",
            Local::now().format("%F %T"),
            next_refresh_time.format("%R")
        )
    }

    pub fn print_next_refresh(&self) {
        info!("Next refresh at {}", self.get_next_refresh_time().format("%H:%M"))
    }

    fn has_unplayed_tracks(&self) -> bool {
        self.sections.unplayed_tracks.enabled
    }

    fn get_unplayed_track(&self, index: usize) -> Option<Track> {
        self.sections.get_unplayed_tracks().get(index).cloned()
    }

    fn has_least_played_tracks(&self) -> bool {
        self.sections.least_played_tracks.enabled
    }

    fn get_least_played_track(&self, index: usize) -> Option<Track> {
        self.sections.get_least_played_tracks().get(index).cloned()
    }

    fn has_oldest_tracks(&self) -> bool {
        self.sections.oldest_tracks.enabled
    }

    fn get_oldest_track(&self, index: usize) -> Option<Track> {
        self.sections.get_oldest_tracks().get(index).cloned()
    }

    fn get_largest_section_length(&self) -> usize {
        *[
            self.sections.get_unplayed_tracks().len(),
            self.sections.get_least_played_tracks().len(),
            self.sections.get_oldest_tracks().len(),
        ]
            .iter()
            .max()
            .unwrap_or(&0)
    }
}

/// Constructs a `vec` of valid refresh minutes from a given refresh intervals
fn build_refresh_minutes(refresh_interval: u32) -> Vec<u32> {
    let refresh_interval = if refresh_interval < 2 {
        5
    } else if refresh_interval > 30 {
        30
    } else {
        refresh_interval
    };

    (1..=60).filter(|i| i % refresh_interval == 0).collect()
}

/// Plex functions
impl Profile {
    pub async fn build_playlist(profile: &mut Profile, app_state: &AppState, action: ProfileAction) -> Result<()> {
        info!("Building `{}` playlist...", profile.title);

        info!("Fetching tracks for section(s)...");
        profile
            .sections
            .fetch_tracks(&profile.clone(), app_state)
            .await?;

        info!("Combining sections into single playlist...");
        let combined = profile.combine_sections()?;
        let items = &combined
            .iter()
            .map(|track| track.id())
            .collect::<Vec<&str>>();

        let plex = app_state.get_plex();
        match action {
            ProfileAction::Create => {
                let save = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Would you like to save this profile?")
                    .default(true)
                    .interact()?;

                if save {
                    info!("Creating playlist in plex...");
                    let playlist_id = plex.create_playlist(profile).await?;
                    profile.set_playlist_id(&playlist_id);

                    info!("Adding tracks to newly created playlist...");
                    plex.add_items_to_playlist(&playlist_id, items).await?;
                }
            }
            ProfileAction::Update => {
                info!("Wiping destination playlist...");
                plex.clear_playlist(&profile.playlist_id).await?;

                info!("Updating destination playlist...");
                plex.add_items_to_playlist(&profile.playlist_id, items)
                    .await?;

                let summary = format!("{}\n{}", profile.get_next_refresh_str(), profile.summary);
                plex.update_summary(&profile.playlist_id, &summary).await?;
            }
            // Other actions are not relevant to this function and are ignored
            _ => {}
        };

        show_results(&combined, action);

        Ok(())
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

fn show_results(tracks: &[Track], action: ProfileAction) {
    let size = tracks.len();

    let duration: i64 = tracks.par_iter().map(|t| t.duration()).sum();
    let duration = Duration::from_millis(duration as u64);
    let duration = humantime::format_duration(duration).to_string();

    let action = if action == ProfileAction::Create {
        "created"
    } else {
        "updated"
    };

    log::info!(
        "Successfully {} playlist!\n\tFinal size: {}\n\tFinal duration: {}",
        action,
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
        str += &format!("\nRefresh Interval: Every {} minutes", self.refresh_interval);
        str += &format!("\nTime Limit:       {}", if self.time_limit == 0 { "None".to_string() } else { format!("{} hours", self.time_limit) });

        str += "\n\nSections:";
        if self.has_unplayed_tracks() {
            str += &format!("\n{}", self.sections.unplayed_tracks)
        }

        if self.has_least_played_tracks() {
            str += &format!("\n{}", self.sections.least_played_tracks)
        }

        if self.has_oldest_tracks() {
            str += &format!("\n{}", self.sections.oldest_tracks)
        }

        write!(f, "{str}")
    }
}

impl Profile {
    pub async fn save_to_file(&self, profiles_directory: &str) -> Result<()> {
        tokio::fs::create_dir_all(profiles_directory).await?;

        let json = serde_json::to_string_pretty(&self)?;

        let mut file = tokio::fs::File::create(&self.profile_path(profiles_directory)).await?;
        file.write_all(json.as_bytes()).await?;

        Ok(())
    }

    pub async fn load_from_disk(path: &str) -> Result<Profile> {
        let mut file = tokio::fs::File::open(path).await?;
        let mut profile = String::default();
        file.read_to_string(&mut profile).await?;
        let profile: Self = serde_json::from_str(&profile)?;
        Ok(profile)
    }

    pub async fn load_profiles(dir: &str) -> Result<Vec<Profile>> {
        debug!("Loading profiles from disk...");
        let dir = Path::new(dir);

        if !dir.exists() {
            panic!("Profiles directory `{}` could not be found.", dir.display())
        }

        if !dir.is_dir() {
            panic!("Profiles directory `{}` is not a directory.", dir.display())
        }

        if dir.read_dir()?.next().is_none() {
            error!("Profiles directory `{}` is empty.", dir.display());
            return Ok(vec![]);
        }

        let mut result = vec![];
        let mut reader = tokio::fs::read_dir(&dir).await?;
        while let Some(entry) = reader.next_entry().await? {
            let profile = Profile::load_from_disk(entry.path().to_str().unwrap()).await?;
            result.push(profile)
        }

        info!("{} profile(s) loaded from disk", &result.len());

        Ok(result)
    }
}

// TESTS ######################################################################

#[cfg(test)]
mod tests {}
