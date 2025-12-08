//! Unit tests for command suggestion state management.

use super::{CommandSuggestion, CommandSuggestionState, SuggestionSource, TriggerMode};

#[test]
fn test_state_initialization() {
    let state = CommandSuggestionState::new();
    assert_eq!(state.suggestions.len(), 0);
    assert_eq!(state.selected_index, 0);
    assert_eq!(state.visible_range, (0, 0));
    assert_eq!(state.trigger_mode, TriggerMode::Auto);
    assert!(!state.is_active);
    assert!(!state.triggered_manually);
    assert!(state.error_message.is_none());
}

#[test]
fn test_set_suggestions() {
    let mut state = CommandSuggestionState::new();
    let suggestions = vec![
        CommandSuggestion::new("/chat".to_string(), "Start chat".to_string(), 100, SuggestionSource::BuiltIn),
        CommandSuggestion::new("/help".to_string(), "Show help".to_string(), 90, SuggestionSource::BuiltIn),
    ];
    
    state.set_suggestions(suggestions.clone());
    assert_eq!(state.suggestions.len(), 2);
    assert_eq!(state.selected_index, 0); // Should reset to 0
}

#[test]
fn test_set_suggestions_resets_selection_when_list_shrinks() {
    let mut state = CommandSuggestionState::new();
    let suggestions = vec![
        CommandSuggestion::new("/chat".to_string(), "Start chat".to_string(), 100, SuggestionSource::BuiltIn),
        CommandSuggestion::new("/help".to_string(), "Show help".to_string(), 90, SuggestionSource::BuiltIn),
        CommandSuggestion::new("/agents".to_string(), "List agents".to_string(), 80, SuggestionSource::BuiltIn),
    ];
    
    state.set_suggestions(suggestions);
    state.selected_index = 2; // Select last item
    
    // Shrink list
    let smaller = vec![
        CommandSuggestion::new("/chat".to_string(), "Start chat".to_string(), 100, SuggestionSource::BuiltIn),
    ];
    state.set_suggestions(smaller);
    
    assert_eq!(state.selected_index, 0); // Should reset to 0
}

#[test]
fn test_select_next_with_wraparound() {
    let mut state = CommandSuggestionState::new();
    let suggestions = vec![
        CommandSuggestion::new("/chat".to_string(), "Start chat".to_string(), 100, SuggestionSource::BuiltIn),
        CommandSuggestion::new("/help".to_string(), "Show help".to_string(), 90, SuggestionSource::BuiltIn),
    ];
    
    state.set_suggestions(suggestions);
    state.selected_index = 1; // Select last item
    
    state.select_next();
    assert_eq!(state.selected_index, 0); // Should wrap to first
}

#[test]
fn test_select_previous_with_wraparound() {
    let mut state = CommandSuggestionState::new();
    let suggestions = vec![
        CommandSuggestion::new("/chat".to_string(), "Start chat".to_string(), 100, SuggestionSource::BuiltIn),
        CommandSuggestion::new("/help".to_string(), "Show help".to_string(), 90, SuggestionSource::BuiltIn),
    ];
    
    state.set_suggestions(suggestions);
    state.selected_index = 0; // Select first item
    
    state.select_previous();
    assert_eq!(state.selected_index, 1); // Should wrap to last
}

#[test]
fn test_select_first() {
    let mut state = CommandSuggestionState::new();
    let suggestions = vec![
        CommandSuggestion::new("/chat".to_string(), "Start chat".to_string(), 100, SuggestionSource::BuiltIn),
        CommandSuggestion::new("/help".to_string(), "Show help".to_string(), 90, SuggestionSource::BuiltIn),
    ];
    
    state.set_suggestions(suggestions);
    state.selected_index = 1;
    
    state.select_first();
    assert_eq!(state.selected_index, 0);
}

#[test]
fn test_select_last() {
    let mut state = CommandSuggestionState::new();
    let suggestions = vec![
        CommandSuggestion::new("/chat".to_string(), "Start chat".to_string(), 100, SuggestionSource::BuiltIn),
        CommandSuggestion::new("/help".to_string(), "Show help".to_string(), 90, SuggestionSource::BuiltIn),
    ];
    
    state.set_suggestions(suggestions);
    state.selected_index = 0;
    
    state.select_last();
    assert_eq!(state.selected_index, 1);
}

