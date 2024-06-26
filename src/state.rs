//! Represents the application state
//!
//! Loads the application configuration file and profiles from disk, as well as building a
//! [`PlexClient`](crate::plex::PlexClient) and loading playlists from the Plex server.

use anyhow::Result;
use derive_builder::Builder;

use crate::config;
use crate::config::Config;
use crate::plex::models::Playlist;
use crate::plex::PlexClient;
use crate::profiles::profile::Profile;

/// Represents the application state
#[derive(Builder, Clone, Debug, Default)]
pub struct AppState {
    /// The application's configuration file
    config: Config,
    /// A wrapper for the Plex API
    plex_client: PlexClient,
    /// [`playlist`](crate::plex::models::Playlist)s fetched from Plex
    playlists: Vec<Playlist>,
    /// [`profile`](crate::profiles::profile::Profile)s loaded from disk
    profiles: Vec<Profile>,
}

impl AppState {
    /// Initializes the application state by loading a configuration file from disk (or creating one
    /// if it does not exist) and loading existing profiles, if any, from the disk.
    /// A ['PlexClient'](crate::plex::PlexClient) is then created, which is used to load playlists
    /// from the Plex server.
    pub async fn initialize() -> Result<Self> {
        let config = config::load_config().await?;

        let dir = config.get_profiles_directory();
        let profiles = Profile::load_profiles(dir).await?;

        let plex_client = PlexClient::initialize(&config).await?;
        let playlists = plex_client.get_playlists().to_vec();

        Ok(AppStateBuilder::default()
            .config(config)
            .plex_client(plex_client)
            .profiles(profiles)
            .playlists(playlists)
            .build()?)
    }
}

// Playlists
impl AppState {
    /// Searches for a [`Playlist`](crate::plex::models::Playlist) by its title from the
    /// application state
    pub fn get_playlist_by_title(&self, title: &str) -> Option<&Playlist> {
        self.playlists.iter().find(|p| p.get_title() == title)
    }
}

// Plex
impl AppState {
    /// Returns a reference to the [`PlexClient`](crate::plex::PlexClient) from the application state
    pub fn get_plex_client(&self) -> &PlexClient {
        &self.plex_client
    }
}

// Profiles
impl AppState {
    /// Returns a `vec` of enabled profiles loaded in the application state
    pub fn get_enabled_profiles(&self) -> Vec<Profile> {
        self.profiles
            .iter()
            .filter_map(move |p| {
                if p.get_enabled() {
                    Some(p.to_owned())
                } else {
                    None
                }
            })
            .collect::<Vec<Profile>>()
    }

    /// Returns a `vec` of titles from all profiles loaded in the application state
    pub fn get_profile_titles(&self) -> Vec<&str> {
        self.profiles
            .iter()
            .map(|p| p.get_title())
            .collect::<Vec<&str>>()
    }

    /// Searches for a specific [`profile`](crate::profiles::profile::Profile) by its title.
    /// Returns `None` if no profile can be found.
    pub fn get_profile_by_title(&self, title: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.get_title() == title)
    }

    /// Returns the directory where ['profile'](crate::profiles::profile::Profile)s are stored on disk.
    pub fn get_profiles_directory(&self) -> &str {
        self.config.get_profiles_directory()
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
}
