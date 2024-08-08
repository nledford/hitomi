use std::env;
use crate::db;
use crate::profiles::profile::Profile;
use anyhow::Result;

pub enum CurrentScreen {
    Main,
}

pub struct App {
    title: String,
    pub current_screen: CurrentScreen,
    profiles: Vec<Profile>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            title: format!("Hitomi v{}", get_app_version()),
            current_screen: CurrentScreen::Main,
            profiles: Vec::default(),
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
