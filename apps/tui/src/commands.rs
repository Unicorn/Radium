//! Command parsing and execution for unified prompt interface.

pub mod models;

/// A parsed command from user input.
#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
}

impl Command {
    /// Parse a command from input string.
    ///
    /// Slash commands start with '/', otherwise treated as chat message.
    pub fn parse(input: &str) -> Option<Self> {
        let input = input.trim();

        if !input.starts_with('/') {
            return None;
        }

        let parts: Vec<&str> = input[1..].split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        Some(Command {
            name: parts[0].to_lowercase(),
            args: parts[1..].iter().map(|s| s.to_string()).collect(),
        })
    }
}

/// Context for what's currently displayed in the main area.
#[derive(Debug, Clone, PartialEq)]
pub enum DisplayContext {
    /// Chatting with an agent
    Chat { agent_id: String, session_id: String },
    /// Viewing list of available agents
    AgentList,
    /// Viewing list of chat sessions
    SessionList,
    /// Viewing dashboard stats
    Dashboard,
    /// Viewing help information
    Help,
    /// Viewing model selector
    ModelSelector,
}

impl Default for DisplayContext {
    fn default() -> Self {
        Self::Help
    }
}

impl DisplayContext {
    /// Get display title for current context.
    pub fn title(&self) -> String {
        match self {
            Self::Chat { agent_id, .. } => format!("Chat with {}", agent_id),
            Self::AgentList => "Available Agents".to_string(),
            Self::SessionList => "Chat Sessions".to_string(),
            Self::Dashboard => "Dashboard".to_string(),
            Self::Help => "Help".to_string(),
            Self::ModelSelector => "Model Selection".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_slash_command() {
        let cmd = Command::parse("/chat my-agent").unwrap();
        assert_eq!(cmd.name, "chat");
        assert_eq!(cmd.args, vec!["my-agent"]);
    }

    #[test]
    fn test_parse_no_slash() {
        assert!(Command::parse("hello world").is_none());
    }

    #[test]
    fn test_parse_empty() {
        assert!(Command::parse("/").is_none());
    }
}
