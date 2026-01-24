use std::process::Command;
use std::path::Path;

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
