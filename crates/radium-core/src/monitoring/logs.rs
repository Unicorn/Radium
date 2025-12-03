//! Log file management for agent execution.

use super::error::Result;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Log manager for agent execution logs.
pub struct LogManager {
    /// Root directory for logs.
    logs_dir: PathBuf,
}

impl LogManager {
    /// Creates a new log manager.
    ///
    /// # Arguments
    /// * `logs_dir` - Directory for log files
    ///
    /// # Errors
    /// Returns error if directory creation fails
    pub fn new(logs_dir: impl AsRef<Path>) -> Result<Self> {
        let logs_dir = logs_dir.as_ref().to_path_buf();
        fs::create_dir_all(&logs_dir)?;
        Ok(Self { logs_dir })
    }

    /// Gets the log file path for an agent.
    ///
    /// # Arguments
    /// * `agent_id` - Agent identifier
    ///
    /// # Returns
    /// Path to the agent's log file
    pub fn log_path(&self, agent_id: &str) -> PathBuf {
        self.logs_dir.join(format!("{}.log", agent_id))
    }

    /// Creates a new log file for an agent.
    ///
    /// # Arguments
    /// * `agent_id` - Agent identifier
    ///
    /// # Returns
    /// File handle for the log file
    ///
    /// # Errors
    /// Returns error if file creation fails
    pub fn create_log(&self, agent_id: &str) -> Result<File> {
        let path = self.log_path(agent_id);
        let file = File::create(path)?;
        Ok(file)
    }

    /// Appends a line to an agent's log.
    ///
    /// # Arguments
    /// * `agent_id` - Agent identifier
    /// * `line` - Log line to append
    ///
    /// # Errors
    /// Returns error if write fails
    pub fn append_log(&self, agent_id: &str, line: &str) -> Result<()> {
        let path = self.log_path(agent_id);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        writeln!(file, "{}", Self::strip_color_codes(line))?;
        Ok(())
    }

    /// Reads an agent's log file.
    ///
    /// # Arguments
    /// * `agent_id` - Agent identifier
    ///
    /// # Returns
    /// Log file contents
    ///
    /// # Errors
    /// Returns error if read fails
    pub fn read_log(&self, agent_id: &str) -> Result<String> {
        let path = self.log_path(agent_id);
        let content = fs::read_to_string(path)?;
        Ok(content)
    }

    /// Reads the last N lines of an agent's log.
    ///
    /// # Arguments
    /// * `agent_id` - Agent identifier
    /// * `lines` - Number of lines to read
    ///
    /// # Returns
    /// Last N lines of the log
    ///
    /// # Errors
    /// Returns error if read fails
    pub fn tail_log(&self, agent_id: &str, lines: usize) -> Result<String> {
        let content = self.read_log(agent_id)?;
        let tail_lines: Vec<&str> = content.lines().rev().take(lines).collect();
        Ok(tail_lines.into_iter().rev().collect::<Vec<_>>().join("\n"))
    }

    /// Strips ANSI color codes from text.
    ///
    /// This is useful for log files where color codes would be noise.
    fn strip_color_codes(text: &str) -> String {
        // Simple regex-free implementation for common ANSI codes
        let mut result = String::with_capacity(text.len());
        let mut chars = text.chars();

        while let Some(c) = chars.next() {
            if c == '\x1B' {
                // Skip ESC sequence
                if chars.next() == Some('[') {
                    // Skip until 'm'
                    while let Some(next) = chars.next() {
                        if next == 'm' {
                            break;
                        }
                    }
                }
            } else {
                result.push(c);
            }
        }

        result
    }

    /// Lists all agent log files.
    ///
    /// # Returns
    /// List of agent IDs with log files
    ///
    /// # Errors
    /// Returns error if directory read fails
    pub fn list_logs(&self) -> Result<Vec<String>> {
        let mut agent_ids = Vec::new();

        for entry in fs::read_dir(&self.logs_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("log") {
                if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                    agent_ids.push(file_stem.to_string());
                }
            }
        }

        Ok(agent_ids)
    }

    /// Deletes an agent's log file.
    ///
    /// # Arguments
    /// * `agent_id` - Agent identifier
    ///
    /// # Errors
    /// Returns error if deletion fails
    pub fn delete_log(&self, agent_id: &str) -> Result<()> {
        let path = self.log_path(agent_id);
        fs::remove_file(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_log_manager_new() {
        let temp_dir = TempDir::new().unwrap();
        let manager = LogManager::new(temp_dir.path()).unwrap();
        assert!(temp_dir.path().exists());
    }

    #[test]
    fn test_log_path() {
        let temp_dir = TempDir::new().unwrap();
        let manager = LogManager::new(temp_dir.path()).unwrap();
        let path = manager.log_path("agent-1");
        assert!(path.to_string_lossy().contains("agent-1.log"));
    }

    #[test]
    fn test_create_and_read_log() {
        let temp_dir = TempDir::new().unwrap();
        let manager = LogManager::new(temp_dir.path()).unwrap();

        let mut file = manager.create_log("agent-1").unwrap();
        writeln!(file, "Test log line").unwrap();
        drop(file);

        let content = manager.read_log("agent-1").unwrap();
        assert!(content.contains("Test log line"));
    }

    #[test]
    fn test_append_log() {
        let temp_dir = TempDir::new().unwrap();
        let manager = LogManager::new(temp_dir.path()).unwrap();

        manager.append_log("agent-1", "Line 1").unwrap();
        manager.append_log("agent-1", "Line 2").unwrap();

        let content = manager.read_log("agent-1").unwrap();
        assert!(content.contains("Line 1"));
        assert!(content.contains("Line 2"));
    }

    #[test]
    fn test_tail_log() {
        let temp_dir = TempDir::new().unwrap();
        let manager = LogManager::new(temp_dir.path()).unwrap();

        for i in 1..=10 {
            manager.append_log("agent-1", &format!("Line {}", i)).unwrap();
        }

        let tail = manager.tail_log("agent-1", 3).unwrap();
        assert!(tail.contains("Line 8"));
        assert!(tail.contains("Line 9"));
        assert!(tail.contains("Line 10"));
        assert!(!tail.contains("Line 7"));
    }

    #[test]
    fn test_strip_color_codes() {
        let colored = "\x1B[32mGreen text\x1B[0m normal \x1B[31mRed\x1B[0m";
        let stripped = LogManager::strip_color_codes(colored);
        assert_eq!(stripped, "Green text normal Red");
    }

    #[test]
    fn test_list_logs() {
        let temp_dir = TempDir::new().unwrap();
        let manager = LogManager::new(temp_dir.path()).unwrap();

        manager.append_log("agent-1", "test").unwrap();
        manager.append_log("agent-2", "test").unwrap();

        let logs = manager.list_logs().unwrap();
        assert_eq!(logs.len(), 2);
        assert!(logs.contains(&"agent-1".to_string()));
        assert!(logs.contains(&"agent-2".to_string()));
    }

    #[test]
    fn test_delete_log() {
        let temp_dir = TempDir::new().unwrap();
        let manager = LogManager::new(temp_dir.path()).unwrap();

        manager.append_log("agent-1", "test").unwrap();
        assert!(manager.log_path("agent-1").exists());

        manager.delete_log("agent-1").unwrap();
        assert!(!manager.log_path("agent-1").exists());
    }
}
