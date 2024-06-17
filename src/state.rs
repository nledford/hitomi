use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
use chrono::{DateTime, Local, Timelike};
use default_struct_builder::DefaultBuilder;
use simplelog::error;
use tokio::time::sleep;

use crate::config::Config;
use crate::plex::models::Playlist;
use crate::plex::Plex;
use crate::profiles::profile::Profile;
use crate::profiles::ProfileAction;

/// Global application state
// pub static APP_STATE: Lazy<Arc<Mutex<AppState>>> =
//     Lazy::new(|| Arc::new(Mutex::new(AppState::default())));

/// Represents the application state
#[derive(Clone, Debug, Default, DefaultBuilder)]
pub struct AppState {
    config: Config,
    current_time: DateTime<Local>,
    plex: Plex,
    playlists: Vec<Playlist>,
    profiles: Vec<Profile>,
    ran_once: bool,
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
                .ran_once(false)
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

    pub async fn update_state(&mut self, run_loop: bool) -> Result<()> {
        self.update_profiles(run_loop).await?;

        if run_loop {
            loop {
                sleep(Duration::from_secs(1)).await;

                if Local::now().second() == 0 {
                    self.update_profiles(run_loop).await?;
                }
            }
        }

        Ok(())
    }

    async fn update_profiles(&mut self, run_loop: bool) -> Result<()> {
        let now = Local::now();
        let app_state = self.clone();

        let mut refresh_failures = HashMap::new();
        for profile in self.profiles.iter_mut() {
            let playlist_id = profile.get_playlist_id().to_string();
            refresh_failures.entry(playlist_id.clone()).or_insert(0);

            if !self.ran_once || now.minute() == profile.get_current_refresh_minute(now) {
                match Profile::build_playlist(profile, &app_state, ProfileAction::Update).await {
                    Ok(_) => {
                        refresh_failures.entry(playlist_id.clone()).and_modify(|v| *v = 0);
                    }
                    Err(err) => {
                        refresh_failures.entry(playlist_id.clone()).and_modify(|v| *v += 1);
                        let failures = refresh_failures.get(&playlist_id.clone()).unwrap();

                        if *failures <= 3 {
                            error!("An error occurred while attempting to build the `{}` playlist: {err}", profile.get_title());
                            error!("Skipping building this playlist. {} build attempt(s) remaining...", 3 - *failures);
                        } else {
                            panic!("Failed to connect to Plex server more than three times.");
                        }
                    }
                }

                if run_loop {
                    profile.print_next_refresh();
                }
            }
        }
        self.ran_once = true;

        Ok(())
    }
}
