//! Policy management commands for Tauri.

use radium_core::policy::{ApprovalMode, ConflictDetector, PolicyEngine, PolicyRule, PolicyAction, PolicyPriority};
use radium_core::workspace::Workspace;
use serde::{Deserialize, Serialize};

/// Policy rule JSON representation.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PolicyRuleJson {
    pub name: String,
    pub priority: String,
    pub action: String,
    pub tool_pattern: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arg_pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl From<PolicyRule> for PolicyRuleJson {
    fn from(rule: PolicyRule) -> Self {
        PolicyRuleJson {
            name: rule.name,
            priority: format!("{:?}", rule.priority).to_lowercase(),
            action: format!("{:?}", rule.action).to_lowercase().replace("askuser", "ask_user"),
            tool_pattern: rule.tool_pattern,
            arg_pattern: rule.arg_pattern,
            reason: rule.reason,
        }
    }
}

impl TryInto<PolicyRule> for PolicyRuleJson {
    type Error = String;

    fn try_into(self) -> Result<PolicyRule, Self::Error> {
        let priority = match self.priority.as_str() {
            "admin" => PolicyPriority::Admin,
            "user" => PolicyPriority::User,
            "default" => PolicyPriority::Default,
            _ => return Err(format!("Invalid priority: {}", self.priority)),
        };

        let action = match self.action.as_str() {
            "allow" => PolicyAction::Allow,
            "deny" => PolicyAction::Deny,
            "ask_user" | "askuser" => PolicyAction::AskUser,
            _ => return Err(format!("Invalid action: {}", self.action)),
        };

        let mut rule = PolicyRule::new(self.name, self.tool_pattern, action)
            .with_priority(priority);
        if let Some(arg_pattern) = self.arg_pattern {
            rule = rule.with_arg_pattern(arg_pattern);
        }
        if let Some(reason) = self.reason {
            rule = rule.with_reason(reason);
        }
        Ok(rule)
    }
}

/// Policy configuration JSON representation.
#[derive(Debug, Serialize, Deserialize)]
pub struct PolicyConfigJson {
    pub approval_mode: String,
    pub rules: Vec<PolicyRuleJson>,
    pub file_exists: bool,
    pub file_path: String,
}

/// Get policy configuration.
#[tauri::command]
pub async fn get_policy_config() -> Result<PolicyConfigJson, String> {
    let workspace = Workspace::discover().map_err(|e| format!("Failed to discover workspace: {}", e))?;
    let policy_file = workspace.root().join(".radium").join("policy.toml");

    let (engine, file_exists) = if policy_file.exists() {
        let engine = PolicyEngine::from_file(&policy_file)
            .map_err(|e| format!("Failed to load policy file: {}", e))?;
        (engine, true)
    } else {
        (PolicyEngine::new(ApprovalMode::Ask)
            .map_err(|e| format!("Failed to create default policy engine: {}", e))?, false)
    };

    let approval_mode = match engine.approval_mode() {
        ApprovalMode::Yolo => "yolo",
        ApprovalMode::AutoEdit => "autoEdit",
        ApprovalMode::Ask => "ask",
    };

    let rules: Vec<PolicyRuleJson> = engine.rules()
        .iter()
        .map(|r| PolicyRuleJson::from(r.clone()))
        .collect();

    Ok(PolicyConfigJson {
        approval_mode: approval_mode.to_string(),
        rules,
        file_exists,
        file_path: policy_file.to_string_lossy().to_string(),
    })
}

