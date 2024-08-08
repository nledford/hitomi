use std::env;
use crate::db;
use crate::profiles::profile::Profile;
use anyhow::Result;
use strum::{Display, EnumCount, FromRepr, VariantArray};

pub enum CurrentScreen {
    Main,
    Run(bool)
}

#[derive(Default, Display, EnumCount, FromRepr, PartialEq, VariantArray)]
pub enum MenuOptions {
    #[default]
    Run,
    #[strum(to_string = "Run Loop")]
    RunLoop,
    #[strum(to_string = "Create Profile")]
    CreateProfile,
    #[strum(to_string = "Edit Profile")]
    EditProfile,
}

pub struct App {
    title: String,
    pub current_screen: CurrentScreen,
    
    // Main Menu
    pub selected_option: usize,
    
    profiles: Vec<Profile>,
    current_profile: Option<Profile>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            title: format!("Hitomi v{}", get_app_version()),
            current_screen: CurrentScreen::Main,
            
            selected_option: 0,
            
            profiles: Vec::default(),
            current_profile: None,
        }
    }
}

impl App {
    pub async fn new() -> Result<Self> {
        // Self::default()
        let profiles = db::profiles::fetch_profiles(false).await?;
        
        let app = Self {
            profiles,
            ..Default::default()
        };
        
        Ok(app)
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }
    
    pub fn get_profiles(&self) -> &[Profile] {
        &self.profiles
    }
}

fn get_app_version() -> String {
    env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string())
}
