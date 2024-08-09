use crate::app::{App, CurrentScreen};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Line, Span, Style, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Constructs the header always displayed at the top of the TUI application
pub fn build_header(f: &mut Frame, app: &App, area: Rect) {
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
pub fn build_footer(f: &mut Frame, app: &App, area: Rect) {
    let current_navigation_text = match app.current_screen {
        CurrentScreen::Main => {
            Span::styled("Home", Style::default().fg(Color::Green))
        }
        CurrentScreen::Run(run_loop) => {
            if run_loop {
                Span::styled("Refreshing Profiles In Loop", Style::default().fg(Color::Green))
            } else {
                Span::styled("Refreshing Profiles. Please wait...", Style::default().fg(Color::Green))
            }
        }
    };
    let mode_footer = Paragraph::new(Line::from(current_navigation_text))
        .block(Block::default().borders(Borders::ALL));

    let current_keys_hint = match app.current_screen {
        CurrentScreen::Main => {
            Span::styled("(q) to quit, Up/Down to change option, Enter to select option", Style::default().fg(Color::Red))
        }
        CurrentScreen::Run(run_loop) => {
            if run_loop {
                Span::styled("Esc to return to home screen", Style::default().fg(Color::Red))
            } else {
                Span::styled("", Style::default())
            }
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