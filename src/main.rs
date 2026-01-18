use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Chart, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::{error::Error, fs::File, io::{self, BufReader}, time::Instant};
use rodio::{Decoder, OutputStream, Sink, Source};

mod scope;
use scope::{display::{oscilloscope::Oscilloscope, DisplayMode, GraphConfig}, Matrix};

struct App {
    current_tab: usize,
    radio_state: ListState,
    radio_stations: Vec<String>,
    // Audio and Oscilloscope
    oscilloscope: Oscilloscope,
    graph_config: GraphConfig,
    audio_data: Matrix<f64>,
    sample_rate: u32,
    channels: usize,
    start_time: Option<Instant>,
    // Keep stream and sink alive
    _stream: Option<OutputStream>,
    _stream_handle: Option<rodio::OutputStreamHandle>,
    sink: Option<Sink>,
    // Error handling
    error_message: Option<String>,
}

impl App {
    fn new() -> App {
        let mut radio_state = ListState::default();
        radio_state.select(Some(3)); // Radio Freedom selected by default

        let mut audio_data = vec![vec![0.0; 1024]; 2];
        let mut sample_rate = 44100;
        let mut channels = 2;
        let mut start_time = None;
        let mut _stream = None;
        let mut _stream_handle = None;
        let mut sink = None;
        let mut error_message = None;

        // Try to initialize audio
        match OutputStream::try_default() {
            Ok((stream, stream_handle)) => {
                match Sink::try_new(&stream_handle) {
                    Ok(s) => {
                        _stream = Some(stream);
                        _stream_handle = Some(stream_handle);
                        sink = Some(s);
                    },
                    Err(e) => error_message = Some(format!("Sink error: {}", e)),
                }
            },
            Err(e) => error_message = Some(format!("Audio init error: {}", e)),
        }

        // Try to load file if audio initialized
        if let Some(sink_ref) = &sink {
            match File::open("audio.mp3") {
                Ok(file) => {
                    match Decoder::new(BufReader::new(file)) {
                        Ok(source) => {
                             sample_rate = source.sample_rate();
                             channels = source.channels() as usize;

                             let samples: Vec<f32> = source.convert_samples().collect();

                             // Re-open for playing
                             if let Ok(file_play) = File::open("audio.mp3") {
                                 if let Ok(source_play) = Decoder::new(BufReader::new(file_play)) {
                                     sink_ref.append(source_play);
                                     sink_ref.play();
                                     start_time = Some(Instant::now());
                                 }
                             }

                             audio_data = vec![Vec::new(); channels];
                             for (i, sample) in samples.iter().enumerate() {
                                 audio_data[i % channels].push(*sample as f64);
                             }
                        },
                        Err(e) => error_message = Some(format!("Format error: {}", e)),
                    }
                },
                Err(_) => {
                    // It's okay if file doesn't exist, just don't play
                    // But maybe user wants to know?
                    // error_message = Some("audio.mp3 not found".to_string());
                }
            }
        }

        let graph_config = GraphConfig {
            samples: 200, // Window size
            sampling_rate: sample_rate,
            scale: 1.0,
            width: 200,
            show_ui: false,
            labels_color: Color::Rgb(51, 255, 51),
            axis_color: Color::DarkGray,
            palette: vec![Color::Rgb(51, 255, 51), Color::Cyan],
            ..Default::default()
        };

        App {
            current_tab: 4, // RADIO tab
            radio_state,
            radio_stations: vec![
                "Classical Radio".to_string(),
                "Diamond City Radio".to_string(),
                "Nuka-Cola Family Radio".to_string(),
                "Radio Freedom".to_string(),
                "Distress Signal".to_string(),
                "Distress Signal".to_string(),
                "Distress Signal".to_string(),
                "Emergency Frequency RJ1138".to_string(),
                "Military Frequency AF95".to_string(),
                "Silver Shroud Radio".to_string(),
            ],
            oscilloscope: Oscilloscope::default(),
            graph_config,
            audio_data,
            sample_rate,
            channels,
            start_time,
            _stream,
            _stream_handle,
            sink,
            error_message,
        }
    }

    fn next_station(&mut self) {
        let i = match self.radio_state.selected() {
            Some(i) => {
                if i >= self.radio_stations.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.radio_state.select(Some(i));
    }

    fn previous_station(&mut self) {
        let i = match self.radio_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.radio_stations.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.radio_state.select(Some(i));
    }

    fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % 5;
    }

    fn previous_tab(&mut self) {
        if self.current_tab == 0 {
            self.current_tab = 4;
        } else {
            self.current_tab -= 1;
        }
    }

    fn get_audio_window(&self) -> Matrix<f64> {
        if let Some(start_time) = self.start_time {
            let elapsed_seconds = start_time.elapsed().as_secs_f64();
            let start_sample = (elapsed_seconds * self.sample_rate as f64) as usize;
            let end_sample = start_sample + self.graph_config.samples as usize;

            let mut window = vec![Vec::new(); self.channels];
            for ch in 0..self.channels {
                if start_sample < self.audio_data[ch].len() {
                    let end = std::cmp::min(end_sample, self.audio_data[ch].len());
                    window[ch] = self.audio_data[ch][start_sample..end].to_vec();
                    // Pad if necessary
                    if window[ch].len() < self.graph_config.samples as usize {
                         window[ch].resize(self.graph_config.samples as usize, 0.0);
                    }
                } else {
                    window[ch] = vec![0.0; self.graph_config.samples as usize];
                }
            }
            window
        } else {
             vec![vec![0.0; self.graph_config.samples as usize]; self.channels]
        }
    }
}

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
        terminal.draw(|f| ui(f, &mut app)).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Draw error: {}", e)))?;

