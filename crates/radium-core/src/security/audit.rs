//! Audit logging system for privacy redactions.

use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use super::privacy_error::{PrivacyError, Result};

/// An audit entry for a privacy redaction operation.
#[derive(Debug, Clone, Serialize)]
pub struct AuditEntry {
    /// Timestamp in RFC3339 format.
    pub timestamp: String,
    /// Agent ID that triggered the redaction (if available).
    pub agent_id: Option<String>,
    /// Type of pattern that matched (e.g., "ipv4", "email").
    pub pattern_type: String,
    /// Number of redactions performed.
    pub redaction_count: usize,
    /// Context where redaction occurred (e.g., "ContextManager.build_context").
    pub context: String,
    /// Redaction mode/style used (e.g., "partial", "full", "hash").
    pub mode: String,
}

/// Audit logger for privacy redactions.
pub struct AuditLogger {
    /// Path to the audit log file.
    file_path: PathBuf,
    /// Whether audit logging is enabled.
    enabled: bool,
    /// Buffered file writer (protected by mutex for thread safety).
    writer: Mutex<Option<BufWriter<File>>>,
}

impl AuditLogger {
    /// Creates a new audit logger.
    ///
    /// # Arguments
    /// * `file_path` - Path to the audit log file (JSONL format)
    /// * `enabled` - Whether audit logging is enabled
    ///
    /// # Errors
    /// Returns error if file cannot be created or opened
    pub fn new(file_path: impl AsRef<Path>, enabled: bool) -> Result<Self> {
        let file_path = file_path.as_ref().to_path_buf();

        // Create parent directory if it doesn't exist
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                PrivacyError::IoError(format!("Failed to create audit log directory: {}", e))
            })?;
        }

        // Open file in append mode
        let writer = if enabled {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_path)
                .map_err(|e| {
                    PrivacyError::IoError(format!("Failed to open audit log file: {}", e))
                })?;
            Some(Mutex::new(Some(BufWriter::new(file))))
        } else {
            None
        };

        Ok(Self {
            file_path,
            enabled,
            writer: Mutex::new(writer),
        })
    }

    /// Logs an audit entry.
    ///
    /// # Arguments
    /// * `entry` - The audit entry to log
    ///
    /// # Errors
    /// Returns error if writing fails
    pub fn log(&self, entry: AuditEntry) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // Serialize entry to JSON
        let json = serde_json::to_string(&entry).map_err(|e| {
            PrivacyError::IoError(format!("Failed to serialize audit entry: {}", e))
        })?;

        // Write to file (thread-safe)
        if let Some(ref writer_mutex) = *self.writer.lock().unwrap() {
            if let Some(ref mut writer) = *writer_mutex {
                writeln!(writer, "{}", json).map_err(|e| {
                    PrivacyError::IoError(format!("Failed to write audit entry: {}", e))
                })?;
                writer.flush().map_err(|e| {
                    PrivacyError::IoError(format!("Failed to flush audit log: {}", e))
                })?;
            }
        }

        Ok(())
    }

    /// Gets the audit log file path.
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Checks if audit logging is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_audit_entry_serialization() {
        let entry = AuditEntry {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            agent_id: Some("agent-123".to_string()),
            pattern_type: "ipv4".to_string(),
            redaction_count: 3,
            context: "ContextManager.build_context".to_string(),
            mode: "partial".to_string(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("ipv4"));
        assert!(json.contains("agent-123"));
        assert!(json.contains("3"));
    }

    #[test]
    fn test_audit_logger_creation() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.jsonl");
        let logger = AuditLogger::new(&log_path, true).unwrap();
        assert!(logger.is_enabled());
        assert_eq!(logger.file_path(), log_path);
    }

    #[test]
    fn test_audit_logger_disabled() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.jsonl");
        let logger = AuditLogger::new(&log_path, false).unwrap();
        assert!(!logger.is_enabled());
        
        // Logging should succeed but not write anything
        let entry = AuditEntry {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            agent_id: None,
            pattern_type: "email".to_string(),
            redaction_count: 1,
            context: "test".to_string(),
            mode: "full".to_string(),
        };
        logger.log(entry).unwrap();
        
        // File should not exist
        assert!(!log_path.exists());
    }

    #[test]
    fn test_audit_logger_writes_jsonl() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.jsonl");
        let logger = AuditLogger::new(&log_path, true).unwrap();

        let entry1 = AuditEntry {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            agent_id: Some("agent-1".to_string()),
            pattern_type: "ipv4".to_string(),
            redaction_count: 2,
            context: "test1".to_string(),
            mode: "partial".to_string(),
        };

        let entry2 = AuditEntry {
            timestamp: "2024-01-15T10:31:00Z".to_string(),
            agent_id: Some("agent-2".to_string()),
            pattern_type: "email".to_string(),
            redaction_count: 1,
            context: "test2".to_string(),
            mode: "full".to_string(),
        };

        logger.log(entry1).unwrap();
        logger.log(entry2).unwrap();

        // Read file and verify JSONL format
        let content = std::fs::read_to_string(&log_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
        
        // Verify each line is valid JSON
        let entry1_parsed: AuditEntry = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(entry1_parsed.pattern_type, "ipv4");
        assert_eq!(entry1_parsed.redaction_count, 2);

        let entry2_parsed: AuditEntry = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(entry2_parsed.pattern_type, "email");
        assert_eq!(entry2_parsed.redaction_count, 1);
    }

    #[test]
    fn test_audit_logger_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.jsonl");
        let logger = Arc::new(AuditLogger::new(&log_path, true).unwrap());

        let mut handles = vec![];
        for i in 0..10 {
            let logger_clone = Arc::clone(&logger);
            let handle = thread::spawn(move || {
                for j in 0..10 {
                    let entry = AuditEntry {
                        timestamp: format!("2024-01-15T10:{}:00Z", i * 10 + j),
                        agent_id: Some(format!("agent-{}", i)),
                        pattern_type: "test".to_string(),
                        redaction_count: j,
                        context: format!("thread-{}", i),
                        mode: "partial".to_string(),
                    };
                    logger_clone.log(entry).unwrap();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all entries were written
        let content = std::fs::read_to_string(&log_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 100); // 10 threads * 10 entries each
    }
}

