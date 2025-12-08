//! ASCII dependency graph visualization.
//!
//! This module provides ASCII art visualization of task dependency graphs
//! for terminal display.

use crate::planning::dag::DependencyGraph;
use crate::workflow::execution_state::{ExecutionState, TaskExecutionStatus};
use std::collections::HashMap;

/// Graph visualizer for ASCII dependency graphs.
pub struct GraphVisualizer;

impl GraphVisualizer {
    /// Renders a dependency graph as ASCII art.
    ///
    /// # Arguments
    /// * `dep_graph` - The dependency graph
    /// * `execution_state` - The execution state for status colors
    ///
    /// # Returns
    /// ASCII art string representation of the graph
    pub fn render(dep_graph: &DependencyGraph, execution_state: &ExecutionState) -> String {
        // Get execution levels for layout
        let levels = dep_graph.calculate_execution_levels();
        
        // Group tasks by level
        let mut tasks_by_level: HashMap<u32, Vec<String>> = HashMap::new();
        for (task_id, level) in levels {
            tasks_by_level.entry(level).or_insert_with(Vec::new).push(task_id);
        }

        // Get max level
        let max_level = levels.values().max().copied().unwrap_or(0);

        let mut output = String::new();
        output.push_str("\n");
        output.push_str("Dependency Graph:\n");
        output.push_str(&"=".repeat(80));
        output.push_str("\n\n");

        // Render each level
        for level in 0..=max_level {
            if let Some(tasks) = tasks_by_level.get(&level) {
                output.push_str(&format!("Level {}: ", level));
                
                for (idx, task_id) in tasks.iter().enumerate() {
                    let status = execution_state.get_status(task_id);
                    let icon = match status {
                        TaskExecutionStatus::Pending => "⏳",
                        TaskExecutionStatus::Running => "▶️",
                        TaskExecutionStatus::Completed => "✅",
                        TaskExecutionStatus::Failed => "❌",
                        TaskExecutionStatus::Blocked => "🚫",
                    };
                    
                    output.push_str(&format!("{} {} ", icon, task_id));
                    
                    if idx < tasks.len() - 1 {
                        output.push_str("│ ");
                    }
                }
                
                output.push_str("\n");
                
                // Draw connections to next level
                if level < max_level {
                    output.push_str("        ");
                    for _ in 0..tasks.len() {
                        output.push_str("│   ");
                    }
                    output.push_str("\n");
                    output.push_str("        ");
                    for _ in 0..tasks.len() {
                        output.push_str("▼   ");
                    }
                    output.push_str("\n");
                }
            }
        }

        output.push_str("\n");
        output.push_str(&"=".repeat(80));
        output.push_str("\n");

        output
    }
}

