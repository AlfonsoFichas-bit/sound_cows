use ratatui::{style::Color, widgets::ListState};
use crate::audio::player::AudioPlayer;
use crate::scope::display::{oscilloscope::Oscilloscope, GraphConfig};
use crate::ui::theme::{PIPBOY_GREEN, COLOR_RED};
use std::sync::mpsc::{channel, Receiver, Sender};

pub enum InputMode {
    Normal,
    Editing,
    SearchResults,
}

#[derive(Clone, Debug)]
pub struct Song {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub url: String,
    pub duration_str: String,
}

// Events sent from background threads to the main UI thread
pub enum AppEvent {
    AudioLoaded(String), // Path to file
    AudioError(String),
    SearchFinished(Vec<Song>), // Results with full metadata
    SearchError(String),
}

pub struct App {
    pub current_tab: usize,
    pub radio_state: ListState,
    // queue is the new 'radio_stations' list
    pub queue: std::collections::VecDeque<Song>,
    pub now_playing: Option<Song>,

    // Components
    pub player: AudioPlayer,
    pub oscilloscope: Oscilloscope,
    pub graph_config: GraphConfig,

    // Search State
    pub input_mode: InputMode,
    pub search_input: String,
    pub cursor_position: usize,
    pub loading_status: Option<String>,
    pub is_loading: bool, // General loading spinner flag

    // Search Results
    pub search_results: Vec<Song>,
    pub search_results_state: ListState,

    // Async Communication
    pub event_tx: Sender<AppEvent>,
    pub event_rx: Receiver<AppEvent>,
}

impl App {
    pub fn new() -> App {
        let mut radio_state = ListState::default();
        // radio_state.select(Some(0)); // Start at top of queue

        let player = AudioPlayer::new();

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

        let (event_tx, event_rx) = channel();

        App {
            current_tab: 4, // RADIO tab
            radio_state,
            queue: std::collections::VecDeque::new(),
            now_playing: None,
            player,
            oscilloscope: Oscilloscope::default(),
            graph_config,
            input_mode: InputMode::Normal,
            search_input: String::new(),
            cursor_position: 0,
            loading_status: None,
            is_loading: false,
            search_results: Vec::new(),
            search_results_state: ListState::default(),
            event_tx,
            event_rx,
        }
    }

    pub fn next_station(&mut self) {
        if self.queue.is_empty() { return; }
        let i = match self.radio_state.selected() {
            Some(i) => {
                if i >= self.queue.len() - 1 {
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
        if self.queue.is_empty() { return; }
        let i = match self.radio_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.queue.len() - 1
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

            self.search_input = self.search_input.chars().take(from_left_to_current_index).chain(self.search_input.chars().skip(current_index)).collect();
            self.move_cursor_left();
        }
    }

    pub fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.search_input.chars().count())
    }

    pub fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }

    // Search Result Navigation
    pub fn next_search_result(&mut self) {
        if self.search_results.is_empty() { return; }
        let i = match self.search_results_state.selected() {
            Some(i) => {
                if i >= self.search_results.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.search_results_state.select(Some(i));
    }

    pub fn previous_search_result(&mut self) {
        if self.search_results.is_empty() { return; }
        let i = match self.search_results_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.search_results.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.search_results_state.select(Some(i));
    }
}
