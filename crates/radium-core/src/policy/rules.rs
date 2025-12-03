//! Policy rule definition and evaluation engine.

use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use super::types::{
    ApprovalMode, PolicyAction, PolicyDecision, PolicyError, PolicyPriority, PolicyResult,
};

/// A single policy rule for tool execution control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Human-readable name for this rule.
    pub name: String,
    /// Glob pattern for matching tool names.
    /// Examples: "read_*", "bash:*", "mcp:*"
    pub tool_pattern: String,
    /// Optional glob pattern for matching tool arguments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arg_pattern: Option<String>,
    /// Action to take when this rule matches.
    pub action: PolicyAction,
    /// Priority level of this rule.
    #[serde(default = "default_priority")]
    pub priority: PolicyPriority,
    /// Human-readable reason for this rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

fn default_priority() -> PolicyPriority {
    PolicyPriority::User
}

impl PolicyRule {
    /// Creates a new policy rule.
    pub fn new(
        name: impl Into<String>,
        tool_pattern: impl Into<String>,
        action: PolicyAction,
    ) -> Self {
        Self {
            name: name.into(),
            tool_pattern: tool_pattern.into(),
            arg_pattern: None,
            action,
            priority: PolicyPriority::User,
            reason: None,
        }
    }

    /// Sets the argument pattern for this rule.
    #[must_use]
    pub fn with_arg_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.arg_pattern = Some(pattern.into());
        self
    }

    /// Sets the priority for this rule.
    #[must_use]
    pub fn with_priority(mut self, priority: PolicyPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the reason for this rule.
    #[must_use]
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Checks if this rule matches a tool execution request.
    ///
    /// # Arguments
    /// * `tool_name` - The name of the tool being executed
    /// * `args` - Arguments passed to the tool
    ///
    /// # Returns
    /// `true` if this rule matches, `false` otherwise.
    pub fn matches(&self, tool_name: &str, args: &[&str]) -> PolicyResult<bool> {
        // Match tool name pattern
        let tool_pattern = Pattern::new(&self.tool_pattern)
            .map_err(|e| PolicyError::PatternError(format!("Invalid tool pattern: {}", e)))?;

        if !tool_pattern.matches(tool_name) {
            return Ok(false);
        }

        // If no arg pattern specified, tool match is sufficient
        let Some(arg_pattern) = &self.arg_pattern else {
            return Ok(true);
        };

        // Match argument pattern
        let arg_pattern = Pattern::new(arg_pattern)
            .map_err(|e| PolicyError::PatternError(format!("Invalid arg pattern: {}", e)))?;

        // Join args and check if pattern matches any arg or the full arg string
        let args_str = args.join(" ");
        Ok(args.iter().any(|arg| arg_pattern.matches(arg)) || arg_pattern.matches(&args_str))
    }
}

/// Policy configuration file structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PolicyConfig {
    /// Approval mode for this policy configuration.
    #[serde(default)]
    approval_mode: ApprovalMode,
    /// List of policy rules.
    #[serde(default)]
    rules: Vec<PolicyRule>,
}

/// Policy engine for evaluating tool execution requests.
pub struct PolicyEngine {
    /// Approval mode.
    approval_mode: ApprovalMode,
    /// Loaded policy rules, sorted by priority (highest first).
    rules: Vec<PolicyRule>,
}

impl PolicyEngine {
    /// Creates a new policy engine with default settings.
    ///
    /// # Arguments
    /// * `approval_mode` - The default approval mode
    ///
    /// # Returns
    /// A new `PolicyEngine` with no custom rules.
    pub fn new(approval_mode: ApprovalMode) -> PolicyResult<Self> {
        Ok(Self { approval_mode, rules: Vec::new() })
    }

    /// Creates a policy engine by loading rules from a TOML file.
    ///
    /// # Arguments
    /// * `policy_file` - Path to the policy TOML file
    ///
    /// # Returns
    /// A new `PolicyEngine` with rules loaded from the file.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed.
    pub fn from_file(policy_file: impl AsRef<Path>) -> PolicyResult<Self> {
        let path = policy_file.as_ref();
        let content = fs::read_to_string(path)
            .map_err(|e| PolicyError::LoadError { path: path.to_path_buf(), source: e })?;

        let config: PolicyConfig = toml::from_str(&content)
            .map_err(|e| PolicyError::ParseError { path: path.to_path_buf(), source: e })?;

        let mut engine = Self { approval_mode: config.approval_mode, rules: config.rules };

        // Sort rules by priority (highest first)
        engine.rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(engine)
    }

