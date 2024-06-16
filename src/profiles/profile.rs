use std::cmp::PartialEq;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, TimeDelta, Timelike};
use default_struct_builder::DefaultBuilder;
use dialoguer::{Confirm, Input, MultiSelect, Select};
use dialoguer::theme::ColorfulTheme;
use futures_lite::future;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use simplelog::info;
use strum::VariantNames;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::plex::models::Track;
use crate::plex::Plex;
use crate::profiles::{ProfileAction, ProfileSource, SectionType};
use crate::profiles::profile_section::{ProfileSection, Sections};
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

    pub fn get_sections_mut(&mut self) -> &mut Sections {
        &mut self.sections
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

    fn refresh_interval_str(&self) -> String {
        format!(
            "{} minutes ({} refreshes per hour)",
            self.refresh_interval,
            self.refreshes_per_hour()
        )
    }

    fn refreshes_per_hour(&self) -> i32 {
        60 / self.refresh_interval as i32
    }

    fn time_limit_str(&self) -> String {
        format!("{} hours", self.time_limit)
    }

    pub fn get_section_time_limit(&self) -> f64 {
        self.time_limit as f64 / self.sections.num_enabled() as f64
    }

    fn get_global_track_total(&self) -> usize {
        self.sections.global_track_total()
    }

    fn get_track_limit(&self) -> Option<i32> {
        if self.track_limit == 0 {
            Some(1111)
        } else {
            Some(self.track_limit)
        }
    }

    pub fn get_current_refresh_minute(&self, now: DateTime<Local>) -> u32 {
        *build_refresh_minutes(self.refresh_interval)
            .iter()
            .find(|x| *x >= &now.minute())
            .unwrap_or(&0)
    }

    pub fn get_next_refresh_minute(&self, now: DateTime<Local>) -> u32 {
        *build_refresh_minutes(self.refresh_interval)
            .iter()
            .find(|x| *x > &now.minute())
            .unwrap_or(&0)
    }

    pub fn get_next_refresh_time(&self, now: DateTime<Local>) -> DateTime<Local> {
        let next_minute = self.get_next_refresh_minute(now);

        now
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .add(TimeDelta::minutes(next_minute as i64))
    }

    pub fn get_next_refresh_str(&self, now: DateTime<Local>) -> String {
        let next_refresh_time = self.get_next_refresh_time(now);
        format!(
            "LAST UPDATE: {}\nNEXT UPDATE: {}",
            now.format("%F %T"),
            next_refresh_time.format("%R")
        )
    }

    pub fn print_next_refresh(&self) {
        info!("Next refresh at {}", self.get_next_refresh_time(Local::now()).format("%H:%M"))
    }

    fn get_unplayed_track(&self, index: usize) -> Option<Track> {
        self.sections.get_unplayed_tracks().get(index).cloned()
    }

    fn get_least_played_track(&self, index: usize) -> Option<Track> {
        self.sections.get_least_played_tracks().get(index).cloned()
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
    pub async fn build_playlist(profile: &mut Profile, action: ProfileAction, plex: &Plex) -> Result<()> {
        info!("Building `{}` playlist...", profile.title);

        let source = profile.get_profile_source();
        let source_id = profile.get_profile_source_id();
        let time_limit = profile.get_section_time_limit();
        profile
            .sections
            .fetch_tracks(plex, source, &source_id.as_deref(), time_limit)
            .await?;

        let combined = profile.combine_sections()?;

        // debug print sample
        for (idx, track) in combined.iter().take(15).enumerate() {
            println!(
                "{idx:02} {:30} {:30} {:4} {:30}",
                track.artist(),
                track.title(),
                track.plays(),
                track.last_played_fmt()
            )
        }

        let items = &combined
            .iter()
            .map(|track| track.id())
            .collect::<Vec<&str>>();

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

                let summary = format!("{}\n{}", profile.get_next_refresh_str(Local::now()), profile.summary);
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

    pub async fn load_profiles(dir: &str) -> anyhow::Result<Vec<Profile>> {
        let dir = Path::new(dir);

        let mut result = vec![];

        if dir.exists() && dir.is_dir() {
            let mut reader = tokio::fs::read_dir(&dir).await?;
            while let Some(entry) = reader.next_entry().await? {
                let profile = Profile::load_from_disk(entry.path().to_str().unwrap()).await?;
                result.push(profile)
            }
        }

        Ok(result)
    }
}


// WIZARD #####################################################################

/// Divisors of 60
static VALID_INTERVALS: [u32; 10] = [2, 3, 4, 5, 6, 10, 12, 15, 20, 30];

pub async fn create_profile_wizard(app_state: &AppState) -> Result<Profile> {
    let profile_name = set_profile_name(app_state).await?;

    let summary = set_summary()?;
    let refresh_interval = select_refresh_interval()?;
    let time_limit = set_time_limit()?;

    let profile_source = select_profile_source()?;
    let profile_source_id = select_profile_source_id(profile_source, app_state).await?;

    let sections = select_profile_sections()?;

    let profile = Profile::with_title(&profile_name)
        .summary(summary)
        .profile_source(profile_source)
        .profile_source_id(profile_source_id)
        .sections(sections)
        .refresh_interval(refresh_interval)
        .time_limit(time_limit);

    Ok(profile)
}

async fn set_profile_name(app_state: &AppState) -> Result<String> {
    let profile_name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("What is the name of your new profile? This will also be the name of the playlist on the plex server.")
        .interact_text()?;

    if app_state.get_profile(&profile_name).is_some() {
        let choice = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Profile `{profile_name}` already exists. Do you want to overwrite this profile?"
            ))
            .default(false)
            .interact()?;

        if !choice {
            return Err(anyhow!("Profile already exists"));
        }
    }

    if app_state.get_playlist_by_title(&profile_name).is_some() {
        let choice = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Playlist `{profile_name}` already exists in plex. Do you want to overwrite this playlist?"))
            .default(false)
            .interact()?;

        if !choice {
            return Err(anyhow!("Playlist already exists in plex"));
        }
    }

    Ok(profile_name)
}

