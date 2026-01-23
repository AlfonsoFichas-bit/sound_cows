use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::ui::theme::{COLOR_RED, COLOR_YELLOW, PIPBOY_BG, PIPBOY_GREEN};

pub fn render(app: &crate::app::state::App) -> Paragraph {
    let mut footer_spans = vec![
        Span::styled("[Enter] ", Style::default().fg(PIPBOY_GREEN).add_modifier(Modifier::BOLD)),
        Span::styled("TURN OFF  ", Style::default().fg(COLOR_YELLOW)),
        Span::styled("[T] ", Style::default().fg(PIPBOY_GREEN).add_modifier(Modifier::BOLD)),
        Span::styled("PERK  ", Style::default().fg(COLOR_YELLOW)),
        Span::styled("[Q] ", Style::default().fg(PIPBOY_GREEN).add_modifier(Modifier::BOLD)),
        Span::styled("QUIT", Style::default().fg(COLOR_YELLOW)),
    ];

    if let Some(err) = &app.player.error_message {
         footer_spans.push(Span::styled(format!("  ERROR: {}", err), Style::default().fg(COLOR_RED).add_modifier(Modifier::BOLD)));
    }

    Paragraph::new(Line::from(footer_spans))
        .style(Style::default().bg(PIPBOY_BG))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(PIPBOY_GREEN))
                .style(Style::default().bg(PIPBOY_BG)),
        )
}