    /// Adds a policy rule to this engine.
    ///
    /// # Arguments
    /// * `rule` - The rule to add
    ///
    /// Rules are automatically sorted by priority after adding.
    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
        // Re-sort rules by priority
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Evaluates a tool execution request against loaded policies.
    ///
    /// # Arguments
    /// * `tool_name` - The name of the tool being executed
    /// * `args` - Arguments passed to the tool
    ///
    /// # Returns
    /// A `PolicyDecision` indicating whether to allow, deny, or ask for the execution.
    ///
    /// # Evaluation Logic
    /// 1. Check all rules in priority order (Admin > User > Default)
    /// 2. Return the action from the first matching rule
    /// 3. If no rules match, apply approval mode defaults:
    ///    - `yolo`: Allow all
    ///    - `autoEdit`: Allow edits (write_file, edit_file), ask for others
    ///    - `ask`: Ask for all
    pub fn evaluate_tool(&self, tool_name: &str, args: &[&str]) -> PolicyResult<PolicyDecision> {
        // Check rules in priority order
        for rule in &self.rules {
            if rule.matches(tool_name, args)? {
                return Ok(PolicyDecision::new(rule.action)
                    .with_rule(&rule.name)
                    .with_reason(rule.reason.clone().unwrap_or_else(|| {
                        format!("Matched rule: {}", rule.name)
                    })));
            }
        }

        // No matching rule, apply approval mode
        let action = match self.approval_mode {
            ApprovalMode::Yolo => PolicyAction::Allow,
            ApprovalMode::AutoEdit => {
                // Auto-approve edit operations
                if Self::is_edit_operation(tool_name) {
                    PolicyAction::Allow
                } else {
                    PolicyAction::AskUser
                }
            }
            ApprovalMode::Ask => PolicyAction::AskUser,
        };

        Ok(PolicyDecision::new(action).with_reason(format!(
            "Default {} mode: {}",
            match self.approval_mode {
                ApprovalMode::Yolo => "yolo",
                ApprovalMode::AutoEdit => "autoEdit",
                ApprovalMode::Ask => "ask",
            },
            match action {
                PolicyAction::Allow => "allowed",
                PolicyAction::Deny => "denied",
                PolicyAction::AskUser => "requires approval",
            }
        )))
    }

    /// Checks if a tool name represents an edit operation.
    fn is_edit_operation(tool_name: &str) -> bool {
        matches!(
            tool_name,
            "write_file" | "edit_file" | "update_file" | "modify_file" | "create_file"
        )
    }

    /// Gets the current approval mode.
    #[must_use]
    pub fn approval_mode(&self) -> ApprovalMode {
        self.approval_mode
    }

    /// Gets the number of loaded rules.
    #[must_use]
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_policy_rule_new() {
        let rule = PolicyRule::new("allow-reads", "read_*", PolicyAction::Allow);
        assert_eq!(rule.name, "allow-reads");
        assert_eq!(rule.tool_pattern, "read_*");
        assert_eq!(rule.action, PolicyAction::Allow);
        assert_eq!(rule.priority, PolicyPriority::User);
    }

    #[test]
    fn test_policy_rule_with_builders() {
        let rule = PolicyRule::new("admin-deny-bash", "bash:*", PolicyAction::Deny)
            .with_priority(PolicyPriority::Admin)
            .with_reason("Security policy: no shell access")
            .with_arg_pattern("*rm*");

        assert_eq!(rule.priority, PolicyPriority::Admin);
        assert_eq!(rule.reason.as_deref(), Some("Security policy: no shell access"));
        assert_eq!(rule.arg_pattern.as_deref(), Some("*rm*"));
    }

    #[test]
    fn test_policy_rule_matches_tool_name() {
        let rule = PolicyRule::new("allow-reads", "read_*", PolicyAction::Allow);

        assert!(rule.matches("read_file", &[]).unwrap());
        assert!(rule.matches("read_config", &[]).unwrap());
        assert!(!rule.matches("write_file", &[]).unwrap());
    }

    #[test]
    fn test_policy_rule_matches_with_arg_pattern() {
        let rule = PolicyRule::new("deny-rm", "bash:*", PolicyAction::Deny)
            .with_arg_pattern("*rm*");

        // Matches because arg contains "rm"
        assert!(rule.matches("bash:sh", &["rm", "-rf", "/"]).unwrap());
        assert!(rule.matches("bash:command", &["rm_old"]).unwrap());
        assert!(rule.matches("bash:exec", &["cleanup_rm_temp"]).unwrap());

        // Doesn't match because arg doesn't contain "rm"
        assert!(!rule.matches("bash:ls", &["-la"]).unwrap());
        assert!(!rule.matches("bash:rm", &["file.txt"]).unwrap());
    }

    #[test]
    fn test_policy_engine_new() {
        let engine = PolicyEngine::new(ApprovalMode::Ask).unwrap();
        assert_eq!(engine.approval_mode(), ApprovalMode::Ask);
        assert_eq!(engine.rule_count(), 0);
    }

    #[test]
    fn test_policy_engine_add_rule() {
        let mut engine = PolicyEngine::new(ApprovalMode::Ask).unwrap();

        engine.add_rule(PolicyRule::new("rule1", "read_*", PolicyAction::Allow));
        assert_eq!(engine.rule_count(), 1);

        engine.add_rule(PolicyRule::new("rule2", "write_*", PolicyAction::Deny));
        assert_eq!(engine.rule_count(), 2);
    }

