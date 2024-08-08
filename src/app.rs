pub enum CurrentScreen {
    Main
}

pub struct App {
    pub current_screen: CurrentScreen,
}

impl Default for App {
    fn default() -> Self {
        Self {
            current_screen: CurrentScreen::Main
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }
}