use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::ui::theme::{PIPBOY_BG, PIPBOY_GREEN, COLOR_YELLOW};

pub fn render_controls(app: &crate::app::state::App) -> Paragraph<'static> {
    let vol_percent = (app.player.volume * 100.0) as u32;
    let mut controls = vec![
        Line::from(Span::styled("   [Shift+Arrows] ZOOM/WIDTH", Style::default().fg(PIPBOY_GREEN))),
        Line::from(Span::styled("   [S] SCATTER  [T] TRIGGER", Style::default().fg(PIPBOY_GREEN))),
        Line::from(Span::styled(format!("   [Space] PAUSE  [+/-] VOL: {}%", vol_percent), Style::default().fg(PIPBOY_GREEN))),
    ];

    if app.player.is_streaming_mode {
        controls.insert(0, Line::from(Span::styled("   [!] OPTIMIZED MODE (NO SCOPE)", Style::default().fg(COLOR_YELLOW))));
    }

    Paragraph::new(controls)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(PIPBOY_GREEN))
                .style(Style::default().bg(PIPBOY_BG))
                .title("SCOPE CTRL"),
        )
}
