use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::{App, CurrentScreen};

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
    
    // TODO build bodies

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
    };
    let mode_footer = Paragraph::new(Line::from(current_navigation_text))
        .block(Block::default().borders(Borders::ALL));

    let current_keys_hint = match app.current_screen {
        CurrentScreen::Main => {
            Span::styled("(q) to quit", Style::default().fg(Color::Red))
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
