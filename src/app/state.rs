use ratatui::{style::Color, widgets::ListState};
use crate::audio::player::AudioPlayer;
use crate::scope::display::{oscilloscope::Oscilloscope, GraphConfig};

pub struct App {
    pub current_tab: usize,
    pub radio_state: ListState,
    pub radio_stations: Vec<String>,

    // Components
    pub player: AudioPlayer,
    pub oscilloscope: Oscilloscope,
    pub graph_config: GraphConfig,
}

impl App {
    pub fn new() -> App {
        let mut radio_state = ListState::default();
        radio_state.select(Some(3)); // Radio Freedom

        let player = AudioPlayer::new();

        let graph_config = GraphConfig {
            samples: 200,
            sampling_rate: player.sample_rate,
            scale: 1.0,
            width: 200,
            show_ui: false,
            labels_color: Color::Rgb(51, 255, 51), // We will use theme later, but config needs Color
            axis_color: Color::DarkGray,
            palette: vec![Color::Rgb(51, 255, 51), Color::Cyan],
            ..Default::default()
        };

        App {
            current_tab: 4, // RADIO tab
            radio_state,
            radio_stations: vec![
                "Classical Radio".to_string(),
                "Diamond City Radio".to_string(),
                "Nuka-Cola Family Radio".to_string(),
                "Radio Freedom".to_string(),
                "Distress Signal".to_string(),
                "Distress Signal".to_string(),
                "Distress Signal".to_string(),
                "Emergency Frequency RJ1138".to_string(),
                "Military Frequency AF95".to_string(),
                "Silver Shroud Radio".to_string(),
            ],
            player,
            oscilloscope: Oscilloscope::default(),
            graph_config,
        }
    }

    pub fn next_station(&mut self) {
        let i = match self.radio_state.selected() {
            Some(i) => {
                if i >= self.radio_stations.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.radio_state.select(Some(i));
    }

    pub fn previous_station(&mut self) {
        let i = match self.radio_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.radio_stations.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.radio_state.select(Some(i));
    }

    pub fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % 5;
    }

    pub fn previous_tab(&mut self) {
        if self.current_tab == 0 {
            self.current_tab = 4;
        } else {
            self.current_tab -= 1;
        }
    }
}
