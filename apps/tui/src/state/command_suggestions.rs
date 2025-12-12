//! Command suggestion state management for auto-completion.
//!
//! Provides a dedicated state structure for managing command auto-completion
//! suggestions with proper encapsulation and helper methods.

/// Source of a command suggestion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuggestionSource {
    /// Built-in TUI command
    BuiltIn,
    /// MCP (Model Context Protocol) command
    MCP,
    /// Agent suggestion
    Agent,
    /// Workflow suggestion
    Workflow,
}

/// Trigger mode for auto-completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerMode {
    /// Auto-completion triggers automatically after N characters
    Auto,
    /// Auto-completion requires explicit trigger (e.g., Ctrl+Space)
    Manual,
}

/// Type of suggestion (command vs parameter).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuggestionType {
    /// Main command or subcommand
    Command,
    /// Parameter value suggestion
    Parameter,
}

/// A single command suggestion with metadata.
#[derive(Debug, Clone)]
pub struct CommandSuggestion {
    /// Full command text (e.g., "/chat agent-id")
    pub command: String,
    /// Description or help text
    pub description: String,
    /// Fuzzy match score (higher is better)
    pub score: i64,
    /// Source of the suggestion
    pub source: SuggestionSource,
    /// Type of suggestion
    pub suggestion_type: SuggestionType,
    /// Parameter name if this is a parameter suggestion
    pub parameter_name: Option<String>,
}

impl CommandSuggestion {
    /// Create a new command suggestion.
    pub fn new(
        command: String,
        description: String,
        score: i64,
        source: SuggestionSource,
    ) -> Self {
        Self {
            command,
            description,
            score,
            source,
            suggestion_type: SuggestionType::Command,
            parameter_name: None,
        }
    }

    /// Create a new parameter suggestion.
    pub fn new_parameter(
        command: String,
        description: String,
        score: i64,
        source: SuggestionSource,
        parameter_name: String,
    ) -> Self {
        Self {
            command,
            description,
            score,
            source,
            suggestion_type: SuggestionType::Parameter,
            parameter_name: Some(parameter_name),
        }
    }
}

/// State for command auto-completion suggestions.
#[derive(Debug, Clone)]
pub struct CommandSuggestionState {
    /// List of suggestions
    pub suggestions: Vec<CommandSuggestion>,
    /// Currently selected suggestion index
    pub selected_index: usize,
    /// Visible range for viewport (start, end)
    pub visible_range: (usize, usize),
    /// Trigger mode (Auto or Manual)
    pub trigger_mode: TriggerMode,
    /// Whether auto-completion is currently active
    pub is_active: bool,
    /// Whether suggestions were manually triggered
    pub triggered_manually: bool,
    /// Error message if any error occurred
    pub error_message: Option<String>,
    /// Result cache for performance optimization
    cache: std::collections::HashMap<String, Vec<CommandSuggestion>>,
    /// Maximum number of suggestions to show in viewport
    pub max_visible: usize,
}

impl CommandSuggestionState {
    /// Create a new command suggestion state.
    pub fn new() -> Self {
        Self {
            suggestions: Vec::new(),
            selected_index: 0,
            visible_range: (0, 0),
            trigger_mode: TriggerMode::Auto,
            is_active: false,
            triggered_manually: false,
            error_message: None,
            cache: std::collections::HashMap::new(),
            max_visible: 8,
        }
    }

    /// Clear all suggestions and reset state.
    pub fn clear(&mut self) {
        self.suggestions.clear();
        self.selected_index = 0;
        self.visible_range = (0, 0);
        self.is_active = false;
        self.triggered_manually = false;
        self.error_message = None;
    }

    /// Set the list of suggestions.
    /// Resets selection if the new list is shorter than current selection.
    pub fn set_suggestions(&mut self, suggestions: Vec<CommandSuggestion>) {
        // Safety check: reset selection if list shrinks
        if suggestions.len() <= self.selected_index {
            self.selected_index = 0;
        }
        
        // Validate list size (prevent excessive memory usage)
        let suggestions = if suggestions.len() > 1000 {
            // Log warning would go here if we had access to logging
            suggestions.into_iter().take(1000).collect()
        } else {
            suggestions
        };

        self.suggestions = suggestions;
        self.update_viewport();
    }

    /// Move selection to next suggestion (with wraparound).
    pub fn select_next(&mut self) {
        if self.suggestions.is_empty() {
            return;
        }
        self.selected_index = (self.selected_index + 1) % self.suggestions.len();
        self.update_viewport();
    }

    /// Move selection to previous suggestion (with wraparound).
    pub fn select_previous(&mut self) {
        if self.suggestions.is_empty() {
            return;
        }
        self.selected_index = (self.selected_index + self.suggestions.len() - 1) % self.suggestions.len();
        self.update_viewport();
    }

    /// Move selection to first suggestion.
    pub fn select_first(&mut self) {
        self.selected_index = 0;
        self.update_viewport();
    }

    /// Move selection to last suggestion.
    pub fn select_last(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected_index = self.suggestions.len() - 1;
            self.update_viewport();
        }
    }

    /// Move selection down by viewport size (page down).
    pub fn select_page_down(&mut self) {
        if self.suggestions.is_empty() {
            return;
        }
        let jump = self.max_visible;
        self.selected_index = (self.selected_index + jump) % self.suggestions.len();
        self.update_viewport();
    }

    /// Move selection up by viewport size (page up).
    pub fn select_page_up(&mut self) {
        if self.suggestions.is_empty() {
            return;
        }
        let jump = self.max_visible;
        self.selected_index = (self.selected_index + self.suggestions.len() - jump) % self.suggestions.len();
        self.update_viewport();
    }

    /// Get the currently selected suggestion.
    /// Returns None if selection is out of bounds.
    pub fn get_selected(&self) -> Option<&CommandSuggestion> {
        if self.selected_index < self.suggestions.len() {
            self.suggestions.get(self.selected_index)
        } else {
            None
        }
    }

    /// Update the visible range based on current selection.
    /// Centers the selection in the viewport when possible.
    pub fn update_viewport(&mut self) {
        let total = self.suggestions.len();
        
        if total == 0 {
            self.visible_range = (0, 0);
            return;
        }

        if total <= self.max_visible {
            // All items fit in viewport
            self.visible_range = (0, total);
        } else {
            // Try to center selection in viewport
            let half_window = self.max_visible / 2;
            let start = self.selected_index.saturating_sub(half_window);
            let end = (start + self.max_visible).min(total);
            
            // Adjust start if we hit the end
            let start = if end == total {
                total.saturating_sub(self.max_visible)
            } else {
                start
            };
            
            self.visible_range = (start, end);
        }
    }

    /// Get cached suggestions for a query, if available.
    pub fn get_cached(&self, query: &str) -> Option<&Vec<CommandSuggestion>> {
        self.cache.get(query)
    }

    /// Cache suggestions for a query.
    /// Implements simple LRU by limiting cache size to 50 entries.
    pub fn cache_suggestions(&mut self, query: String, suggestions: Vec<CommandSuggestion>) {
        // Simple cache size limit (LRU would be better but this is simpler)
        if self.cache.len() >= 50 {
            // Remove oldest entry (simple approach - could use proper LRU)
            if let Some(key) = self.cache.keys().next().cloned() {
                self.cache.remove(&key);
            }
        }
        self.cache.insert(query, suggestions);
    }

    /// Clear the suggestion cache.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for CommandSuggestionState {
    fn default() -> Self {
        Self::new()
    }
}




