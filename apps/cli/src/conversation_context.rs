//! Conversation context tracking for CLI chat sessions.
//!
//! Provides structured tracking of topics, decisions, tasks, and entities
//! across multi-turn conversations to enable context-aware interactions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tracks the semantic context of a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    /// Active topics being discussed
    pub topics: Vec<Topic>,

    /// Key decisions made in conversation
    pub decisions: Vec<Decision>,

    /// Tasks identified or completed
    pub tasks: Vec<Task>,

    /// Files and code entities referenced
    pub entities: HashMap<String, EntityContext>,

    /// Current user intent
    pub intent: Option<UserIntent>,
}

/// A topic of discussion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub name: String,
    pub first_mentioned_turn: usize,
    pub last_mentioned_turn: usize,
    pub relevance_score: f32,
}

/// A decision made during conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub description: String,
    pub turn_number: usize,
    pub rationale: Option<String>,
}

/// A task identified or completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub description: String,
    pub status: TaskStatus,
    pub created_turn: usize,
    pub completed_turn: Option<usize>,
}

/// Status of a task
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Identified,
    InProgress,
    Completed,
    Blocked,
}

/// Context about an entity (file, function, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityContext {
    pub entity_type: EntityType,
    pub first_referenced: usize,
    pub last_referenced: usize,
    pub operations: Vec<String>, // e.g., "read", "modified", "analyzed"
}

/// Type of entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    File,
    Directory,
    GitCommit,
    Function,
    Module,
}

/// User's current intent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserIntent {
    Exploration,      // "What is...", "Show me..."
    Implementation,   // "Add feature...", "Fix bug..."
    Analysis,         // "Why does...", "How does..."
    Refactoring,      // "Clean up...", "Improve..."
    Documentation,    // "Document...", "Explain..."
}

