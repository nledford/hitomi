use std::io::stdout;
use anyhow::Result;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::crossterm::{event, execute};
use ratatui::crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::Terminal;
use hitomi::app::{App, CurrentScreen};
use hitomi::ui::ui;
// use hitomi::logger;

#[tokio::main]
async fn main() -> Result<()> {
    // logger::initialize_logger()?;

    // Setup terminal
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();
    run_app(&mut terminal, &mut app)?;


    // Dismantle terminal
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {

    loop {
        terminal.draw(|f| ui(f, app))?;

        // For now, `q` or `Q` exits the application
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                continue;
            }
            
            match app.current_screen {
                CurrentScreen::Main => match key.code {
                    KeyCode::Char('q') => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}