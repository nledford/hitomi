use anyhow::Result;
use derive_builder::Builder;

use crate::config::Config;
use crate::plex::models::Playlist;
use crate::plex::PlexClient;
use crate::profiles::profile::Profile;

/// Represents the application state
#[derive(Builder, Clone, Debug, Default)]
pub struct AppState {
    config: Config,
    plex: PlexClient,
    playlists: Vec<Playlist>,
    profiles: Vec<Profile>,
}

impl AppState {
    pub async fn initialize() -> Result<Self> {
        let config = Config::load_config().await?;

        let dir = config.get_profiles_directory();
        let profiles = Profile::load_profiles(dir).await?;

        let plex = PlexClient::initialize(&config).await?;
        let playlists = plex.get_playlists().to_vec();

        Ok(AppStateBuilder::default()
            .config(config)
            .plex(plex)
            .profiles(profiles)
            .playlists(playlists)
            .build()?)
    }
}

// Playlists
impl AppState {
    pub fn get_playlist_by_title(&self, title: &str) -> Option<&Playlist> {
        self.playlists.iter().find(|p| p.title == title)
    }
}

// Plex
impl AppState {
    pub fn get_plex(&self) -> &PlexClient {
        &self.plex
    }
}

// Profiles
impl AppState {
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

    pub fn get_profile_titles(&self) -> Vec<&str> {
        self.profiles
            .iter()
            .map(|p| p.get_title())
            .collect::<Vec<&str>>()
    }

    pub fn get_profile(&self, title: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.get_title() == title)
    }

    pub fn get_profiles_directory(&self) -> &str {
        self.config.get_profiles_directory()
    }

    pub fn have_profiles(&self) -> bool {
        !self.profiles.is_empty()
    }

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
