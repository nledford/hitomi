use anyhow::Result;
use chrono::{DateTime, Local, Timelike};
use default_struct_builder::DefaultBuilder;

use crate::config::Config;
use crate::plex::models::Playlist;
use crate::plex::Plex;
use crate::profiles::profile::Profile;

/// Represents the application state
#[derive(Clone, Debug, Default, DefaultBuilder)]
pub struct AppState {
    config: Config,
    current_time: DateTime<Local>,
    plex: Plex,
    playlists: Vec<Playlist>,
    profiles: Vec<Profile>,
}

impl AppState {
    pub async fn initialize() -> Result<Self> {
        let config = Config::load_config(None).await?;

        let dir = config.get_profiles_directory();
        let profiles = Profile::load_profiles(dir).await?;

        let plex = Plex::initialize(&config).await?;
        let playlists = plex.get_playlists().to_vec();

        Ok(
            Self::default()
                .config(config)
                .current_time(Local::now())
                .plex(plex)
                .profiles(profiles)
                .playlists(playlists)
        )
    }
}

// Config
impl AppState {
    pub fn get_config(&self) -> &Config {
        &self.config
    }
}

// Current Time
impl AppState {
    pub fn update_time(&mut self) {
        self.current_time = Local::now()
    }

    pub fn get_current_minute(&self) -> u32 {
        self.current_time.minute()
    }

    pub fn get_current_second(&self) -> u32 {
        self.current_time.second()
    }
}

// Playlists
impl AppState {
    pub fn get_playlists(&self) -> &[Playlist] {
        &self.playlists
    }

    pub fn get_playlist_by_title(&self, title: &str) -> Option<&Playlist> {
        self.playlists.iter().find(|p| p.title == title)
    }
}

// Plex
impl AppState {
    pub fn get_plex(&self) -> &Plex {
        &self.plex
    }
}

// Profiles
impl AppState {
    pub fn get_profiles(&self) -> &[Profile] {
        &self.profiles
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
