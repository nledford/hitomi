use itertools::Itertools;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;
use strum::VariantArray;
use crate::app::{App, CurrentScreen, MenuOptions};

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

    build_header(f, app, chunks[0]);

    match app.current_screen {
        CurrentScreen::Main => build_home_screen(f, app, chunks[1]),
        CurrentScreen::Run(run_loop) => todo!()
    }

    build_footer(f, app, chunks[2]);
}

/// Constructs the header always displayed at the top of the TUI application
fn build_header(f: &mut Frame, app: &App, area: Rect) {
    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let title = Paragraph::new(Text::styled(
        app.get_title(),
        Style::default().fg(Color::Green),
    ))
        .block(title_block);
    f.render_widget(title, area);
}

/// Constructs the footer always displayed at the bottom of the TUI application
fn build_footer(f: &mut Frame, app: &App, area: Rect) {
    let current_navigation_text = match app.current_screen {
        CurrentScreen::Main => {
            Span::styled("Home", Style::default().fg(Color::Green))
        }
        CurrentScreen::Run(run_loop) => {
            todo!()
        }
    };
    let mode_footer = Paragraph::new(Line::from(current_navigation_text))
        .block(Block::default().borders(Borders::ALL));

    let current_keys_hint = match app.current_screen {
        CurrentScreen::Main => {
            Span::styled("(q) to quit", Style::default().fg(Color::Red))
        }
        CurrentScreen::Run(run_loop) => {
            todo!()
        }
    };
    let key_notes_footer = Paragraph::new(Line::from(current_keys_hint))
        .block(Block::default().borders(Borders::ALL));

    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    f.render_widget(mode_footer, footer_chunks[0]);
    f.render_widget(key_notes_footer, footer_chunks[1]);
}

fn build_home_screen(f: &mut Frame, app: &App, area: Rect) {
    let list_items = MenuOptions::VARIANTS
        .iter()
        .map(|menu_item| {
            let style = if MenuOptions::from_repr(app.selected_option).unwrap() == *menu_item {
               Style::default().bg(Color::White).fg(Color::Black)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(Span::styled(menu_item.to_string(), style)))
        })
        .collect_vec();
    let list = List::new(list_items);

    let centered = centered_rect(50, 100, area);
    f.render_widget(list, centered)
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