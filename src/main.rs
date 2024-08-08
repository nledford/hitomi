use anyhow::Result;
use hitomi::app::{App, CurrentScreen};
use hitomi::ui::ui;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::crossterm::{event, execute};
use ratatui::Terminal;
use std::io::stdout;
use hitomi::db;

#[tokio::main]
async fn main() -> Result<()> {
    // color_eyre::install()?;
    
    db::initialize_pool(None).await?;

    // Setup terminal
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new().await?;
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
                },
            }
        }
    }

    Ok(())
}
