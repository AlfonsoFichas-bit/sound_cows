use ratatui::{
    layout::Alignment,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::ui::theme::{PIPBOY_BG, PIPBOY_DARK, PIPBOY_GREEN};

pub fn render(app: &crate::app::state::App) -> Paragraph {
    let tabs = vec!["STAT", "INV", "DATA", "MAP", "RADIO"];
    let tab_spans: Vec<Span> = tabs
        .iter()
        .enumerate()
        .flat_map(|(i, t)| {
            let style = if i == app.current_tab {
                Style::default()
                    .fg(PIPBOY_DARK)
                    .bg(PIPBOY_GREEN)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(PIPBOY_GREEN)
            };
            vec![
                Span::raw("  "),
                Span::styled(format!("{}", t), style),
                Span::raw("  "),
            ]
        })
        .collect();

    Paragraph::new(Line::from(tab_spans))
        .style(Style::default().bg(PIPBOY_BG))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(PIPBOY_GREEN))
                .style(Style::default().bg(PIPBOY_BG)),
        )
}
