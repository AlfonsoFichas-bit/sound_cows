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

use app::state::{App, InputMode};
use scope::display::{update_value_f, update_value_i, DisplayMode};

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

// Added where clause to fix E0310
fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<(), Box<dyn Error>>
where <B as Backend>::Error: 'static {
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

                // Handle Input Mode
                match app.input_mode {
                    InputMode::Normal => {
                        match key.code {
                            KeyCode::Char('/') if app.current_tab == 2 => { // DATA tab is index 2 ("DATA")
                                app.input_mode = InputMode::Editing;
                            }
                            KeyCode::Char('q') => return Ok(()),

                            // Specific controls for Scope that don't conflict or use modifiers
                            KeyCode::Up if key.modifiers.contains(KeyModifiers::SHIFT) && app.current_tab == 4 => {
                                update_value_f(&mut app.graph_config.scale, 0.01, magnitude, 0.0..10.0);
                            }
                            KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) && app.current_tab == 4 => {
                                update_value_f(&mut app.graph_config.scale, -0.01, magnitude, 0.0..10.0);
                            }
                            KeyCode::Right if key.modifiers.contains(KeyModifiers::SHIFT) && app.current_tab == 4 => {
                                update_value_i(&mut app.graph_config.samples, true, 25, magnitude, 0..app.graph_config.width * 2);
                            }
                            KeyCode::Left if key.modifiers.contains(KeyModifiers::SHIFT) && app.current_tab == 4 => {
                                update_value_i(&mut app.graph_config.samples, false, 25, magnitude, 0..app.graph_config.width * 2);
                            }
                            // Toggle features
                            KeyCode::Char('s') if app.current_tab == 4 => app.graph_config.scatter = !app.graph_config.scatter,
                            KeyCode::Char(' ') if app.current_tab == 4 => {
                                app.graph_config.pause = !app.graph_config.pause;
                                app.player.toggle_pause();
                            },
                            KeyCode::Char('+') => app.player.volume_up(),
                            KeyCode::Char('-') => app.player.volume_down(),

                            // Standard Navigation
                            KeyCode::Down if !key.modifiers.contains(KeyModifiers::SHIFT) => app.next_station(),
                            KeyCode::Up if !key.modifiers.contains(KeyModifiers::SHIFT) => app.previous_station(),
                            KeyCode::Left if !key.modifiers.contains(KeyModifiers::SHIFT) => app.previous_tab(),
                            KeyCode::Right if !key.modifiers.contains(KeyModifiers::SHIFT) => app.next_tab(),
                            KeyCode::Tab => app.next_tab(),
                            _ => {}
                        }
                    },
                    InputMode::Editing => {
                        match key.code {
                            KeyCode::Enter => {
                                let query = app.search_input.clone();
                                app.loading_status = Some(format!("Downloading: {}...", query));
                                // In a real app, this should be async or in a thread.
                                // For now it blocks UI, but shows intention.
                                // We need to force a redraw here if we want the user to see "Downloading"
                                // But since this is a single thread loop, it will freeze.
                                // Improvement: Offload download to thread later.
                                terminal.draw(|f| ui::layout::draw(f, &mut app)).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Draw error: {}", e)))?;

                                app.player.load_source(&query);
                                app.loading_status = Some("Ready".to_string());
                                // Clear input
                                app.search_input.clear();
                                app.reset_cursor();
                                app.input_mode = InputMode::Normal;
                                // Auto switch to Radio tab (index 4) to see visualization?
                                app.current_tab = 4;
                            }
                            KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                            }
                            KeyCode::Backspace => {
                                app.delete_char();
                            }
                            KeyCode::Left => {
                                app.move_cursor_left();
                            }
                            KeyCode::Right => {
                                app.move_cursor_right();
                            }
                            KeyCode::Char(to_insert) => {
                                app.enter_char(to_insert);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}