impl ConversationContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self {
            topics: Vec::new(),
            decisions: Vec::new(),
            tasks: Vec::new(),
            entities: HashMap::new(),
            intent: None,
        }
    }

    /// Update context after each turn (Phase 1 - simple pattern matching)
    pub fn update_from_turn(
        &mut self,
        turn_number: usize,
        user_msg: &str,
        assistant_msg: &str,
        tool_calls: &[String],
    ) {
        // Extract topics from keywords
        self.extract_topics(turn_number, user_msg);

        // Track file references from tool calls
        self.extract_entities(turn_number, tool_calls);

        // Detect intent from user message
        self.detect_intent(user_msg);

        // Detect decisions (simple pattern matching)
        self.detect_decisions(turn_number, user_msg, assistant_msg);

        // Detect tasks
        self.detect_tasks(turn_number, user_msg);
    }

    /// Generate context summary for system prompt
    pub fn to_system_context(&self) -> String {
        let mut parts = Vec::new();

        if !self.topics.is_empty() {
            let topic_names: Vec<String> = self.topics
                .iter()
                .take(5)
                .map(|t| format!("{} ({:.1})", t.name, t.relevance_score))
                .collect();
            parts.push(format!("Topics: {}", topic_names.join(", ")));
        }

        if !self.decisions.is_empty() {
            parts.push(format!("Decisions made: {}", self.decisions.len()));
            for decision in self.decisions.iter().rev().take(3) {
                parts.push(format!("  - {}", decision.description));
            }
        }

        if !self.tasks.is_empty() {
            let active_tasks: Vec<_> = self.tasks
                .iter()
                .filter(|t| t.status != TaskStatus::Completed)
                .collect();
            if !active_tasks.is_empty() {
                parts.push(format!("Active tasks: {}", active_tasks.len()));
                for task in active_tasks.iter().take(3) {
                    parts.push(format!("  - {} ({:?})", task.description, task.status));
                }
            }
        }

        if let Some(ref intent) = self.intent {
            parts.push(format!("Current intent: {:?}", intent));
        }

        if parts.is_empty() {
            String::new()
        } else {
            format!("## Conversation Context\n\n{}\n", parts.join("\n"))
        }
    }

    /// Extract topics from keywords (Phase 1 - simple)
    fn extract_topics(&mut self, turn: usize, user_msg: &str) {
        let msg_lower = user_msg.to_lowercase();

        // Programming language topics
        if msg_lower.contains("rust") || msg_lower.contains(".rs") {
            self.update_or_create_topic("rust", turn);
        }
        if msg_lower.contains("typescript") || msg_lower.contains(".ts") || msg_lower.contains(".tsx") {
            self.update_or_create_topic("typescript", turn);
        }
        if msg_lower.contains("python") || msg_lower.contains(".py") {
            self.update_or_create_topic("python", turn);
        }

        // Development topics
        if msg_lower.contains("test") || msg_lower.contains("testing") {
            self.update_or_create_topic("testing", turn);
        }
        if msg_lower.contains("git") || msg_lower.contains("commit") || msg_lower.contains("branch") {
            self.update_or_create_topic("git", turn);
        }
        if msg_lower.contains("deploy") || msg_lower.contains("deployment") {
            self.update_or_create_topic("deployment", turn);
        }
        if msg_lower.contains("performance") || msg_lower.contains("optimize") {
            self.update_or_create_topic("performance", turn);
        }
        if msg_lower.contains("security") || msg_lower.contains("auth") {
            self.update_or_create_topic("security", turn);
        }
        if msg_lower.contains("database") || msg_lower.contains("db") || msg_lower.contains("sql") {
            self.update_or_create_topic("database", turn);
        }
        if msg_lower.contains("api") || msg_lower.contains("endpoint") {
            self.update_or_create_topic("api", turn);
        }

        // Decay relevance of topics not mentioned
        for topic in &mut self.topics {
            if topic.last_mentioned_turn < turn {
                topic.relevance_score *= 0.9;
            }
        }

        // Sort by relevance
        self.topics.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
    }

    /// Update existing topic or create new one
    fn update_or_create_topic(&mut self, name: &str, turn: usize) {
        if let Some(topic) = self.topics.iter_mut().find(|t| t.name == name) {
            topic.last_mentioned_turn = turn;
            topic.relevance_score = (topic.relevance_score * 0.8) + 0.3; // Decay + boost
        } else {
            self.topics.push(Topic {
                name: name.to_string(),
                first_mentioned_turn: turn,
                last_mentioned_turn: turn,
                relevance_score: 1.0,
            });
        }
    }

    /// Extract file/entity references from tool calls
    fn extract_entities(&mut self, _turn: usize, tool_calls: &[String]) {
        for tool_call in tool_calls {
            // Simple pattern matching for file operations
            if tool_call.contains("read_file") || tool_call.contains("write_file") {
                // In a real implementation, would parse the actual file path
                // For now, just track that file operations occurred
            }
        }
    }

    /// Detect user intent from message
    fn detect_intent(&mut self, user_msg: &str) {
        let msg_lower = user_msg.to_lowercase();

        self.intent = if msg_lower.starts_with("what")
            || msg_lower.starts_with("show")
            || msg_lower.starts_with("list")
            || msg_lower.contains("find")
            || msg_lower.contains("search") {
            Some(UserIntent::Exploration)
        } else if msg_lower.contains("add")
            || msg_lower.contains("implement")
            || msg_lower.contains("create")
            || msg_lower.contains("fix bug")
            || msg_lower.contains("solve") {
            Some(UserIntent::Implementation)
        } else if msg_lower.starts_with("why")
            || msg_lower.starts_with("how")
            || msg_lower.contains("analyze")
            || msg_lower.contains("understand")
            || msg_lower.contains("explain") {
            Some(UserIntent::Analysis)
        } else if msg_lower.contains("refactor")
            || msg_lower.contains("clean up")
            || msg_lower.contains("improve")
            || msg_lower.contains("optimize") {
            Some(UserIntent::Refactoring)
        } else if msg_lower.contains("document")
            || msg_lower.contains("comment")
            || msg_lower.contains("describe") {
            Some(UserIntent::Documentation)
        } else {
            self.intent.clone() // Keep previous intent if unclear
        };
    }

    /// Detect decisions from conversation
    fn detect_decisions(&mut self, _turn: usize, user_msg: &str, _assistant_msg: &str) {
        let user_lower = user_msg.to_lowercase();

        // Detect decision changes
        if user_lower.contains("actually") || user_lower.contains("instead") || user_lower.contains("let's use") {
            self.decisions.push(Decision {
                description: format!("Changed approach based on: {}",
                    user_msg.chars().take(100).collect::<String>()),
                turn_number: turn,
                rationale: None,
            });
        }

        // Detect confirmations
        if user_lower.starts_with("yes") || user_lower.starts_with("ok") || user_lower.starts_with("sure") {
            self.decisions.push(Decision {
                description: "User confirmed previous suggestion".to_string(),
                turn_number: turn,
                rationale: None,
            });
        }
    }

    /// Detect tasks from conversation
    fn detect_tasks(&mut self, turn: usize, user_msg: &str) {
        let msg_lower = user_msg.to_lowercase();

        // Detect new tasks
        if msg_lower.contains("need to") || msg_lower.contains("should") || msg_lower.contains("want to") {
            // Extract task description (simplified)
            let description = user_msg.chars().take(100).collect::<String>();

            self.tasks.push(Task {
                description,
                status: TaskStatus::Identified,
                created_turn: turn,
                completed_turn: None,
            });
        }

        // Detect task completion
        if msg_lower.contains("done") || msg_lower.contains("finished") || msg_lower.contains("completed") {
            // Mark recent tasks as completed
            if let Some(task) = self.tasks.iter_mut()
                .filter(|t| t.status != TaskStatus::Completed)
                .last() {
                task.status = TaskStatus::Completed;
                task.completed_turn = Some(turn);
            }
        }
    }
}

impl Default for ConversationContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_extraction() {
        let mut context = ConversationContext::new();
        context.update_from_turn(1, "Let's work on the Rust codebase", "", &[]);

        assert_eq!(context.topics.len(), 1);
        assert_eq!(context.topics[0].name, "rust");
    }

    #[test]
    fn test_intent_detection() {
        let mut context = ConversationContext::new();

        context.update_from_turn(1, "What is the main function?", "", &[]);
        assert_eq!(context.intent, Some(UserIntent::Exploration));

        context.update_from_turn(2, "Add a new feature", "", &[]);
        assert_eq!(context.intent, Some(UserIntent::Implementation));

        context.update_from_turn(3, "Why does this fail?", "", &[]);
        assert_eq!(context.intent, Some(UserIntent::Analysis));
    }

    #[test]
    fn test_decision_tracking() {
        let mut context = ConversationContext::new();
        context.update_from_turn(1, "Actually, let's use a different approach", "", &[]);

        assert_eq!(context.decisions.len(), 1);
        assert!(context.decisions[0].description.contains("Changed approach"));
    }
}
