use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::Chart,
    Frame,
};
use crate::app::state::App;
use crate::scope::display::{DisplayMode, Dimension};
use crate::ui::theme::{PIPBOY_BG, PIPBOY_GREEN};
use ratatui::widgets::{Block, Borders};
use ratatui::style::Style;

use super::components;

pub fn draw(f: &mut Frame, app: &mut App) {
    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header with tabs
            Constraint::Min(0),     // Content area
            Constraint::Length(3),  // Footer
        ])
        .split(f.area());

    // Header
    f.render_widget(components::header::render(app), chunks[0]);

    // Content area
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(65),  // Left panel (radio list)
            Constraint::Percentage(35),  // Right panel (waveform + controls)
        ])
        .split(chunks[1]);

    // Playlist
    let playlist_widget = components::playlist::render(&app.radio_stations);
    f.render_stateful_widget(
        playlist_widget,
        content_chunks[0],
        &mut app.radio_state
    );

    // Right panel
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),  // Waveform
            Constraint::Percentage(25),  // Progress
            Constraint::Percentage(25),  // Controls
        ])
        .split(content_chunks[1]);

    // Oscilloscope (Inline generation because of borrow checker issues with Chart data)
    let window_size = app.graph_config.samples as usize;
    let data = app.player.get_window(window_size);
    let datasets_data = app.oscilloscope.process(&app.graph_config, &data);

    let ratatui_datasets: Vec<ratatui::widgets::Dataset> = datasets_data
        .iter()
        .map(|ds| ds.into())
        .collect();

    let chart = Chart::new(ratatui_datasets)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(PIPBOY_GREEN))
                .style(Style::default().bg(PIPBOY_BG)),
        )
        .x_axis(app.oscilloscope.axis(&app.graph_config, Dimension::X))
        .y_axis(app.oscilloscope.axis(&app.graph_config, Dimension::Y));

    f.render_widget(chart, right_chunks[0]);

    // Progress Bar
    f.render_widget(components::progress::render(app), right_chunks[1]);

    // Controls
    f.render_widget(components::scope_view::render_controls(), right_chunks[2]);

    // Footer
    f.render_widget(components::footer::render(app), chunks[2]);
}
