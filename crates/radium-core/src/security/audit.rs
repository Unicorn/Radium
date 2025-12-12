//! Audit logging for secret operations.
//!
//! Records all secret access operations with timestamps and operation types,
//! enabling security monitoring and compliance.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::error::{SecurityError, SecurityResult};

/// Maximum audit log file size before rotation (10MB).
const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024;

/// Number of rotated log files to keep.
const MAX_ROTATED_LOGS: usize = 5;

/// Types of secret operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AuditOperation {
    /// Secret stored.
    Store,
    /// Secret retrieved.
    Get,
    /// Secrets listed.
    List,
    /// Secret rotated.
    Rotate,
    /// Secret removed.
    Remove,
}

/// Audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Timestamp of the operation.
    pub timestamp: String,
    /// Type of operation.
    pub operation: AuditOperation,
    /// Name of the secret (never the value).
    pub secret_name: String,
    /// Whether the operation succeeded.
    pub success: bool,
    /// Error message if operation failed (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// Session ID (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// User ID for authentication auditing (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// Policy decision for tool execution auditing (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_decision: Option<String>,
    /// Agent ID for privacy auditing (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    /// Pattern type for redaction auditing (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern_type: Option<String>,
    /// Number of redactions for auditing (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redaction_count: Option<usize>,
    /// Context for the operation (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Mode for privacy operations (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

/// Filter for querying audit log entries.
#[derive(Debug, Clone)]
pub struct AuditFilter {
    /// Filter by operation type (if Some).
    pub operation: Option<AuditOperation>,
    /// Filter by secret name (if Some).
    pub secret_name: Option<String>,
    /// Start time for time range filter (if Some).
    pub start_time: Option<chrono::DateTime<Utc>>,
    /// End time for time range filter (if Some).
    pub end_time: Option<chrono::DateTime<Utc>>,
}

impl Default for AuditFilter {
    fn default() -> Self {
        Self {
            operation: None,
            secret_name: None,
            start_time: None,
            end_time: None,
        }
    }
}

/// Audit logger for recording secret operations.
pub struct AuditLogger {
    /// Path to the audit log file.
    log_path: PathBuf,
}

