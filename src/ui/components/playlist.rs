use ratatui::{
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
};
use crate::ui::theme::{PIPBOY_BG, PIPBOY_DARK, PIPBOY_GREEN};

pub fn render(radio_stations: &[String]) -> List<'_> {
    let items: Vec<ListItem> = radio_stations
        .iter()
        .map(|station| {
            ListItem::new(station.clone())
                .style(Style::default().fg(PIPBOY_GREEN))
        })
        .collect();

    List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(PIPBOY_GREEN))
                .style(Style::default().bg(PIPBOY_BG)),
        )
        .highlight_style(
            Style::default()
                .bg(PIPBOY_GREEN)
                .fg(PIPBOY_DARK)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▮ ")
}
