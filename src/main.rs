use hitomi::app::{App, AppResult};
use hitomi::event::{Event, EventHandler};
use hitomi::handler::handle_key_events;
use hitomi::tui::Tui;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;
use std::io;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Create an application.
    let mut app = App::new().await?;

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}

/*fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app).unwrap())?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }

                match app.current_screen {
                    CurrentScreen::Main => match key.code {
                        KeyCode::Char('q') => {
                            break;
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
                                MenuOptions::CreateProfile => todo!(),
                                MenuOptions::EditProfile => todo!()
                            }
                        }
                        _ => {}
                    },
                    CurrentScreen::Run(run_loop) => {
                        if run_loop && key.code == KeyCode::Esc {
                            app.current_screen = CurrentScreen::Main
                        }
                    }
                }
            }
        }
    }

    Ok(())
}*/
