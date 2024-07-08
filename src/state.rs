//! Represents the application state
//!
//! Loads the application configuration file and profiles from disk, as well as building a
//! [`PlexClient`] and loading playlists from the Plex server.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use chrono::Local;
use derive_builder::Builder;
use itertools::Itertools;
use once_cell::sync::Lazy;
use simplelog::info;
use tokio::sync::RwLock;

use crate::config;
use crate::config::Config;
use crate::plex::models::playlists::Playlist;
use crate::plex::types::PlexId;
use crate::plex::PlexClient;
use crate::profiles::profile::Profile;
use crate::types::Title;

pub static APP_STATE: Lazy<Arc<RwLock<AppState>>> =
    Lazy::new(|| Arc::new(RwLock::new(AppState::default())));

/// Represents the application state
#[derive(Builder, Clone, Debug)]
pub struct AppState {
    /// The application's configuration file
    config: Option<Config>,
    /// A wrapper for the Plex API
    plex_client: Option<PlexClient>,
    /// [`Playlist`]s fetched from Plex
    playlists: Vec<Playlist>,
    /// [`Profile`]s loaded from disk
    profiles: Vec<Profile>,
    refresh_failures: HashMap<String, u32>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: Some(Config::default()),
            plex_client: None,
            playlists: vec![],
            profiles: vec![],
            refresh_failures: HashMap::new(),
        }
    }
}

impl AppState {
    /// Initializes the application state by loading a configuration file from disk (or creating one
    /// if it does not exist) and loading existing profiles, if any, from the disk.
    /// A ['PlexClient'](crate::plex::PlexClient) is then created, which is used to load playlists
    /// from the Plex server.
    pub async fn initialize(&mut self) -> Result<()> {
        let config = config::load_config().await?;

        let dir = config.get_profiles_directory();
        let profiles = Profile::load_profiles(dir).await?;

        let plex_client = PlexClient::initialize(&config).await?;
        let playlists = plex_client.get_playlists().to_vec();
        let refresh_failures = HashMap::new();

        let state = AppStateBuilder::default()
            .config(Some(config))
            .plex_client(Some(plex_client))
            .profiles(profiles)
            .playlists(playlists)
            .refresh_failures(refresh_failures)
            .build()?;

        *self = state;

        Ok(())
    }
}

// Config
impl AppState {
    pub fn have_config(&self) -> bool {
        self.config.is_some()
    }
    pub fn get_config(&self) -> Result<&Config> {
        let config = self
            .config
            .as_ref()
            .ok_or(anyhow!("Configuration file not loaded"))?;
        Ok(config)
    }
}

// Plex
impl AppState {
    /// Returns a reference to the [`PlexClient`] from the application state
    pub fn get_plex_client(&self) -> Result<&PlexClient> {
        let client = self
            .plex_client
            .as_ref()
            .ok_or(anyhow!("Plex client not found"))?;
        Ok(client)
    }
}

// Playlists
impl AppState {
    /// Searches for a [`Playlist`] by its title from the
    /// application state
    pub fn get_playlist_by_title(&self, title: &Title) -> Option<&Playlist> {
        self.playlists
            .iter()
            .find(|p| p.get_title() == title.as_ref())
    }

    pub fn get_playlist_by_id(&self, id: &PlexId) -> Option<&Playlist> {
        self.playlists.iter().find(|p| p.get_id() == id.as_ref())
    }

    pub fn update_refresh_failures(&mut self, id: &PlexId) {
        *self.refresh_failures.entry(id.to_string()).or_default() += 1;
    }
}

// Profiles
impl AppState {
    /// Returns a `vec` of enabled profiles loaded in the application state
    pub fn get_enabled_profiles(&self) -> Vec<Profile> {
        self.profiles
            .iter()
            .sorted_unstable_by_key(|p| p.get_title().to_owned())
            .filter_map(move |p| {
                if p.get_enabled() {
                    Some(p.to_owned())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn get_profiles_to_refresh(&self, ran_once: bool) -> Vec<Profile> {
        if ran_once && !self.any_profile_refresh() {
            return vec![];
        }

        // If the application has run once, we DO NOT want to override refreshing profiles
        self.get_enabled_profiles()
            .into_iter()
            .sorted_unstable_by_key(|p| p.get_title().to_owned())
            .filter(|profile| profile.check_for_refresh(!ran_once))
            .collect::<Vec<_>>()
    }

    /// Returns a `vec` of titles from all profiles loaded in the application state
    pub fn get_profile_titles(&self) -> Vec<&str> {
        let profiles = self
            .profiles
            .iter()
            .sorted_unstable_by_key(|p| p.get_title().to_owned())
            .map(|p| p.get_title())
            .collect::<Vec<_>>();
        profiles
    }

    /// Searches for a specific [`profile`](crate::profiles::profile::Profile) by its title.
    /// Returns `None` if no profile can be found.
    pub fn get_profile_by_title(&self, title: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.get_title() == title)
    }

    /// Returns the directory where ['profile'](crate::profiles::profile::Profile)s are stored on disk.
    pub fn get_profiles_directory(&self) -> Result<&str> {
        Ok(self.get_config()?.get_profiles_directory())
    }

    /// Checks if [`profile`](crate::profiles::profile::Profile)s have been loaded to the application state.
    pub fn have_profiles(&self) -> bool {
        !self.profiles.is_empty()
    }

    /// Prints a list of all [`profile`](crate::profiles::profile::Profile)s loaded from disk
    pub fn list_profiles(&self) {
        let mut titles = self
            .profiles
            .iter()
            .map(|p| p.get_title())
            .collect::<Vec<&str>>();
        titles.sort_unstable();

        if titles.is_empty() {
            println!("No profiles found.")
        } else {
            println!("Existing profiles found");
            for title in titles {
                println!("  - {}", title)
            }
        }
    }

    pub fn any_profile_refresh(&self) -> bool {
        self.get_enabled_profiles()
            .iter()
            .any(|profile| profile.check_for_refresh(false))
    }

    pub fn print_update(&self, playlists_updated: usize) {
        info!(
            "Updated {playlists_updated} playlists at {}",
            Local::now().format("%F %T")
        );

        let str = self
            .get_enabled_profiles()
            .into_iter()
            .fold(
                HashMap::new(),
                |mut acc: HashMap<String, Vec<String>>, profile| {
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
}
