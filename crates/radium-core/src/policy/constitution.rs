//! Session-based constitution system for per-session rule enforcement.
//!
//! Provides lightweight "constitution" rules that are enforced per session ID.
//! Rules are automatically cleaned up after TTL expiration to prevent memory leaks.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::time::{Duration, interval};

/// Maximum number of rules per session.
const MAX_RULES_PER_SESSION: usize = 50;

/// Time-to-live for session constitutions (1 hour).
const SESSION_TTL_MS: u64 = 60 * 60 * 1000;

/// Constitution entry for a session.
#[derive(Debug, Clone)]
struct ConstitutionEntry {
    /// Rules for this session.
    rules: Vec<String>,
    /// Last update timestamp.
    updated: DateTime<Utc>,
}

impl ConstitutionEntry {
    /// Creates a new constitution entry.
    fn new() -> Self {
        Self { rules: Vec::new(), updated: Utc::now() }
    }

    /// Checks if this entry is stale (past TTL).
    fn is_stale(&self) -> bool {
        let now = Utc::now();
        let elapsed = now.signed_duration_since(self.updated);
        elapsed.num_milliseconds() as u64 > SESSION_TTL_MS
    }
}

/// Manager for session constitutions.
///
/// Thread-safe manager that stores per-session rules and automatically
/// cleans up stale entries.
#[derive(Clone)]
pub struct ConstitutionManager {
    /// Map of session ID to constitution entry.
    constitutions: Arc<RwLock<HashMap<String, ConstitutionEntry>>>,
}

impl ConstitutionManager {
    /// Creates a new constitution manager.
    pub fn new() -> Self {
        let manager = Self { constitutions: Arc::new(RwLock::new(HashMap::new())) };

        // Start cleanup task
        manager.start_cleanup_task();

        manager
    }

    /// Updates the constitution for a session by adding or merging rules.
    ///
    /// # Arguments
    /// * `session_id` - The session identifier
    /// * `rule` - The rule to add
    ///
    /// Rules are appended to the session's rule list, up to MAX_RULES_PER_SESSION.
    pub fn update_constitution(&self, session_id: &str, rule: String) {
        if session_id.is_empty() || rule.is_empty() {
            return;
        }

        let mut constitutions = self.constitutions.write().unwrap();
        let entry =
            constitutions.entry(session_id.to_string()).or_insert_with(ConstitutionEntry::new);

        // Enforce max rules limit (remove oldest if at limit)
        if entry.rules.len() >= MAX_RULES_PER_SESSION {
            entry.rules.remove(0);
        }

        entry.rules.push(rule);
        entry.updated = Utc::now();
    }

    /// Resets the constitution for a session with new rules.
    ///
    /// # Arguments
    /// * `session_id` - The session identifier
    /// * `rules` - The new rules to set
    pub fn reset_constitution(&self, session_id: &str, rules: Vec<String>) {
        if session_id.is_empty() {
            return;
        }

        let mut constitutions = self.constitutions.write().unwrap();
        let entry =
            constitutions.entry(session_id.to_string()).or_insert_with(ConstitutionEntry::new);

        // Limit to MAX_RULES_PER_SESSION
        let rules: Vec<String> = rules.into_iter().take(MAX_RULES_PER_SESSION).collect();
        entry.rules = rules;
        entry.updated = Utc::now();
    }

    /// Gets the effective rules for a session.
    ///
    /// # Arguments
    /// * `session_id` - The session identifier
    ///
    /// # Returns
    /// Vector of rules for the session, or empty vector if session not found
    pub fn get_constitution(&self, session_id: &str) -> Vec<String> {
        let constitutions = self.constitutions.read().unwrap();
        let _entry = match constitutions.get(session_id) {
            Some(entry) => entry,
            None => return vec![],
        };

        // Update timestamp on access
        drop(constitutions);
        let mut constitutions = self.constitutions.write().unwrap();
        if let Some(entry) = constitutions.get_mut(session_id) {
            entry.updated = Utc::now();
            entry.rules.clone()
        } else {
            vec![]
        }
    }

