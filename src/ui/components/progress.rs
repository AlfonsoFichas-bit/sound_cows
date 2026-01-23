use std::time::Duration;
use ratatui::{
    style::Style,
    widgets::{Block, Borders, Gauge},
};
use crate::ui::theme::{PIPBOY_BG, PIPBOY_DARK, PIPBOY_GREEN};

fn format_time(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let minutes = seconds / 60;
    let seconds = seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

pub fn render(app: &crate::app::state::App) -> Gauge {
    let mut ratio = 0.0;
    let mut label = String::from("00:00 / 00:00");

    if let (Some(start), Some(total)) = (app.player.start_time, app.player.total_duration) {
        let elapsed = start.elapsed();
        let total_secs = total.as_secs_f64();
        if total_secs > 0.0 {
            ratio = (elapsed.as_secs_f64() / total_secs).min(1.0);
        }
        label = format!("{} / {}", format_time(elapsed), format_time(total));
    }

    Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .title("PROGRESS")
            .border_style(Style::default().fg(PIPBOY_GREEN))
            .style(Style::default().bg(PIPBOY_BG)))
        .gauge_style(Style::default().fg(PIPBOY_GREEN).bg(PIPBOY_DARK))
        .ratio(ratio)
        .label(label)
}
