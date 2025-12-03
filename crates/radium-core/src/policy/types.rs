//! Core types for the policy engine.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Action to take for a tool execution request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyAction {
    /// Allow the tool execution without prompting.
    Allow,
    /// Deny the tool execution.
    Deny,
    /// Ask the user for approval before executing.
    AskUser,
}

/// Priority level for policy rules.
///
/// Higher priority rules override lower priority rules when multiple rules match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyPriority {
    /// Default system policies (lowest priority).
    Default = 0,
    /// User-defined policies (medium priority).
    User = 1,
    /// Administrator policies (highest priority).
    Admin = 2,
}

/// Approval mode for tool execution.
///
/// Determines the default behavior when no specific policy rule matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalMode {
    /// Auto-approve all tool executions (use with caution).
    Yolo,
    /// Auto-approve file edits, ask for other operations.
    AutoEdit,
    /// Ask for approval on all tool executions (safest, default).
    Ask,
}

impl Default for ApprovalMode {
    fn default() -> Self {
        Self::Ask
    }
}

impl ApprovalMode {
    /// Returns the default action for this approval mode.
    #[must_use]
    pub fn default_action(self) -> PolicyAction {
        match self {
            Self::Yolo => PolicyAction::Allow,
            Self::AutoEdit | Self::Ask => PolicyAction::AskUser,
        }
    }

    /// Checks if this mode auto-approves edit operations.
    #[must_use]
    pub fn auto_approves_edits(self) -> bool {
        matches!(self, Self::Yolo | Self::AutoEdit)
    }
}

/// Decision result from policy evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyDecision {
    /// The action to take.
    pub action: PolicyAction,
    /// Human-readable reason for the decision.
    pub reason: Option<String>,
    /// The rule that made this decision (if any).
    pub matched_rule: Option<String>,
}

impl PolicyDecision {
    /// Creates a new policy decision.
    pub fn new(action: PolicyAction) -> Self {
        Self { action, reason: None, matched_rule: None }
    }

    /// Adds a reason to the decision.
    #[must_use]
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Sets the matched rule.
    #[must_use]
    pub fn with_rule(mut self, rule_name: impl Into<String>) -> Self {
        self.matched_rule = Some(rule_name.into());
        self
    }

    /// Checks if the action is to allow execution.
    #[must_use]
    pub fn is_allowed(&self) -> bool {
        self.action == PolicyAction::Allow
    }

    /// Checks if the action is to deny execution.
    #[must_use]
    pub fn is_denied(&self) -> bool {
        self.action == PolicyAction::Deny
    }

    /// Checks if the action is to ask the user.
    #[must_use]
    pub fn requires_approval(&self) -> bool {
        self.action == PolicyAction::AskUser
    }
}

/// Errors that can occur during policy evaluation.
#[derive(Error, Debug)]
pub enum PolicyError {
    /// Failed to load policy file.
    #[error("Failed to load policy file at {path}: {source}")]
    LoadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to parse policy TOML.
    #[error("Failed to parse policy file at {path}: {source}")]
    ParseError {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    /// Invalid policy configuration.
    #[error("Invalid policy configuration: {0}")]
    InvalidConfig(String),

    /// Pattern matching error.
    #[error("Pattern matching error: {0}")]
    PatternError(String),
}

/// Result type alias for policy operations.
pub type PolicyResult<T> = std::result::Result<T, PolicyError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_action_serialization() {
        let action = PolicyAction::Allow;
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, "\"allow\"");

        let action = PolicyAction::Deny;
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, "\"deny\"");

        let action = PolicyAction::AskUser;
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, "\"askuser\"");
    }

    #[test]
    fn test_policy_priority_ordering() {
        assert!(PolicyPriority::Admin > PolicyPriority::User);
        assert!(PolicyPriority::User > PolicyPriority::Default);
        assert!(PolicyPriority::Admin > PolicyPriority::Default);
    }

    #[test]
    fn test_approval_mode_default_action() {
        assert_eq!(ApprovalMode::Yolo.default_action(), PolicyAction::Allow);
        assert_eq!(ApprovalMode::AutoEdit.default_action(), PolicyAction::AskUser);
        assert_eq!(ApprovalMode::Ask.default_action(), PolicyAction::AskUser);
    }

    #[test]
    fn test_approval_mode_auto_approves_edits() {
        assert!(ApprovalMode::Yolo.auto_approves_edits());
        assert!(ApprovalMode::AutoEdit.auto_approves_edits());
        assert!(!ApprovalMode::Ask.auto_approves_edits());
    }

    #[test]
    fn test_policy_decision_new() {
        let decision = PolicyDecision::new(PolicyAction::Allow);
        assert_eq!(decision.action, PolicyAction::Allow);
        assert!(decision.reason.is_none());
        assert!(decision.matched_rule.is_none());
        assert!(decision.is_allowed());
        assert!(!decision.is_denied());
        assert!(!decision.requires_approval());
    }

    #[test]
    fn test_policy_decision_with_reason() {
        let decision = PolicyDecision::new(PolicyAction::Deny)
            .with_reason("Security policy violation");

        assert_eq!(decision.action, PolicyAction::Deny);
        assert_eq!(decision.reason.as_deref(), Some("Security policy violation"));
        assert!(decision.is_denied());
    }

    #[test]
    fn test_policy_decision_with_rule() {
        let decision = PolicyDecision::new(PolicyAction::Allow)
            .with_rule("allow-read-files");

        assert_eq!(decision.matched_rule.as_deref(), Some("allow-read-files"));
    }

    #[test]
    fn test_policy_decision_requires_approval() {
        let decision = PolicyDecision::new(PolicyAction::AskUser);
        assert!(decision.requires_approval());
        assert!(!decision.is_allowed());
        assert!(!decision.is_denied());
    }
}
