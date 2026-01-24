use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::app::state::{App, InputMode};
use crate::ui::theme::{PIPBOY_BG, PIPBOY_GREEN, COLOR_YELLOW};

pub fn render(app: &App) -> Paragraph {
    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("/", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to search audio URL..."),
            ],
            Style::default().fg(PIPBOY_GREEN),
        ),
        InputMode::Editing => (
            vec![
                Span::raw("> "),
                Span::styled(&app.search_input, Style::default().fg(COLOR_YELLOW)),
                Span::styled("█", Style::default().fg(PIPBOY_GREEN).add_modifier(Modifier::SLOW_BLINK)), // Fake cursor
            ],
            Style::default().fg(COLOR_YELLOW),
        ),
    };

    let mut text = vec![Line::from(msg)];

    if let Some(status) = &app.loading_status {
        text.push(Line::from(Span::styled(format!("[STATUS]: {}", status), Style::default().fg(PIPBOY_GREEN))));
    }

    Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("DATA / SEARCH")
                .border_style(style)
                .style(Style::default().bg(PIPBOY_BG)),
        )
}
