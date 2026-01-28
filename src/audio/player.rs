use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};
use std::path::Path;
use std::thread;
use std::sync::{mpsc::Sender, Arc, Mutex};
use std::collections::VecDeque;
use rodio::{Decoder, OutputStream, Sink, Source, Sample};
use crate::scope::Matrix;
use crate::app::state::AppEvent;
use super::stream::{download_audio, search_audio};

// Type alias for the shared buffer
// We use a VecDeque for a rolling window of samples.
// It stores interleaved samples (L, R, L, R...).
type SharedAudioBuffer = Arc<Mutex<VecDeque<f32>>>;

// Custom Source that inspects samples as they pass through
pub struct InspectionSource<I>
where
    I: Source,
    I::Item: Sample + Send,
{
    input: I,
    buffer: SharedAudioBuffer,
}

impl<I> InspectionSource<I>
where
    I: Source,
    I::Item: Sample + Send,
{
    pub fn new(input: I, buffer: SharedAudioBuffer) -> Self {
        InspectionSource { input, buffer }
    }
}

impl<I> Iterator for InspectionSource<I>
where
    I: Source,
    I::Item: Sample + Send,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.input.next()?;

        // Push sample to buffer
        if let Ok(mut buf) = self.buffer.lock() {
            // Convert sample to f32 for visualization
            let sample_f32 = item.to_f32();
            buf.push_back(sample_f32);

            // Limit buffer size to keep memory low but enough for visualization window
            // If sample rate is 44100, stereo = 88200 samples per second.
            // We just need enough for the visualizer window (e.g. 1024 or 2048 samples).
            // Let's keep a bit more, say 8192 to be safe.
            if buf.len() > 8192 {
                buf.pop_front();
            }
        }

        Some(item)
    }
}

impl<I> Source for InspectionSource<I>
where
    I: Source,
    I::Item: Sample + Send,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.input.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.input.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.input.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }
}

pub struct AudioPlayer {
    _stream: Option<OutputStream>,
    _stream_handle: Option<rodio::OutputStreamHandle>,
    sink: Option<Sink>,

    // Visualization data - Changed to shared buffer
    pub visualization_buffer: SharedAudioBuffer,

    pub sample_rate: u32,
    pub channels: usize,

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
            visualization_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(8192))),
            sample_rate: 44100,
            channels: 2,
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
                Ok(yt_results) => {
                    // Convert YtDlpResult to Song
                    let songs: Vec<crate::app::state::Song> = yt_results.into_iter().map(|r| {
                        crate::app::state::Song {
                            title: r.title,
                            artist: r.artist.or(r.uploader).unwrap_or("Unknown".to_string()),
                            album: r.album.unwrap_or("Unknown".to_string()),
                            url: r.url,
                            duration_str: r.duration_string.unwrap_or("00:00".to_string()),
                        }
                    }).collect();
                    let _ = tx.send(AppEvent::SearchFinished(songs));
                },
                Err(e) => {
                    let _ = tx.send(AppEvent::SearchError(e));
                }
            }
        });
    }

    pub fn play_file(&mut self, path: &Path) {
        if let Some(handle) = &self._stream_handle {
            // Drop old sink to stop previous playback
            self.sink = None;

            match File::open(path) {
                Ok(file) => {
                    match Decoder::new(BufReader::new(file)) {
                        Ok(source) => {
                             self.sample_rate = source.sample_rate();
                             self.channels = source.channels() as usize;

                             // Estimate duration from file size
                             let metadata = std::fs::metadata(path).ok();
                             let file_size = metadata.map(|m| m.len()).unwrap_or(0);
                             let approx_seconds = if file_size > 0 { file_size / 16000 } else { 0 }; // 128kbps approx
                             self.total_duration = Some(Duration::from_secs(approx_seconds));

                             if let Ok(new_sink) = Sink::try_new(handle) {
                                 new_sink.set_volume(self.volume);

                                 // Clear visualization buffer
                                 if let Ok(mut buf) = self.visualization_buffer.lock() {
                                     buf.clear();
                                 }

                                 // Use InspectionSource to tap into the stream
                                 let source_f32 = source.convert_samples::<f32>();
                                 let inspection_source = InspectionSource::new(source_f32, self.visualization_buffer.clone());

                                 new_sink.append(inspection_source);

                                 self.sink = Some(new_sink);
                                 self.start_time = Some(Instant::now());
                                 self.elapsed_when_paused = Duration::from_secs(0);
                                 self.is_paused = false;
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
        if self.is_paused {
            return vec![vec![0.0; window_size]; self.channels];
        }

        // Read from visualization_buffer
        if let Ok(buf) = self.visualization_buffer.lock() {
            let mut window = vec![Vec::new(); self.channels];

            // We want the last `window_size` samples per channel.
            let total_needed = window_size * self.channels;
            let available = buf.len();
            let start_index = if available > total_needed { available - total_needed } else { 0 };

            // Extract and de-interleave
            for (i, &sample) in buf.iter().skip(start_index).enumerate() {
                let channel = i % self.channels;
                if window[channel].len() < window_size {
                    window[channel].push(sample as f64);
                }
            }

            // Pad if necessary
             for ch in 0..self.channels {
                if window[ch].len() < window_size {
                    window[ch].resize(window_size, 0.0);
                }
            }
            return window;
        }

        vec![vec![0.0; window_size]; self.channels]
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
