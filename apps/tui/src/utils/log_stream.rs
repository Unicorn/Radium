//! Log file streaming utility for real-time log updates.

use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::interval;

/// Log stream state
#[derive(Debug, Clone)]
pub struct LogStreamState {
    /// Log file path
    pub path: std::path::PathBuf,
    /// Current lines
    pub lines: Vec<String>,
    /// Current file position
    pub position: u64,
    /// Whether the agent is still running
    pub is_running: bool,
    /// File size in bytes
    pub file_size: u64,
}

impl LogStreamState {
    /// Creates a new log stream state.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            lines: Vec::new(),
            position: 0,
            is_running: true,
            file_size: 0,
        }
    }

    /// Reads new lines from the log file.
    pub fn read_new_lines(&mut self) -> std::io::Result<Vec<String>> {
        let file = File::open(&self.path)?;
        let metadata = file.metadata()?;
        self.file_size = metadata.len();

        // If file was truncated or position is beyond file size, reset
        if self.position > self.file_size {
            self.position = 0;
            self.lines.clear();
        }

        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(self.position))?;

        let mut new_lines = Vec::new();
        let mut line = String::new();
        let mut current_pos = self.position;

        while reader.read_line(&mut line)? > 0 {
            let line_len = line.len() as u64;
            if !line.trim().is_empty() {
                let trimmed = line.trim_end().to_string();
                new_lines.push(trimmed.clone());
                self.lines.push(trimmed);
            }
            current_pos += line_len;
            line.clear();
        }

        self.position = current_pos;
        Ok(new_lines)
    }

    /// Returns all current lines.
    pub fn get_lines(&self) -> &[String] {
        &self.lines
    }

    /// Returns the file size in bytes.
    pub fn get_file_size(&self) -> u64 {
        self.file_size
    }

    /// Marks the stream as no longer running.
    pub fn mark_stopped(&mut self) {
        self.is_running = false;
    }
}

/// Log stream manager for handling multiple log streams.
pub struct LogStreamManager {
    streams: Arc<Mutex<std::collections::HashMap<String, LogStreamState>>>,
}

impl LogStreamManager {
    /// Creates a new log stream manager.
    pub fn new() -> Self {
        Self {
            streams: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Registers a log file to stream.
    pub async fn register<P: AsRef<Path>>(&self, id: String, path: P) {
        let mut streams = self.streams.lock().await;
        streams.insert(id, LogStreamState::new(path));
    }

    /// Updates a log stream (reads new lines).
    pub async fn update_stream(&self, id: &str) -> std::io::Result<Vec<String>> {
        let mut streams = self.streams.lock().await;
        if let Some(stream) = streams.get_mut(id) {
            stream.read_new_lines()
        } else {
            Ok(Vec::new())
        }
    }

    /// Gets the current lines for a stream.
    pub async fn get_lines(&self, id: &str) -> Vec<String> {
        let streams = self.streams.lock().await;
        streams
            .get(id)
            .map(|s| s.lines.clone())
            .unwrap_or_default()
    }

    /// Gets the file size for a stream.
    pub async fn get_file_size(&self, id: &str) -> u64 {
        let streams = self.streams.lock().await;
        streams.get(id).map(|s| s.file_size).unwrap_or(0)
    }

    /// Marks a stream as stopped.
    pub async fn mark_stopped(&self, id: &str) {
        let mut streams = self.streams.lock().await;
        if let Some(stream) = streams.get_mut(id) {
            stream.mark_stopped();
        }
    }

    /// Checks if a stream is running.
    pub async fn is_running(&self, id: &str) -> bool {
        let streams = self.streams.lock().await;
        streams.get(id).map(|s| s.is_running).unwrap_or(false)
    }

    /// Removes a stream.
    pub async fn remove(&self, id: &str) {
        let mut streams = self.streams.lock().await;
        streams.remove(id);
    }
}

impl Default for LogStreamManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Background task for updating log streams.
pub async fn start_log_stream_updater(_manager: Arc<LogStreamManager>, _update_interval: Duration) {
    let mut interval = interval(_update_interval);
    loop {
        interval.tick().await;
        // Update all streams
        // This would typically be called from the main app loop
        // For now, it's a placeholder for the update mechanism
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_log_stream_basic() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Line 1").unwrap();
        writeln!(file, "Line 2").unwrap();
        file.flush().unwrap();

        let mut stream = LogStreamState::new(file.path());
        let new_lines = stream.read_new_lines().unwrap();
        
        assert_eq!(new_lines.len(), 2);
        assert_eq!(new_lines[0], "Line 1");
        assert_eq!(new_lines[1], "Line 2");
    }

    #[tokio::test]
    async fn test_log_stream_incremental() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Line 1").unwrap();
        file.flush().unwrap();

        let mut stream = LogStreamState::new(file.path());
        let new_lines = stream.read_new_lines().unwrap();
        assert_eq!(new_lines.len(), 1);

        // Add more lines
        writeln!(file, "Line 2").unwrap();
        writeln!(file, "Line 3").unwrap();
        file.flush().unwrap();

        let new_lines = stream.read_new_lines().unwrap();
        assert_eq!(new_lines.len(), 2);
        assert_eq!(stream.lines.len(), 3);
    }

    #[tokio::test]
    async fn test_log_stream_manager() {
        let manager = LogStreamManager::new();
        let file = NamedTempFile::new().unwrap();
        
        manager.register("test-id".to_string(), file.path()).await;
        
        let lines = manager.get_lines("test-id").await;
        assert_eq!(lines.len(), 0);
        
        let is_running = manager.is_running("test-id").await;
        assert!(is_running);
        
        manager.mark_stopped("test-id").await;
        let is_running = manager.is_running("test-id").await;
        assert!(!is_running);
    }
}

