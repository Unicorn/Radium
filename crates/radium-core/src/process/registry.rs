//! Process registry for managing spawned processes.

use anyhow::{Context, Result, anyhow};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command as TokioCommand};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Status of a spawned process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessStatus {
    /// Process is currently running.
    Running,
    /// Process has stopped cleanly.
    Stopped,
    /// Process has crashed (non-zero exit code).
    Crashed,
    /// Process is being restarted.
    Restarting,
}

impl std::fmt::Display for ProcessStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessStatus::Running => write!(f, "Running"),
            ProcessStatus::Stopped => write!(f, "Stopped"),
            ProcessStatus::Crashed => write!(f, "Crashed"),
            ProcessStatus::Restarting => write!(f, "Restarting"),
        }
    }
}

/// Handle to a spawned process with metadata.
#[derive(Debug, Clone)]
pub struct ProcessHandle {
    /// Unique identifier for this process.
    pub id: String,
    /// System process ID.
    pub pid: Option<u32>,
    /// Command used to spawn the process.
    pub command: String,
    /// Arguments passed to the command.
    pub args: Vec<String>,
    /// Working directory.
    pub working_dir: PathBuf,
    /// Current status.
    pub status: ProcessStatus,
    /// When the process was spawned.
    pub spawned_at: SystemTime,
    /// When the process exited (if applicable).
    pub exited_at: Option<SystemTime>,
    /// Exit code (if process has exited).
    pub exit_code: Option<i32>,
    /// Number of times the process has been restarted.
    pub restart_count: u32,
    /// Path to log file where stdout/stderr is captured.
    pub log_file: PathBuf,
    /// Whether shutdown was explicitly requested.
    pub shutdown_requested: bool,
}

impl ProcessHandle {
    /// Returns the uptime of the process in seconds.
    pub fn uptime_seconds(&self) -> u64 {
        if let Ok(duration) = SystemTime::now().duration_since(self.spawned_at) {
            duration.as_secs()
        } else {
            0
        }
    }

    /// Returns a human-readable uptime string.
    pub fn uptime_display(&self) -> String {
        let seconds = self.uptime_seconds();
        if seconds < 60 {
            format!("{}s", seconds)
        } else if seconds < 3600 {
            format!("{}m {}s", seconds / 60, seconds % 60)
        } else {
            format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
        }
    }
}

/// Internal process state with the child handle.
#[allow(dead_code)]
struct ProcessState {
    handle: ProcessHandle,
    child: Option<Child>,
    log_sender: Option<mpsc::Sender<String>>,
}

/// Registry for managing spawned processes.
pub struct ProcessRegistry {
    /// Map of process ID to process state.
    processes: Arc<RwLock<HashMap<String, ProcessState>>>,
    /// Directory where log files are stored.
    log_dir: PathBuf,
}

