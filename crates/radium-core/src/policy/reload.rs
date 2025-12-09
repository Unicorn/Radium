//! Hot-reload mechanism for policy rules.

use super::rules::PolicyEngine;
use super::types::{ApprovalMode, PolicyError, PolicyResult};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Policy reloader that watches for file changes and hot-reloads rules.
pub struct PolicyReloader {
    /// Path to the policy file being watched.
    policy_file: PathBuf,
    /// The policy engine (wrapped in Arc<RwLock> for thread-safe updates).
    engine: Arc<RwLock<PolicyEngine>>,
    /// File system watcher.
    watcher: RecommendedWatcher,
    /// Current rules snapshot for rollback.
    rollback_snapshot: Option<PolicyEngine>,
}

impl PolicyReloader {
    /// Creates a new policy reloader.
    ///
    /// # Arguments
    /// * `policy_file` - Path to the policy TOML file to watch
    /// * `engine` - Policy engine to update (wrapped in Arc<RwLock>)
    ///
    /// # Returns
    /// A new `PolicyReloader` that watches the file for changes.
    pub fn new(
        policy_file: impl AsRef<Path>,
        engine: Arc<RwLock<PolicyEngine>>,
    ) -> PolicyResult<Self> {
        let policy_file = policy_file.as_ref().to_path_buf();

        // Create file watcher
        let mut watcher = notify::recommended_watcher(Self::create_event_handler(Arc::clone(&engine), policy_file.clone()))
            .map_err(|e| PolicyError::InvalidConfig(format!("Failed to create file watcher: {}", e)))?;

        // Watch the policy file
        watcher.watch(&policy_file, RecursiveMode::NonRecursive)
            .map_err(|e| PolicyError::InvalidConfig(format!("Failed to watch policy file: {}", e)))?;

        info!(policy_file = %policy_file.display(), "Started watching policy file for changes");

        Ok(Self {
            policy_file,
            engine,
            watcher,
            rollback_snapshot: None,
        })
    }

