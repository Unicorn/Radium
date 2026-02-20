//! Log file watcher for monitoring process logs.

use anyhow::{Context, Result};
use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tracing::{debug, error, warn};

/// Log watcher that monitors a log file for changes and streams new lines.
pub struct LogWatcher {
    /// Path to the log file being watched.
    log_file: PathBuf,
    /// Current position in the file (for tail functionality).
    position: Arc<Mutex<u64>>,
    /// Sender for log lines.
    log_sender: mpsc::Sender<String>,
    /// File system watcher.
    _watcher: Option<RecommendedWatcher>,
}

impl LogWatcher {
    /// Creates a new log watcher for the specified file.
    ///
    /// # Arguments
    /// * `log_file` - Path to the log file to watch
    /// * `log_sender` - Channel sender for streaming log lines
    pub fn new<P: AsRef<Path>>(
        log_file: P,
        log_sender: mpsc::Sender<String>,
    ) -> Result<Self> {
        let log_file = log_file.as_ref().to_path_buf();

        // Initialize position to end of file (tail mode)
        let position = if log_file.exists() {
            std::fs::metadata(&log_file)
                .map(|m| m.len())
                .unwrap_or(0)
        } else {
            0
        };

        let position = Arc::new(Mutex::new(position));

        debug!(
            log_file = ?log_file,
            initial_position = position.try_lock().map(|p| *p).unwrap_or(0),
            "Creating log watcher"
        );

        Ok(Self {
            log_file,
            position,
            log_sender,
            _watcher: None,
        })
    }

    /// Starts watching the log file.
    ///
    /// This method sets up file system notifications and begins streaming
    /// new log lines as they're written to the file.
    pub fn start(&mut self) -> Result<()> {
        let log_file = self.log_file.clone();
        let position = self.position.clone();
        let log_sender = self.log_sender.clone();

        // Create file watcher
        let (tx, mut rx) = mpsc::channel(100);

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.blocking_send(event);
                }
            },
            Config::default(),
        )
        .context("Failed to create file watcher")?;

        // Watch the log file
        watcher
            .watch(&log_file, RecursiveMode::NonRecursive)
            .context("Failed to watch log file")?;

        debug!(log_file = ?log_file, "Started watching log file");

        // Spawn task to handle file events
        let log_file_clone = log_file.clone();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) => {
                        // File was modified or created, read new lines
                        if let Err(e) = Self::read_new_lines(
                            &log_file_clone,
                            &position,
                            &log_sender,
                        )
                        .await
                        {
                            error!(
                                log_file = ?log_file_clone,
                                error = %e,
                                "Failed to read new lines"
                            );
                        }
                    }
                    EventKind::Remove(_) => {
                        warn!(log_file = ?log_file_clone, "Log file was removed");
                    }
                    _ => {}
                }
            }
        });

        self._watcher = Some(watcher);

        Ok(())
    }

    /// Reads new lines from the log file since the last position.
    async fn read_new_lines(
        log_file: &Path,
        position: &Arc<Mutex<u64>>,
        log_sender: &mpsc::Sender<String>,
    ) -> Result<()> {
        use tokio::io::{AsyncReadExt, AsyncSeekExt};

        // Open file
        let mut file = tokio::fs::File::open(log_file)
            .await
            .context("Failed to open log file")?;

        // Get current file size
        let metadata = file.metadata().await?;
        let file_size = metadata.len();

        // Get last read position
        let mut pos = position.lock().await;

        // If file was truncated (e.g., log rotation), reset position
        if file_size < *pos {
            debug!(
                log_file = ?log_file,
                old_position = *pos,
                new_size = file_size,
                "Log file truncated, resetting position"
            );
            *pos = 0;
        }

        // Seek to last position
        file.seek(std::io::SeekFrom::Start(*pos)).await?;

        // Read new content
        let mut buffer = Vec::new();
        let bytes_read = file.read_to_end(&mut buffer).await?;

        if bytes_read == 0 {
            return Ok(());
        }

        // Update position
        *pos += bytes_read as u64;

        // Parse lines and send
        let content = String::from_utf8_lossy(&buffer);
        for line in content.lines() {
            if !line.is_empty() {
                let _ = log_sender.send(line.to_string()).await;
            }
        }

        debug!(
            log_file = ?log_file,
            bytes_read = bytes_read,
            new_position = *pos,
            "Read new log lines"
        );

        Ok(())
    }

    /// Reads the entire log file from the beginning.
    pub async fn read_full_log(&self) -> Result<String> {
        tokio::fs::read_to_string(&self.log_file)
            .await
            .context("Failed to read log file")
    }

    /// Gets the current file position.
    pub async fn get_position(&self) -> u64 {
        *self.position.lock().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_log_watcher_reads_new_lines() {
        // Create temporary log file
        let temp_file = NamedTempFile::new().unwrap();
        let log_path = temp_file.path().to_path_buf();

        // Write initial content
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .open(&log_path)
            .await
            .unwrap();
        file.write_all(b"Initial line\n").await.unwrap();
        file.flush().await.unwrap();
        drop(file);

        // Create channel for log lines
        let (tx, mut rx) = mpsc::channel(100);

        // Create and start watcher
        let mut watcher = LogWatcher::new(&log_path, tx).unwrap();
        watcher.start().unwrap();

        // Wait a bit for watcher to initialize
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Write new content
        let mut file = tokio::fs::OpenOptions::new()
            .append(true)
            .open(&log_path)
            .await
            .unwrap();
        file.write_all(b"New line 1\n").await.unwrap();
        file.write_all(b"New line 2\n").await.unwrap();
        file.flush().await.unwrap();
        drop(file);

        // Wait for file system event to be processed
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Collect received lines
        let mut lines = Vec::new();
        while let Ok(line) = rx.try_recv() {
            lines.push(line);
        }

        // Should have received the new lines (not the initial one)
        assert!(lines.contains(&"New line 1".to_string()));
        assert!(lines.contains(&"New line 2".to_string()));
    }

    #[tokio::test]
    async fn test_read_full_log() {
        // Create temporary log file
        let temp_file = NamedTempFile::new().unwrap();
        let log_path = temp_file.path().to_path_buf();

        // Write content
        tokio::fs::write(&log_path, b"Line 1\nLine 2\nLine 3\n")
            .await
            .unwrap();

        // Create watcher
        let (tx, _rx) = mpsc::channel(100);
        let watcher = LogWatcher::new(&log_path, tx).unwrap();

        // Read full log
        let content = watcher.read_full_log().await.unwrap();
        assert_eq!(content, "Line 1\nLine 2\nLine 3\n");
    }
}