impl ProcessRegistry {
    /// Creates a new process registry.
    ///
    /// # Arguments
    /// * `log_dir` - Directory where process logs will be stored
    pub fn new<P: AsRef<Path>>(log_dir: P) -> Result<Self> {
        let log_dir = log_dir.as_ref().to_path_buf();

        // Create log directory if it doesn't exist
        if !log_dir.exists() {
            std::fs::create_dir_all(&log_dir)
                .context("Failed to create log directory")?;
        }

        Ok(Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            log_dir,
        })
    }

    /// Spawns a new process and registers it.
    ///
    /// # Arguments
    /// * `command` - Command to execute
    /// * `args` - Command arguments
    /// * `working_dir` - Working directory for the process
    ///
    /// # Returns
    /// The process ID that can be used to manage the process
    pub async fn spawn_process(
        &self,
        command: impl Into<String>,
        args: Vec<String>,
        working_dir: impl Into<PathBuf>,
    ) -> Result<String> {
        let command = command.into();
        let working_dir = working_dir.into();
        let process_id = Uuid::new_v4().to_string();

        info!(
            process_id = %process_id,
            command = %command,
            args = ?args,
            working_dir = ?working_dir,
            "Spawning process"
        );

        // Create log file
        let log_file = self.log_dir.join(format!("{}.log", process_id));
        let log_file_clone = log_file.clone();

        // Create channel for log lines
        let (log_tx, mut log_rx) = mpsc::channel::<String>(1000);

        // Spawn background task to write logs to file
        tokio::spawn(async move {
            let file = match tokio::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file_clone)
                .await
            {
                Ok(f) => f,
                Err(e) => {
                    error!("Failed to open log file: {}", e);
                    return;
                }
            };

            let mut writer = tokio::io::BufWriter::new(file);

            while let Some(line) = log_rx.recv().await {
                use tokio::io::AsyncWriteExt;
                if let Err(e) = writer.write_all(line.as_bytes()).await {
                    error!("Failed to write to log file: {}", e);
                    break;
                }
                if let Err(e) = writer.write_all(b"\n").await {
                    error!("Failed to write newline to log file: {}", e);
                    break;
                }
                if let Err(e) = writer.flush().await {
                    error!("Failed to flush log file: {}", e);
                    break;
                }
            }
        });

        // Spawn the process
        let mut cmd = TokioCommand::new(&command);
        cmd.args(&args)
            .current_dir(&working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let mut child = cmd.spawn()
            .context(format!("Failed to spawn process: {}", command))?;

        let pid = child.id();

        debug!(
            process_id = %process_id,
            pid = ?pid,
            "Process spawned successfully"
        );

        // Capture stdout
        if let Some(stdout) = child.stdout.take() {
            let log_tx_clone = log_tx.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = log_tx_clone.send(format!("[stdout] {}", line)).await;
                }
            });
        }

        // Capture stderr
        if let Some(stderr) = child.stderr.take() {
            let log_tx_clone = log_tx.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = log_tx_clone.send(format!("[stderr] {}", line)).await;
                }
            });
        }

        // Create process handle
        let handle = ProcessHandle {
            id: process_id.clone(),
            pid,
            command: command.clone(),
            args: args.clone(),
            working_dir,
            status: ProcessStatus::Running,
            spawned_at: SystemTime::now(),
            exited_at: None,
            exit_code: None,
            restart_count: 0,
            log_file,
            shutdown_requested: false,
        };

        // Store in registry
        let state = ProcessState {
            handle: handle.clone(),
            child: Some(child),
            log_sender: Some(log_tx),
        };

        self.processes.write().await.insert(process_id.clone(), state);

        // Spawn task to monitor process exit
        let processes = self.processes.clone();
        let process_id_clone = process_id.clone();
        tokio::spawn(async move {
            // Wait for process to exit
            let exit_status = {
                let mut processes_guard = processes.write().await;
                if let Some(state) = processes_guard.get_mut(&process_id_clone) {
                    if let Some(child) = &mut state.child {
                        child.wait().await
                    } else {
                        return;
                    }
                } else {
                    return;
                }
            };

            // Update process state
            let mut processes_guard = processes.write().await;
            if let Some(state) = processes_guard.get_mut(&process_id_clone) {
                state.handle.exited_at = Some(SystemTime::now());

                match exit_status {
                    Ok(status) => {
                        state.handle.exit_code = status.code();

                        if state.handle.shutdown_requested {
                            state.handle.status = ProcessStatus::Stopped;
                            info!(
                                process_id = %process_id_clone,
                                exit_code = ?status.code(),
                                "Process stopped cleanly"
                            );
                        } else if status.success() {
                            state.handle.status = ProcessStatus::Stopped;
                            info!(
                                process_id = %process_id_clone,
                                "Process exited successfully"
                            );
                        } else {
                            state.handle.status = ProcessStatus::Crashed;
                            warn!(
                                process_id = %process_id_clone,
                                exit_code = ?status.code(),
                                "Process crashed"
                            );
                        }
                    }
                    Err(e) => {
                        state.handle.status = ProcessStatus::Crashed;
                        error!(
                            process_id = %process_id_clone,
                            error = %e,
                            "Failed to get process exit status"
                        );
                    }
                }

                // Clean up child handle
                state.child = None;
            }
        });

        Ok(process_id)
    }

    /// Kills a running process.
    ///
    /// # Arguments
    /// * `process_id` - ID of the process to kill
    pub async fn kill_process(&self, process_id: &str) -> Result<()> {
        info!(process_id = %process_id, "Killing process");

        let mut processes = self.processes.write().await;
        let state = processes.get_mut(process_id)
            .ok_or_else(|| anyhow!("Process not found: {}", process_id))?;

        state.handle.shutdown_requested = true;

        if let Some(child) = &mut state.child {
            child.kill().await
                .context("Failed to kill process")?;
            debug!(process_id = %process_id, "Process killed");
        } else {
            warn!(process_id = %process_id, "Process already exited");
        }

        Ok(())
    }

    /// Restarts a process by killing it and spawning a new one.
    ///
    /// # Arguments
    /// * `process_id` - ID of the process to restart
    ///
    /// # Returns
    /// The new process ID
    pub async fn restart_process(&self, process_id: &str) -> Result<String> {
        info!(process_id = %process_id, "Restarting process");

        // Get process info before killing
        let (command, args, working_dir, restart_count) = {
            let processes = self.processes.read().await;
            let state = processes.get(process_id)
                .ok_or_else(|| anyhow!("Process not found: {}", process_id))?;

            (
                state.handle.command.clone(),
                state.handle.args.clone(),
                state.handle.working_dir.clone(),
                state.handle.restart_count,
            )
        };

        // Kill the old process
        self.kill_process(process_id).await?;

        // Wait a bit for the process to fully terminate
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Spawn new process
        let new_process_id = self.spawn_process(command, args, working_dir).await?;

        // Update restart count
        {
            let mut processes = self.processes.write().await;
            if let Some(state) = processes.get_mut(&new_process_id) {
                state.handle.restart_count = restart_count + 1;
                state.handle.status = ProcessStatus::Running;
            }
        }

        debug!(
            old_process_id = %process_id,
            new_process_id = %new_process_id,
            restart_count = restart_count + 1,
            "Process restarted"
        );

        Ok(new_process_id)
    }

    /// Gets the status of a process.
    ///
    /// # Arguments
    /// * `process_id` - ID of the process
    pub async fn get_process_status(&self, process_id: &str) -> Option<ProcessStatus> {
        let processes = self.processes.read().await;
        processes.get(process_id).map(|state| state.handle.status.clone())
    }

    /// Gets a process handle by ID.
    ///
    /// # Arguments
    /// * `process_id` - ID of the process
    pub async fn get_process_handle(&self, process_id: &str) -> Option<ProcessHandle> {
        let processes = self.processes.read().await;
        processes.get(process_id).map(|state| state.handle.clone())
    }

    /// Lists all registered processes.
    pub async fn list_processes(&self) -> Vec<ProcessHandle> {
        let processes = self.processes.read().await;
        processes.values()
            .map(|state| state.handle.clone())
            .collect()
    }

    /// Removes a process from the registry.
    ///
    /// # Arguments
    /// * `process_id` - ID of the process to remove
    pub async fn remove_process(&self, process_id: &str) -> Result<()> {
        info!(process_id = %process_id, "Removing process from registry");

        let mut processes = self.processes.write().await;
        processes.remove(process_id)
            .ok_or_else(|| anyhow!("Process not found: {}", process_id))?;

        Ok(())
    }

    /// Shuts down all processes and cleans up.
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down process registry");

        let process_ids: Vec<String> = {
            let processes = self.processes.read().await;
            processes.keys().cloned().collect()
        };

        for process_id in process_ids {
            if let Err(e) = self.kill_process(&process_id).await {
                warn!(
                    process_id = %process_id,
                    error = %e,
                    "Failed to kill process during shutdown"
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_spawn_and_list_process() {
        let temp_dir = TempDir::new().unwrap();
        let registry = ProcessRegistry::new(temp_dir.path()).unwrap();

        // Spawn a simple process
        let process_id = registry
            .spawn_process("echo", vec!["hello".to_string()], temp_dir.path())
            .await
            .unwrap();

        // List processes
        let processes = registry.list_processes().await;
        assert_eq!(processes.len(), 1);
        assert_eq!(processes[0].id, process_id);
        assert_eq!(processes[0].command, "echo");

        // Wait for process to finish
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Check status changed to Stopped
        let status = registry.get_process_status(&process_id).await;
        assert_eq!(status, Some(ProcessStatus::Stopped));
    }

    #[tokio::test]
    async fn test_kill_process() {
        let temp_dir = TempDir::new().unwrap();
        let registry = ProcessRegistry::new(temp_dir.path()).unwrap();

        // Spawn a long-running process
        let process_id = registry
            .spawn_process("sleep", vec!["10".to_string()], temp_dir.path())
            .await
            .unwrap();

        // Verify it's running
        let status = registry.get_process_status(&process_id).await;
        assert_eq!(status, Some(ProcessStatus::Running));

        // Kill it
        registry.kill_process(&process_id).await.unwrap();

        // Wait a bit for status to update
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Check it's stopped
        let status = registry.get_process_status(&process_id).await;
        assert_eq!(status, Some(ProcessStatus::Stopped));
    }
}
