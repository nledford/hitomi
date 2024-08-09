use crate::db;
use crate::profiles::manager::ProfileManager;
use anyhow::Result;
use std::{env, error};
use strum::{Display, EnumCount, FromRepr, VariantArray};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(PartialEq)]
pub enum CurrentScreen {
    Main,
    Run(bool),
}

#[derive(Default, Display, EnumCount, FromRepr, PartialEq, VariantArray)]
pub enum MenuOptions {
    #[default]
    #[strum(to_string = "Refresh Profiles Once")]
    RefreshProfiles,
    #[strum(to_string = "Refresh Profiles In Loop")]
    RefreshLoop,
    #[strum(to_string = "Create Profile")]
    CreateProfile,
    #[strum(to_string = "Edit Profile")]
    EditProfile,
}

pub struct App {
    pub running: bool,
    title: String,
    profile_manager: ProfileManager,

    pub current_screen: CurrentScreen,

    // Main Menu
    pub selected_option: usize,
    // current_profile: Option<Profile>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            title: format!("Hitomi v{}", get_app_version()),
            profile_manager: ProfileManager::default(),

            current_screen: CurrentScreen::Main,

            selected_option: 0,
            // current_profile: None,
        }
    }
}

impl App {
    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    // pub fn increment_counter(&mut self) {
    //     if let Some(res) = self.counter.checked_add(1) {
    //         self.counter = res;
    //     }
    // }
    //
    // pub fn decrement_counter(&mut self) {
    //     if let Some(res) = self.counter.checked_sub(1) {
    //         self.counter = res;
    //     }
    // }
}

impl App {
    pub async fn new() -> Result<Self> {
        db::initialize_pool(None).await?;
        let profile_manager = ProfileManager::new().await?;

        let app = Self {
            profile_manager,
            ..Default::default()
        };

        Ok(app)
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn get_profile_manager(&self) -> &ProfileManager {
        &self.profile_manager
    }

    pub fn get_main_menu_selected_option(&self) -> MenuOptions {
        MenuOptions::from_repr(self.selected_option).unwrap()
    }
}

fn get_app_version() -> String {
    env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string())
}
