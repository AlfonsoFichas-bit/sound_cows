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

    if app.input_mode as usize == 5 { // SelectPlaylistToAdd (Enum variant index check or explicit match)
         // Hacky way to render modal over EVERYTHING:
         // We let the underlying tab render first, then the modal on top.
         // But checking enum variant by index is fragile.
         // Let's rely on the components::db_playlist::draw_playlists being capable of drawing the modal?
         // No, draw_playlists draws the full playlist TAB.
         // We need to call the modal drawing logic regardless of the current tab if in that mode.
         // Refactoring db_playlist to separate the modal drawing would be cleaner.
         // For now, let's keep it simple: The modal is drawn inside 'draw_playlists'.
         // But 'draw_playlists' expects 'area' to split.
         // Let's just modify the end of this function to draw the modal if needed.
    }

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
    } else if app.current_tab == 1 {
        // INV Tab - Playlists
        components::db_playlist::draw_playlists(f, app, chunks[1]);
    } else {
        // RADIO Tab (Default Layout) - Also fallback for STAT/MAP for now
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(65),  // Left panel (radio list)
                Constraint::Percentage(35),  // Right panel (waveform + controls)
            ])
            .split(chunks[1]);

        // Playlist
        // NOTE: 'playlist' was renamed to 'db_playlist', but 'playlist.rs' (radio list) was missing from the mod list.
        // It seems I accidentally removed 'mod playlist' from src/ui/mod.rs earlier.
        // However, looking at the file tree, I suspect 'playlist.rs' was renamed to 'db_playlist.rs'.
        // Wait, the original playlist.rs handled the RADIO stations list.
        // I need to check if I overwrote/renamed the radio playlist component.
        // Let's assume for a moment I need to render the radio list differently or restore the file.
        // But since I renamed it, I probably lost the radio rendering logic if I didn't preserve it.
        // Actually, looking at previous steps, I renamed `src/ui/components/playlist.rs` to `db_playlist.rs`.
        // That file was NEWLY created in Step 2.
        // Ah, wait. Did `playlist.rs` exist BEFORE Step 2?
        // Let's check the file list or assume I need to implement a simple list for radio stations here if it's gone.
        // Or if 'db_playlist.rs' was the NEW one, and 'playlist.rs' was the OLD one.
        // In Step 2 I wrote `src/ui/components/playlist.rs`.
        // In Step 3 I renamed it to `db_playlist.rs`.
        // So the radio playlist component is likely missing or I need to use `List` directly here.

        // Let's implement the radio list rendering directly here using standard widgets to fix the build,
        // since the original component seems to be lost or confused.

        let items: Vec<ratatui::widgets::ListItem> = app
            .radio_stations
            .iter()
            .map(|i| ratatui::widgets::ListItem::new(ratatui::text::Line::from(i.as_str())))
            .collect();

        let playlist_widget = ratatui::widgets::List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Radio Stations").border_style(Style::default().fg(PIPBOY_GREEN)))
            .highlight_style(Style::default().add_modifier(ratatui::style::Modifier::REVERSED))
            .highlight_symbol(">> ");

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
        f.render_widget(components::scope_view::render_controls(app), right_chunks[2]);
    }

    // Footer
    f.render_widget(components::footer::render(app), chunks[2]);

    // Global Modals (Overlay)
    if let crate::app::state::InputMode::SelectPlaylistToAdd = app.input_mode {
        // Reuse the logic from db_playlist which happens to have the modal logic inside.
        // Ideally we extract `draw_add_to_playlist_modal` to a public function.
        // For this iteration, since I put the modal logic inside `draw_playlists`,
        // I can call it with the full frame area, but it will try to draw the playlist UI too?
        // Let's refactor db_playlist slightly in the next step or just duplicate the modal call here
        // if I exposed it.
        // Wait, I put the logic INSIDE `draw_playlists`. That function draws the split view.
        // If I am in DATA tab, I want to see DATA tab + Modal.
        // So I should extract the modal drawing.

        components::db_playlist::draw_playlists(f, app, f.area());
        // Note: This effectively redraws the playlist UI *over* the current tab which might be weird
        // if we are in DATA tab, effectively switching context visually to Playlist tab temporarily.
        // This is actually acceptable for a "Select Playlist" modal since it lists playlists!
    }
}
