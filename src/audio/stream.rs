use std::process::Command;
use std::path::Path;
use serde_derive::Deserialize; // We need serde for JSON parsing

#[derive(Deserialize, Debug)]
pub struct YtDlpResult {
    pub title: String,
    pub url: String, // Or webpage_url
    #[allow(dead_code)]
    pub webpage_url: Option<String>,
    pub artist: Option<String>,
    pub uploader: Option<String>, // Fallback for artist
    pub album: Option<String>,
    pub duration_string: Option<String>, // "3:45"
    #[allow(dead_code)]
    pub duration: Option<f64>, // seconds
}

pub fn download_audio(url: &str, output_path: &Path) -> Result<(), String> {
    let output = Command::new("./yt-dlp")
        .arg("-x") // Extract audio
        .arg("--audio-format")
        .arg("mp3")
        .arg("-o")
        .arg(output_path)
        .arg("--force-overwrites") // Overwrite if exists
        .arg(url)
        .output();

    match output {
        Ok(o) => {
            if o.status.success() {
                Ok(())
            } else {
                Err(format!("yt-dlp error: {}", String::from_utf8_lossy(&o.stderr)))
            }
        },
        Err(e) => Err(format!("Failed to execute yt-dlp: {}", e)),
    }
}

// Return full struct instead of tuple
pub fn search_audio(query: &str) -> Result<Vec<YtDlpResult>, String> {
    // ytsearch5:query means "search youtube for query and get 5 results"
    let search_query = format!("ytsearch5:{}", query);

    let output = Command::new("./yt-dlp")
        .arg("--flat-playlist") // Don't download, just list
        .arg("--dump-json")     // Output as JSON
        .arg("--no-warnings")
        .arg(&search_query)
        .output();

    match output {
        Ok(o) => {
            if o.status.success() {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let mut results = Vec::new();

                // yt-dlp outputs one JSON object per line
                for line in stdout.lines() {
                    if let Ok(entry) = serde_json::from_str::<YtDlpResult>(line) {
                        results.push(entry);
                    }
                }
                Ok(results)
            } else {
                Err(format!("yt-dlp search error: {}", String::from_utf8_lossy(&o.stderr)))
            }
        },
        Err(e) => Err(format!("Failed to execute yt-dlp search: {}", e)),
    }
}
