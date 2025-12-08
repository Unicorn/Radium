//! Dry-run preview generation for policy engine.

use super::types::{DryRunPreview, PolicyResult};

/// Generates a preview of what would be executed for a tool call.
///
/// # Arguments
/// * `tool_name` - The name of the tool
/// * `args` - The arguments that would be passed
///
/// # Returns
/// A `DryRunPreview` with details about what would happen.
pub fn generate_preview(tool_name: &str, args: &[&str]) -> PolicyResult<DryRunPreview> {
    let arguments: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let affected_resources = analyze_affected_resources(tool_name, args)?;
    let details = generate_details(tool_name, args)?;

    Ok(DryRunPreview {
        tool_name: tool_name.to_string(),
        arguments,
        affected_resources,
        details,
    })
}

/// Analyzes what resources would be affected by a tool execution.
fn analyze_affected_resources(tool_name: &str, args: &[&str]) -> PolicyResult<Vec<String>> {
    let mut resources = Vec::new();

    match tool_name {
        // File operations
        "read_file" | "write_file" | "edit_file" | "delete_file" | "create_file" => {
            if let Some(first_arg) = args.first() {
                resources.push(format!("File: {}", first_arg));
            }
        }
        // Terminal commands
        "run_terminal_cmd" => {
            if !args.is_empty() {
                // Try to identify files or resources from command
                let cmd = args.join(" ");
                if cmd.contains("terraform") {
                    resources.push("Terraform state".to_string());
                    // Look for .tf files in args
                    for arg in args {
                        if arg.ends_with(".tf") || arg.ends_with(".tfvars") {
                            resources.push(format!("Terraform file: {}", arg));
                        }
                    }
                } else if cmd.contains("git") {
                    resources.push("Git repository".to_string());
                } else if cmd.contains("docker") || cmd.contains("podman") {
                    resources.push("Container runtime".to_string());
                } else if cmd.contains("kubectl") {
                    resources.push("Kubernetes cluster".to_string());
                } else {
                    resources.push(format!("Command: {}", args[0]));
                }
            }
        }
        // MCP tools - extract server and tool info
        tool if tool.starts_with("mcp_") => {
            let parts: Vec<&str> = tool.split('_').collect();
            if parts.len() >= 2 {
                resources.push(format!("MCP server: {}", parts[1]));
            }
            if !args.is_empty() {
                resources.push(format!("Tool arguments: {}", args.join(" ")));
            }
        }
        _ => {
            // Generic fallback
            if !args.is_empty() {
                resources.push(format!("Tool: {} with {} argument(s)", tool_name, args.len()));
            }
        }
    }

    Ok(resources)
}

/// Generates detailed description of what would happen.
fn generate_details(tool_name: &str, args: &[&str]) -> PolicyResult<Option<String>> {
    let details = match tool_name {
        "read_file" => Some(format!("Would read file: {}", args.first().unwrap_or(&"<unknown>"))),
        "write_file" => Some(format!("Would write to file: {}", args.first().unwrap_or(&"<unknown>"))),
        "edit_file" => Some(format!("Would modify file: {}", args.first().unwrap_or(&"<unknown>"))),
        "delete_file" => Some(format!("Would delete file: {}", args.first().unwrap_or(&"<unknown>"))),
        "create_file" => Some(format!("Would create file: {}", args.first().unwrap_or(&"<unknown>"))),
        "run_terminal_cmd" => {
            let cmd = args.join(" ");
            if cmd.contains("terraform apply") {
                Some("Would apply Terraform configuration and create/modify infrastructure resources".to_string())
            } else if cmd.contains("terraform destroy") {
                Some("Would destroy Terraform-managed infrastructure resources".to_string())
            } else if cmd.contains("git push") && cmd.contains("--force") {
                Some("Would force push to remote repository (potentially destructive)".to_string())
            } else if cmd.contains("rm -rf") {
                Some("Would recursively delete files and directories (destructive)".to_string())
            } else if cmd.contains("sudo") {
                Some("Would execute command with elevated privileges".to_string())
            } else {
                Some(format!("Would execute shell command: {}", cmd))
            }
        }
        tool if tool.starts_with("mcp_") => {
            Some(format!("Would call MCP tool: {} with arguments", tool))
        }
        _ => None,
    };

    Ok(details)
}

/// Formats a dry-run preview as a human-readable string.
pub fn format_preview(preview: &DryRunPreview) -> String {
    let mut output = String::new();
    output.push_str("Dry-Run Preview\n");
    output.push_str("===============\n");
    output.push_str(&format!("Tool: {}\n", preview.tool_name));
    
    if !preview.arguments.is_empty() {
        output.push_str(&format!("Arguments: {}\n", preview.arguments.join(" ")));
    }
    
    if !preview.affected_resources.is_empty() {
        output.push_str("Affected Resources:\n");
        for resource in &preview.affected_resources {
            output.push_str(&format!("  - {}\n", resource));
        }
    }
    
    if let Some(ref details) = preview.details {
        output.push_str(&format!("Details: {}\n", details));
    }
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_preview_file_operation() {
        let preview = generate_preview("write_file", &["test.txt"]).unwrap();
        assert_eq!(preview.tool_name, "write_file");
        assert_eq!(preview.arguments, vec!["test.txt"]);
        assert!(!preview.affected_resources.is_empty());
    }

    #[test]
    fn test_generate_preview_terminal_command() {
        let preview = generate_preview("run_terminal_cmd", &["terraform", "apply"]).unwrap();
        assert_eq!(preview.tool_name, "run_terminal_cmd");
        assert!(preview.affected_resources.iter().any(|r| r.contains("Terraform")));
        assert!(preview.details.is_some());
    }

    #[test]
    fn test_analyze_affected_resources_file_ops() {
        let resources = analyze_affected_resources("read_file", &["config.toml"]).unwrap();
        assert!(resources.iter().any(|r| r.contains("config.toml")));
    }

    #[test]
    fn test_analyze_affected_resources_terraform() {
        let resources = analyze_affected_resources("run_terminal_cmd", &["terraform", "apply", "main.tf"]).unwrap();
        assert!(resources.iter().any(|r| r.contains("Terraform")));
        assert!(resources.iter().any(|r| r.contains("main.tf")));
    }

    #[test]
    fn test_format_preview() {
        let preview = DryRunPreview {
            tool_name: "write_file".to_string(),
            arguments: vec!["test.txt".to_string()],
            affected_resources: vec!["File: test.txt".to_string()],
            details: Some("Would write to file".to_string()),
        };
        let formatted = format_preview(&preview);
        assert!(formatted.contains("write_file"));
        assert!(formatted.contains("test.txt"));
        assert!(formatted.contains("Affected Resources"));
    }
}

