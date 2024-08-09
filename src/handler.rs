use crate::app::{App, AppResult, CurrentScreen, MenuOptions};
use crossterm::event::{KeyCode, KeyEvent};
use strum::EnumCount;

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match app.current_screen {
        CurrentScreen::Main => match key_event.code {
            // Exit application on `ESC` or `q`
            KeyCode::Esc | KeyCode::Char('q') => {
                app.quit();
            }
            KeyCode::Up => {
                if app.selected_option == 0 {
                    app.selected_option = MenuOptions::COUNT - 1;
                } else {
                    app.selected_option -= 1;
                }
            }
            KeyCode::Down => {
                if app.selected_option == MenuOptions::COUNT - 1 {
                    app.selected_option = 0;
                } else {
                    app.selected_option += 1;
                }
            }
            KeyCode::Enter => {
                let selected = app.get_main_menu_selected_option();

                app.current_screen = match selected {
                    MenuOptions::RefreshProfiles => CurrentScreen::Run(false),
                    MenuOptions::RefreshLoop => CurrentScreen::Run(true),
                    _ => todo!(),
                }
            }
            _ => {}
        },
        CurrentScreen::Run(run_loop) => {
            todo!("Run view not implemented")
        }
    }

    Ok(())
}
