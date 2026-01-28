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

    if app.current_tab == 2 {
        // DATA Tab - Search Interface
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search Input
                Constraint::Min(0),    // Results List
            ])
            .split(chunks[1]);

        f.render_widget(components::search::render_input(app), content_chunks[0]);

        // Render results list statefully - Passing fields instead of full app to fix borrow error
        let results_widget = components::search::render_results(&app.search_results, &app.input_mode);
        f.render_stateful_widget(
            results_widget,
            content_chunks[1],
            &mut app.search_results_state
        );

    } else {
        // RADIO Tab (Default Layout)
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(65),  // Left panel (Now Playing + Queue)
                Constraint::Percentage(35),  // Right panel (waveform + controls)
            ])
            .split(chunks[1]);

        // Left Panel Split (Now Playing / Queue)
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30), // Now Playing
                Constraint::Percentage(70), // Queue
            ])
            .split(content_chunks[0]);

        // --- Now Playing ---
        let now_playing_text = if let Some(song) = &app.now_playing {
            vec![
                ratatui::text::Line::from(ratatui::text::Span::styled(format!("Title: {}", song.title), Style::default().add_modifier(ratatui::style::Modifier::BOLD).fg(crate::ui::theme::COLOR_YELLOW))),
                ratatui::text::Line::from(format!("Artist: {}", song.artist)),
                ratatui::text::Line::from(format!("Album: {}", song.album)),
                ratatui::text::Line::from(format!("Duration: {}", song.duration_str)),
            ]
        } else {
            vec![ratatui::text::Line::from("No song playing.")]
        };

        let now_playing_widget = ratatui::widgets::Paragraph::new(now_playing_text)
            .block(Block::default().borders(Borders::ALL).title(" En Reproducción ").border_style(Style::default().fg(PIPBOY_GREEN)))
            .style(Style::default().fg(PIPBOY_GREEN));

        f.render_widget(now_playing_widget, left_chunks[0]);

        // --- Queue (A Continuación) ---
        let items: Vec<ratatui::widgets::ListItem> = app
            .queue
            .iter()
            .map(|song| {
                ratatui::widgets::ListItem::new(ratatui::text::Line::from(vec![
                    ratatui::text::Span::raw(format!("{} - ", song.title)),
                    ratatui::text::Span::styled(&song.artist, Style::default().fg(crate::ui::theme::COLOR_YELLOW)),
                ]))
            })
            .collect();

        let queue_widget = ratatui::widgets::List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" A Continuación ").border_style(Style::default().fg(PIPBOY_GREEN)))
            .highlight_style(Style::default().add_modifier(ratatui::style::Modifier::REVERSED))
            .highlight_symbol(">> ");

        f.render_stateful_widget(
            queue_widget,
            left_chunks[1],
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
        f.render_widget(components::scope_view::render_controls(app), right_chunks[2]);
    }

    // Footer
    f.render_widget(components::footer::render(app), chunks[2]);
}
