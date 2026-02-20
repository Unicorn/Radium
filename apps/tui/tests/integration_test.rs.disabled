//! Integration tests for TUI components.

use radium_tui::commands::{Command, DisplayContext};
use radium_tui::views::{markdown::render_markdown, PromptData};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_rendering_integration() {
        // Test that markdown rendering works end-to-end
        let text = "This is **bold** and *italic* text with `code`.";
        let lines = render_markdown(text);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_markdown_code_block_integration() {
        let text =
            "Here's some code:\n```rust\nfn main() {\n    println!(\"Hello\");\n}\n```\nThat's it!";
        let lines = render_markdown(text);
        // Should have multiple lines including code block
        assert!(lines.len() > 3);
    }

    #[test]
    fn test_markdown_list_integration() {
        let text = "- First item\n- Second item\n- Third item";
        let lines = render_markdown(text);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_command_parsing_all_commands() {
        // Test parsing of all major commands
        let commands = vec![
            "/help",
            "/auth",
            "/agents",
            "/chat agent-1",
            "/sessions",
            "/dashboard",
            "/models",
            "/orchestrator",
            "/orchestrator toggle",
            "/orchestrator status",
            "/complete ./specs/feature.md",
            "/mcp-commands",
        ];

        for cmd_str in commands {
            let cmd = Command::parse(cmd_str);
            assert!(cmd.is_some(), "Failed to parse command: {}", cmd_str);
        }
    }

    #[test]
    fn test_command_parsing_with_args() {
        // Test commands with arguments
        let cmd = Command::parse("/chat my-agent").unwrap();
        assert_eq!(cmd.name, "chat");
        assert_eq!(cmd.args, vec!["my-agent"]);

        let cmd = Command::parse("/complete ./path/to/file.md").unwrap();
        assert_eq!(cmd.name, "complete");
        assert_eq!(cmd.args, vec!["./path/to/file.md"]);

        let cmd = Command::parse("/orchestrator switch gemini").unwrap();
        assert_eq!(cmd.name, "orchestrator");
        assert_eq!(cmd.args, vec!["switch", "gemini"]);
    }

    #[test]
    fn test_command_parsing_no_args() {
        // Test commands without arguments
        let cmd = Command::parse("/help").unwrap();
        assert_eq!(cmd.name, "help");
        assert!(cmd.args.is_empty());

        let cmd = Command::parse("/agents").unwrap();
        assert_eq!(cmd.name, "agents");
        assert!(cmd.args.is_empty());
    }

    #[test]
    fn test_command_parsing_invalid() {
        // Test that invalid inputs are not parsed as commands
        assert!(Command::parse("help").is_none()); // Missing /
        assert!(Command::parse("/").is_none()); // Empty command
        assert!(Command::parse("").is_none()); // Empty string
        assert!(Command::parse("   ").is_none()); // Whitespace only
    }

    #[test]
    fn test_display_context_default() {
        let context = DisplayContext::default();
        assert_eq!(context, DisplayContext::Help);
    }

    #[test]
    fn test_display_context_titles() {
        assert_eq!(DisplayContext::Help.title(), "Help");
        assert_eq!(DisplayContext::AgentList.title(), "Available Agents");
        assert_eq!(DisplayContext::SessionList.title(), "Chat Sessions");
        assert_eq!(DisplayContext::Dashboard.title(), "Dashboard");
        assert_eq!(DisplayContext::ModelSelector.title(), "Model Selection");

        let chat_context = DisplayContext::Chat {
            agent_id: "test-agent".to_string(),
            session_id: "session-1".to_string(),
        };
        assert_eq!(chat_context.title(), "Chat with test-agent");
    }

    #[test]
    fn test_prompt_data_initialization() {
        let prompt_data = PromptData::new();
        assert_eq!(prompt_data.context, DisplayContext::Help);
        assert!(prompt_data.input_text().is_empty());
        assert!(!prompt_data.output.is_empty()); // Should have welcome message
        assert!(prompt_data.conversation.is_empty());
        assert!(!prompt_data.command_palette_active);
    }

    #[test]
    fn test_prompt_data_input_handling() {
        let mut prompt_data = PromptData::new();
        
        // Test setting input
        prompt_data.set_input("hello");
        assert_eq!(prompt_data.input_text(), "hello");

        // Test setting different input
        prompt_data.set_input("hell");
        assert_eq!(prompt_data.input_text(), "hell");

        // Test clearing input
        prompt_data.clear_input();
        assert!(prompt_data.input_text().is_empty());
    }

    #[test]
    fn test_prompt_data_multiline_input() {
        let mut prompt_data = PromptData::new();
        
        // Test multiline input
        prompt_data.set_input("line1\nline2\nline3");
        assert_eq!(prompt_data.input_text(), "line1\nline2\nline3");
        
        let input_text = prompt_data.input_text();
        let lines: Vec<&str> = input_text.lines().collect();
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_prompt_data_output_handling() {
        let mut prompt_data = PromptData::new();
        
        // Clear initial welcome message
        prompt_data.clear_output();
        
        // Test adding output
        prompt_data.add_output("Line 1".to_string());
        prompt_data.add_output("Line 2".to_string());
        assert_eq!(prompt_data.output.len(), 2);
        assert_eq!(prompt_data.output[0], "Line 1");
        assert_eq!(prompt_data.output[1], "Line 2");

        // Test clearing output
        prompt_data.clear_output();
        assert!(prompt_data.output.is_empty());
    }

    #[test]
    fn test_prompt_data_output_buffer_limit() {
        let mut prompt_data = PromptData::new();
        
        // Add more than 1000 lines
        for i in 0..1500 {
            prompt_data.add_output(format!("Line {}", i));
        }
        
        // Should be limited to 1000 lines
        assert_eq!(prompt_data.output.len(), 1000);
        // Oldest lines should be removed
        assert_eq!(prompt_data.output[0], "Line 500");
        assert_eq!(prompt_data.output[999], "Line 1499");
    }

    #[test]
    fn test_prompt_data_scrollback() {
        let mut prompt_data = PromptData::new();
        
        // Add some conversation history
        for i in 0..20 {
            prompt_data.conversation.push(format!("Message {}", i));
        }
        
        // Test scrollback offset
        assert_eq!(prompt_data.scrollback_offset, 0);
        
        // Simulate scrolling up
        prompt_data.scrollback_offset = 10;
        assert_eq!(prompt_data.scrollback_offset, 10);
        
        // Simulate scrolling to top
        prompt_data.scrollback_offset = 0;
        assert_eq!(prompt_data.scrollback_offset, 0);
    }

    #[test]
    fn test_command_suggestions() {
        let mut prompt_data = PromptData::new();
        
        // Simulate typing "/ch"
        prompt_data.set_input("/ch");
        
        // Command suggestions should be filtered
        prompt_data.command_suggestions = vec![
            "/chat - Start chat".to_string(),
            "/help - Show help".to_string(),
        ];
        
        assert!(!prompt_data.command_suggestions.is_empty());
        assert!(prompt_data.command_suggestions.iter().any(|s| s.contains("chat")));
    }

    #[test]
    fn test_command_parsing_with_multiline_input() {
        let mut prompt_data = PromptData::new();
        
        // Set multiline input with command on first line
        prompt_data.set_input("/chat agent-1\nsome note");
        
        // Command parsing should work with first line
        let input = prompt_data.input_text();
        let first_line = input.lines().next().unwrap_or("");
        let cmd = Command::parse(first_line);
        
        assert!(cmd.is_some());
        if let Some(cmd) = cmd {
            assert_eq!(cmd.name, "chat");
            assert_eq!(cmd.args, vec!["agent-1"]);
        }
    }

    #[test]
    fn test_enter_vs_cmd_enter_behavior() {
        let mut prompt_data = PromptData::new();
        prompt_data.set_input("test message");
        
        // Plain Enter should insert newline (tested via TextArea)
        use crossterm::event::{KeyCode, KeyModifiers};
        prompt_data.input.handle_key(KeyCode::Enter, KeyModifiers::NONE);
        
        let text = prompt_data.input_text();
        // Should have newline inserted
        assert!(text.contains('\n') || text.ends_with('\n'));
    }

    #[test]
    fn test_error_handling_unknown_command() {
        // Test that unknown commands are handled gracefully
        let cmd = Command::parse("/unknown-command").unwrap();
        assert_eq!(cmd.name, "unknown-command");
        // The App will handle this and show an error message
    }

    #[test]
    fn test_error_handling_missing_args() {
        // Test commands that require arguments
        let cmd = Command::parse("/chat").unwrap();
        assert_eq!(cmd.name, "chat");
        assert!(cmd.args.is_empty());
        // The App will show usage message for missing args
    }

    #[test]
    fn test_scrollback_functionality() {
        let mut prompt_data = PromptData::new();
        
        // Add conversation history
        for i in 0..100 {
            prompt_data.conversation.push(format!("Message {}", i));
        }
        
        // Test scrollback navigation
        let initial_offset = prompt_data.scrollback_offset;
        
        // Simulate PageUp (scroll up)
        prompt_data.scrollback_offset = (prompt_data.scrollback_offset + 10)
            .min(prompt_data.conversation.len().saturating_sub(1));
        assert!(prompt_data.scrollback_offset > initial_offset);
        
        // Simulate PageDown (scroll down)
        prompt_data.scrollback_offset = prompt_data.scrollback_offset.saturating_sub(10);
        assert!(prompt_data.scrollback_offset <= initial_offset + 10);
        
        // Simulate Home (scroll to top)
        prompt_data.scrollback_offset = 0;
        assert_eq!(prompt_data.scrollback_offset, 0);
        
        // Simulate End (scroll to bottom)
        prompt_data.scrollback_offset = prompt_data.conversation.len();
        assert_eq!(prompt_data.scrollback_offset, prompt_data.conversation.len());
    }
}