fn set_summary() -> Result<String> {
    let summary = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("What is the summary for your new profile? This will also be the summary of the playlist on the plex server.")
        .default(String::default())
        .interact_text()?;

    Ok(summary)
}

fn select_refresh_interval() -> Result<u32> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select the refresh interval for this profile:")
        .default(0)
        .items(&VALID_INTERVALS.map(|i| format!("{i} minutes")))
        .interact()?;

    Ok(VALID_INTERVALS[selection])
}

fn set_time_limit() -> Result<u32> {
    let time_limit = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter a time limit in hours for the profile, or `0` for no time limit:")
        .default("24".to_string())
        .interact_text()?
        .parse::<u32>()?;

    Ok(time_limit)
}

fn select_profile_source() -> Result<ProfileSource> {
    let choices = ProfileSource::VARIANTS;
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select the source for this profile:")
        .default(0)
        .items(choices)
        .interact()?;

    Ok(ProfileSource::from_repr(selection).unwrap())
}

async fn select_profile_source_id(profile_source: ProfileSource, app_state: &AppState) -> Result<Option<String>> {
    let plex = app_state.get_plex();

    let id = match profile_source {
        ProfileSource::Library => None,
        ProfileSource::Collection => {
            let collections = plex.get_collections();
            let titles = collections
                .iter()
                .map(|x| x.title.as_str())
                .collect::<Vec<&str>>();

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a collection:")
                .default(0)
                .items(&titles)
                .interact()?;

            Some(collections[selection].rating_key.to_owned())
        }
        ProfileSource::Playlist => {
            let playlists = plex.get_playlists();
            let titles = playlists
                .iter()
                .map(|x| x.title.as_str())
                .collect::<Vec<&str>>();

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a playlist")
                .default(0)
                .items(&titles)
                .interact()?;

            Some(playlists[selection].rating_key.to_owned())
        }
        ProfileSource::SingleArtist => {
            let artist: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Search for an artist:")
                .interact_text()?;

            info!("Searching for artists. Please wait...");
            let artists = plex.search_for_artist(&artist).await?;

            let names = &artists
                .iter()
                .map(|x| x.title.to_owned())
                .collect::<Vec<String>>();

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select an artist:")
                .default(0)
                .items(names)
                .interact()?;

            Some(artists[selection].id().to_owned())
        }
    };

    Ok(id)
}

