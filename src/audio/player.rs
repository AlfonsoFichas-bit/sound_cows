use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};
use std::path::Path;
use std::thread;
use std::sync::mpsc::Sender;
use rodio::{Decoder, OutputStream, Sink, Source};
use crate::scope::Matrix;
use crate::app::state::AppEvent;
use super::stream::{download_audio, search_audio};

pub struct AudioPlayer {
    // We keep these alive
    _stream: Option<OutputStream>,
    _stream_handle: Option<rodio::OutputStreamHandle>,
    sink: Option<Sink>,

    // Visualization data
    pub audio_data: Matrix<f64>,
    pub sample_rate: u32,
    pub channels: usize,
    pub is_streaming_mode: bool, // New flag for optimization

    // Playback Timing State
    pub start_time: Option<Instant>,
    pub elapsed_when_paused: Duration,
    pub total_duration: Option<Duration>,

    // Errors
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
            is_streaming_mode: false,
            start_time: None,
            elapsed_when_paused: Duration::from_secs(0),
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
    }

    // Synchronous load (legacy / local)
    #[allow(dead_code)]
    pub fn load_source(&mut self, path_or_url: &str) {
        if self.sink.is_none() {
            return;
        }

        self.error_message = None;

        let path = if path_or_url.starts_with("http") {
            let temp_path = Path::new("stream_cache.mp3");
            match download_audio(path_or_url, temp_path) {
                Ok(_) => temp_path,
                Err(e) => {
                    self.error_message = Some(e);
                    return;
                }
            }
        } else {
            Path::new(path_or_url)
        };

        self.play_file(path);
    }

    // Async load wrapper
    pub fn load_source_async(url: String, tx: Sender<AppEvent>) {
        thread::spawn(move || {
            let temp_path = Path::new("stream_cache.mp3");
            match download_audio(&url, temp_path) {
                Ok(_) => {
                    let _ = tx.send(AppEvent::AudioLoaded(temp_path.to_string_lossy().to_string()));
                },
                Err(e) => {
                    let _ = tx.send(AppEvent::AudioError(e));
                }
            }
        });
    }

    pub fn search_async(query: String, tx: Sender<AppEvent>) {
        thread::spawn(move || {
            match search_audio(&query) {
                Ok(results) => {
                    let _ = tx.send(AppEvent::SearchFinished(results));
                },
                Err(e) => {
                    let _ = tx.send(AppEvent::SearchError(e));
                }
            }
        });
    }

    pub fn play_file(&mut self, path: &Path) {
        if let Some(sink) = &self.sink {
            sink.stop();

            match File::open(path) {
                Ok(file) => {
                    match Decoder::new(BufReader::new(file)) {
                        Ok(source) => {
                             self.sample_rate = source.sample_rate();
                             self.channels = source.channels() as usize;

                             // Calculate duration properly?
                             // Rodio source might support `total_duration()`.
                             // MP3 decoder often returns None for total_duration until scanned.
                             // We can estimate from file size if we knew bitrate, but let's try reading a bit.
                             // Actually, if we want to optimize, we CANNOT run `convert_samples().collect()` on the whole file.

                             // Strategy:
                             // 1. Try to guess duration.
                             // 2. If it seems long, or we just want to be safe, enable Streaming Mode.
                             // 3. For now, we unfortunately need to iterate to know duration reliably for VBR MP3s without scanning.
                             // BUT, we can just check file size as a heuristic for "Long file".
                             // 10 minutes of MP3 128kbps is approx 10MB.
                             // Let's say if file > 20MB, we assume it's long and skip loading.

                             let metadata = std::fs::metadata(path).ok();
                             let file_size = metadata.map(|m| m.len()).unwrap_or(0);
                             let threshold_bytes = 20 * 1024 * 1024; // 20 MB threshold

                             // Re-open for playing (we consumed `source` for metadata check if we did, but we haven't yet)
                             // Actually `source` is fresh here.

                             if file_size > threshold_bytes {
                                 // --- STREAMING MODE (Optimization) ---
                                 self.is_streaming_mode = true;
                                 self.audio_data = vec![Vec::new(); self.channels]; // Empty buffer
                                 // We won't know exact total_duration easily without scanning.
                                 // Let's guess or leave it None.
                                 // If we leave it None, progress bar might break.
                                 // We can approximate: 128kbps = 16KB/s roughly.
                                 // Duration = size / 16000.
                                 let approx_seconds = file_size / 16000;
                                 self.total_duration = Some(Duration::from_secs(approx_seconds));

                                 // We need to consume the `source` we created? No, we can use it.
                                 // But we need a clone or reopen for Sink?
                                 // Rodio Sink takes ownership of Source.
                                 if let Some(handle) = &self._stream_handle {
                                     if let Ok(new_sink) = Sink::try_new(handle) {
                                         new_sink.set_volume(self.volume);
                                         new_sink.append(source); // Use the source directly! No collecting!
                                         self.sink = Some(new_sink);
                                         self.start_time = Some(Instant::now());
                                         self.elapsed_when_paused = Duration::from_secs(0);
                                         self.is_paused = false;
                                     }
                                 }
                             } else {
                                 // --- FULL LOAD MODE (Visualizer Active) ---
                                 self.is_streaming_mode = false;

                                 let samples: Vec<f32> = source.convert_samples().collect(); // Expensive step!
                                 let total_samples = samples.len() / self.channels;
                                 self.total_duration = Some(Duration::from_secs_f64(total_samples as f64 / self.sample_rate as f64));

                                 // We consumed source, so reopen for sink
                                 if let Ok(file_play) = File::open(path) {
                                     if let Ok(source_play) = Decoder::new(BufReader::new(file_play)) {
                                         if let Some(handle) = &self._stream_handle {
                                             if let Ok(new_sink) = Sink::try_new(handle) {
                                                 new_sink.set_volume(self.volume);
                                                 new_sink.append(source_play);
                                                 self.sink = Some(new_sink);
                                                 self.start_time = Some(Instant::now());
                                                 self.elapsed_when_paused = Duration::from_secs(0);
                                                 self.is_paused = false;
                                             }
                                         }
                                     }
                                 }

                                 self.audio_data = vec![Vec::new(); self.channels];
                                 for (i, sample) in samples.iter().enumerate() {
                                     self.audio_data[i % self.channels].push(*sample as f64);
                                 }
                             }
                        },
                        Err(e) => self.error_message = Some(format!("Format error: {}", e)),
                    }
                },
                Err(_) => {
                     self.error_message = Some(format!("File not found: {}", path.display()));
                }
            }
        }
    }

    /// Helper to get the current playback position
    pub fn get_current_time(&self) -> Duration {
        if self.is_paused {
            self.elapsed_when_paused
        } else {
            if let Some(start) = self.start_time {
                self.elapsed_when_paused + start.elapsed()
            } else {
                Duration::from_secs(0)
            }
        }
    }

    pub fn get_window(&self, window_size: usize) -> Matrix<f64> {
        // If paused or streaming (no data), return a flat line
        if self.is_paused || self.is_streaming_mode {
            return vec![vec![0.0; window_size]; self.channels];
        }

        let elapsed_seconds = self.get_current_time().as_secs_f64();
        let start_sample = (elapsed_seconds * self.sample_rate as f64) as usize;

        // Safety check if audio_data is empty (should cover streaming mode, but double check)
        if self.audio_data.is_empty() || self.audio_data[0].is_empty() {
             return vec![vec![0.0; window_size]; self.channels];
        }

        let mut window = vec![Vec::new(); self.channels];
        for ch in 0..self.channels {
            if start_sample < self.audio_data[ch].len() {
                let end = std::cmp::min(start_sample + window_size, self.audio_data[ch].len());
                window[ch] = self.audio_data[ch][start_sample..end].to_vec();
                if window[ch].len() < window_size {
                     window[ch].resize(window_size, 0.0);
                }
            } else {
                window[ch] = vec![0.0; window_size];
            }
        }
        window
    }

    pub fn toggle_pause(&mut self) {
        if let Some(sink) = &self.sink {
            if self.is_paused {
                // RESUME
                sink.play();
                self.is_paused = false;
                self.start_time = Some(Instant::now());
            } else {
                // PAUSE
                sink.pause();
                self.is_paused = true;
                // Capture elapsed time up to this moment
                if let Some(start) = self.start_time {
                    self.elapsed_when_paused += start.elapsed();
                }
                self.start_time = None;
            }
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        if let Some(sink) = &self.sink {
            self.volume = volume.clamp(0.0, 10.0);
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
