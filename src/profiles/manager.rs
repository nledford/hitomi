use std::cmp::PartialEq;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;
use std::time::Duration;

use anyhow::{anyhow, Result};
use chrono::Local;
use dialoguer::Confirm;
use dialoguer::theme::ColorfulTheme;
use futures::future;
use itertools::{fold, Itertools};
use simplelog::{error, info};
use slotmap::{new_key_type, SecondaryMap, SlotMap};
use state::InitCell;
use tokio::sync::{Mutex, OnceCell, RwLock};
use tokio::task::JoinSet;
use uuid::Uuid;

use crate::{files, plex};
use crate::plex::models::tracks::Track;
use crate::plex::types::PlexId;
use crate::profiles::{ProfileAction, ProfileSource, SectionType};
use crate::profiles::profile::Profile;
use crate::profiles::profile_section::ProfileSection;
use crate::state::APP_STATE;

pub static PROFILE_MANAGER: InitCell<RwLock<ProfileManager>> = InitCell::new();

pub async fn initialize_profile_manager(profiles_directory: &str) -> Result<()> {
    let manager = ProfileManager::new(profiles_directory).await?;
    PROFILE_MANAGER.set(RwLock::new(manager));
    Ok(())
}

new_key_type! {
    pub struct ProfileKey;
}

#[derive(Clone, Debug, Default)]
pub struct ProfileManager {
    /// Profiles that have been loaded from disk
    profiles: Vec<Profile>,
    /// Profiles being managed by the application
    managed_profiles: SlotMap<ProfileKey, Profile>,
    managed_unplayed_sections: SecondaryMap<ProfileKey, ProfileSection>,
    managed_least_played_sections: SecondaryMap<ProfileKey, ProfileSection>,
    managed_oldest_sections: SecondaryMap<ProfileKey, ProfileSection>,
    unplayed_tracks: SecondaryMap<ProfileKey, Vec<Track>>,
    least_played_tracks: SecondaryMap<ProfileKey, Vec<Track>>,
    oldest_tracks: SecondaryMap<ProfileKey, Vec<Track>>,

}

impl ProfileManager {
    pub async fn new(profiles_directory: &str) -> Result<Self> {
        let mut manager = ProfileManager::default();
        manager.profiles = files::load_profiles_from_disk(profiles_directory).await?;
        manager.build_managed_profiles_and_sections();
        Ok(manager)
    }

    fn build_managed_profiles_and_sections(&mut self) {
        let mut managed_profiles = SlotMap::with_key();
        let mut managed_unplayed_sections = SecondaryMap::new();
        let mut managed_least_played_sections = SecondaryMap::new();
        let mut managed_oldest_sections = SecondaryMap::new();

        for profile in &self.profiles {
            let profile_key = managed_profiles.insert(profile.clone());

            for section in profile.get_sections() {
                if section.is_unplayed_section() {
                    let _ = managed_unplayed_sections.insert(profile_key, section.clone());
                }

                if section.is_least_played_section() {
                    let _ = managed_least_played_sections.insert(profile_key, section.clone());
                }

                if section.is_oldest_section() {
                    let _ = managed_oldest_sections.insert(profile_key, section.clone());
                }
            }
        }

        self.managed_profiles = managed_profiles;
        self.managed_unplayed_sections = managed_unplayed_sections;
        self.managed_least_played_sections = managed_least_played_sections;
        self.managed_oldest_sections = managed_oldest_sections;
    }

    pub fn set_playlist_id(&mut self, profile_key: ProfileKey, id: &PlexId) {
        let profile = self.managed_profiles.get_mut(profile_key).unwrap();
        profile.set_playlist_id(id.clone())
    }

    fn get_num_profiles(&self) -> usize {
        self.managed_profiles.len()
    }

    pub fn have_profiles(&self) -> bool {
        !self.managed_profiles.is_empty()
    }

