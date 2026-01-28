use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{error::Error, io, path::Path};

mod app;
mod audio;
mod scope;
mod ui;

use app::state::{App, InputMode, AppEvent};
use scope::display::{update_value_f, update_value_i, DisplayMode};
use audio::player::AudioPlayer;

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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<(), Box<dyn Error>>
where <B as Backend>::Error: 'static {
    loop {
        terminal.draw(|f| ui::layout::draw(f, &mut app)).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Draw error: {}", e)))?;

        // Check for async events non-blockingly
        if let Ok(event) = app.event_rx.try_recv() {
            match event {
                AppEvent::AudioLoaded(path) => {
                    app.is_loading = false;
                    app.player.play_file(Path::new(&path));
                    app.loading_status = Some("Playing...".to_string());
                    app.current_tab = 4; // Switch to Radio
                },
                AppEvent::AudioError(e) => {
                    app.is_loading = false;
                    app.loading_status = Some(format!("Error: {}", e));
                },
                AppEvent::SearchFinished(results) => {
                    app.is_loading = false;
                    app.search_results = results;
                    app.loading_status = Some(format!("Found {} results", app.search_results.len()));
                    if !app.search_results.is_empty() {
                        app.search_results_state.select(Some(0));
                        app.input_mode = InputMode::SearchResults;
                    } else {
                        app.input_mode = InputMode::Normal;
                    }
                },
                AppEvent::SearchError(e) => {
                    app.is_loading = false;
                    app.loading_status = Some(format!("Search Error: {}", e));
                    app.input_mode = InputMode::Normal;
                }
            }
        }

        if event::poll(std::time::Duration::from_millis(16))? {
            let event = event::read().map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Event error: {}", e)))?;

            if app.current_tab == 4 {
                app.oscilloscope.handle(event.clone());
            }

            if let Event::Key(key) = event {
                // Global Scope Controls
                let magnitude = match key.modifiers {
                    KeyModifiers::SHIFT => 10.0,
                    KeyModifiers::CONTROL => 5.0,
                    KeyModifiers::ALT => 0.2,
                    _ => 1.0,
                };

                match app.input_mode {
                    InputMode::Normal => {
                        match key.code {
                            KeyCode::Char('/') if app.current_tab == 2 => {
                                app.input_mode = InputMode::Editing;
                            }
                            KeyCode::Char('q') => return Ok(()),

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
                            KeyCode::Char('s') if app.current_tab == 4 => app.graph_config.scatter = !app.graph_config.scatter,
                            KeyCode::Char(' ') if app.current_tab == 4 => {
                                app.graph_config.pause = !app.graph_config.pause;
                                app.player.toggle_pause();
                            },
                            KeyCode::Char('+') => app.player.volume_up(),
                            KeyCode::Char('-') => app.player.volume_down(),

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

                                if query.starts_with("http://") || query.starts_with("https://") {
                                    // Direct URL handling - Async
                                    app.loading_status = Some(format!("Downloading URL: {}...", query));
                                    app.is_loading = true;

                                    // Need to pass the sender to the static function.
                                    // app.player.load_source_async needs to be static or we clone sender
                                    let tx = app.event_tx.clone();
                                    AudioPlayer::load_source_async(query, tx);

                                    app.search_input.clear();
                                    app.reset_cursor();
                                    app.input_mode = InputMode::Normal;

                                } else {
                                    // Search Query handling - Async
                                    app.loading_status = Some(format!("Searching: {}...", query));
                                    app.is_loading = true;

                                    let tx = app.event_tx.clone();
                                    AudioPlayer::search_async(query, tx);

                                    app.search_input.clear();
                                    app.reset_cursor();
                                }
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
                    },
                    InputMode::SearchResults => {
                        match key.code {
                            KeyCode::Down => app.next_search_result(),
                            KeyCode::Up => app.previous_search_result(),
                            KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                                app.search_results.clear();
                            },
                            KeyCode::Enter => {
                                let selected_track = if let Some(selected_idx) = app.search_results_state.selected() {
                                    app.search_results.get(selected_idx).cloned()
                                } else {
                                    None
                                };

                                if let Some(song) = selected_track {
                                    app.loading_status = Some(format!("Downloading: {}...", song.title));
                                    app.is_loading = true;

                                    app.now_playing = Some(song.clone());

                                    let tx = app.event_tx.clone();
                                    AudioPlayer::load_source_async(song.url, tx);

                                    app.input_mode = InputMode::Normal;
                                }
                            },
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}
