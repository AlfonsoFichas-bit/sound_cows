use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{error::Error, io};

mod app;
mod audio;
mod scope;
mod ui;

use app::state::App;
use scope::display::{update_value_f, update_value_i, DisplayMode}; // Import DisplayMode trait

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| ui::layout::draw(f, &mut app)).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Draw error: {}", e)))?;

        // Poll for events with timeout to allow refreshing
        if event::poll(std::time::Duration::from_millis(16))? { // ~60 FPS
            let event = event::read().map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Event error: {}", e)))?;

            // Pass event to oscilloscope if in Radio tab
            if app.current_tab == 4 {
                app.oscilloscope.handle(event.clone());
            }

            if let Event::Key(key) = event {
                // Global Scope Controls (Shift + Arrows)
                let magnitude = match key.modifiers {
                    KeyModifiers::SHIFT => 10.0,
                    KeyModifiers::CONTROL => 5.0,
                    KeyModifiers::ALT => 0.2,
                    _ => 1.0,
                };

                // Specific controls for Scope that don't conflict or use modifiers
                if app.current_tab == 4 {
                     match key.code {
                        KeyCode::Up if key.modifiers.contains(KeyModifiers::SHIFT) => {
                            update_value_f(&mut app.graph_config.scale, 0.01, magnitude, 0.0..10.0);
                        }
                        KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) => {
                            update_value_f(&mut app.graph_config.scale, -0.01, magnitude, 0.0..10.0);
                        }
                        KeyCode::Right if key.modifiers.contains(KeyModifiers::SHIFT) => {
                            update_value_i(&mut app.graph_config.samples, true, 25, magnitude, 0..app.graph_config.width * 2);
                        }
                        KeyCode::Left if key.modifiers.contains(KeyModifiers::SHIFT) => {
                            update_value_i(&mut app.graph_config.samples, false, 25, magnitude, 0..app.graph_config.width * 2);
                        }
                        // Toggle features
                        KeyCode::Char('s') => app.graph_config.scatter = !app.graph_config.scatter,
                        KeyCode::Char(' ') => app.graph_config.pause = !app.graph_config.pause,
                        _ => {}
                    }
                }

                // Standard Navigation
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down if !key.modifiers.contains(KeyModifiers::SHIFT) => app.next_station(),
                    KeyCode::Up if !key.modifiers.contains(KeyModifiers::SHIFT) => app.previous_station(),
                    KeyCode::Left if !key.modifiers.contains(KeyModifiers::SHIFT) => app.previous_tab(),
                    KeyCode::Right if !key.modifiers.contains(KeyModifiers::SHIFT) => app.next_tab(),
                    KeyCode::Tab => app.next_tab(),
                    _ => {}
                }
            }
        }
    }
}
