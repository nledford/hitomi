use std::env;

pub enum CurrentScreen {
    Main
}

pub struct App {
    title: String,
    pub current_screen: CurrentScreen,
}

impl Default for App {
    fn default() -> Self {
        Self {
            title: format!("Hitomi v{}", get_app_version()),
            current_screen: CurrentScreen::Main
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn get_title(&self) -> &str {
        &self.title
    }
}

fn get_app_version() -> String {
    env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string())
}