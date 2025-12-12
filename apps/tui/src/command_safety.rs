//! Command Safety Module
//!
//! Provides safe command execution with three-tier classification:
//! - **Safe**: Read-only commands that auto-execute (ls, git status, cat, etc.)
//! - **Dangerous**: Commands that modify state and require confirmation (rm, sudo, npm install, etc.)
//! - **Blocked**: Commands that are never allowed (fork bombs, disk operations, etc.)
//!
//! # Examples
//!
//! ```rust
//! use radium_tui::command_safety::{CommandSafety, CommandClassification};
//!
//! let safety = CommandSafety::default();
//! let analysis = safety.analyze("git status");
//!
//! match analysis.classification {
//!     CommandClassification::Safe => {
//!         // Execute immediately
//!     }
//!     CommandClassification::Dangerous => {
//!         // Request user confirmation
//!     }
//!     CommandClassification::Blocked => {
//!         // Reject immediately
//!     }
//! }
//! ```

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Classification of command safety levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandClassification {
    /// Safe read-only command that can auto-execute
    Safe,
    /// Dangerous command that requires user confirmation
    Dangerous,
    /// Blocked command that should never execute
    Blocked,
}

/// Result of analyzing a command's safety
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandAnalysis {
    pub classification: CommandClassification,
    pub root_command: String,
    pub full_command: String,
    pub danger_reason: Option<String>,
}

/// Command safety classifier
pub struct CommandSafety {
    safe_commands: HashSet<String>,
    dangerous_commands: HashSet<String>,
    blocked_commands: HashSet<String>,
}

impl Default for CommandSafety {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandSafety {
    /// Create a new CommandSafety instance with default command lists
    pub fn new() -> Self {
        let safe_commands = Self::default_safe_commands();
        let dangerous_commands = Self::default_dangerous_commands();
        let blocked_commands = Self::default_blocked_commands();

        Self {
            safe_commands,
            dangerous_commands,
            blocked_commands,
        }
    }

