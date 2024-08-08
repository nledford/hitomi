mod components;
mod home;

use crate::app::{App, CurrentScreen};
use itertools::Itertools;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;
use strum::VariantArray;

/// Constructs the user interface of the TUI application
pub fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(f.area());

    components::build_header(f, app, chunks[0]);

    match app.current_screen {
        CurrentScreen::Main => home::build_home_screen(f, app, chunks[1]),
        CurrentScreen::Run(run_loop) => todo!()
    }

    components::build_footer(f, app, chunks[2]);
}


/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}