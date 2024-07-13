use std::cmp::PartialEq;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use chrono::Local;
use dialoguer::Confirm;
use dialoguer::theme::ColorfulTheme;
use itertools::Itertools;
use simplelog::{error, info};
use slotmap::{new_key_type, SecondaryMap, SlotMap};
use tokio::sync::{OnceCell, RwLock};
use tokio::time::sleep;
use uuid::Uuid;

use crate::{files, plex};
use crate::plex::models::tracks::Track;
use crate::plex::types::PlexId;
use crate::profiles::{ProfileAction, ProfileSource};
use crate::profiles::profile::Profile;
use crate::profiles::profile_section::ProfileSection;
use crate::profiles::types::ProfileSourceId;

pub static PROFILE_MANAGER: OnceCell<Arc<RwLock<ProfileManager>>> = OnceCell::const_new();

pub async fn initialize_profile_manager(profiles_directory: &str) -> Result<()> {
    let manager = ProfileManager::new(profiles_directory).await?;
    PROFILE_MANAGER.set(Arc::new(RwLock::new(manager)))?;
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

    pub fn add_new_profile(&mut self, new_profile: &Profile) -> ProfileKey {
        self.managed_profiles.insert(new_profile.clone())
    }

    pub fn remove_new_profile(&mut self, new_profile_key: ProfileKey) {
        let _ = self.managed_profiles.remove(new_profile_key);
    }

    fn set_playlist_id(&mut self, profile_key: ProfileKey, id: &PlexId) {
        let profile = self.managed_profiles.get_mut(profile_key).unwrap();
        profile.set_playlist_id(id.clone())
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
            .sorted_unstable_by_key(|(_, v)| v.get_title().to_owned())
            .collect::<HashMap<ProfileKey, &Profile>>()
    }

    pub fn get_enabled_profiles(&self) -> HashMap<ProfileKey, &Profile> {
        self.get_profiles()
            .into_iter()
            .filter_map(|(k, v)| if v.get_enabled() { Some((k, v)) } else { None })
            .collect::<HashMap<ProfileKey, &Profile>>()
    }

    pub fn get_profiles_to_refresh(&self, ran_once: bool) -> HashMap<ProfileKey, &Profile> {
        if ran_once && !self.get_any_profile_refresh() {
            return HashMap::default();
        }

        // If the application has run once, we DO NOT want to override refreshing profiles
        self.get_enabled_profiles()
            .into_iter()
            .filter(|(_, v)| v.check_for_refresh(!ran_once))
            .collect::<HashMap<ProfileKey, &Profile>>()
    }

    pub fn get_profile_titles(&self) -> Vec<String> {
        self.get_profiles()
            .iter()
            .map(|(_, v)| v.get_title().to_string())
            .collect::<Vec<_>>()
    }

    pub fn get_profile_by_key(&self, profile_key: ProfileKey) -> Option<&Profile> {
        let profile = self
            .managed_profiles
            .iter()
            .find(|(k, _)| *k == profile_key);

        if let Some((_, v)) = profile {
            Some(v)
        } else {
            None
        }
    }

    pub fn get_profile_by_id(&self, id: Uuid) -> Option<&Profile> {
        self.get_profiles().into_iter().find_map(|(_, v)| {
            if v.get_profile_id() == id {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn get_profile_by_title(&self, title: &str) -> Option<&Profile> {
        self.get_profiles().into_iter().find_map(|(_, v)| {
            if v.get_title() == title {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn get_profile_key(&self, title: &str) -> Option<ProfileKey> {
        let profile = self
            .managed_profiles
            .iter()
            .find(|(_, v)| v.get_title() == title);
        if let Some((k, _)) = profile {
            Some(k)
        } else {
            None
        }
    }

    fn get_profile_time_limit(&self, profile_key: ProfileKey) -> Option<f64> {
        if let Some(profile) = self.managed_profiles.get(profile_key) {
            Some(profile.get_section_time_limit())
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

    pub fn get_any_profile_refresh(&self) -> bool {
        self.get_enabled_profiles()
            .iter()
            .any(|(_, v)| v.check_for_refresh(false))
    }

    fn print_update(&self, playlists_updated: usize) {
        info!(
            "Updated {playlists_updated} at {}",
            Local::now().format("%F %T")
        );

        let str = self
            .get_enabled_profiles()
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

    pub async fn run_refreshes(&self, run_loop: bool) -> Result<()> {
        self.refresh_playlists_from_profiles(run_loop, false).await?;

        if run_loop {
            loop {
                sleep(Duration::from_secs(1)).await;

                self.refresh_playlists_from_profiles(run_loop, true).await?;
            }
        }

        Ok(())
    }

    pub async fn refresh_playlists_from_profiles(
        &self,
        run_loop: bool,
        ran_once: bool,
    ) -> Result<()> {
        if ran_once && !self.get_any_profile_refresh() {
            return Ok(());
        }

        let profiles = self.get_profiles_to_refresh(ran_once);
        let tasks = profiles
            .into_iter()
            .map(|(key, _)| {
                self.update_playlist(key, None)
            })
            .collect::<Vec<_>>();
        let refreshed = tasks.len();

        match futures::future::try_join_all(tasks).await {
            Ok(_) => {
                if run_loop {
                    self.print_update(refreshed)
                }
            }
            Err(err) => {
                error!("An error occurred while attempting to refresh playlists: {err}")
            }
        }

        Ok(())
    }

    pub async fn create_playlist(&mut self,
                                 profile: &Profile,
                                 profile_key: ProfileKey,
                                 tracks: FetchSectionTracksResult) -> Result<()> {
        let plex_client = plex::get_plex_client().await;
        let save = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Would you like to save this profile?")
            .default(true)
            .interact()?;

        if save {
            info!("Creating playlist in plex...");
            let playlist_id = plex_client.create_playlist(profile).await?;
            let playlist_id = PlexId::try_new(playlist_id)?;
            self.set_playlist_id(profile_key, &playlist_id);

            info!("Adding tracks to newly created playlist...");
            plex_client
                .add_items_to_playlist(&playlist_id, &tracks.get_track_ids())
                .await?;

            show_results(&tracks.combined, profile.get_title(), ProfileAction::Create);
        } else {
            info!("Playlist not saved");
        }


        Ok(())
    }

    pub async fn preview_playlist(&self, profile: &Profile) -> Result<()> {
        let profile_key = self.get_profile_key(profile.get_title()).unwrap();
        let tracks = self.fetch_sections_tracks(profile_key, None).await?;
        tracks.print_preview();

        Ok(())
    }

    pub async fn update_playlist(&self, profile_key: ProfileKey, limit: Option<i32>) -> Result<()> {
        let plex_client = plex::get_plex_client().await;
        let profile = self.get_profile_by_key(profile_key).unwrap();
        let tracks = self.fetch_profile_tracks(profile_key, limit).await?;
        info!("Updating `{}` playlist...", profile.get_title());

        info!("Wiping destination playlist...");
        plex_client
            .clear_playlist(&profile.get_playlist_id())
            .await?;

        info!("Updating destination playlist...");
        plex_client
            .add_items_to_playlist(&profile.get_playlist_id(), &tracks.get_track_ids())
            .await?;

        let summary = format!(
            "{}\n{}",
            profile.get_next_refresh_str(),
            profile.get_summary()
        );
        plex_client
            .update_summary(&profile.get_playlist_id(), &summary)
            .await?;

        show_results(&tracks.combined, profile.get_title(), ProfileAction::Update);

        Ok(())
    }

    pub async fn fetch_profile_tracks(&self, profile_key: ProfileKey, limit: Option<i32>) -> Result<FetchSectionTracksResult> {
        let mut tracks = self.fetch_sections_tracks(profile_key, limit).await?;
        let time_limit = self.get_profile_time_limit(profile_key).unwrap();

        if let Some(section) = self.managed_unplayed_sections.get(profile_key) {
            section.run_manual_filters(&mut tracks.unplayed, time_limit, None);
        }

        if let Some(section) = self.managed_least_played_sections.get(profile_key) {
            section.run_manual_filters(&mut tracks.least_played, time_limit, None);
        }

        if let Some(section) = self.managed_oldest_sections.get(profile_key) {
            section.run_manual_filters(&mut tracks.oldest, time_limit, None);
        }

        tracks.combine();

        Ok(tracks)
    }

    async fn fetch_sections_tracks(
        &self,
        profile_key: ProfileKey,
        limit: Option<i32>,
    ) -> Result<FetchSectionTracksResult> {
        let unplayed = self.managed_unplayed_sections.get(profile_key);
        let least_played = self.managed_least_played_sections.get(profile_key);
        let oldest = self.managed_oldest_sections.get(profile_key);

        let (source, source_id) = self
            .get_profile_by_key(profile_key)
            .unwrap()
            .get_profile_source_and_id();

        let tracks =
            fetch_sections_tracks(source, source_id, unplayed, least_played, oldest, limit).await?;

        Ok(tracks)
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

async fn fetch_sections_tracks(
    profile_source: &ProfileSource,
    profile_source_id: Option<&ProfileSourceId>,
    unplayed: Option<&ProfileSection>,
    least_played: Option<&ProfileSection>,
    oldest: Option<&ProfileSection>,
    limit: Option<i32>,
) -> Result<FetchSectionTracksResult> {
    let mut result = FetchSectionTracksResult::default();

    if let Some(section) = unplayed {
        result.unplayed =
            fetch_section_tracks(section, profile_source, profile_source_id, limit).await?;
    }

    if let Some(section) = least_played {
        result.least_played =
            fetch_section_tracks(section, profile_source, profile_source_id, limit).await?;
    }

    if let Some(section) = oldest {
        result.oldest =
            fetch_section_tracks(section, profile_source, profile_source_id, limit).await?;
    }

    Ok(result)
}

async fn fetch_section_tracks(
    section: &ProfileSection,
    profile_source: &ProfileSource,
    profile_source_id: Option<&ProfileSourceId>,
    limit: Option<i32>,
) -> Result<Vec<Track>> {
    let mut tracks = vec![];

    if !section.is_enabled() {
        return Ok(tracks);
    }
    let plex_client = plex::get_plex_client().await;

    let mut filters = HashMap::new();
    if section.get_minimum_track_rating() != 0 {
        filters.insert(
            "userRating>>".to_string(),
            section.get_minimum_track_rating().to_string(),
        );
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
            let collection = plex_client
                .fetch_collection(profile_source_id.unwrap())
                .await?;
            let artists = plex_client
                .fetch_artists_from_collection(&collection)
                .await?;
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

    tracks = plex_client
        .fetch_music(filters, section.get_sorting(), limit)
        .await?;

    Ok(tracks)
}

#[derive(Debug, Default)]
pub struct FetchSectionTracksResult {
    unplayed: Vec<Track>,
    least_played: Vec<Track>,
    oldest: Vec<Track>,
    combined: Vec<Track>,
}

impl FetchSectionTracksResult {
    fn are_none_valid(&self) -> bool {
        self.get_num_valid() <= 0
    }

    fn get_num_valid(&self) -> usize {
        vec![
            self.unplayed.is_empty(),
            self.least_played.is_empty(),
            self.oldest.is_empty(),
        ]
            .iter()
            .filter(|x| **x == false)
            .count()
    }

    fn get_largest_section_length(&self) -> usize {
        *vec![
            self.unplayed.len(),
            self.least_played.len(),
            self.oldest.len(),
        ]
            .iter()
            .max()
            .unwrap_or(&0_usize)
    }

    fn get_track_ids(&self) -> Vec<String> {
        if self.combined.is_empty() {
            vec![]
        } else {
            self.combined
                .iter()
                .map(|track| track.id().to_string())
                .collect::<Vec<_>>()
        }
    }
    fn print_preview(&self) {
        if self.combined.is_empty() {
            return;
        }

        let preview = self.combined
            .iter()
            .take(25)
            .collect::<Vec<_>>();

        for (i, track) in preview.iter().enumerate() {
            println!("{:2} {}", i + 1, track)
        }
    }

    fn combine(&mut self) {
        if self.are_none_valid() {
            return;
        }
        info!("Combining playlists...");

        self.combined = Vec::new();

        info!("Combing {} sections...", self.get_num_valid());

        for i in 0..self.get_largest_section_length() {
            if let Some(track) = self.unplayed.get(i) {
                self.combined.push(track.clone())
            }

            if let Some(track) = self.least_played.get(i) {
                self.combined.push(track.clone())
            }

            if let Some(track) = self.oldest.get(i) {
                self.combined.push(track.clone())
            }
        }
    }
}
