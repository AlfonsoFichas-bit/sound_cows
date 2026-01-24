use ratatui::{style::Color, widgets::ListState};
use crate::audio::player::AudioPlayer;
use crate::scope::display::{oscilloscope::Oscilloscope, GraphConfig};
use crate::ui::theme::{PIPBOY_GREEN, COLOR_RED};

pub enum InputMode {
    Normal,
    Editing,
}

pub struct App {
    pub current_tab: usize,
    pub radio_state: ListState,
    pub radio_stations: Vec<String>,

    // Components
    pub player: AudioPlayer,
    pub oscilloscope: Oscilloscope,
    pub graph_config: GraphConfig,

    // Search State
    pub input_mode: InputMode,
    pub search_input: String,
    pub cursor_position: usize,
    pub loading_status: Option<String>,
}

impl App {
    pub fn new() -> App {
        let mut radio_state = ListState::default();
        radio_state.select(Some(3)); // Radio Freedom

        let mut player = AudioPlayer::new();
        player.load_source("audio.mp3"); // Load local by default

        let graph_config = GraphConfig {
            samples: 200,
            sampling_rate: player.sample_rate,
            scale: 1.0,
            width: 200,
            show_ui: false,
            labels_color: PIPBOY_GREEN,
            axis_color: Color::DarkGray,
            palette: vec![PIPBOY_GREEN, COLOR_RED],
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
            input_mode: InputMode::Normal,
            search_input: String::new(),
            cursor_position: 0,
            loading_status: None,
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

    // Input Handling Helper Methods
    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        self.search_input.insert(self.cursor_position, new_char);
        self.move_cursor_right();
    }

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.search_input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.search_input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By retrieving the content without the deleted character,
            // we can reset the search input with this new content.
            self.search_input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    pub fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.search_input.chars().count())
    }

    pub fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }
}
