//! Policy engine for safe tool execution with user confirmations.
//!
//! Provides configurable safety policies that can:
//! - Automatically allow safe operations (read-only)
//! - Request user confirmation for dangerous operations (writes, git push)
//! - Deny unsafe operations (sudo, rm -rf)
//! - Support user-defined whitelist/blacklist patterns

use anyhow::{anyhow, Result};
use radium_abstraction::ToolCall;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, Write};

/// Policy engine for managing tool execution safety
#[derive(Debug, Clone)]
pub struct PolicyEngine {
    /// Built-in safety rules
    rules: Vec<PolicyRule>,
    /// User-specific policies (whitelist/blacklist)
    user_policy: UserPolicy,
}

/// A safety rule for tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Tool name pattern (supports wildcards)
    pub tool_pattern: String,
    /// Optional argument pattern to match
    pub arg_pattern: Option<String>,
    /// Action to take when matched
    pub action: PolicyAction,
    /// Human-readable reason for this rule
    pub reason: String,
}

/// Action to take for a tool execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)]
pub enum PolicyAction {
    /// Allow without confirmation
    Allow,
    /// Ask user for confirmation
    AskUser,
    /// Deny execution
    Deny,
}

/// User-specific policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct UserPolicy {
    /// Always allow these tool patterns
    pub whitelist: Vec<String>,
    /// Always deny these tool patterns
    pub blacklist: Vec<String>,
    /// Remember user decisions for specific operations
    pub remembered_decisions: HashMap<String, PolicyAction>,
}

/// Result of a policy check
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PolicyDecision {
    /// Operation is allowed
    Allow,
    /// Operation requires user confirmation
    AskUser { message: String },
    /// Operation is denied
    Deny { reason: String },
}

impl PolicyEngine {
    /// Create a new policy engine with default rules
    pub fn new() -> Self {
        Self {
            rules: Self::default_rules(),
            user_policy: UserPolicy::default(),
        }
    }

    /// Create policy engine with custom user policy
    pub fn with_user_policy(user_policy: UserPolicy) -> Self {
        Self {
            rules: Self::default_rules(),
            user_policy,
        }
    }

    /// Default safety rules
    fn default_rules() -> Vec<PolicyRule> {
        vec![
            // Read operations - allow
            PolicyRule {
                tool_pattern: "read_file".to_string(),
                arg_pattern: None,
                action: PolicyAction::Allow,
                reason: "Read-only operation".to_string(),
            },
            PolicyRule {
                tool_pattern: "list_dir".to_string(),
                arg_pattern: None,
                action: PolicyAction::Allow,
                reason: "Read-only operation".to_string(),
            },
            PolicyRule {
                tool_pattern: "glob_file_search".to_string(),
                arg_pattern: None,
                action: PolicyAction::Allow,
                reason: "Read-only operation".to_string(),
            },
            PolicyRule {
                tool_pattern: "git_log".to_string(),
                arg_pattern: None,
                action: PolicyAction::Allow,
                reason: "Read-only git operation".to_string(),
            },
            PolicyRule {
                tool_pattern: "git_show".to_string(),
                arg_pattern: None,
                action: PolicyAction::Allow,
                reason: "Read-only git operation".to_string(),
            },
            PolicyRule {
                tool_pattern: "git_blame".to_string(),
                arg_pattern: None,
                action: PolicyAction::Allow,
                reason: "Read-only git operation".to_string(),
            },
            PolicyRule {
                tool_pattern: "find_references".to_string(),
                arg_pattern: None,
                action: PolicyAction::Allow,
                reason: "Read-only operation".to_string(),
            },
            PolicyRule {
                tool_pattern: "analyze_code_structure".to_string(),
                arg_pattern: None,
                action: PolicyAction::Allow,
                reason: "Read-only operation".to_string(),
            },
            PolicyRule {
                tool_pattern: "project_scan".to_string(),
                arg_pattern: None,
                action: PolicyAction::Allow,
                reason: "Read-only operation".to_string(),
            },
            PolicyRule {
                tool_pattern: "read_lints".to_string(),
                arg_pattern: None,
                action: PolicyAction::Allow,
                reason: "Read-only operation".to_string(),
            },
            // Write operations - ask user
            PolicyRule {
                tool_pattern: "write_file".to_string(),
                arg_pattern: None,
                action: PolicyAction::AskUser,
                reason: "Modifies file system".to_string(),
            },
            PolicyRule {
                tool_pattern: "search_replace".to_string(),
                arg_pattern: None,
                action: PolicyAction::AskUser,
                reason: "Modifies files".to_string(),
            },
            // Dangerous terminal commands - ask user or deny
            PolicyRule {
                tool_pattern: "run_terminal_cmd".to_string(),
                arg_pattern: Some("sudo *".to_string()),
                action: PolicyAction::Deny,
                reason: "Sudo commands are not allowed for security reasons".to_string(),
            },
            PolicyRule {
                tool_pattern: "run_terminal_cmd".to_string(),
                arg_pattern: Some("rm -rf*".to_string()),
                action: PolicyAction::Deny,
                reason: "Recursive deletion is not allowed for safety".to_string(),
            },
            PolicyRule {
                tool_pattern: "run_terminal_cmd".to_string(),
                arg_pattern: Some("git push*".to_string()),
                action: PolicyAction::AskUser,
                reason: "Pushes code to remote repository".to_string(),
            },
            PolicyRule {
                tool_pattern: "run_terminal_cmd".to_string(),
                arg_pattern: Some("git commit*".to_string()),
                action: PolicyAction::AskUser,
                reason: "Creates git commit".to_string(),
            },
            // Default for terminal commands - ask user
            PolicyRule {
                tool_pattern: "run_terminal_cmd".to_string(),
                arg_pattern: None,
                action: PolicyAction::AskUser,
                reason: "Terminal command execution".to_string(),
            },
        ]
    }

