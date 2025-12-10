//! Policy rule definition and evaluation engine.

use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Arc;

use super::alerts::AlertManager;
use super::analytics::PolicyAnalytics;
use super::dry_run::generate_preview;
use super::types::{
    ApprovalMode, PolicyAction, PolicyDecision, PolicyError, PolicyPriority, PolicyResult,
};
use crate::hooks::registry::{HookRegistry, HookType};
use crate::hooks::types::HookContext;

/// A single policy rule for tool execution control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Human-readable name for this rule.
    pub name: String,
    /// Glob pattern for matching tool names.
    /// Examples: "read_*", "bash:*", "mcp:*", "server:tool", "*:dangerous", "server1:*", "code_execution"
    /// 
    /// For MCP tools, patterns can match:
    /// - `mcp_*` - All MCP tools (orchestration format: mcp_server_tool)
    /// - `mcp_server1_*` - All tools from server1
    /// - `*:tool` - Tool named "tool" from any server (if using server:tool format)
    /// - `server1:*` - All tools from server1 (if using server:tool format)
    /// 
    /// For code execution (provider-specific tools like Gemini's code_execution):
    /// - `code_execution` - Matches code execution tool requests
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
    /// Optional hook registry for tool execution interception.
    hook_registry: Option<Arc<HookRegistry>>,
    /// Optional alert manager for violation notifications.
    alert_manager: Option<Arc<AlertManager>>,
    /// Optional analytics manager for tracking policy events.
    analytics: Option<Arc<PolicyAnalytics>>,
}

impl PolicyEngine {
    /// Creates a new policy engine with default settings.
    ///
    /// # Arguments
    /// * `approval_mode` - The default approval mode
    /// * `hook_registry` - Optional hook registry for tool execution interception
    ///
    /// # Returns
    /// A new `PolicyEngine` with no custom rules.
    pub fn new(approval_mode: ApprovalMode) -> PolicyResult<Self> {
        Ok(Self {
            approval_mode,
            rules: Vec::new(),
            hook_registry: None,
            alert_manager: None,
            analytics: None,
        })
    }

    /// Creates a new policy engine with hook registry.
    ///
    /// # Arguments
    /// * `approval_mode` - The default approval mode
    /// * `hook_registry` - Hook registry for tool execution interception
    ///
    /// # Returns
    /// A new `PolicyEngine` with no custom rules.
    pub fn with_hooks(approval_mode: ApprovalMode, hook_registry: Arc<HookRegistry>) -> PolicyResult<Self> {
        Ok(Self {
            approval_mode,
            rules: Vec::new(),
            hook_registry: Some(hook_registry),
            alert_manager: None,
            analytics: None,
        })
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

        let mut engine = Self {
            approval_mode: config.approval_mode,
            rules: config.rules,
            hook_registry: None,
            alert_manager: None,
            analytics: None,
        };

        // Sort rules by priority (highest first)
        engine.rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(engine)
    }

    /// Sets the hook registry for this policy engine.
    pub fn set_hook_registry(&mut self, hook_registry: Arc<HookRegistry>) {
        self.hook_registry = Some(hook_registry);
    }

    /// Sets the alert manager for this policy engine.
    pub fn set_alert_manager(&mut self, alert_manager: Arc<AlertManager>) {
        self.alert_manager = Some(alert_manager);
    }

