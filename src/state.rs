//! Represents the application state
//!
//! Loads the application configuration file and profiles from disk, as well as building a
//! [`PlexClient`] and loading playlists from the Plex server.

use anyhow::{anyhow, Result};
use derive_builder::Builder;
use tokio::sync::{OnceCell, RwLock};

use crate::config::Config;
use crate::plex::models::playlists::Playlist;
use crate::plex::types::PlexId;
use crate::plex::PLEX_CLIENT;
use crate::profiles::manager;
use crate::profiles::manager::ProfileManager;
use crate::types::Title;
use crate::{config, plex};

pub static APP_STATE: OnceCell<RwLock<AppState>> = OnceCell::const_new();

pub async fn initialize_app_state() -> Result<()> {
    let app_state = AppState::initialize().await?;
    APP_STATE.set(RwLock::new(app_state))?;
    Ok(())
}

/// Represents the application state
#[derive(Builder, Debug)]
pub struct AppState {
    /// The application's configuration file
    config: Option<Config>,
    /// [`Playlist`]s fetched from Plex
    playlists: Vec<Playlist>,
    profile_manager: ProfileManager,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: Some(Config::default()),
            playlists: vec![],
            profile_manager: ProfileManager::default(),
        }
    }
}

impl AppState {
    /// Initializes the application state by loading a configuration file from disk (or creating one
    /// if it does not exist) and loading existing profiles, if any, from the disk.
    /// A ['PlexClient'](PlexClient) is then created, which is used to load playlists
    /// from the Plex server.
    pub async fn initialize() -> Result<Self> {
        let config = config::load_config().await?;
        plex::initialize_plex_client(&config).await?;

        let dir = config.get_profiles_directory();
        manager::initialize_profile_manager(dir).await?;
        let manager = ProfileManager::new(dir).await?;

        let playlists = PLEX_CLIENT.get().unwrap().get_playlists().to_vec();

        let state = AppStateBuilder::default()
            .config(Some(config))
            .playlists(playlists)
            .profile_manager(manager)
            .build()?;

        Ok(state)
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
}

// Profiles
impl AppState {
    pub fn get_profile_manager(&self) -> &ProfileManager {
        &self.profile_manager
    }
}
