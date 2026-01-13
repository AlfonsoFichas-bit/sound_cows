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
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::{error::Error, io};

struct App {
    current_tab: usize,
    radio_state: ListState,
    radio_stations: Vec<String>,
}

impl App {
    fn new() -> App {
        let mut radio_state = ListState::default();
        radio_state.select(Some(3)); // Radio Freedom selected by default
        
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

    // Waveform display (ASCII art)
    let waveform = vec![
        "    ╱╲    ╱╲    ╱╲",
        "   ╱  ╲  ╱  ╲  ╱  ╲",
        "  ╱    ╲╱    ╲╱    ╲",
        " ╱                  ╲",
        "╱                    ╲",
    ];

    let waveform_text: Vec<Line> = waveform
        .iter()
        .map(|line| Line::from(Span::styled(*line, Style::default().fg(pipboy_green))))
        .collect();

    let waveform_widget = Paragraph::new(waveform_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(pipboy_green))
                .style(Style::default().bg(pipboy_bg)),
        )
        .alignment(Alignment::Center);

    f.render_widget(waveform_widget, right_chunks[0]);

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
    let footer_text = Line::from(vec![
        Span::styled("[Enter] ", Style::default().fg(pipboy_green).add_modifier(Modifier::BOLD)),
        Span::styled("TURN OFF RADIO  ", Style::default().fg(Color::Yellow)),
        Span::styled("[T] ", Style::default().fg(pipboy_green).add_modifier(Modifier::BOLD)),
        Span::styled("PERK CHART  ", Style::default().fg(Color::Yellow)),
        Span::styled("[Q] ", Style::default().fg(pipboy_green).add_modifier(Modifier::BOLD)),
        Span::styled("QUIT", Style::default().fg(Color::Yellow)),
    ]);

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