    /// Sets the analytics manager for this policy engine.
    pub fn set_analytics(&mut self, analytics: Arc<PolicyAnalytics>) {
        self.analytics = Some(analytics);
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
    /// 1. Execute BeforeTool hooks (if registered) to allow modification of tool name/args
    /// 2. Check all rules in priority order (Admin > User > Default)
    /// 3. Return the action from the first matching rule
    /// 4. If no rules match, apply approval mode defaults:
    ///    - `yolo`: Allow all
    ///    - `autoEdit`: Allow edits (write_file, edit_file), ask for others
    ///    - `ask`: Ask for all
    pub async fn evaluate_tool(&self, tool_name: &str, args: &[&str]) -> PolicyResult<PolicyDecision> {
        // Execute BeforeTool hooks to allow modification
        let mut effective_tool_name = tool_name.to_string();
        let mut effective_args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

        if let Some(registry) = &self.hook_registry {
            let hook_context = HookContext::new(
                "before_tool",
                serde_json::json!({
                    "tool_name": tool_name,
                    "args": args,
                }),
            );

            if let Ok(results) = registry.execute_hooks(HookType::BeforeTool, &hook_context).await {
                for result in results {
                    // If hook says to stop, deny execution
                    if !result.should_continue {
                        return Ok(PolicyDecision::new(PolicyAction::Deny).with_reason(
                            result.message.unwrap_or_else(|| "Tool execution denied by hook".to_string()),
                        ));
                    }

                    // If hook modifies tool name or args, use the modified version
                    if let Some(modified_data) = result.modified_data {
                        if let Some(new_tool_name) = modified_data.get("tool_name").and_then(|v| v.as_str()) {
                            effective_tool_name = new_tool_name.to_string();
                        }
                        if let Some(new_args) = modified_data.get("args").and_then(|v| v.as_array()) {
                            effective_args = new_args
                                .iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect::<Vec<_>>();
                        }
                    }
                }
            }
        }

        // Convert effective_args back to &[&str] for policy evaluation
        let args_refs: Vec<&str> = effective_args.iter().map(|s| s.as_str()).collect();

        // Check rules in priority order
        for rule in &self.rules {
            if rule.matches(&effective_tool_name, &args_refs)? {
                let mut decision = PolicyDecision::new(rule.action)
                    .with_rule(&rule.name)
                    .with_reason(
                        rule.reason.clone().unwrap_or_else(|| format!("Matched rule: {}", rule.name)),
                    );

                // Generate preview for dry-run actions
                if rule.action == PolicyAction::DryRunFirst {
                    let preview = generate_preview(&effective_tool_name, &args_refs)?;
                    decision = decision.with_preview(preview);
                }

                // Send alert for violations (non-allow actions)
                if let Some(ref alert_manager) = self.alert_manager {
                    if decision.action != PolicyAction::Allow {
                        let args_str: Vec<&str> = args_refs.iter().copied().collect();
                        alert_manager.send_alert(&decision, &effective_tool_name, &args_str, None).await;
                    }
                }

                // Record analytics event
                if let Some(ref analytics) = self.analytics {
                    let args_str: Vec<&str> = args_refs.iter().copied().collect();
                    analytics.record_event(&decision, &effective_tool_name, &args_str, None);
                }

                return Ok(decision);
            }
        }

        // No matching rule, apply approval mode
        let action = match self.approval_mode {
            ApprovalMode::Yolo => PolicyAction::Allow,
            ApprovalMode::AutoEdit => {
                // Auto-approve edit operations
                if Self::is_edit_operation(&effective_tool_name) {
                    PolicyAction::Allow
                } else {
                    PolicyAction::AskUser
                }
            }
            ApprovalMode::Ask => PolicyAction::AskUser,
        };

        let decision = PolicyDecision::new(action).with_reason(format!(
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
                PolicyAction::DryRunFirst => "requires dry-run preview",
            }
        ));

        // Send alert for violations (non-allow actions)
        if let Some(ref alert_manager) = self.alert_manager {
            if decision.action != PolicyAction::Allow {
                let args_str: Vec<&str> = args_refs.iter().copied().collect();
                alert_manager.send_alert(&decision, &effective_tool_name, &args_str, None).await;
            }
        }

        // Record analytics event
        if let Some(ref analytics) = self.analytics {
            let args_str: Vec<&str> = args_refs.iter().copied().collect();
            analytics.record_event(&decision, &effective_tool_name, &args_str, None);
        }

        Ok(decision)
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

    /// Gets all loaded rules (immutable reference).
    #[must_use]
    pub fn rules(&self) -> &[PolicyRule] {
        &self.rules
    }

    /// Updates the approval mode and rules from another PolicyEngine.
    ///
    /// This is useful for hot-reloading policy configuration while preserving
    /// other components like hook_registry, alert_manager, and analytics.
    pub fn update_from(&mut self, other: PolicyEngine) {
        self.approval_mode = other.approval_mode;
        self.rules = other.rules;
    }

    /// Detects conflicts in the current set of rules.
    ///
    /// # Returns
    /// Vector of detected conflicts.
    ///
    /// # Errors
    /// Returns error if pattern parsing fails.
    pub fn detect_conflicts(&self) -> PolicyResult<Vec<super::conflict_resolution::PolicyConflict>> {
        super::conflict_resolution::ConflictDetector::detect_conflicts(&self.rules)
    }

    /// Resolves conflicts using auto-resolution strategy.
    ///
    /// # Returns
    /// Vector of rule names that were removed.
    pub fn auto_resolve_conflicts(&mut self) -> PolicyResult<Vec<String>> {
        let conflicts = self.detect_conflicts()?;
        let removed = super::conflict_resolution::ConflictResolver::auto_resolve(&conflicts, &mut self.rules);
        Ok(removed)
    }

    /// Resolves conflicts using a specified strategy.
    ///
    /// # Arguments
    /// * `strategy` - Resolution strategy to apply
    ///
    /// # Returns
    /// Vector of rule names that were removed.
    pub fn resolve_conflicts(
        &mut self,
        strategy: super::conflict_resolution::ResolutionStrategy,
    ) -> PolicyResult<Vec<String>> {
        let conflicts = self.detect_conflicts()?;
        let removed = super::conflict_resolution::ConflictResolver::resolve_conflicts(
            &conflicts,
            strategy,
            &mut self.rules,
        );
        Ok(removed)
    }

    /// Executes AfterTool hooks after tool execution.
    ///
    /// # Arguments
    /// * `tool_name` - The name of the tool that was executed
    /// * `args` - Arguments that were passed to the tool
    /// * `result` - The result of tool execution
    ///
    /// # Returns
    /// Vector of hook results, or error if hook execution failed.
    pub async fn execute_after_tool_hooks(
        &self,
        tool_name: &str,
        args: &[&str],
        result: &serde_json::Value,
    ) -> Result<Vec<crate::hooks::types::HookResult>, PolicyError> {
        if let Some(registry) = &self.hook_registry {
            let hook_context = HookContext::new(
                "after_tool",
                serde_json::json!({
                    "tool_name": tool_name,
                    "args": args,
                    "result": result,
                }),
            );

            registry.execute_hooks(HookType::AfterTool, &hook_context).await.map_err(|e| {
                PolicyError::PatternError(format!("Hook execution failed: {}", e))
            })
        } else {
            Ok(Vec::new())
        }
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
        let rule =
            PolicyRule::new("deny-rm", "bash:*", PolicyAction::Deny).with_arg_pattern("*rm*");

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

    #[tokio::test]
    async fn test_policy_engine_evaluate_matching_rule() {
        let mut engine = PolicyEngine::new(ApprovalMode::Ask).unwrap();
        engine.add_rule(PolicyRule::new("allow-reads", "read_*", PolicyAction::Allow));

        let decision = engine.evaluate_tool("read_file", &["config.toml"]).await.unwrap();
        assert!(decision.is_allowed());
        assert_eq!(decision.matched_rule.as_deref(), Some("allow-reads"));
    }

    #[tokio::test]
    async fn test_policy_engine_evaluate_no_match_yolo() {
        let engine = PolicyEngine::new(ApprovalMode::Yolo).unwrap();

        let decision = engine.evaluate_tool("some_tool", &[]).await.unwrap();
        assert!(decision.is_allowed());
        assert!(decision.matched_rule.is_none());
    }

    #[tokio::test]
    async fn test_policy_engine_evaluate_no_match_ask() {
        let engine = PolicyEngine::new(ApprovalMode::Ask).unwrap();

        let decision = engine.evaluate_tool("some_tool", &[]).await.unwrap();
        assert!(decision.requires_approval());
    }

    #[tokio::test]
    async fn test_policy_engine_evaluate_auto_edit_mode() {
        let engine = PolicyEngine::new(ApprovalMode::AutoEdit).unwrap();

        // Edit operations should be auto-approved
        let decision = engine.evaluate_tool("write_file", &["file.txt"]).await.unwrap();
        assert!(decision.is_allowed());

        let decision = engine.evaluate_tool("edit_file", &["file.txt"]).await.unwrap();
        assert!(decision.is_allowed());

        // Non-edit operations should require approval
        let decision = engine.evaluate_tool("delete_file", &["file.txt"]).await.unwrap();
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

    #[tokio::test]
    async fn test_policy_engine_rule_priority_override() {
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
        let decision = engine.evaluate_tool("bash:sh", &["rm", "-rf"]).await.unwrap();
        assert!(decision.is_denied());
        assert_eq!(decision.matched_rule.as_deref(), Some("deny-rm"));

        // Without rm arg, admin rule doesn't match, user rule allows
        let decision = engine.evaluate_tool("bash:ls", &["-la"]).await.unwrap();
        assert!(decision.is_allowed());
        assert_eq!(decision.matched_rule.as_deref(), Some("allow-all-bash"));
    }

    #[tokio::test]
    async fn test_policy_engine_mcp_tool_patterns() {
        let mut engine = PolicyEngine::new(ApprovalMode::Ask).unwrap();

        // Test pattern matching for MCP tool names
        // MCP tools in orchestration use format: mcp_server_tool
        let rule1 = PolicyRule {
            name: "allow-all-mcp".to_string(),
            tool_pattern: "mcp_*".to_string(), // Matches tools starting with mcp_
            arg_pattern: None,
            action: PolicyAction::Allow,
            priority: PolicyPriority::User,
            reason: Some("Allow all MCP tools".to_string()),
        };
        engine.add_rule(rule1);

        let rule2 = PolicyRule {
            name: "deny-specific-server".to_string(),
            tool_pattern: "mcp_untrusted_*".to_string(), // Matches tools from untrusted server
            arg_pattern: None,
            action: PolicyAction::Deny,
            priority: PolicyPriority::Admin, // Higher priority
            reason: Some("Deny untrusted server".to_string()),
        };
        engine.add_rule(rule2);

        // Test MCP tool name matching
        let decision1 = engine.evaluate_tool("mcp_server1_tool1", &[]).await.unwrap();
        assert!(decision1.is_allowed());
        assert_eq!(decision1.matched_rule, Some("allow-all-mcp".to_string()));

        let decision2 = engine.evaluate_tool("mcp_untrusted_tool1", &[]).await.unwrap();
        assert!(decision2.is_denied());
        assert_eq!(decision2.matched_rule, Some("deny-specific-server".to_string()));

        // Test with server:tool format (if tools are registered with that format)
        let decision3 = engine.evaluate_tool("mcp_trusted_read", &[]).await.unwrap();
        assert!(decision3.is_allowed());
    }

    #[tokio::test]
    async fn test_policy_engine_mcp_tool_glob_patterns() {
        let mut engine = PolicyEngine::new(ApprovalMode::Ask).unwrap();

        // Test glob patterns that would match server:tool format
        // Pattern: *:dangerous matches any server with dangerous tool
        let rule = PolicyRule {
            name: "deny-dangerous-tools".to_string(),
            tool_pattern: "*:dangerous".to_string(),
            arg_pattern: None,
            action: PolicyAction::Deny,
            priority: PolicyPriority::Admin,
            reason: Some("Deny dangerous tools from any server".to_string()),
        };
        engine.add_rule(rule);

        // Test that pattern matches (if tool names use server:tool format)
        let decision = engine.evaluate_tool("server1:dangerous", &[]).await.unwrap();
        assert!(decision.is_denied());

        // Test pattern: server1:* matches all tools from server1
        let rule2 = PolicyRule {
            name: "allow-server1".to_string(),
            tool_pattern: "server1:*".to_string(),
            arg_pattern: None,
            action: PolicyAction::Allow,
            priority: PolicyPriority::User,
            reason: None,
        };
        engine.add_rule(rule2);

        let decision2 = engine.evaluate_tool("server1:read", &[]).await.unwrap();
        assert!(decision2.is_allowed());
    }

    #[tokio::test]
    async fn test_policy_engine_dry_run_action() {
        let mut engine = PolicyEngine::new(ApprovalMode::Ask).unwrap();
        engine.add_rule(
            PolicyRule::new("terraform-dry-run", "run_terminal_cmd", PolicyAction::DryRunFirst)
                .with_arg_pattern("terraform apply *")
                .with_reason("Terraform operations require dry-run preview"),
        );

        let decision = engine.evaluate_tool("run_terminal_cmd", &["terraform", "apply"]).await.unwrap();
        assert!(decision.requires_dry_run());
        assert!(decision.preview.is_some());
        
        let preview = decision.preview.unwrap();
        assert_eq!(preview.tool_name, "run_terminal_cmd");
        assert!(preview.arguments.contains(&"terraform".to_string()));
        assert!(preview.affected_resources.iter().any(|r| r.contains("Terraform")));
        assert!(preview.details.is_some());
    }

    #[tokio::test]
    async fn test_policy_engine_dry_run_file_operation() {
        let mut engine = PolicyEngine::new(ApprovalMode::Ask).unwrap();
        engine.add_rule(
            PolicyRule::new("file-write-dry-run", "write_*", PolicyAction::DryRunFirst)
                .with_reason("File writes require preview"),
        );

        let decision = engine.evaluate_tool("write_file", &["test.txt", "content"]).await.unwrap();
        assert!(decision.requires_dry_run());
        assert!(decision.preview.is_some());
        
        let preview = decision.preview.unwrap();
        assert_eq!(preview.tool_name, "write_file");
        assert!(!preview.affected_resources.is_empty());
    }

    #[tokio::test]
    async fn test_policy_engine_dry_run_from_toml() {
        let temp_dir = TempDir::new().unwrap();
        let policy_file = temp_dir.path().join("policy.toml");

        let toml_content = r#"
approval_mode = "ask"

[[rules]]
name = "terraform-dry-run"
tool_pattern = "run_terminal_cmd"
arg_pattern = "terraform apply *"
action = "dry_run_first"
priority = "user"
reason = "Terraform apply requires dry-run preview"
"#;

        fs::write(&policy_file, toml_content).unwrap();

        let engine = PolicyEngine::from_file(&policy_file).unwrap();
        let decision = engine.evaluate_tool("run_terminal_cmd", &["terraform", "apply", "main.tf"]).await.unwrap();
        
        assert!(decision.requires_dry_run());
        assert!(decision.preview.is_some());
        assert_eq!(decision.matched_rule.as_deref(), Some("terraform-dry-run"));
    }

    #[tokio::test]
    async fn test_policy_engine_with_alert_manager() {
        use crate::policy::alerts::{AlertConfig, AlertManager, WebhookConfig};
        
        let alert_config = AlertConfig {
            enabled: false, // Disable for testing
            webhooks: vec![],
            rate_limit_per_minute: 10,
        };
        let alert_manager = Arc::new(AlertManager::new(alert_config));
        
        let mut engine = PolicyEngine::new(ApprovalMode::Ask).unwrap();
        engine.set_alert_manager(Arc::clone(&alert_manager));
        engine.add_rule(
            PolicyRule::new("deny-rm", "run_terminal_cmd", PolicyAction::Deny)
                .with_arg_pattern("*rm*")
                .with_reason("Dangerous command"),
        );

        let decision = engine.evaluate_tool("run_terminal_cmd", &["rm", "-rf", "/tmp"]).await.unwrap();
        assert!(decision.is_denied());
    }
}