    pub fn get_profiles(&self) -> HashMap<ProfileKey, &Profile> {
        if !self.have_profiles() {
            return HashMap::default();
        }

        self.managed_profiles
            .iter()
            .sorted_unstable_by_key(|(k, v)| v.get_title().to_owned())
            .collect::<HashMap<ProfileKey, &Profile>>()
    }

    pub fn get_enabled_profiles(&self) -> HashMap<ProfileKey, &Profile> {
        self.get_profiles()
            .into_iter()
            .filter_map(|(k, v)| {
                if v.get_enabled() {
                    Some((k, v))
                } else {
                    None
                }
            })
            .collect::<HashMap<ProfileKey, &Profile>>()
    }

    pub fn get_profiles_to_refresh(&self, ran_once: bool) -> HashMap<ProfileKey, &Profile> {
        if ran_once && !self.get_any_profile_refresh() {
            return HashMap::default();
        }

        // If the application has run once, we DO NOT want to override refreshing profiles
        self.get_enabled_profiles()
            .into_iter()
            .filter(|(k, v)| v.check_for_refresh(!ran_once))
            .collect::<HashMap<ProfileKey, &Profile>>()
    }

    pub fn get_profile_titles(&self) -> Vec<String> {
        self.get_profiles()
            .iter()
            .map(|(_, v)| v.get_title().to_string())
            .collect::<Vec<_>>()
    }

    pub fn get_profile_by_key(&self, profile_key: ProfileKey) -> Option<&Profile> {
        let profile = self.managed_profiles
            .iter()
            .find(|(k, v)| *k == profile_key);

        if let Some((k, v)) = profile {
            Some(v)
        } else {
            None
        }
    }

    pub fn get_profile_by_id(&self, id: Uuid) -> Option<&Profile> {
        self.get_profiles()
            .into_iter()
            .find_map(|(_, v)| {
                if v.get_profile_id() == id {
                    Some(v)
                } else {
                    None
                }
            })
    }

    pub fn get_profile_by_title(&self, title: &str) -> Option<&Profile> {
        self.get_profiles()
            .into_iter()
            .find_map(|(_, v)| {
                if v.get_title() == title {
                    Some(v)
                } else {
                    None
                }
            })
    }

    pub fn get_profile_key(&self, title: &str) -> Option<ProfileKey> {
        let profile = self.managed_profiles.iter().find(|(_, v)| v.get_title() == title);
        if let Some((k, _)) = profile {
            Some(k)
        } else {
            None
        }
    }

    pub fn list_profiles(&self) {
        let titles = self.get_profile_titles();
        if titles.is_empty() {
            println!("No profiles found.")
        } else {
            println!("Existing profiles found");
            for title in titles {
                println!("  - {}", title)
            }
        }
    }

    pub fn list_profiles_and_sections(&self) {
        let profiles = &self.managed_profiles;

        for (k, v) in profiles {
            println!("{}", v.get_title());

            if self.managed_unplayed_sections.contains_key(k) {
                println!(" - Unplayed")
            }

            if self.managed_least_played_sections.contains_key(k) {
                println!(" - Least Played")
            }

            if self.managed_oldest_sections.contains_key(k) {
                println!(" - Oldest")
            }
        }
    }

    fn get_enabled_sections_count(&self, profile_key: ProfileKey) -> i32 {
        let mut count = 0;

        if let Some(section) = self.managed_unplayed_sections.get(profile_key) {
            if section.is_enabled() {
                count += 1;
            }
        }

        if let Some(section) = self.managed_least_played_sections.get(profile_key) {
            if section.is_enabled() {
                count += 1;
            }
        }

        if let Some(section) = self.managed_oldest_sections.get(profile_key) {
            if section.is_enabled() {
                count += 1;
            }
        }

        count
    }

    fn get_largest_section_length(&self, profile_key: ProfileKey) -> usize {
        let unplayed = if let Some(section) = self.unplayed_tracks.get(profile_key) {
            section.len()
        } else {
            0
        };

        let least_played = if let Some(section) = self.least_played_tracks.get(profile_key) {
            section.len()
        } else {
            0
        };

        let oldest = if let Some(section) = self.oldest_tracks.get(profile_key) {
            section.len()
        } else {
            0
        };

        *vec![unplayed, least_played, oldest].iter().max().unwrap_or(&0_usize)
    }

