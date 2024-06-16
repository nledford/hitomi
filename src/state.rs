use std::sync::Arc;

use anyhow::Result;
use default_struct_builder::DefaultBuilder;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;

use crate::config::Config;
use crate::plex::models::Playlist;
use crate::plex::Plex;
use crate::profiles::profile::Profile;

/// Global application state
pub static APP_STATE: Lazy<Arc<Mutex<AppState>>> =
    Lazy::new(|| Arc::new(Mutex::new(AppState::default())));

/// Represents the application state
#[derive(Debug, Default, DefaultBuilder)]
pub struct AppState {
    config: Config,
    plex: Plex,
    profiles: Vec<Profile>,
}

impl AppState {
    pub async fn initialize() -> Result<Self> {
        let config = Config::load_config(None).await?;

        let dir = config.get_profiles_directory();
        let profiles = Profile::load_profiles(dir).await?;

        let plex = Plex::initialize(&config).await?;

        Ok(Self::default().config(config).profiles(profiles).plex(plex))
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }

    pub fn get_playlists(&self) -> Vec<Playlist> {
        self.plex.get_playlists()
    }

    pub fn get_playlist(&self, title: &str) -> Option<Playlist> {
        let playlists = self.get_playlists();
        playlists.iter().find(|p| p.title == title).cloned()
    }

    pub fn get_plex(&self) -> &Plex {
        &self.plex
    }

    pub fn get_profiles(&self) -> Vec<Profile> {
        self.profiles.clone()
    }

    pub fn get_profile(&self, title: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.get_title() == title)
    }

    pub fn get_profiles_directory(&self) -> &str {
        self.config.get_profiles_directory()
    }

    pub fn num_profiles(&self) -> usize {
        self.profiles.len()
    }

    pub fn add_profile(&mut self, profile: Profile) {
        self.profiles.push(profile);
    }

    pub fn list_profiles(&self) {
        let titles = self
            .profiles
            .iter()
            .map(|p| p.get_title())
            .collect::<Vec<&str>>();

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