    /// Check if a tool execution is allowed
    pub async fn check_tool_execution(&self, tool_call: &ToolCall) -> Result<PolicyDecision> {
        let operation_key = self.get_operation_key(tool_call);

        // Check user blacklist first
        if self.user_policy.is_blacklisted(&tool_call.name) {
            return Ok(PolicyDecision::Deny {
                reason: "Tool is blacklisted by user policy".to_string(),
            });
        }

        // Check user whitelist
        if self.user_policy.is_whitelisted(&tool_call.name) {
            return Ok(PolicyDecision::Allow);
        }

        // Check remembered decisions
        if let Some(action) = self.user_policy.remembered_decisions.get(&operation_key) {
            return match action {
                PolicyAction::Allow => Ok(PolicyDecision::Allow),
                PolicyAction::Deny => Ok(PolicyDecision::Deny {
                    reason: "Denied by previous user decision".to_string(),
                }),
                PolicyAction::AskUser => {
                    // Fallthrough to ask user again (shouldn't happen)
                    self.evaluate_rules(tool_call)
                }
            };
        }

        // Evaluate rules
        self.evaluate_rules(tool_call)
    }

    /// Evaluate rules to determine policy decision
    fn evaluate_rules(&self, tool_call: &ToolCall) -> Result<PolicyDecision> {
        // Find matching rule
        for rule in &self.rules {
            if self.matches_rule(tool_call, rule) {
                return match rule.action {
                    PolicyAction::Allow => Ok(PolicyDecision::Allow),
                    PolicyAction::Deny => Ok(PolicyDecision::Deny {
                        reason: rule.reason.clone(),
                    }),
                    PolicyAction::AskUser => {
                        let message = self.format_confirmation_message(tool_call, &rule.reason);
                        Ok(PolicyDecision::AskUser { message })
                    }
                };
            }
        }

        // Default: ask user for unknown operations
        let message = self.format_confirmation_message(tool_call, "Unknown operation");
        Ok(PolicyDecision::AskUser { message })
    }

    /// Check if a tool call matches a rule
    fn matches_rule(&self, tool_call: &ToolCall, rule: &PolicyRule) -> bool {
        // Check tool name pattern
        if !self.matches_pattern(&tool_call.name, &rule.tool_pattern) {
            return false;
        }

        // Check argument pattern if specified
        if let Some(ref arg_pattern) = rule.arg_pattern {
            // Extract command argument from tool call if it's a terminal command
            if tool_call.name == "run_terminal_cmd" {
                if let Some(cmd) = tool_call.arguments.get("command") {
                    if let Some(cmd_str) = cmd.as_str() {
                        return self.matches_pattern(cmd_str, arg_pattern);
                    }
                }
            }
            return false;
        }

        true
    }