impl AuditLogger {
    /// Creates a new audit logger.
    ///
    /// # Arguments
    ///
    /// * `log_path` - Path to the audit log file
    ///
    /// # Errors
    ///
    /// Returns an error if the log file cannot be created or accessed.
    pub fn new(log_path: PathBuf) -> SecurityResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| SecurityError::Io(e))?;
        }

        // Create or open log file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|e| SecurityError::Io(e))?;

        // Set file permissions to 0600 (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&log_path, perms)
                .map_err(|e| SecurityError::Io(e))?;
        }

        drop(file);

        Ok(Self { log_path })
    }

    /// Logs a secret operation.
    ///
    /// # Arguments
    ///
    /// * `operation` - Type of operation
    /// * `secret_name` - Name of the secret (never the value)
    /// * `success` - Whether the operation succeeded
    /// * `error_message` - Error message if operation failed (optional)
    ///
    /// # Errors
    ///
    /// Returns an error if logging fails.
    pub fn log_operation(
        &self,
        operation: AuditOperation,
        secret_name: &str,
        success: bool,
        error_message: Option<&str>,
    ) -> SecurityResult<()> {
        // Check if rotation is needed
        self.rotate_if_needed()?;

        let entry = AuditEntry {
            timestamp: Utc::now().to_rfc3339(),
            operation,
            secret_name: secret_name.to_string(),
            success,
            error_message: error_message.map(|s| s.to_string()),
            session_id: None, // Could be added later if session tracking is needed
            agent_id: None,
            pattern_type: None,
            redaction_count: None,
            context: None,
            mode: None,
        };

        // Serialize to JSON
        let json = serde_json::to_string(&entry)
            .map_err(|e| SecurityError::Serialization(e))?;

        // Append to log file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .map_err(|e| SecurityError::Io(e))?;

        writeln!(file, "{}", json)
            .map_err(|e| SecurityError::Io(e))?;

        // Ensure permissions are still 0600
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&self.log_path, perms)
                .map_err(|e| SecurityError::Io(e))?;
        }

        Ok(())
    }

    /// Logs a pre-constructed audit entry.
    ///
    /// This method is useful when you have already constructed an AuditEntry
    /// with all fields populated (e.g., for privacy redaction auditing).
    ///
    /// # Arguments
    ///
    /// * `entry` - The audit entry to log
    ///
    /// # Errors
    ///
    /// Returns an error if logging fails.
    pub fn log(&self, entry: AuditEntry) -> SecurityResult<()> {
        // Check if rotation is needed
        self.rotate_if_needed()?;

        // Serialize to JSON
        let json = serde_json::to_string(&entry)
            .map_err(|e| SecurityError::Serialization(e))?;

        // Append to log file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .map_err(|e| SecurityError::Io(e))?;

        writeln!(file, "{}", json)
            .map_err(|e| SecurityError::Io(e))?;

        // Ensure permissions are still 0600
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&self.log_path, perms)
                .map_err(|e| SecurityError::Io(e))?;
        }

        Ok(())
    }

    /// Queries audit log entries matching the filter.
    ///
    /// # Arguments
    ///
    /// * `filter` - Filter criteria
    ///
    /// # Returns
    ///
    /// Vector of matching audit entries
    ///
    /// # Errors
    ///
    /// Returns an error if the log file cannot be read.
    pub fn query_log(&self, filter: &AuditFilter) -> SecurityResult<Vec<AuditEntry>> {
        if !self.log_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.log_path)
            .map_err(|e| SecurityError::Io(e))?;

        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| SecurityError::Io(e))?;
            if line.trim().is_empty() {
                continue;
            }

            let entry: AuditEntry = serde_json::from_str(&line)
                .map_err(|e| SecurityError::Serialization(e))?;

            // Apply filters
            if let Some(op) = filter.operation {
                if entry.operation != op {
                    continue;
                }
            }

            if let Some(ref name) = filter.secret_name {
                if entry.secret_name != *name {
                    continue;
                }
            }

            if let Some(start) = filter.start_time {
                if let Ok(entry_time) = chrono::DateTime::parse_from_rfc3339(&entry.timestamp) {
                    if entry_time < start {
                        continue;
                    }
                }
            }

            if let Some(end) = filter.end_time {
                if let Ok(entry_time) = chrono::DateTime::parse_from_rfc3339(&entry.timestamp) {
                    if entry_time > end {
                        continue;
                    }
                }
            }

            entries.push(entry);
        }

        Ok(entries)
    }

    /// Rotates the log file if it exceeds the maximum size.
    fn rotate_if_needed(&self) -> SecurityResult<()> {
        if !self.log_path.exists() {
            return Ok(());
        }

        let metadata = std::fs::metadata(&self.log_path)
            .map_err(|e| SecurityError::Io(e))?;

        if metadata.len() < MAX_LOG_SIZE {
            return Ok(());
        }

        // Rotate logs
        for i in (1..MAX_ROTATED_LOGS).rev() {
            let old_path = self.log_path.with_extension(format!("log.{}", i));
            let new_path = self.log_path.with_extension(format!("log.{}", i + 1));

            if old_path.exists() {
                if new_path.exists() {
                    std::fs::remove_file(&new_path)
                        .map_err(|e| SecurityError::Io(e))?;
                }
                std::fs::rename(&old_path, &new_path)
                    .map_err(|e| SecurityError::Io(e))?;
            }
        }

        // Rename current log to .1
        let rotated_path = self.log_path.with_extension("log.1");
        std::fs::rename(&self.log_path, &rotated_path)
            .map_err(|e| SecurityError::Io(e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_log_operation() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");

        let logger = AuditLogger::new(log_path.clone()).unwrap();
        logger.log_operation(
            AuditOperation::Store,
            "test_secret",
            true,
            None,
        ).unwrap();

        assert!(log_path.exists());
    }

    #[test]
    fn test_query_log() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");

        let logger = AuditLogger::new(log_path.clone()).unwrap();
        logger.log_operation(AuditOperation::Store, "secret1", true, None).unwrap();
        logger.log_operation(AuditOperation::Get, "secret1", true, None).unwrap();
        logger.log_operation(AuditOperation::Store, "secret2", true, None).unwrap();

        let filter = AuditFilter {
            operation: Some(AuditOperation::Store),
            ..Default::default()
        };

        let entries = logger.query_log(&filter).unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.operation == AuditOperation::Store));
    }

    #[test]
    fn test_query_by_secret_name() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");

        let logger = AuditLogger::new(log_path.clone()).unwrap();
        logger.log_operation(AuditOperation::Store, "secret1", true, None).unwrap();
        logger.log_operation(AuditOperation::Get, "secret1", true, None).unwrap();
        logger.log_operation(AuditOperation::Store, "secret2", true, None).unwrap();

        let filter = AuditFilter {
            secret_name: Some("secret1".to_string()),
            ..Default::default()
        };

        let entries = logger.query_log(&filter).unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.secret_name == "secret1"));
    }
}