    /// Default safe commands (read-only operations)
    fn default_safe_commands() -> HashSet<String> {
        vec![
            // File inspection
            "ls", "cat", "head", "tail", "less", "more", "file", "stat",
            "pwd", "which", "whereis", "find", "locate", "tree",

            // Git read-only
            "git status", "git log", "git diff", "git show", "git branch",
            "git tag", "git remote", "git config --get", "git config --list",
            "git rev-parse", "git describe", "git reflog",

            // Text processing
            "grep", "egrep", "fgrep", "rg", "ag", "ack",
            "wc", "sort", "uniq", "cut", "tr", "sed -n",
            "awk", "column", "jq", "yq",

            // System info
            "date", "whoami", "hostname", "uname", "uptime",
            "df -h", "df", "du -h", "du", "free -h", "free",
            "env", "printenv", "id", "groups",

            // Process inspection
            "ps", "top -n 1", "pgrep", "jobs", "pstree",

            // Network inspection
            "ping -c", "curl -I", "wget --spider", "dig", "nslookup",
            "ifconfig", "ip addr", "netstat", "ss",

            // Package inspection
            "npm list", "npm ls", "pip list", "pip freeze",
            "cargo tree", "cargo search", "gem list",
            "apt list", "brew list", "yum list",

            // Rust specific
            "cargo check", "cargo test --no-run", "cargo fmt -- --check",
            "cargo clippy -- -D warnings", "rustc --version", "rustup show",

            // Build inspection (no execution)
            "make -n", "cmake --version", "gcc --version", "clang --version",

            // Documentation
            "man", "help", "which", "type", "alias",

            // Archive inspection
            "tar -tf", "zip -l", "unzip -l", "7z l",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    /// Default dangerous commands (require confirmation)
    fn default_dangerous_commands() -> HashSet<String> {
        vec![
            // File modification
            "rm", "rmdir", "mv", "cp", "touch", "mkdir",
            "chmod", "chown", "chgrp", "ln", "unlink",

            // Git write operations
            "git add", "git commit", "git push", "git pull",
            "git merge", "git rebase", "git reset", "git checkout",
            "git cherry-pick", "git stash", "git clean", "git gc",

            // System modification
            "sudo", "su", "kill", "killall", "pkill", "xkill",
            "reboot", "shutdown", "halt", "poweroff",

            // Network operations
            "curl", "wget", "ssh", "scp", "rsync", "sftp", "ftp",
            "nc", "netcat", "telnet",

            // Package management
            "npm install", "npm uninstall", "npm update", "npm ci",
            "pip install", "pip uninstall", "pip install --upgrade",
            "cargo install", "cargo uninstall", "cargo update",
            "apt install", "apt remove", "apt update", "apt upgrade",
            "yum install", "yum remove", "yum update",
            "brew install", "brew uninstall", "brew upgrade",
            "gem install", "gem uninstall",

            // Compilation and build
            "make", "cmake", "gcc", "clang", "rustc", "cargo build",
            "npm run", "npm start", "npm build",

            // Shell operations
            "source", "eval", "exec", "bash", "sh", "zsh", "fish",

            // Docker/Container
            "docker run", "docker exec", "docker build", "docker rm",
            "docker-compose up", "docker-compose down",
            "podman run", "podman exec",

            // Database operations
            "psql", "mysql", "sqlite3", "mongo", "redis-cli",

            // Archive operations
            "tar -x", "unzip", "7z x", "gunzip", "bunzip2",

            // System tools
            "crontab", "systemctl", "service", "launchctl",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    /// Default blocked commands (never execute)
    fn default_blocked_commands() -> HashSet<String> {
        vec![
            // Fork bombs and malicious patterns
            ":(){ :|:& };:",
            "fork while fork",

            // Disk operations
            "mkfs", "fdisk", "parted", "gparted", "diskutil",
            "dd if=/dev/random", "dd if=/dev/zero",
            "shred", "wipe",

            // Dangerous system modifications
            "rm -rf /", "rm -rf /*", "rm -rf ~/*",
            "chmod -R 777 /", "chown -R",

            // Kernel modifications
            "insmod", "modprobe", "rmmod",

            // Boot loader
            "grub-install", "update-grub",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    /// Analyze a command and classify its safety
    pub fn analyze(&self, command: &str) -> CommandAnalysis {
        let roots = extract_command_roots(command);

        // Check if any root command is blocked
        for root in &roots {
            if self.is_blocked(root) {
                return CommandAnalysis {
                    classification: CommandClassification::Blocked,
                    root_command: root.clone(),
                    full_command: command.to_string(),
                    danger_reason: Some(format!(
                        "Command '{}' is blocked for safety. This command can cause severe system damage.",
                        root
                    )),
                };
            }
        }

        // Check for exact command matches in blocked list (for patterns like fork bombs)
        if self.blocked_commands.contains(command.trim()) {
            return CommandAnalysis {
                classification: CommandClassification::Blocked,
                root_command: command.to_string(),
                full_command: command.to_string(),
                danger_reason: Some("This command pattern is blocked for safety.".to_string()),
            };
        }

        // Check if any root command is dangerous
        for root in &roots {
            if self.is_dangerous(root) {
                return CommandAnalysis {
                    classification: CommandClassification::Dangerous,
                    root_command: root.clone(),
                    full_command: command.to_string(),
                    danger_reason: Some(format!(
                        "'{}' can modify system state or execute code. User confirmation required.",
                        root
                    )),
                };
            }

            // Check for dangerous command with arguments (e.g., "git push")
            for dangerous_cmd in &self.dangerous_commands {
                if command.trim().starts_with(dangerous_cmd) {
                    return CommandAnalysis {
                        classification: CommandClassification::Dangerous,
                        root_command: root.clone(),
                        full_command: command.to_string(),
                        danger_reason: Some(format!(
                            "'{}' can modify system state. User confirmation required.",
                            dangerous_cmd
                        )),
                    };
                }
            }
        }

        // Check if command matches any safe command pattern
        for safe_cmd in &self.safe_commands {
            if command.trim().starts_with(safe_cmd) || roots.iter().any(|r| safe_cmd.starts_with(r)) {
                let first_root = roots.first().cloned().unwrap_or_default();
                return CommandAnalysis {
                    classification: CommandClassification::Safe,
                    root_command: first_root,
                    full_command: command.to_string(),
                    danger_reason: None,
                };
            }
        }

        // Default to safe if all roots are recognized as safe
        let first_root = roots.first().cloned().unwrap_or_default();
        if roots.iter().all(|r| self.safe_commands.contains(r)) {
            CommandAnalysis {
                classification: CommandClassification::Safe,
                root_command: first_root,
                full_command: command.to_string(),
                danger_reason: None,
            }
        } else {
            // Unknown commands are treated as dangerous by default (conservative)
            CommandAnalysis {
                classification: CommandClassification::Dangerous,
                root_command: first_root.clone(),
                full_command: command.to_string(),
                danger_reason: Some(format!(
                    "Unknown command '{}'. Confirmation required for safety.",
                    first_root
                )),
            }
        }
    }

    /// Check if a command root is blocked
    fn is_blocked(&self, command_root: &str) -> bool {
        self.blocked_commands.contains(command_root)
    }

    /// Check if a command root is dangerous
    fn is_dangerous(&self, command_root: &str) -> bool {
        self.dangerous_commands.contains(command_root)
    }

    /// Check if a command root is safe
    #[allow(dead_code)]
    fn is_safe(&self, command_root: &str) -> bool {
        self.safe_commands.contains(command_root)
    }

    /// Add a custom safe command
    pub fn add_safe_command(&mut self, command: String) {
        self.safe_commands.insert(command);
    }

    /// Add a custom dangerous command
    pub fn add_dangerous_command(&mut self, command: String) {
        self.dangerous_commands.insert(command);
    }

    /// Add a custom blocked command
    pub fn add_blocked_command(&mut self, command: String) {
        self.blocked_commands.insert(command);
    }

    /// Load user's command allowlist from disk
    pub fn load_allowlist(workspace_root: &Path) -> Result<HashSet<String>> {
        let allowlist_path = workspace_root.join(".radium/_internals/command_allowlist.json");

        if allowlist_path.exists() {
            let content = fs::read_to_string(allowlist_path)?;
            let allowlist: Vec<String> = serde_json::from_str(&content)?;
            Ok(allowlist.into_iter().collect())
        } else {
            Ok(HashSet::new())
        }
    }

    /// Save user's command allowlist to disk
    pub fn save_allowlist(allowlist: &HashSet<String>, workspace_root: &Path) -> Result<()> {
        let allowlist_path = workspace_root.join(".radium/_internals/command_allowlist.json");

        // Create directory if it doesn't exist
        if let Some(parent) = allowlist_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let allowlist_vec: Vec<_> = allowlist.iter().cloned().collect();
        let content = serde_json::to_string_pretty(&allowlist_vec)?;
        fs::write(allowlist_path, content)?;

        Ok(())
    }
}

/// Extract root commands from a full command string
///
/// Splits the command by pipes (|), semicolons (;), and double ampersands (&&),
/// then extracts the first token of each segment.
///
/// # Examples
///
/// ```
/// use radium_tui::command_safety::extract_command_roots;
///
/// let roots = extract_command_roots("ls -la | grep foo");
/// assert_eq!(roots, vec!["ls", "grep"]);
///
/// let roots = extract_command_roots("cd /tmp && rm file.txt");
/// assert_eq!(roots, vec!["cd", "rm"]);
/// ```
pub fn extract_command_roots(command: &str) -> Vec<String> {
    let mut roots = Vec::new();

    // Split by common command separators
    let segments = command.split(&['|', ';'][..]);

    for segment in segments {
        // Further split by && (logical AND)
        let sub_segments = segment.split("&&");

        for sub_segment in sub_segments {
            let trimmed = sub_segment.trim();
            if let Some(first_token) = trimmed.split_whitespace().next() {
                if !first_token.is_empty() {
                    roots.push(first_token.to_string());
                }
            }
        }
    }

    roots
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_command_classification() {
        let safety = CommandSafety::new();
        let analysis = safety.analyze("ls -la");
        assert_eq!(analysis.classification, CommandClassification::Safe);
        assert!(analysis.danger_reason.is_none());
    }

    #[test]
    fn test_dangerous_command_classification() {
        let safety = CommandSafety::new();
        let analysis = safety.analyze("rm -rf /tmp/file");
        assert_eq!(analysis.classification, CommandClassification::Dangerous);
        assert!(analysis.danger_reason.is_some());
    }

    #[test]
    fn test_blocked_command_classification() {
        let safety = CommandSafety::new();
        let analysis = safety.analyze(":(){ :|:& };:");
        assert_eq!(analysis.classification, CommandClassification::Blocked);
        assert!(analysis.danger_reason.is_some());
    }

    #[test]
    fn test_pipe_command_safe() {
        let safety = CommandSafety::new();
        let analysis = safety.analyze("cat file.txt | grep pattern");
        assert_eq!(analysis.classification, CommandClassification::Safe);
    }

    #[test]
    fn test_pipe_command_mixed_dangerous() {
        let safety = CommandSafety::new();
        let analysis = safety.analyze("cat file.txt | rm dangerous.txt");
        assert_eq!(analysis.classification, CommandClassification::Dangerous);
    }

    #[test]
    fn test_command_root_extraction() {
        let roots = extract_command_roots("ls -la && git status");
        assert_eq!(roots, vec!["ls", "git"]);
    }

    #[test]
    fn test_command_root_extraction_pipes() {
        let roots = extract_command_roots("ps aux | grep rust | wc -l");
        assert_eq!(roots, vec!["ps", "grep", "wc"]);
    }

    #[test]
    fn test_git_safe_commands() {
        let safety = CommandSafety::new();

        assert_eq!(
            safety.analyze("git status").classification,
            CommandClassification::Safe
        );
        assert_eq!(
            safety.analyze("git log").classification,
            CommandClassification::Safe
        );
        assert_eq!(
            safety.analyze("git diff").classification,
            CommandClassification::Safe
        );
    }

    #[test]
    fn test_git_dangerous_commands() {
        let safety = CommandSafety::new();

        assert_eq!(
            safety.analyze("git push").classification,
            CommandClassification::Dangerous
        );
        assert_eq!(
            safety.analyze("git commit -m 'test'").classification,
            CommandClassification::Dangerous
        );
    }

    #[test]
    fn test_unknown_command_conservative() {
        let safety = CommandSafety::new();
        let analysis = safety.analyze("someunknowncommand --flag");
        // Unknown commands should be treated as dangerous (conservative)
        assert_eq!(analysis.classification, CommandClassification::Dangerous);
    }

    #[test]
    fn test_custom_safe_command() {
        let mut safety = CommandSafety::new();
        safety.add_safe_command("customsafe".to_string());

        let analysis = safety.analyze("customsafe --flag");
        assert_eq!(analysis.classification, CommandClassification::Safe);
    }

    #[test]
    fn test_custom_dangerous_command() {
        let mut safety = CommandSafety::new();
        safety.add_dangerous_command("customdangerous".to_string());

        let analysis = safety.analyze("customdangerous");
        assert_eq!(analysis.classification, CommandClassification::Dangerous);
    }

    #[test]
    fn test_disk_operations_blocked() {
        let safety = CommandSafety::new();

        assert_eq!(
            safety.analyze("mkfs").classification,
            CommandClassification::Blocked
        );
        assert_eq!(
            safety.analyze("fdisk").classification,
            CommandClassification::Blocked
        );
    }
}