    fn get_track(&self, profile_key: ProfileKey, section_type: SectionType, idx: usize) -> Option<&Track> {
        match section_type {
            SectionType::Unplayed => {
                if let Some(tracks) = self.unplayed_tracks.get(profile_key) {
                    tracks.get(idx)
                } else {
                    None
                }
            }
            SectionType::LeastPlayed => {
                if let Some(tracks) = self.least_played_tracks.get(profile_key) {
                    tracks.get(idx)
                } else {
                    None
                }
            }
            SectionType::Oldest => {
                if let Some(tracks) = self.oldest_tracks.get(profile_key) {
                    tracks.get(idx)
                } else {
                    None
                }
            }
        }
    }

    fn get_any_profile_refresh(&self) -> bool {
        self.get_enabled_profiles()
            .iter()
            .any(|(_, v)| v.check_for_refresh(false))
    }

    fn print_update(&self, playlists_updated: i32) {
        info!("Updated {playlists_updated} at {}", Local::now().format("%F %T"));

        let str = self.get_enabled_profiles()
            .into_iter()
            .fold(
                HashMap::new(),
                |mut acc: HashMap<String, Vec<String>>, (_, profile)| {
                    acc.entry(profile.get_next_refresh_hour_minute())
                        .or_default()
                        .push(profile.get_title().to_owned());
                    acc
                },
            )
            .into_iter()
            .sorted()
            .fold(String::default(), |mut acc, (k, v)| {
                acc += &format!("  <b>Refreshing at {k}:</b>\n");
                for title in v {
                    acc += &format!("    - {title}\n");
                }
                acc
            });
        info!("Upcoming refreshes:\n{str}")
    }

    pub async fn perform_refresh(&self, run_loop: bool, ran_once: bool) -> Result<()> {
        let profiles = self.get_profiles_to_refresh(ran_once);
        let refreshed = refresh_playlists_from_profiles(profiles, ran_once).await?;
        if run_loop {
            self.print_update(refreshed)
        }
        Ok(())
    }

    async fn fetch_sections_tracks(&mut self, profile_key: ProfileKey, limit: Option<i32>) -> Result<()> {
        if let Some(_) = self.managed_unplayed_sections.get(profile_key) {
            self.fetch_section_tracks(profile_key, SectionType::Unplayed, limit).await?;
        }

        if let Some(_) = self.managed_least_played_sections.get(profile_key) {
            self.fetch_section_tracks(profile_key, SectionType::LeastPlayed, limit).await?;
        }

        if let Some(_) = self.managed_oldest_sections.get(profile_key) {
            self.fetch_section_tracks(profile_key, SectionType::Oldest, limit).await?;
        }

        Ok(())
    }

