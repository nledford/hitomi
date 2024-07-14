//! Manages profiles

use std::cmp::PartialEq;
use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
use chrono::{Local, Timelike, Utc};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use itertools::Itertools;
use simplelog::{error, info};
use slotmap::{new_key_type, SecondaryMap, SlotMap};
use tokio::time::sleep;

use crate::config::Config;
use crate::files;
use crate::plex::models::playlists::Playlist;
use crate::plex::models::tracks::Track;
use crate::plex::types::PlexId;
use crate::plex::PlexClient;
use crate::profiles::merger::SectionTracksMerger;
use crate::profiles::profile::Profile;
use crate::profiles::profile_section::ProfileSection;
use crate::profiles::types::ProfileSourceId;
use crate::profiles::{ProfileAction, ProfileSource};

new_key_type! {
    pub struct ProfileKey;
}

#[derive(Clone, Debug, Default)]
pub struct ProfileManager {
    config: Config,
    plex_client: PlexClient,
    playlists: Vec<Playlist>,
    /// Profiles that have been loaded from disk
    profiles: Vec<Profile>,
    /// Profiles being managed by the application
    managed_profiles: SlotMap<ProfileKey, Profile>,
    managed_unplayed_sections: SecondaryMap<ProfileKey, ProfileSection>,
    managed_least_played_sections: SecondaryMap<ProfileKey, ProfileSection>,
    managed_oldest_sections: SecondaryMap<ProfileKey, ProfileSection>,
}

// INITIALIZATION
impl ProfileManager {
    pub async fn new() -> Result<Self> {
        let config = crate::config::load_config().await?;
        let plex_client = PlexClient::initialize(&config).await?;
        let playlists = plex_client.get_playlists().to_vec();
        let profiles = files::load_profiles_from_disk(config.get_profiles_directory()).await?;

        let mut manager = ProfileManager {
            config,
            plex_client,
            playlists,
            profiles,
            ..Default::default()
        };
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
}

// CONFIG
impl ProfileManager {
    pub fn get_config_profiles_directory(&self) -> &str {
        self.config.get_profiles_directory()
    }
}

// PlEX
impl ProfileManager {
    pub fn get_plex_client(&self) -> &PlexClient {
        &self.plex_client
    }
}

// PLAYLISTS
impl ProfileManager {
    pub fn get_playlist_by_title(&self, title: &str) -> Option<&Playlist> {
        self.playlists.iter().find(|p| p.get_title() == title)
    }
}

impl ProfileManager {
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
            .values()
            .map(|profile| profile.get_title().to_string())
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
        self.managed_profiles
            .get(profile_key)
            .map(|profile| profile.get_section_time_limit())
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
        Utc::now().second() == 0
            && self
            .get_enabled_profiles()
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
                for title in v.iter().sorted() {
                    acc += &format!("    - {title}\n");
                }
                acc
            });
        info!("Upcoming refreshes:\n{str}")
    }

    pub async fn run_refreshes(&self, run_loop: bool) -> Result<()> {
        self.refresh_playlists_from_profiles(run_loop, false)
            .await?;

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
            .into_keys()
            .map(|key| self.update_playlist(key, None))
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

    pub async fn create_playlist(
        &mut self,
        profile: &Profile,
        profile_key: ProfileKey,
        merger: &SectionTracksMerger,
    ) -> Result<()> {
        let save = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Would you like to save this profile?")
            .default(true)
            .interact()?;

        if save {
            info!("Creating playlist in plex...");
            let playlist_id = self.plex_client.create_playlist(profile).await?;
            let playlist_id = PlexId::try_new(playlist_id)?;
            self.set_playlist_id(profile_key, &playlist_id);

            info!("Adding tracks to newly created playlist...");
            self.plex_client
                .add_items_to_playlist(&playlist_id, &merger.get_track_ids())
                .await?;

            print_refresh_results(
                merger.get_combined_tracks(),
                profile.get_title(),
                ProfileAction::Create,
            );
        } else {
            info!("Playlist not saved");
        }

        Ok(())
    }

    pub async fn preview_playlist(&self, profile: &Profile) -> Result<()> {
        let profile_key = self.get_profile_key(profile.get_title()).unwrap();
        let merger = self.fetch_sections_tracks(profile_key, None).await?;
        merger.print_preview();

        Ok(())
    }

    pub async fn update_playlist(&self, profile_key: ProfileKey, limit: Option<i32>) -> Result<()> {
        let profile = self.get_profile_by_key(profile_key).unwrap();
        let merger = self.fetch_profile_tracks(profile_key, limit).await?;
        info!("Updating `{}` playlist...", profile.get_title());

        info!("Wiping destination playlist...");
        self.plex_client
            .clear_playlist(profile.get_playlist_id())
            .await?;

        info!("Updating destination playlist...");
        self.plex_client
            .add_items_to_playlist(profile.get_playlist_id(), &merger.get_track_ids())
            .await?;

        let summary = format!(
            "{}\n{}",
            profile.get_next_refresh_str(),
            profile.get_summary()
        );
        self.plex_client
            .update_summary(profile.get_playlist_id(), &summary)
            .await?;

        print_refresh_results(
            merger.get_combined_tracks(),
            profile.get_title(),
            ProfileAction::Update,
        );

        Ok(())
    }

    pub async fn fetch_profile_tracks(
        &self,
        profile_key: ProfileKey,
        limit: Option<i32>,
    ) -> Result<SectionTracksMerger> {
        let mut merger = self.fetch_sections_tracks(profile_key, limit).await?;
        let time_limit = self.get_profile_time_limit(profile_key).unwrap();

        if let Some(section) = self.managed_unplayed_sections.get(profile_key) {
            merger.run_manual_filters(section, time_limit)
        }

        if let Some(section) = self.managed_least_played_sections.get(profile_key) {
            merger.run_manual_filters(section, time_limit)
        }

        if let Some(section) = self.managed_oldest_sections.get(profile_key) {
            merger.run_manual_filters(section, time_limit)
        }

        merger.merge();

        Ok(merger)
    }

    async fn fetch_sections_tracks(
        &self,
        profile_key: ProfileKey,
        limit: Option<i32>,
    ) -> Result<SectionTracksMerger> {
        let unplayed = self.managed_unplayed_sections.get(profile_key);
        let least_played = self.managed_least_played_sections.get(profile_key);
        let oldest = self.managed_oldest_sections.get(profile_key);

        let (source, source_id) = self
            .get_profile_by_key(profile_key)
            .unwrap()
            .get_profile_source_and_id();

        let tracks = fetch_sections_tracks(
            &self.plex_client,
            source,
            source_id,
            unplayed,
            least_played,
            oldest,
            limit,
        )
            .await?;

        Ok(tracks)
    }
}