    /// Simple pattern matching (supports * wildcard)
    fn matches_pattern(&self, value: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len() - 1];
            return value.starts_with(prefix);
        }

        if pattern.starts_with('*') {
            let suffix = &pattern[1..];
            return value.ends_with(suffix);
        }

        value == pattern
    }

    /// Format confirmation message for user
    fn format_confirmation_message(&self, tool_call: &ToolCall, reason: &str) -> String {
        let args_json = serde_json::to_string_pretty(&tool_call.arguments)
            .unwrap_or_else(|_| "{}".to_string());

        format!(
            "🔒 Security Check\n\
             Tool: {}\n\
             Reason: {}\n\
             Arguments:\n{}\n\n\
             Allow this operation? [y/N/always/never]:",
            tool_call.name, reason, args_json
        )
    }

    /// Get unique key for an operation (for remembering decisions)
    fn get_operation_key(&self, tool_call: &ToolCall) -> String {
        format!("{}__{}", tool_call.name, tool_call.id)
    }

    /// Prompt user for confirmation and remember decision if requested
    pub async fn prompt_user(&mut self, decision: &PolicyDecision) -> Result<bool> {
        if let PolicyDecision::AskUser { message } = decision {
            print!("\n{} ", message);
            io::stdout().flush()?;

            let mut response = String::new();
            io::stdin().read_line(&mut response)?;
            let response = response.trim().to_lowercase();

            match response.as_str() {
                "y" | "yes" => Ok(true),
                "always" => {
                    // TODO: Remember this decision
                    Ok(true)
                }
                "never" => {
                    // TODO: Remember this decision
                    Ok(false)
                }
                _ => Ok(false), // Default to deny
            }
        } else {
            Err(anyhow!("prompt_user called for non-AskUser decision"))
        }
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl UserPolicy {
    /// Create default user policy
    pub fn new() -> Self {
        Self {
            whitelist: Vec::new(),
            blacklist: Vec::new(),
            remembered_decisions: HashMap::new(),
        }
    }

    /// Check if a tool is whitelisted
    pub fn is_whitelisted(&self, tool_name: &str) -> bool {
        self.whitelist.iter().any(|pattern| {
            if pattern.ends_with('*') {
                let prefix = &pattern[..pattern.len() - 1];
                tool_name.starts_with(prefix)
            } else {
                tool_name == pattern
            }
        })
    }

    /// Check if a tool is blacklisted
    pub fn is_blacklisted(&self, tool_name: &str) -> bool {
        self.blacklist.iter().any(|pattern| {
            if pattern.ends_with('*') {
                let prefix = &pattern[..pattern.len() - 1];
                tool_name.starts_with(prefix)
            } else {
                tool_name == pattern
            }
        })
    }
}

impl Default for UserPolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_read_operations_allowed() {
        let engine = PolicyEngine::new();
        let tool_call = ToolCall {
            id: "test-1".to_string(),
            name: "read_file".to_string(),
            arguments: json!({"path": "/some/file.txt"}),
        };

        let decision = engine.evaluate_rules(&tool_call).unwrap();
        assert!(matches!(decision, PolicyDecision::Allow));
    }

    #[test]
    fn test_write_operations_ask_user() {
        let engine = PolicyEngine::new();
        let tool_call = ToolCall {
            id: "test-2".to_string(),
            name: "write_file".to_string(),
            arguments: json!({"path": "/some/file.txt", "content": "test"}),
        };

        let decision = engine.evaluate_rules(&tool_call).unwrap();
        assert!(matches!(decision, PolicyDecision::AskUser { .. }));
    }

    #[test]
    fn test_sudo_denied() {
        let engine = PolicyEngine::new();
        let tool_call = ToolCall {
            id: "test-3".to_string(),
            name: "run_terminal_cmd".to_string(),
            arguments: json!({"command": "sudo rm -rf /"}),
        };

        let decision = engine.evaluate_rules(&tool_call).unwrap();
        assert!(matches!(decision, PolicyDecision::Deny { .. }));
    }

    #[test]
    fn test_whitelist() {
        let mut user_policy = UserPolicy::new();
        user_policy.whitelist.push("write_*".to_string());

        let engine = PolicyEngine::with_user_policy(user_policy);

        let tool_call = ToolCall {
            id: "test-4".to_string(),
            name: "write_file".to_string(),
            arguments: json!({"path": "/some/file.txt"}),
        };

        // Should be whitelisted and allowed
        assert!(engine.user_policy.is_whitelisted(&tool_call.name));
    }

    #[test]
    fn test_blacklist() {
        let mut user_policy = UserPolicy::new();
        user_policy.blacklist.push("run_terminal_cmd".to_string());

        let engine = PolicyEngine::with_user_policy(user_policy);

        let tool_call = ToolCall {
            id: "test-5".to_string(),
            name: "run_terminal_cmd".to_string(),
            arguments: json!({"command": "ls"}),
        };

        // Should be blacklisted
        assert!(engine.user_policy.is_blacklisted(&tool_call.name));
    }
}