    async fn fetch_section_tracks(&mut self, profile_key: ProfileKey, section_type: SectionType, limit: Option<i32>) -> Result<()> {
        let (section, tracks) = match section_type {
            SectionType::Unplayed => {
                (self.managed_unplayed_sections.get(profile_key), self.unplayed_tracks.get_mut(profile_key))
            }
            SectionType::LeastPlayed => {
                (self.managed_least_played_sections.get(profile_key), self.least_played_tracks.get_mut(profile_key))
            }
            SectionType::Oldest => {
                (self.managed_oldest_sections.get(profile_key), self.oldest_tracks.get_mut(profile_key))
            }
        };

        let section = if let Some(section) = section {
            if !section.is_enabled() {
                return Ok(());
            }
            section
        } else {
            return Ok(());
        };
        let profile = self.managed_profiles.get(profile_key).unwrap();
        let profile_source = profile.get_profile_source();
        let profile_source_id = profile.get_profile_source_id();

        let plex_client = plex::get_plex_client().await;

        let mut filters = HashMap::new();
        if section.get_minimum_track_rating() != 0 {
            filters.insert("userRating>>".to_string(), section.get_minimum_track_rating().to_string());
        }

        if section.is_unplayed_section() {
            filters.insert("viewCount".to_string(), "0".to_string());
        } else {
            filters.insert("viewCount>>".to_string(), "0".to_string());
        }

        match profile_source {
            // Nothing special needs to be done for a library source, so this branch is left blank
            ProfileSource::Library => {}
            ProfileSource::Collection => {
                let collection = plex_client.fetch_collection(profile_source_id.unwrap()).await?;
                let artists = plex_client.fetch_artists_from_collection(&collection).await?;
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

        let mut tracks = tracks.unwrap();
        *tracks = plex_client.fetch_music(filters, section.get_sorting(), limit).await?;

        Ok(())
    }

    async fn combine_sections(&self, profile_key: ProfileKey) -> Vec<Track> {
        info!("Combing {} sections...", self.get_enabled_sections_count(profile_key));

        let mut combined = vec![];
        for i in 0..self.get_largest_section_length(profile_key) {
            if let Some(track) = self.get_track(profile_key, SectionType::Unplayed, i) {
                combined.push(track.clone())
            }

            if let Some(track) = self.get_track(profile_key, SectionType::LeastPlayed, i) {
                combined.push(track.clone())
            }

            if let Some(track) = self.get_track(profile_key, SectionType::Oldest, i) {
                combined.push(track.clone())
            }
        }

        combined
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

async fn refresh_playlists_from_profiles(profiles: HashMap<ProfileKey, &Profile>, ran_once: bool) -> Result<i32> {
    let manager = PROFILE_MANAGER.get().read().await;

    if ran_once && !manager.get_any_profile_refresh() {
        return Ok(0);
    }

    let mut set = JoinSet::new();
    for (key, profile) in profiles {
        info!("Building `{}` playlist...", profile.get_title());
        set.spawn(build_playlist(ProfileAction::Update, key, None));
    }

    let mut refreshed = 0;
    while let Some(res) = set.join_next().await {
        let res = res?;
        if let Err(err) = res {
            error!("An error occurred while attempt to refresh profiles: {err}")
        } else {
            refreshed += 1;
        }
    }

    Ok(refreshed)
}

async fn build_playlist(action: ProfileAction, profile_key: ProfileKey, limit: Option<i32>) -> Result<()> {
    {
        info!("Fetching tracks for section(s)...");
        let mut manager = PROFILE_MANAGER.get().write().await;
        manager.fetch_sections_tracks(profile_key, limit).await?;
    }

    let manager = PROFILE_MANAGER.get().read().await;
    let combined = manager.combine_sections(profile_key).await;
    let items = combined
        .iter()
        .map(|track| track.id())
        .collect::<Vec<_>>();

    let plex_client = plex::get_plex_client().await;

    match action {
        ProfileAction::Create => {
            let save = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Would you like to save this profile?")
                .default(true)
                .interact()?;

            if save {
                info!("Creating playlist in plex...");
                let playlist_id = plex_client.create_playlist(profile_key).await?;
                let playlist_id = PlexId::try_new(playlist_id)?;
                {
                    let mut manager = PROFILE_MANAGER.get().write().await;
                    manager.set_playlist_id(profile_key, &playlist_id);
                }

                info!("Adding tracks to newly created playlist...");
                plex_client
                    .add_items_to_playlist(&playlist_id, &items)
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
            let manager = PROFILE_MANAGER.get().read().await;
            let profile = manager.get_profile_by_key(profile_key).unwrap();

            info!("Wiping destination playlist...");
            plex_client.clear_playlist(&profile.get_playlist_id()).await?;

            info!("Updating destination playlist...");
            plex_client
                .add_items_to_playlist(&profile.get_playlist_id(), &items)
                .await?;

            let summary = format!("{}\n{}", profile.get_next_refresh_str(), profile.get_summary());
            plex_client
                .update_summary(&profile.get_playlist_id(), &summary)
                .await?;
        }
        // Other actions are not relevant to this function and are ignored
        _ => {}
    }

    Ok(())
}