// UTILITY FUNCTIONS #############################################################

fn print_refresh_results(tracks: &[Track], playlist_title: &str, action: ProfileAction) {
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
        playlist_title,
        size,
        duration
    );
}

async fn fetch_sections_tracks(
    plex_client: &PlexClient,
    profile_source: &ProfileSource,
    profile_source_id: Option<&ProfileSourceId>,
    unplayed: Option<&ProfileSection>,
    least_played: Option<&ProfileSection>,
    oldest: Option<&ProfileSection>,
    limit: Option<i32>,
) -> Result<SectionTracksMerger> {
    let mut result = SectionTracksMerger::default();

    if let Some(section) = unplayed {
        let tracks = fetch_section_tracks(
            plex_client,
            section,
            profile_source,
            profile_source_id,
            limit,
        )
            .await?;
        result.set_unplayed_tracks(tracks);
    }

    if let Some(section) = least_played {
        let tracks = fetch_section_tracks(
            plex_client,
            section,
            profile_source,
            profile_source_id,
            limit,
        )
            .await?;
        result.set_least_played_tracks(tracks)
    }

    if let Some(section) = oldest {
        let tracks = fetch_section_tracks(
            plex_client,
            section,
            profile_source,
            profile_source_id,
            limit,
        )
            .await?;
        result.set_oldest_tracks(tracks)
    }

    Ok(result)
}

async fn fetch_section_tracks(
    plex_client: &PlexClient,
    section: &ProfileSection,
    profile_source: &ProfileSource,
    profile_source_id: Option<&ProfileSourceId>,
    limit: Option<i32>,
) -> Result<Vec<Track>> {
    let mut tracks = vec![];

    if !section.is_enabled() {
        return Ok(tracks);
    }
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