        // Poll for events with timeout to allow refreshing
        if event::poll(std::time::Duration::from_millis(16))? { // ~60 FPS
            if let Event::Key(key) = event::read().map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Event error: {}", e)))? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => app.next_station(),
                    KeyCode::Up => app.previous_station(),
                    KeyCode::Left => app.previous_tab(),
                    KeyCode::Right => app.next_tab(),
                    KeyCode::Tab => app.next_tab(),
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let pipboy_green = Color::Rgb(51, 255, 51);
    let pipboy_dark = Color::Rgb(0, 20, 0);
    let pipboy_bg = Color::Rgb(0, 10, 0);

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header with tabs
            Constraint::Min(0),     // Content area
            Constraint::Length(3),  // Footer
        ])
        .split(f.area());

    // Header with tabs
    let tabs = vec!["STAT", "INV", "DATA", "MAP", "RADIO"];
    let tab_spans: Vec<Span> = tabs
        .iter()
        .enumerate()
        .flat_map(|(i, t)| {
            let style = if i == app.current_tab {
                Style::default()
                    .fg(pipboy_dark)
                    .bg(pipboy_green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(pipboy_green)
            };
            vec![
                Span::raw("  "),
                Span::styled(format!("{}", t), style),
                Span::raw("  "),
            ]
        })
        .collect();

    let header = Paragraph::new(Line::from(tab_spans))
        .style(Style::default().bg(pipboy_bg))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(pipboy_green))
                .style(Style::default().bg(pipboy_bg)),
        );
    f.render_widget(header, chunks[0]);

    // Content area
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(65),  // Left panel (radio list)
            Constraint::Percentage(35),  // Right panel (waveform + controls)
        ])
        .split(chunks[1]);

    // Radio stations list
    let items: Vec<ListItem> = app
        .radio_stations
        .iter()
        .map(|station| {
            ListItem::new(station.clone())
                .style(Style::default().fg(pipboy_green))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(pipboy_green))
                .style(Style::default().bg(pipboy_bg)),
        )
        .highlight_style(
            Style::default()
                .bg(pipboy_green)
                .fg(pipboy_dark)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▮ ");

    f.render_stateful_widget(list, content_chunks[0], &mut app.radio_state);

    // Right panel
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),  // Waveform
            Constraint::Percentage(25),  // RADS meter
            Constraint::Percentage(25),  // Controls
        ])
        .split(content_chunks[1]);

    // Waveform display
    let data = app.get_audio_window();
    let datasets = app.oscilloscope.process(&app.graph_config, &data);

    // Convert scope::display::DataSet to ratatui::widgets::Dataset
    let ratatui_datasets: Vec<ratatui::widgets::Dataset> = datasets
        .iter()
        .map(|ds| ds.into())
        .collect();

    let chart = Chart::new(ratatui_datasets)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(pipboy_green))
                .style(Style::default().bg(pipboy_bg)),
        )
        .x_axis(app.oscilloscope.axis(&app.graph_config, scope::display::Dimension::X))
        .y_axis(app.oscilloscope.axis(&app.graph_config, scope::display::Dimension::Y));

    f.render_widget(chart, right_chunks[0]);

    // RADS meter
    let rads_display = vec![
        Line::from(Span::styled("    RADS", Style::default().fg(pipboy_green))),
        Line::from(Span::styled("   ┌───┐", Style::default().fg(pipboy_green))),
        Line::from(vec![
            Span::styled("   │", Style::default().fg(pipboy_green)),
            Span::styled(" ▓ ", Style::default().fg(Color::Yellow)),
            Span::styled("│", Style::default().fg(pipboy_green)),
        ]),
        Line::from(Span::styled("   └───┘", Style::default().fg(pipboy_green))),
    ];

    let rads_widget = Paragraph::new(rads_display)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(pipboy_green))
                .style(Style::default().bg(pipboy_bg)),
        );

    f.render_widget(rads_widget, right_chunks[1]);

    // Controls display
    let controls = vec![
        Line::from(Span::styled("    ● POWER", Style::default().fg(Color::Red))),
        Line::from(Span::styled("    ◐ TUNE", Style::default().fg(pipboy_green))),
    ];

    let controls_widget = Paragraph::new(controls)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(pipboy_green))
                .style(Style::default().bg(pipboy_bg)),
        );

    f.render_widget(controls_widget, right_chunks[2]);

    // Footer with instructions
    let mut footer_spans = vec![
        Span::styled("[Enter] ", Style::default().fg(pipboy_green).add_modifier(Modifier::BOLD)),
        Span::styled("TURN OFF RADIO  ", Style::default().fg(Color::Yellow)),
        Span::styled("[T] ", Style::default().fg(pipboy_green).add_modifier(Modifier::BOLD)),
        Span::styled("PERK CHART  ", Style::default().fg(Color::Yellow)),
        Span::styled("[Q] ", Style::default().fg(pipboy_green).add_modifier(Modifier::BOLD)),
        Span::styled("QUIT", Style::default().fg(Color::Yellow)),
    ];

    if let Some(err) = &app.error_message {
         footer_spans.push(Span::styled(format!("  ERROR: {}", err), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));
    }

    let footer_text = Line::from(footer_spans);

    let footer = Paragraph::new(footer_text)
        .style(Style::default().bg(pipboy_bg))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(pipboy_green))
                .style(Style::default().bg(pipboy_bg)),
        );

    f.render_widget(footer, chunks[2]);
}