    /// Cleans up stale session constitutions.
    ///
    /// Removes entries that haven't been accessed within the TTL period.
    fn cleanup_stale_sessions(&self) {
        let mut constitutions = self.constitutions.write().unwrap();
        constitutions.retain(|_, entry| !entry.is_stale());
    }

    /// Starts the background cleanup task.
    fn start_cleanup_task(&self) {
        let manager = self.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(SESSION_TTL_MS));
            loop {
                interval.tick().await;
                manager.cleanup_stale_sessions();
            }
        });
    }
}

impl Default for ConstitutionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constitution_manager_new() {
        let manager = ConstitutionManager::new();
        let rules = manager.get_constitution("test-session");
        assert!(rules.is_empty());
    }

    #[test]
    fn test_update_constitution() {
        let manager = ConstitutionManager::new();
        manager.update_constitution("session-1", "no external network calls".to_string());
        manager.update_constitution("session-1", "prefer unit tests".to_string());

        let rules = manager.get_constitution("session-1");
        assert_eq!(rules.len(), 2);
        assert!(rules.contains(&"no external network calls".to_string()));
        assert!(rules.contains(&"prefer unit tests".to_string()));
    }

    #[test]
    fn test_update_constitution_empty_session() {
        let manager = ConstitutionManager::new();
        manager.update_constitution("", "rule".to_string());
        let rules = manager.get_constitution("");
        assert!(rules.is_empty());
    }

    #[test]
    fn test_update_constitution_empty_rule() {
        let manager = ConstitutionManager::new();
        manager.update_constitution("session-1", "".to_string());
        let rules = manager.get_constitution("session-1");
        assert!(rules.is_empty());
    }

    #[test]
    fn test_reset_constitution() {
        let manager = ConstitutionManager::new();
        manager.update_constitution("session-1", "old rule".to_string());
        manager.reset_constitution(
            "session-1",
            vec!["new rule 1".to_string(), "new rule 2".to_string()],
        );

        let rules = manager.get_constitution("session-1");
        assert_eq!(rules.len(), 2);
        assert!(!rules.contains(&"old rule".to_string()));
        assert!(rules.contains(&"new rule 1".to_string()));
        assert!(rules.contains(&"new rule 2".to_string()));
    }

    #[test]
    fn test_reset_constitution_max_rules() {
        let manager = ConstitutionManager::new();
        let many_rules: Vec<String> =
            (0..=MAX_RULES_PER_SESSION).map(|i| format!("rule-{}", i)).collect();
        manager.reset_constitution("session-1", many_rules);

        let rules = manager.get_constitution("session-1");
        assert_eq!(rules.len(), MAX_RULES_PER_SESSION);
    }

    #[test]
    fn test_get_constitution_nonexistent() {
        let manager = ConstitutionManager::new();
        let rules = manager.get_constitution("nonexistent");
        assert!(rules.is_empty());
    }

    #[test]
    fn test_constitution_entry_is_stale() {
        let mut entry = ConstitutionEntry::new();
        assert!(!entry.is_stale());

        // Simulate stale entry by setting old timestamp
        entry.updated = Utc::now() - chrono::Duration::milliseconds(SESSION_TTL_MS as i64 + 1000);
        assert!(entry.is_stale());
    }

    #[test]
    fn test_update_constitution_max_rules() {
        let manager = ConstitutionManager::new();
        for i in 0..=MAX_RULES_PER_SESSION {
            manager.update_constitution("session-1", format!("rule-{}", i));
        }

        let rules = manager.get_constitution("session-1");
        assert_eq!(rules.len(), MAX_RULES_PER_SESSION);
        // First rule should be removed
        assert!(!rules.contains(&"rule-0".to_string()));
        // Last rule should be present
        assert!(rules.contains(&format!("rule-{}", MAX_RULES_PER_SESSION)));
    }
}
