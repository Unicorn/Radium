//! Custom commands system with TOML-based definitions.
//!
//! Supports:
//! - TOML-based command definitions
//! - Shell command injection: `!{command}`
//! - File content injection: `@{file}`
//! - Argument placeholders: `{{args}}`, `{{arg1}}`, etc.
//! - User vs project command precedence
//! - Namespaced commands via directory structure

use super::error::{CommandError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Custom command definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomCommand {
    /// Command name.
    pub name: String,

    /// Command description.
    #[serde(default)]
    pub description: String,

    /// Command template with injection syntax.
    pub template: String,

    /// Optional arguments definition.
    #[serde(default)]
    pub args: Vec<String>,

    /// Optional namespace (from directory structure).
    #[serde(skip)]
    pub namespace: Option<String>,
}

impl CustomCommand {
    /// Executes the command with provided arguments.
    ///
    /// # Arguments
    /// * `args` - Arguments to substitute into template
    /// * `base_dir` - Base directory for file resolution
    ///
    /// # Returns
    /// The rendered command output
    ///
    /// # Errors
    /// Returns error if execution fails
    pub fn execute(&self, args: &[String], base_dir: &Path) -> Result<String> {
        let mut rendered = self.template.clone();

        // Substitute arguments
        rendered = self.substitute_args(&rendered, args)?;

        // Execute shell commands (!{command})
        rendered = self.execute_shell_commands(&rendered)?;

        // Inject file contents (@{file})
        rendered = self.inject_files(&rendered, base_dir)?;

        Ok(rendered)
    }

    /// Substitutes argument placeholders.
    fn substitute_args(&self, template: &str, args: &[String]) -> Result<String> {
        let mut result = template.to_string();

        // Substitute {{args}} with all arguments joined
        if result.contains("{{args}}") {
            let args_str = args.join(" ");
            result = result.replace("{{args}}", &args_str);
        }

        // Substitute {{arg1}}, {{arg2}}, etc.
        for (i, arg) in args.iter().enumerate() {
            let placeholder = format!("{{{{arg{}}}}}", i + 1);
            result = result.replace(&placeholder, arg);
        }

        Ok(result)
    }

    /// Executes shell command injections.
    fn execute_shell_commands(&self, template: &str) -> Result<String> {
        let mut result = template.to_string();

        // Find all !{...} patterns
        while let Some(start) = result.find("!{") {
            let Some(end) = result[start..].find('}') else {
                return Err(CommandError::TemplateRender(
                    "Unclosed shell command injection".to_string(),
                ));
            };

            let end = start + end;
            let command_str = &result[start + 2..end];

            // Execute shell command
            let output = Command::new("sh")
                .arg("-c")
                .arg(command_str)
                .output()
                .map_err(|e| CommandError::ShellExecution(e.to_string()))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(CommandError::ShellExecution(format!(
                    "Command failed: {}",
                    stderr
                )));
            }

            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            result.replace_range(start..=end, &stdout);
        }

        Ok(result)
    }

    /// Injects file contents.
    fn inject_files(&self, template: &str, base_dir: &Path) -> Result<String> {
        let mut result = template.to_string();

        // Find all @{...} patterns
        while let Some(start) = result.find("@{") {
            let Some(end) = result[start..].find('}') else {
                return Err(CommandError::TemplateRender(
                    "Unclosed file injection".to_string(),
                ));
            };

            let end = start + end;
            let file_path_str = &result[start + 2..end];
            let file_path = base_dir.join(file_path_str);

            // Read file contents
            let content = fs::read_to_string(&file_path).map_err(|e| {
                CommandError::FileInjection(format!(
                    "Failed to read {}: {}",
                    file_path.display(),
                    e
                ))
            })?;

            result.replace_range(start..=end, &content);
        }

        Ok(result)
    }
}

/// Custom command discovery and registry.
pub struct CommandRegistry {
    /// Registered commands by name.
    commands: HashMap<String, CustomCommand>,

