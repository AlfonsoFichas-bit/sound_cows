use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::ui::theme::{PIPBOY_BG, PIPBOY_GREEN};

pub fn render_controls() -> Paragraph<'static> {
    let controls = vec![
        Line::from(Span::styled("   [Shift+Arrows] ZOOM/WIDTH", Style::default().fg(PIPBOY_GREEN))),
        Line::from(Span::styled("   [S] SCATTER  [T] TRIGGER", Style::default().fg(PIPBOY_GREEN))),
        Line::from(Span::styled("   [Space] PAUSE", Style::default().fg(PIPBOY_GREEN))),
    ];

    Paragraph::new(controls)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(PIPBOY_GREEN))
                .style(Style::default().bg(PIPBOY_BG))
                .title("SCOPE CTRL"),
        )
}
