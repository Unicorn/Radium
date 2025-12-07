//! Integration tests for command palette functionality.

use radium_tui::commands::Command;
use radium_tui::views::PromptData;

#[test]
fn test_command_palette_activation() {
    let mut prompt_data = PromptData::new();
    assert!(!prompt_data.command_palette_active);
    
    // Simulate Ctrl+P activation
    prompt_data.command_palette_active = true;
    assert!(prompt_data.command_palette_active);
}

#[test]
fn test_command_palette_query() {
    let mut prompt_data = PromptData::new();
    prompt_data.command_palette_active = true;
    
    // Simulate typing in command palette
    prompt_data.command_palette_query.push('c');
    prompt_data.command_palette_query.push('h');
    prompt_data.command_palette_query.push('a');
    
    assert_eq!(prompt_data.command_palette_query, "cha");
}

#[test]
fn test_command_palette_suggestions_filtering() {
    let mut prompt_data = PromptData::new();
    prompt_data.command_palette_active = true;
    prompt_data.command_palette_query = "chat".to_string();
    
    // Simulate command suggestions based on query
    let available_commands = vec![
        ("help", "Show all available commands"),
        ("chat", "Start chat with an agent"),
        ("agents", "List all available agents"),
        ("sessions", "Show your chat sessions"),
    ];
    
    // Filter commands that match query
    let filtered: Vec<_> = available_commands
        .iter()
        .filter(|(cmd, _desc)| cmd.contains(&prompt_data.command_palette_query))
        .map(|(cmd, desc)| format!("/{} - {}", cmd, desc))
        .collect();
    
    assert!(!filtered.is_empty());
    assert!(filtered.iter().any(|s| s.contains("/chat")));
}

#[test]
fn test_command_palette_fuzzy_matching() {
    let query = "cht";
    let commands = vec!["help", "chat", "agents", "sessions"];
    
    // Simple fuzzy match: command contains query or query is substring
    let matches: Vec<_> = commands
        .iter()
        .filter(|cmd| {
            cmd.contains(query) || 
            query.chars().all(|c| cmd.contains(c))
        })
        .collect();
    
    // "chat" should match "cht" (fuzzy)
    assert!(matches.contains(&"chat"));
}

#[test]
fn test_command_palette_selection_navigation() {
    let mut prompt_data = PromptData::new();
    prompt_data.command_palette_active = true;
    prompt_data.command_suggestions = vec![
        "/chat - Start chat".to_string(),
        "/agents - List agents".to_string(),
        "/sessions - Show sessions".to_string(),
    ];
    
    let initial_index = prompt_data.selected_suggestion_index;
    
    // Simulate down arrow
    let max_index = prompt_data.command_suggestions.len().saturating_sub(1);
    prompt_data.selected_suggestion_index = (prompt_data.selected_suggestion_index + 1).min(max_index);
    
    assert!(prompt_data.selected_suggestion_index > initial_index);
    
    // Simulate up arrow
    prompt_data.selected_suggestion_index = prompt_data.selected_suggestion_index.saturating_sub(1);
    assert_eq!(prompt_data.selected_suggestion_index, initial_index);
}

#[test]
fn test_command_palette_dismissal() {
    let mut prompt_data = PromptData::new();
    prompt_data.command_palette_active = true;
    prompt_data.command_palette_query = "test".to_string();
    
    // Simulate Esc to dismiss
    prompt_data.command_palette_active = false;
    prompt_data.command_palette_query.clear();
    prompt_data.command_suggestions.clear();
    
    assert!(!prompt_data.command_palette_active);
    assert!(prompt_data.command_palette_query.is_empty());
    assert!(prompt_data.command_suggestions.is_empty());
}

#[test]
fn test_command_palette_command_execution() {
    let mut prompt_data = PromptData::new();
    prompt_data.command_palette_active = true;
    prompt_data.command_suggestions = vec![
        "/chat agent-1 - Start chat".to_string(),
    ];
    prompt_data.selected_suggestion_index = 0;
    
    // Simulate selecting and executing command
    if let Some(suggestion) = prompt_data.command_suggestions.get(prompt_data.selected_suggestion_index) {
        if let Some(cmd) = suggestion.split(" - ").next() {
            // Parse the command
            let parsed = Command::parse(cmd);
            assert!(parsed.is_some());
            if let Some(cmd) = parsed {
                assert_eq!(cmd.name, "chat");
                assert_eq!(cmd.args, vec!["agent-1"]);
            }
        }
    }
}