    /// Project commands directory.
    project_dir: Option<PathBuf>,

    /// User commands directory.
    user_dir: Option<PathBuf>,
}

impl CommandRegistry {
    /// Creates a new command registry.
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            project_dir: None,
            user_dir: None,
        }
    }

    /// Sets the project commands directory.
    pub fn with_project_dir(mut self, dir: impl AsRef<Path>) -> Self {
        self.project_dir = Some(dir.as_ref().to_path_buf());
        self
    }

    /// Sets the user commands directory.
    pub fn with_user_dir(mut self, dir: impl AsRef<Path>) -> Self {
        self.user_dir = Some(dir.as_ref().to_path_buf());
        self
    }

    /// Discovers all custom commands.
    ///
    /// Project commands take precedence over user commands.
    ///
    /// # Errors
    /// Returns error if discovery fails
    pub fn discover(&mut self) -> Result<()> {
        // Clone directory paths to avoid borrow checker issues
        let user_dir = self.user_dir.clone();
        let project_dir = self.project_dir.clone();

        // Load user commands first (lower precedence)
        if let Some(user_dir) = user_dir {
            if user_dir.exists() {
                self.discover_in_directory(&user_dir, None)?;
            }
        }

        // Load project commands (higher precedence, will overwrite user commands)
        if let Some(project_dir) = project_dir {
            if project_dir.exists() {
                self.discover_in_directory(&project_dir, None)?;
            }
        }

        Ok(())
    }

    /// Discovers commands in a directory.
    fn discover_in_directory(&mut self, dir: &Path, namespace: Option<String>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
                // Load command from TOML file
                let content = fs::read_to_string(&path)?;
                let mut command: CustomCommand = toml::from_str(&content)?;

                // Set namespace
                command.namespace = namespace.clone();

                // Register command
                let name = if let Some(ref ns) = namespace {
                    format!("{}:{}", ns, command.name)
                } else {
                    command.name.clone()
                };

                self.commands.insert(name, command);
            } else if path.is_dir() {
                // Recurse into subdirectories for namespaced commands
                let dir_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                let new_namespace = if let Some(ref ns) = namespace {
                    format!("{}:{}", ns, dir_name)
                } else {
                    dir_name
                };
                self.discover_in_directory(&path, Some(new_namespace))?;
            }
        }

        Ok(())
    }

    /// Gets a command by name.
    pub fn get(&self, name: &str) -> Option<&CustomCommand> {
        self.commands.get(name)
    }

    /// Lists all command names.
    pub fn list(&self) -> Vec<String> {
        self.commands.keys().cloned().collect()
    }

    /// Returns the number of registered commands.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Returns true if no commands are registered.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_custom_command_substitute_args() {
        let command = CustomCommand {
            name: "test".to_string(),
            description: String::new(),
            template: "Hello {{arg1}} and {{arg2}}!".to_string(),
            args: vec![],
            namespace: None,
        };

        let args = vec!["Alice".to_string(), "Bob".to_string()];
        let result = command.substitute_args(&command.template, &args).unwrap();
        assert_eq!(result, "Hello Alice and Bob!");
    }

    #[test]
    fn test_custom_command_substitute_args_all() {
        let command = CustomCommand {
            name: "test".to_string(),
            description: String::new(),
            template: "Args: {{args}}".to_string(),
            args: vec![],
            namespace: None,
        };

        let args = vec!["one".to_string(), "two".to_string(), "three".to_string()];
        let result = command.substitute_args(&command.template, &args).unwrap();
        assert_eq!(result, "Args: one two three");
    }

    #[test]
    fn test_custom_command_inject_files() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("content.txt");

        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"Hello from file!").unwrap();
        drop(file);

        let command = CustomCommand {
            name: "test".to_string(),
            description: String::new(),
            template: "Content: @{content.txt}".to_string(),
            args: vec![],
            namespace: None,
        };

        let result = command.inject_files(&command.template, temp_dir.path()).unwrap();
        assert_eq!(result, "Content: Hello from file!");
    }

    #[test]
    fn test_custom_command_execute_shell() {
        let command = CustomCommand {
            name: "test".to_string(),
            description: String::new(),
            template: "Date: !{date +%Y}".to_string(),
            args: vec![],
            namespace: None,
        };

        let result = command.execute_shell_commands(&command.template).unwrap();
        assert!(result.starts_with("Date: 20")); // Year starts with 20
    }

    #[test]
    fn test_command_registry_new() {
        let registry = CommandRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_command_registry_discover() {
        let temp_dir = TempDir::new().unwrap();
        let commands_dir = temp_dir.path().join("commands");
        fs::create_dir(&commands_dir).unwrap();

        // Create a test command file
        let command_file = commands_dir.join("test.toml");
        let mut file = File::create(&command_file).unwrap();
        file.write_all(br#"
name = "test"
description = "Test command"
template = "Hello {{args}}!"
"#).unwrap();

        let mut registry = CommandRegistry::new().with_project_dir(&commands_dir);
        registry.discover().unwrap();

        assert_eq!(registry.len(), 1);
        assert!(registry.get("test").is_some());
    }

    #[test]
    fn test_command_registry_namespace() {
        let temp_dir = TempDir::new().unwrap();
        let commands_dir = temp_dir.path().join("commands");
        fs::create_dir_all(&commands_dir.join("git")).unwrap();

        // Create a namespaced command file
        let command_file = commands_dir.join("git").join("status.toml");
        let mut file = File::create(&command_file).unwrap();
        file.write_all(br#"
name = "status"
description = "Git status command"
template = "!{git status}"
"#).unwrap();

        let mut registry = CommandRegistry::new().with_project_dir(&commands_dir);
        registry.discover().unwrap();

        assert_eq!(registry.len(), 1);
        assert!(registry.get("git:status").is_some());
    }

    #[test]
    fn test_command_registry_precedence() {
        let temp_dir = TempDir::new().unwrap();
        let user_dir = temp_dir.path().join("user");
        let project_dir = temp_dir.path().join("project");
        fs::create_dir(&user_dir).unwrap();
        fs::create_dir(&project_dir).unwrap();

        // Create user command
        let user_command = user_dir.join("test.toml");
        let mut file = File::create(&user_command).unwrap();
        file.write_all(br#"
name = "test"
description = "User command"
template = "User template"
"#).unwrap();

        // Create project command (should override)
        let project_command = project_dir.join("test.toml");
        let mut file = File::create(&project_command).unwrap();
        file.write_all(br#"
name = "test"
description = "Project command"
template = "Project template"
"#).unwrap();

        let mut registry = CommandRegistry::new()
            .with_user_dir(&user_dir)
            .with_project_dir(&project_dir);
        registry.discover().unwrap();

        let command = registry.get("test").unwrap();
        assert_eq!(command.template, "Project template");
    }

    #[test]
    fn test_command_registry_get_not_found() {
        let registry = CommandRegistry::new();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_command_registry_list() {
        let temp_dir = TempDir::new().unwrap();
        let commands_dir = temp_dir.path().join("commands");
        fs::create_dir(&commands_dir).unwrap();

        // Create multiple commands
        for name in &["cmd1", "cmd2", "cmd3"] {
            let command_file = commands_dir.join(format!("{}.toml", name));
            let mut file = File::create(&command_file).unwrap();
            file.write_all(format!(r#"
name = "{}"
description = "Test"
template = "test"
"#, name).as_bytes()).unwrap();
        }

        let mut registry = CommandRegistry::new().with_project_dir(&commands_dir);
        registry.discover().unwrap();

        let commands = registry.list();
        assert_eq!(commands.len(), 3);
        assert!(commands.contains(&"cmd1".to_string()));
        assert!(commands.contains(&"cmd2".to_string()));
        assert!(commands.contains(&"cmd3".to_string()));
    }

    #[test]
    fn test_command_registry_len() {
        let temp_dir = TempDir::new().unwrap();
        let commands_dir = temp_dir.path().join("commands");
        fs::create_dir(&commands_dir).unwrap();

        let mut registry = CommandRegistry::new().with_project_dir(&commands_dir);
        assert_eq!(registry.len(), 0);

        // Create a command
        let command_file = commands_dir.join("test.toml");
        let mut file = File::create(&command_file).unwrap();
        file.write_all(br#"
name = "test"
description = "Test"
template = "test"
"#).unwrap();

        registry.discover().unwrap();
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_command_registry_is_empty() {
        let registry = CommandRegistry::new();
        assert!(registry.is_empty());

        let temp_dir = TempDir::new().unwrap();
        let commands_dir = temp_dir.path().join("commands");
        fs::create_dir(&commands_dir).unwrap();

        let command_file = commands_dir.join("test.toml");
        let mut file = File::create(&command_file).unwrap();
        file.write_all(br#"
name = "test"
description = "Test"
template = "test"
"#).unwrap();

        let mut registry = CommandRegistry::new().with_project_dir(&commands_dir);
        registry.discover().unwrap();
        assert!(!registry.is_empty());
    }

    #[test]
    fn test_custom_command_execute_error_missing_file() {
        let command = CustomCommand {
            name: "test".to_string(),
            description: String::new(),
            template: "Content: @{nonexistent.txt}".to_string(),
            args: vec![],
            namespace: None,
        };

        let temp_dir = TempDir::new().unwrap();
        let result = command.execute(&[], temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_command_substitute_empty_args() {
        let command = CustomCommand {
            name: "test".to_string(),
            description: String::new(),
            template: "No args: {{args}}".to_string(),
            args: vec![],
            namespace: None,
        };

        let args = vec![];
        let result = command.substitute_args(&command.template, &args).unwrap();
        assert_eq!(result, "No args: ");
    }

    #[test]
    fn test_custom_command_multiple_substitutions() {
        let command = CustomCommand {
            name: "test".to_string(),
            description: String::new(),
            template: "{{arg1}} {{arg2}} {{arg1}}".to_string(),
            args: vec![],
            namespace: None,
        };

        let args = vec!["A".to_string(), "B".to_string()];
        let result = command.substitute_args(&command.template, &args).unwrap();
        assert_eq!(result, "A B A");
    }

    #[test]
    fn test_command_registry_multiple_namespaces() {
        let temp_dir = TempDir::new().unwrap();
        let commands_dir = temp_dir.path().join("commands");
        fs::create_dir_all(&commands_dir.join("git")).unwrap();
        fs::create_dir_all(&commands_dir.join("docker")).unwrap();

        // Git command
        let git_file = commands_dir.join("git").join("status.toml");
        let mut file = File::create(&git_file).unwrap();
        file.write_all(br#"
name = "status"
description = "Git status"
template = "git status"
"#).unwrap();

        // Docker command
        let docker_file = commands_dir.join("docker").join("ps.toml");
        let mut file = File::create(&docker_file).unwrap();
        file.write_all(br#"
name = "ps"
description = "Docker ps"
template = "docker ps"
"#).unwrap();

        let mut registry = CommandRegistry::new().with_project_dir(&commands_dir);
        registry.discover().unwrap();

        assert!(registry.get("git:status").is_some());
        assert!(registry.get("docker:ps").is_some());
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_custom_command_mixed_substitutions() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("data.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"file content").unwrap();
        drop(file);

        let command = CustomCommand {
            name: "test".to_string(),
            description: String::new(),
            template: "Arg: {{arg1}}, File: @{data.txt}, Shell: !{echo hello}".to_string(),
            args: vec![],
            namespace: None,
        };

        let args = vec!["value".to_string()];
        let result = command.execute(&args, temp_dir.path()).unwrap();
        assert!(result.contains("Arg: value"));
        assert!(result.contains("File: file content"));
        assert!(result.contains("Shell: hello"));
    }
}