    #[test]
    fn test_policy_engine_priority_sorting() {
        let mut engine = PolicyEngine::new(ApprovalMode::Ask).unwrap();

        // Add rules in reverse priority order
        engine.add_rule(
            PolicyRule::new("default-rule", "*", PolicyAction::Allow)
                .with_priority(PolicyPriority::Default),
        );
        engine.add_rule(
            PolicyRule::new("admin-rule", "*", PolicyAction::Deny)
                .with_priority(PolicyPriority::Admin),
        );
        engine.add_rule(
            PolicyRule::new("user-rule", "*", PolicyAction::AskUser)
                .with_priority(PolicyPriority::User),
        );

        // Admin rule should be first due to highest priority
        assert_eq!(engine.rules[0].name, "admin-rule");
        assert_eq!(engine.rules[1].name, "user-rule");
        assert_eq!(engine.rules[2].name, "default-rule");
    }

    #[test]
    fn test_policy_engine_evaluate_matching_rule() {
        let mut engine = PolicyEngine::new(ApprovalMode::Ask).unwrap();
        engine.add_rule(PolicyRule::new("allow-reads", "read_*", PolicyAction::Allow));

        let decision = engine.evaluate_tool("read_file", &["config.toml"]).unwrap();
        assert!(decision.is_allowed());
        assert_eq!(decision.matched_rule.as_deref(), Some("allow-reads"));
    }

    #[test]
    fn test_policy_engine_evaluate_no_match_yolo() {
        let engine = PolicyEngine::new(ApprovalMode::Yolo).unwrap();

        let decision = engine.evaluate_tool("some_tool", &[]).unwrap();
        assert!(decision.is_allowed());
        assert!(decision.matched_rule.is_none());
    }

    #[test]
    fn test_policy_engine_evaluate_no_match_ask() {
        let engine = PolicyEngine::new(ApprovalMode::Ask).unwrap();

        let decision = engine.evaluate_tool("some_tool", &[]).unwrap();
        assert!(decision.requires_approval());
    }

    #[test]
    fn test_policy_engine_evaluate_auto_edit_mode() {
        let engine = PolicyEngine::new(ApprovalMode::AutoEdit).unwrap();

        // Edit operations should be auto-approved
        let decision = engine.evaluate_tool("write_file", &["file.txt"]).unwrap();
        assert!(decision.is_allowed());

        let decision = engine.evaluate_tool("edit_file", &["file.txt"]).unwrap();
        assert!(decision.is_allowed());

        // Non-edit operations should require approval
        let decision = engine.evaluate_tool("delete_file", &["file.txt"]).unwrap();
        assert!(decision.requires_approval());
    }

    #[test]
    fn test_policy_engine_from_toml() {
        let temp_dir = TempDir::new().unwrap();
        let policy_file = temp_dir.path().join("policy.toml");

        let toml_content = r#"
approval_mode = "ask"

[[rules]]
name = "allow-read-files"
tool_pattern = "read_*"
action = "allow"
priority = "user"
reason = "Read operations are safe"

[[rules]]
name = "deny-shell-commands"
tool_pattern = "bash:*"
action = "deny"
priority = "admin"
reason = "Shell commands disabled for security"
"#;

        fs::write(&policy_file, toml_content).unwrap();

        let engine = PolicyEngine::from_file(&policy_file).unwrap();
        assert_eq!(engine.approval_mode(), ApprovalMode::Ask);
        assert_eq!(engine.rule_count(), 2);

        // Admin rule should be first
        assert_eq!(engine.rules[0].name, "deny-shell-commands");
        assert_eq!(engine.rules[0].priority, PolicyPriority::Admin);
    }

    #[test]
    fn test_policy_engine_rule_priority_override() {
        let mut engine = PolicyEngine::new(ApprovalMode::Ask).unwrap();

        // Add general allow rule (user priority)
        engine.add_rule(
            PolicyRule::new("allow-all-bash", "bash:*", PolicyAction::Allow)
                .with_priority(PolicyPriority::User),
        );

        // Add specific deny rule (admin priority)
        engine.add_rule(
            PolicyRule::new("deny-rm", "bash:*", PolicyAction::Deny)
                .with_priority(PolicyPriority::Admin)
                .with_arg_pattern("*rm*"),
        );

        // Admin rule should match first and deny
        let decision = engine.evaluate_tool("bash:sh", &["rm", "-rf"]).unwrap();
        assert!(decision.is_denied());
        assert_eq!(decision.matched_rule.as_deref(), Some("deny-rm"));

        // Without rm arg, admin rule doesn't match, user rule allows
        let decision = engine.evaluate_tool("bash:ls", &["-la"]).unwrap();
        assert!(decision.is_allowed());
        assert_eq!(decision.matched_rule.as_deref(), Some("allow-all-bash"));
    }
}