/// Save policy configuration.
#[tauri::command]
pub async fn save_policy_config(
    approval_mode: String,
    rules: Vec<PolicyRuleJson>,
) -> Result<String, String> {
    let workspace = Workspace::discover().map_err(|e| format!("Failed to discover workspace: {}", e))?;
    let policy_file = workspace.root().join(".radium").join("policy.toml");

    // Ensure .radium directory exists
    if let Some(parent) = policy_file.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create .radium directory: {}", e))?;
    }

    let approval_mode_enum = match approval_mode.as_str() {
        "yolo" => ApprovalMode::Yolo,
        "autoEdit" => ApprovalMode::AutoEdit,
        "ask" => ApprovalMode::Ask,
        _ => return Err(format!("Invalid approval mode: {}", approval_mode)),
    };

    let mut engine = PolicyEngine::new(approval_mode_enum)
        .map_err(|e| format!("Failed to create policy engine: {}", e))?;

    for rule_json in rules {
        let rule: PolicyRule = rule_json.try_into()
            .map_err(|e| format!("Failed to convert rule: {}", e))?;
        engine.add_rule(rule);
    }

    // Serialize to TOML
    use toml::{Value, map::Map};
    let mut config = Map::new();
    config.insert("approval_mode".to_string(), Value::String(approval_mode.to_string()));
    
    let rules_array: Vec<Value> = engine.rules().iter().map(|rule| {
        let mut rule_map = Map::new();
        rule_map.insert("name".to_string(), Value::String(rule.name.clone()));
        rule_map.insert("tool_pattern".to_string(), Value::String(rule.tool_pattern.clone()));
        rule_map.insert("action".to_string(), Value::String(format!("{:?}", rule.action).to_lowercase().replace("askuser", "ask_user")));
        rule_map.insert("priority".to_string(), Value::String(format!("{:?}", rule.priority).to_lowercase()));
        if let Some(ref arg_pattern) = rule.arg_pattern {
            rule_map.insert("arg_pattern".to_string(), Value::String(arg_pattern.clone()));
        }
        if let Some(ref reason) = rule.reason {
            rule_map.insert("reason".to_string(), Value::String(reason.clone()));
        }
        Value::Table(rule_map)
    }).collect();
    
    config.insert("rules".to_string(), Value::Array(rules_array));
    
    let toml_string = toml::to_string_pretty(&Value::Table(config))
        .map_err(|e| format!("Failed to serialize policy to TOML: {}", e))?;
    
    std::fs::write(&policy_file, toml_string)
        .map_err(|e| format!("Failed to write policy file: {}", e))?;

    Ok(format!("Policy saved to {}", policy_file.display()))
}

/// Validate policy configuration.
#[tauri::command]
pub async fn validate_policy_config(
    approval_mode: String,
    rules: Vec<PolicyRuleJson>,
) -> Result<String, String> {
    let approval_mode_enum = match approval_mode.as_str() {
        "yolo" => ApprovalMode::Yolo,
        "autoEdit" => ApprovalMode::AutoEdit,
        "ask" => ApprovalMode::Ask,
        _ => return Err(format!("Invalid approval mode: {}", approval_mode)),
    };

    let mut engine = PolicyEngine::new(approval_mode_enum)
        .map_err(|e| format!("Failed to create policy engine: {}", e))?;

    for rule_json in rules {
        let rule: PolicyRule = rule_json.try_into()
            .map_err(|e| format!("Invalid rule: {}", e))?;
        engine.add_rule(rule);
    }

    Ok("Policy configuration is valid".to_string())
}

/// Check if a tool would be allowed.
#[tauri::command]
pub async fn check_policy_tool(
    tool_name: String,
    args: Vec<String>,
) -> Result<serde_json::Value, String> {
    let workspace = Workspace::discover().map_err(|e| format!("Failed to discover workspace: {}", e))?;
    let policy_file = workspace.root().join(".radium").join("policy.toml");

    let engine = if policy_file.exists() {
        PolicyEngine::from_file(&policy_file)
            .map_err(|e| format!("Failed to load policy file: {}", e))?
    } else {
        PolicyEngine::new(ApprovalMode::Ask)
            .map_err(|e| format!("Failed to create default policy engine: {}", e))?
    };

    let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let decision = engine.evaluate_tool(&tool_name, &args_str)
        .await
        .map_err(|e| format!("Failed to evaluate tool: {}", e))?;

    Ok(serde_json::json!({
        "allowed": decision.is_allowed(),
        "denied": decision.is_denied(),
        "requires_approval": decision.requires_approval(),
        "action": format!("{:?}", decision.action).to_lowercase().replace("askuser", "ask_user"),
        "matched_rule": decision.matched_rule.clone(),
        "reason": decision.reason.clone(),
    }))
}

/// Detect conflicts in policy rules.
#[tauri::command]
pub async fn detect_policy_conflicts() -> Result<serde_json::Value, String> {
    let workspace = Workspace::discover().map_err(|e| format!("Failed to discover workspace: {}", e))?;
    let policy_file = workspace.root().join(".radium").join("policy.toml");

    if !policy_file.exists() {
        return Ok(serde_json::json!({
            "conflicts": [],
            "count": 0
        }));
    }

    let engine = PolicyEngine::from_file(&policy_file)
        .map_err(|e| format!("Failed to load policy file: {}", e))?;

    let conflicts = ConflictDetector::detect_conflicts(engine.rules())
        .map_err(|e| format!("Failed to detect conflicts: {}", e))?;

    let conflicts_json: Vec<serde_json::Value> = conflicts.iter().map(|c| {
        serde_json::json!({
            "rule1": c.rule1.name,
            "rule2": c.rule2.name,
            "conflict_type": format!("{:?}", c.conflict_type).to_lowercase(),
        })
    }).collect();

    Ok(serde_json::json!({
        "conflicts": conflicts_json,
        "count": conflicts.len()
    }))
}