    /// Creates an event handler for file system events.
    fn create_event_handler(
        engine: Arc<RwLock<PolicyEngine>>,
        policy_file: PathBuf,
    ) -> impl Fn(Result<Event, notify::Error>) + Send + Sync + 'static {
        move |result: Result<Event, notify::Error>| {
            match result {
                Ok(event) => {
                    // Only process write/modify events for the policy file
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        for path in &event.paths {
                            if path == &policy_file {
                                // Spawn async task to reload
                                let engine_clone = Arc::clone(&engine);
                                let file_clone = policy_file.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = Self::reload_policy(&engine_clone, &file_clone).await {
                                        let error_msg = format!("Failed to reload policy file: {}", e);
                                        tracing::error!("{}", error_msg);
                                    }
                                });
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!(error = %e, "File watcher error");
                }
            }
        }
    }

    /// Reloads policy rules from the file.
    ///
    /// # Arguments
    /// * `engine` - Policy engine to update
    /// * `policy_file` - Path to the policy file
    async fn reload_policy(engine: &Arc<RwLock<PolicyEngine>>, policy_file: &Path) -> PolicyResult<()> {
        info!(policy_file = %policy_file.display(), "Policy file changed, reloading...");

        // Save current state for rollback
        let current_engine = engine.read().await;
        let rollback_snapshot = PolicyEngine {
            approval_mode: current_engine.approval_mode(),
            rules: current_engine.rules().to_vec(),
            hook_registry: None, // Don't copy hook registry
            alert_manager: None, // Don't copy alert manager
            analytics: None, // Don't copy analytics
        };
        drop(current_engine);

        // Try to load new policy
        match PolicyEngine::from_file(policy_file) {
            Ok(mut new_engine) => {
                // Validate the new engine
                if let Err(e) = Self::validate_policy(&new_engine) {
                    error!(
                        error = %e,
                        "Policy validation failed, rolling back to previous rules"
                    );
                    // Rollback
                    let mut engine_write = engine.write().await;
                    *engine_write = rollback_snapshot;
                    return Err(e);
                }

                // Preserve hook registry, alert manager, and analytics from current engine
                let current_engine = engine.read().await;
                // Note: We can't easily preserve these without making PolicyEngine Clone,
                // so for now we'll just update the rules and approval mode
                drop(current_engine);

                // Apply new rules atomically
                {
                    let mut engine_write = engine.write().await;
                    engine_write.update_from(new_engine);
                    // Note: hook_registry, alert_manager, analytics are preserved
                }

                let rule_count = engine.read().await.rules().len();
                info!(
                    rule_count = rule_count,
                    "Policy rules reloaded successfully"
                );
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                error!(
                    error = %error_msg,
                    "Failed to parse policy file, rolling back to previous rules"
                );
                // Rollback
                let mut engine_write = engine.write().await;
                *engine_write = rollback_snapshot;
                Err(e)
            }
        }
    }

    /// Validates a policy engine configuration.
    fn validate_policy(engine: &PolicyEngine) -> PolicyResult<()> {
        // Check for conflicts
        if let Ok(conflicts) = engine.detect_conflicts() {
            if !conflicts.is_empty() {
                return Err(PolicyError::InvalidConfig(format!(
                    "Policy contains {} conflict(s)",
                    conflicts.len()
                )));
            }
        }

        // Validate rule patterns
        for rule in engine.rules() {
            // Try to match a dummy pattern to validate glob syntax
            if let Err(e) = rule.matches("test_tool", &["test_arg"]) {
                return Err(PolicyError::InvalidConfig(format!(
                    "Invalid pattern in rule '{}': {}",
                    rule.name, e
                )));
            }
        }

        Ok(())
    }

    /// Manually triggers a reload (useful for testing or API calls).
    pub async fn reload(&self) -> PolicyResult<()> {
        Self::reload_policy(&self.engine, &self.policy_file).await
    }

    /// Gets the path to the policy file being watched.
    pub fn policy_file(&self) -> &Path {
        &self.policy_file
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_policy_reload_success() {
        let temp_dir = TempDir::new().unwrap();
        let policy_file = temp_dir.path().join("policy.toml");

        // Create initial policy
        let initial_policy = r#"
approval_mode = "ask"

[[rules]]
name = "allow-reads"
tool_pattern = "read_*"
action = "allow"
"#;
        std::fs::write(&policy_file, initial_policy).unwrap();

        let engine = Arc::new(RwLock::new(
            PolicyEngine::from_file(&policy_file).unwrap()
        ));

        let reloader = PolicyReloader::new(&policy_file, Arc::clone(&engine)).unwrap();

        // Modify policy
        let new_policy = r#"
approval_mode = "ask"

[[rules]]
name = "allow-reads"
tool_pattern = "read_*"
action = "allow"

[[rules]]
name = "deny-writes"
tool_pattern = "write_*"
action = "deny"
"#;
        std::fs::write(&policy_file, new_policy).unwrap();

        // Wait a bit for file system event
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Manually trigger reload
        reloader.reload().await.unwrap();

        // Verify new rules are loaded
        let engine_read = engine.read().await;
        assert_eq!(engine_read.rules().len(), 2);
    }

    #[tokio::test]
    async fn test_policy_reload_rollback_on_invalid() {
        let temp_dir = TempDir::new().unwrap();
        let policy_file = temp_dir.path().join("policy.toml");

        // Create initial valid policy
        let initial_policy = r#"
approval_mode = "ask"

[[rules]]
name = "allow-reads"
tool_pattern = "read_*"
action = "allow"
"#;
        std::fs::write(&policy_file, initial_policy).unwrap();

        let engine = Arc::new(RwLock::new(
            PolicyEngine::from_file(&policy_file).unwrap()
        ));
        let initial_rule_count = engine.read().await.rules().len();

        let reloader = PolicyReloader::new(&policy_file, Arc::clone(&engine)).unwrap();

        // Write invalid policy
        std::fs::write(&policy_file, "invalid toml syntax {").unwrap();

        // Try to reload (should fail and rollback)
        let result = reloader.reload().await;
        assert!(result.is_err());

        // Verify original rules are still active
        let engine_read = engine.read().await;
        assert_eq!(engine_read.rules().len(), initial_rule_count);
    }
}

