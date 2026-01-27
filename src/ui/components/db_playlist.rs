use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Modifier},
    text::{Span, Line},
    layout::Position,
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear},
    Frame,
};
use crate::app::state::{App, InputMode};
use crate::ui::theme::{PIPBOY_GREEN, COLOR_YELLOW as PIPBOY_YELLOW};

pub fn draw_playlists(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(area);

    // --- Left Panel: Playlists List ---
    let playlists: Vec<ListItem> = app
        .playlists
        .iter()
        .map(|p| {
            ListItem::new(Line::from(vec![
                Span::raw(format!("> {}", p.name)),
            ]))
        })
        .collect();

    let playlist_block = Block::default()
        .borders(Borders::ALL)
        .title(" Playlists (P to create) ")
        .border_style(Style::default().fg(PIPBOY_GREEN));

    let playlist_list = List::new(playlists)
        .block(playlist_block)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    f.render_stateful_widget(playlist_list, chunks[0], &mut app.playlist_state);

    // --- Right Panel: Songs in Playlist ---
    let song_block = Block::default()
        .borders(Borders::ALL)
        .title(if let Some(id) = app.viewing_playlist_id {
             if let Some(p) = app.playlists.iter().find(|p| p.id == id) {
                 format!(" Songs in '{}' (Enter to play) ", p.name)
             } else {
                 " Songs ".to_string()
             }
        } else {
            " Songs ".to_string()
        })
        .border_style(Style::default().fg(PIPBOY_GREEN));

    let songs: Vec<ListItem> = app
        .playlist_songs
        .iter()
        .map(|s| {
            ListItem::new(Line::from(vec![
                Span::raw(format!("{}. {}", s.position, s.title)),
            ]))
        })
        .collect();

    let song_list = List::new(songs)
        .block(song_block)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    f.render_stateful_widget(song_list, chunks[1], &mut app.playlist_songs_state);


    // --- Creation Modal (if active) ---
    if let InputMode::PlaylistNameInput = app.input_mode {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Create Playlist ")
            .style(Style::default().fg(PIPBOY_YELLOW));

        let area = centered_rect(60, 20, f.area());
        f.render_widget(Clear, area); // Clear background

        let input = Paragraph::new(app.playlist_input_name.as_str())
            .style(Style::default().fg(PIPBOY_YELLOW))
            .block(block);

        f.render_widget(input, area);

        // Cursor
        f.set_cursor_position(Position::new(
            area.x + app.playlist_input_name.len() as u16 + 1,
            area.y + 1,
        ));
    }

    // --- Select Playlist Modal (Add Song) ---
    if let InputMode::SelectPlaylistToAdd = app.input_mode {
        if let Some((title, _)) = &app.song_to_add {
            let block = Block::default()
                .borders(Borders::ALL)
                .title(format!(" Add '{}' to... ", title))
                .style(Style::default().fg(PIPBOY_YELLOW));

            let area = centered_rect(60, 40, f.area());
            f.render_widget(Clear, area);
            f.render_widget(block.clone(), area);

            let inner_area = block.inner(area);

            let playlists: Vec<ListItem> = app
                .playlists
                .iter()
                .map(|p| {
                    ListItem::new(Line::from(vec![
                        Span::raw(format!("> {}", p.name)),
                    ]))
                })
                .collect();

            let list = List::new(playlists)
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, inner_area, &mut app.playlist_state);
        }
    }
}

// Helper for centering the modal
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1]);

    layout[1]
}