fn select_profile_sections() -> Result<Sections> {
    let defaults = &[false, false, false];
    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Which sections do you want to include in your profile?")
        .items(SectionType::VARIANTS)
        .defaults(defaults)
        .interact()?;

    let selections = if selections.is_empty() {
        vec![0, 1, 2]
    } else {
        selections
    };

    let mut sections = Sections::default();

    if selections.contains(&0) {
        sections = sections.unplayed_tracks(build_profile_section(SectionType::Unplayed)?)
    }

    if selections.contains(&1) {
        sections = sections.least_played_tracks(build_profile_section(SectionType::LeastPlayed)?)
    }

    if selections.contains(&2) {
        sections = sections.oldest_tracks(build_profile_section(SectionType::Oldest)?)
    }

    Ok(sections)
}

fn build_profile_section(section_type: SectionType) -> Result<ProfileSection> {
    println!("\nBuilding Section: {section_type}");

    let deduplicate_tracks_by_guid = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to deduplicate tracks by their Plex GUID?")
        .default(true)
        .interact()?;

    let deduplicate_by_track_and_artist = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to deduplicate tracks with the same title and artist?")
        .default(true)
        .interact()?;

    let maximum_tracks_by_artists =
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter a maximum number of tracks that can appear in a playlist by a single artist. (A value of `0` disables any limit.)")
            .default(25)
            .validate_with(|input: &i32| -> Result<(), &str> {
                if *input >= 0 {
                    Ok(())
                } else {
                    Err("Value cannot be less than zero")
                }
            })
            .interact_text()?;

    let minimum_track_rating = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter a minimum star rating for included tracks:")
        .default(3)
        .validate_with(|input: &u32| -> Result<(), &str> {
            if *input <= 5 {
                Ok(())
            } else {
                Err("Minimum rating cannot be greater than five")
            }
        })
        .interact_text()?;

    let randomize = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to randomize the track order?")
        .default(true)
        .interact()?;

    // TODO get valid sort fields from plex
    let sorting = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter a comma separated list of fields to sort")
        .default(get_default_sorting(section_type))
        // TODO validate
        .interact_text()?;

    let section = ProfileSection::with_enabled()
        .section_type(section_type)
        .deduplicate_tracks_by_guid(deduplicate_tracks_by_guid)
        .deduplicate_tracks_by_title_and_artist(deduplicate_by_track_and_artist)
        .maximum_tracks_by_artist(maximum_tracks_by_artists)
        .minimum_track_rating(minimum_track_rating)
        .randomize_tracks(randomize)
        .sorting(sorting);

    Ok(section)
}

fn get_default_sorting(section_type: SectionType) -> String {
    match section_type {
        SectionType::Unplayed => vec!["userRating:desc", "viewCount", "lastViewedAt", "guid", "mediaBitrate:desc"],
        SectionType::LeastPlayed => vec!["viewCount", "lastViewedAt", "guid", "mediaBitrate:desc"],
        SectionType::Oldest => vec!["lastViewedAt", "viewCount", "guid", "mediaBitrate:desc"],
    }
        .join(",")
}

// TESTS ######################################################################

#[cfg(test)]
mod tests {}
