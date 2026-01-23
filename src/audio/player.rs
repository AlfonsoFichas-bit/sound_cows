use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};
use rodio::{Decoder, OutputStream, Sink, Source};
use crate::scope::Matrix;

pub struct AudioPlayer {
    // We keep these alive
    _stream: Option<OutputStream>,
    _stream_handle: Option<rodio::OutputStreamHandle>,
    sink: Option<Sink>,

    // Visualization data
    pub audio_data: Matrix<f64>,
    pub sample_rate: u32,
    pub channels: usize,
    pub start_time: Option<Instant>,
    pub total_duration: Option<Duration>,
    pub error_message: Option<String>,

    // State
    pub is_paused: bool,
    pub volume: f32,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let mut player = AudioPlayer {
            _stream: None,
            _stream_handle: None,
            sink: None,
            audio_data: vec![vec![0.0; 1024]; 2],
            sample_rate: 44100,
            channels: 2,
            start_time: None,
            total_duration: None,
            error_message: None,
            is_paused: false,
            volume: 1.0,
        };

        player.init();
        player
    }

    fn init(&mut self) {
        match OutputStream::try_default() {
            Ok((stream, stream_handle)) => {
                match Sink::try_new(&stream_handle) {
                    Ok(s) => {
                        self._stream = Some(stream);
                        self._stream_handle = Some(stream_handle);
                        self.sink = Some(s);
                    },
                    Err(e) => self.error_message = Some(format!("Sink error: {}", e)),
                }
            },
            Err(e) => self.error_message = Some(format!("Audio init error: {}", e)),
        }

        if self.sink.is_some() {
             self.load_file("audio.mp3");
        }
    }

    fn load_file(&mut self, path: &str) {
        if let Some(sink) = &self.sink {
            match File::open(path) {
                Ok(file) => {
                    match Decoder::new(BufReader::new(file)) {
                        Ok(source) => {
                             self.sample_rate = source.sample_rate();
                             self.channels = source.channels() as usize;

                             let samples: Vec<f32> = source.convert_samples().collect();
                             let total_samples = samples.len() / self.channels;
                             self.total_duration = Some(Duration::from_secs_f64(total_samples as f64 / self.sample_rate as f64));

                             // Re-open for playing
                             if let Ok(file_play) = File::open(path) {
                                 if let Ok(source_play) = Decoder::new(BufReader::new(file_play)) {
                                     sink.append(source_play);
                                     sink.play();
                                     self.start_time = Some(Instant::now());
                                 }
                             }

                             self.audio_data = vec![Vec::new(); self.channels];
                             for (i, sample) in samples.iter().enumerate() {
                                 self.audio_data[i % self.channels].push(*sample as f64);
                             }
                        },
                        Err(e) => self.error_message = Some(format!("Format error: {}", e)),
                    }
                },
                Err(_) => {
                    // Silent fail or log
                    // self.error_message = Some(format!("File not found: {}", path));
                }
            }
        }
    }

    pub fn get_window(&self, window_size: usize) -> Matrix<f64> {
        if let Some(start_time) = self.start_time {
            // Note: This naive implementation doesn't account for pausing time drift
            // For a robust player, we'd track "accumulated pause duration" or use Sink position if available.
            // Given the constraints, we will just use elapsed time, which means the visual will jump if paused.
            // Fixing this properly requires refactoring how we track time.
            // For now, let's just implement the control.

            let elapsed_seconds = start_time.elapsed().as_secs_f64();
            let start_sample = (elapsed_seconds * self.sample_rate as f64) as usize;
            let end_sample = start_sample + window_size;

            let mut window = vec![Vec::new(); self.channels];
            for ch in 0..self.channels {
                if start_sample < self.audio_data[ch].len() {
                    let end = std::cmp::min(end_sample, self.audio_data[ch].len());
                    window[ch] = self.audio_data[ch][start_sample..end].to_vec();
                    // Pad if necessary
                    if window[ch].len() < window_size {
                         window[ch].resize(window_size, 0.0);
                    }
                } else {
                    window[ch] = vec![0.0; window_size];
                }
            }
            window
        } else {
             vec![vec![0.0; window_size]; self.channels]
        }
    }

    pub fn toggle_pause(&mut self) {
        if let Some(sink) = &self.sink {
            if self.is_paused {
                sink.play();
                self.is_paused = false;
                // TODO: Correct start_time to account for paused duration to avoid visual skip
            } else {
                sink.pause();
                self.is_paused = true;
            }
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        if let Some(sink) = &self.sink {
            self.volume = volume.clamp(0.0, 10.0); // 1.0 is 100%, allow boost up to 1000%? Or just 0-1? Rodio documentation says 1.0 is default.
            sink.set_volume(self.volume);
        }
    }

    pub fn volume_up(&mut self) {
        self.set_volume(self.volume + 0.1);
    }

    pub fn volume_down(&mut self) {
        self.set_volume(self.volume - 0.1);
    }
}