#[test]
fn test_select_page_down() {
    let mut state = CommandSuggestionState::new();
    state.max_visible = 8;
    let suggestions: Vec<_> = (0..20)
        .map(|i| CommandSuggestion::new(
            format!("/cmd{}", i),
            format!("Command {}", i),
            100 - i as i64,
            SuggestionSource::BuiltIn,
        ))
        .collect();
    
    state.set_suggestions(suggestions);
    state.selected_index = 0;
    
    state.select_page_down();
    assert_eq!(state.selected_index, 8); // Should jump by viewport size
}

#[test]
fn test_select_page_up() {
    let mut state = CommandSuggestionState::new();
    state.max_visible = 8;
    let suggestions: Vec<_> = (0..20)
        .map(|i| CommandSuggestion::new(
            format!("/cmd{}", i),
            format!("Command {}", i),
            100 - i as i64,
            SuggestionSource::BuiltIn,
        ))
        .collect();
    
    state.set_suggestions(suggestions);
    state.selected_index = 10;
    
    state.select_page_up();
    assert_eq!(state.selected_index, 2); // Should jump backward by viewport size
}

#[test]
fn test_get_selected_valid() {
    let mut state = CommandSuggestionState::new();
    let suggestions = vec![
        CommandSuggestion::new("/chat".to_string(), "Start chat".to_string(), 100, SuggestionSource::BuiltIn),
        CommandSuggestion::new("/help".to_string(), "Show help".to_string(), 90, SuggestionSource::BuiltIn),
    ];
    
    state.set_suggestions(suggestions);
    state.selected_index = 0;
    
    let selected = state.get_selected();
    assert!(selected.is_some());
    assert_eq!(selected.unwrap().command, "/chat");
}

#[test]
fn test_get_selected_out_of_bounds() {
    let mut state = CommandSuggestionState::new();
    let suggestions = vec![
        CommandSuggestion::new("/chat".to_string(), "Start chat".to_string(), 100, SuggestionSource::BuiltIn),
    ];
    
    state.set_suggestions(suggestions);
    state.selected_index = 999; // Out of bounds
    
    let selected = state.get_selected();
    assert!(selected.is_none());
}

#[test]
fn test_update_viewport_small_list() {
    let mut state = CommandSuggestionState::new();
    state.max_visible = 8;
    let suggestions: Vec<_> = (0..5)
        .map(|i| CommandSuggestion::new(
            format!("/cmd{}", i),
            format!("Command {}", i),
            100,
            SuggestionSource::BuiltIn,
        ))
        .collect();
    
    state.set_suggestions(suggestions);
    state.selected_index = 2;
    state.update_viewport();
    
    // All items should be visible
    assert_eq!(state.visible_range, (0, 5));
}

#[test]
fn test_update_viewport_large_list() {
    let mut state = CommandSuggestionState::new();
    state.max_visible = 8;
    let suggestions: Vec<_> = (0..20)
        .map(|i| CommandSuggestion::new(
            format!("/cmd{}", i),
            format!("Command {}", i),
            100,
            SuggestionSource::BuiltIn,
        ))
        .collect();
    
    state.set_suggestions(suggestions);
    state.selected_index = 10;
    state.update_viewport();
    
    // Viewport should be centered on selection
    let (start, end) = state.visible_range;
    assert!(start <= 10);
    assert!(end > 10);
    assert_eq!(end - start, 8); // Should show 8 items
}

#[test]
fn test_cache_suggestions() {
    let mut state = CommandSuggestionState::new();
    let suggestions = vec![
        CommandSuggestion::new("/chat".to_string(), "Start chat".to_string(), 100, SuggestionSource::BuiltIn),
    ];
    
    state.cache_suggestions("test".to_string(), suggestions.clone());
    
    let cached = state.get_cached("test");
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().len(), 1);
}

#[test]
fn test_clear() {
    let mut state = CommandSuggestionState::new();
    let suggestions = vec![
        CommandSuggestion::new("/chat".to_string(), "Start chat".to_string(), 100, SuggestionSource::BuiltIn),
    ];
    
    state.set_suggestions(suggestions);
    state.selected_index = 0;
    state.is_active = true;
    state.triggered_manually = true;
    state.error_message = Some("Error".to_string());
    
    state.clear();
    
    assert_eq!(state.suggestions.len(), 0);
    assert_eq!(state.selected_index, 0);
    assert!(!state.is_active);
    assert!(!state.triggered_manually);
    assert!(state.error_message.is_none());
}

