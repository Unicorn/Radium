//! Planner module for task decomposition and agent assignment
//!
//! This module provides the planner workflow that analyzes user requests and
//! decomposes them into independent subtasks suitable for parallel execution
//! by specialized agents.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::agent_tools::AgentToolRegistry;
use super::context::OrchestrationContext;
use super::OrchestrationProvider;
use crate::error::{OrchestrationError, Result};

/// A subtask that can be executed by a specialized agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtask {
    /// Agent ID to assign this subtask to
    pub agent_id: String,
    /// Task description for the agent
    pub task_description: String,
    /// Additional context for the task (optional)
    pub context: Option<serde_json::Value>,
    /// Expected output type (e.g., "code_review", "implementation", "analysis")
    pub expected_output_type: String,
}

/// A plan for decomposing a request into subtasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecompositionPlan {
    /// List of subtasks to execute
    pub subtasks: Vec<Subtask>,
    /// Whether delegation should occur (false means execute directly without delegation)
    pub should_delegate: bool,
    /// Reasoning for the decomposition decision
    pub reasoning: String,
}

/// Planner for decomposing tasks into subtasks
pub struct Planner {
    /// Agent registry for discovering available agents
    agent_registry: Arc<AgentToolRegistry>,
    /// Orchestration provider for LLM-based decomposition
    provider: Arc<dyn OrchestrationProvider>,
}

impl Planner {
    /// Create a new planner
    pub fn new(
        agent_registry: Arc<AgentToolRegistry>,
        provider: Arc<dyn OrchestrationProvider>,
    ) -> Self {
        Self { agent_registry, provider }
    }

    /// Analyze a user request and determine if it should be decomposed into subtasks
    ///
    /// Returns a decomposition plan with subtasks if delegation is beneficial,
    /// or a single-task plan if the request is simple enough to execute directly.
    pub async fn decompose(
        &self,
        user_request: &str,
        context: &OrchestrationContext,
    ) -> Result<DecompositionPlan> {
        // First, check if we should delegate at all
        let should_delegate = self.should_delegate(user_request).await?;

        if !should_delegate {
            // Simple request - return single task plan (no delegation)
            return Ok(DecompositionPlan {
                subtasks: vec![],
                should_delegate: false,
                reasoning: "Request is simple enough to execute directly without delegation".to_string(),
            });
        }

        // Complex request - decompose into subtasks
        let subtasks = self.generate_subtasks(user_request, context).await?;
        let subtask_count = subtasks.len();

        Ok(DecompositionPlan {
            subtasks,
            should_delegate: true,
            reasoning: format!("Decomposed into {} independent subtasks for parallel execution", subtask_count),
        })
    }

    /// Determine if a request should be delegated to multiple agents
    ///
    /// Simple requests (single file, single concern) should not be delegated.
    /// Complex requests (multiple components, multiple concerns) should be delegated.
    async fn should_delegate(&self, user_request: &str) -> Result<bool> {
        // Heuristic-based decision: check request complexity
        let request_lower = user_request.to_lowercase();
        
        // Simple indicators that suggest no delegation needed:
        // - Single file operations
        // - Simple questions
        // - Single concern requests
        let simple_indicators = [
            "review this file",
            "explain this",
            "what does this do",
            "fix this bug",
            "add a comment",
            "rename this",
        ];

        for indicator in &simple_indicators {
            if request_lower.contains(indicator) && request_lower.len() < 200 {
                return Ok(false);
            }
        }

        // Complex indicators that suggest delegation:
        // - Multiple components mentioned
        // - Multiple files mentioned
        // - Multiple concerns
        // - Words like "and", "also", "multiple", "several"
        let complex_indicators = [
            "multiple",
            "several",
            "both",
            "all",
            "and also",
            "as well as",
            "implement",
            "create",
            "build",
            "design",
        ];

        let mut complexity_score = 0;
        for indicator in &complex_indicators {
            if request_lower.contains(indicator) {
                complexity_score += 1;
            }
        }

        // Count mentions of files/components
        let file_mentions = request_lower.matches(".rs").count()
            + request_lower.matches(".ts").count()
            + request_lower.matches(".js").count()
            + request_lower.matches(".py").count()
            + request_lower.matches("file").count();

        // Delegate if complexity score is high or multiple files mentioned
        Ok(complexity_score >= 2 || file_mentions >= 2 || request_lower.len() > 300)
    }

    /// Generate subtasks from a complex request using LLM-based decomposition
    async fn generate_subtasks(
        &self,
        user_request: &str,
        context: &OrchestrationContext,
    ) -> Result<Vec<Subtask>> {
        // Get available agents
        let available_agents: Vec<_> = self.agent_registry.get_tools()
            .iter()
            .filter_map(|tool| {
                // Extract agent ID from tool name (format: "agent_{id}")
                if tool.name.starts_with("agent_") {
                    let agent_id = tool.name.strip_prefix("agent_")?.to_string();
                    let agent_meta = self.agent_registry.get_agent(&agent_id)?;
                    Some((agent_id, agent_meta.description.clone(), tool.description.clone()))
                } else {
                    None
                }
            })
            .collect();

        if available_agents.is_empty() {
            // No agents available - return empty plan (will execute directly)
            return Ok(vec![]);
        }

        // Build prompt for LLM-based decomposition
        let agent_list: String = available_agents
            .iter()
            .map(|(id, desc, _)| format!("- {}: {}", id, desc))
            .collect::<Vec<_>>()
            .join("\n");

        let decomposition_prompt = format!(
            r#"Analyze the following user request and decompose it into independent subtasks that can be executed in parallel by specialized agents.

User Request: {}

Available Agents:
{}

Instructions:
1. Break down the request into 2-5 independent subtasks
2. Each subtask should be self-contained and can be executed in parallel
3. Assign each subtask to the most appropriate agent from the available list
4. Provide clear task descriptions
5. Specify the expected output type for each subtask (e.g., "code_review", "implementation", "analysis", "documentation")

Respond in JSON format:
{{
  "subtasks": [
    {{
      "agent_id": "agent_id",
      "task_description": "clear description of what the agent should do",
      "expected_output_type": "type of output expected"
    }}
  ]
}}

Only include subtasks that are truly independent and can run in parallel. If the request is too simple, return an empty subtasks array."#,
            user_request, agent_list
        );

        // Use the orchestration provider to get LLM response
        let empty_tools = vec![];
        let result = self.provider.execute_with_tools(
            &decomposition_prompt,
            &empty_tools,
            context,
        ).await?;

        // Parse the response as JSON
        let parsed: DecompositionResponse = serde_json::from_str(&result.response)
            .map_err(|e| OrchestrationError::Other(format!(
                "Failed to parse decomposition response: {}. Response was: {}",
                e, result.response
            )))?;

        // Convert to Subtask structs
        let subtasks: Vec<Subtask> = parsed.subtasks
            .into_iter()
            .map(|st| Subtask {
                agent_id: st.agent_id,
                task_description: st.task_description,
                context: None, // Can be enhanced later
                expected_output_type: st.expected_output_type,
            })
            .collect();

        Ok(subtasks)
    }
}

/// Internal structure for parsing LLM decomposition response
#[derive(Debug, Deserialize)]
struct DecompositionResponse {
    subtasks: Vec<SubtaskResponse>,
}

#[derive(Debug, Deserialize)]
struct SubtaskResponse {
    agent_id: String,
    task_description: String,
    expected_output_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::agent_tools::AgentMetadata;
    use std::collections::HashMap;

    // Mock provider for testing
    struct MockProvider;
    impl OrchestrationProvider for MockProvider {
        async fn execute_with_tools(
            &self,
            _input: &str,
            _tools: &[super::super::tool::Tool],
            _context: &OrchestrationContext,
        ) -> Result<super::super::OrchestrationResult> {
            // Return mock decomposition
            let response = r#"{
                "subtasks": [
                    {
                        "agent_id": "code_agent",
                        "task_description": "Review the code for bugs",
                        "expected_output_type": "code_review"
                    },
                    {
                        "agent_id": "test_agent",
                        "task_description": "Write tests for the code",
                        "expected_output_type": "test_suite"
                    }
                ]
            }"#;
            Ok(super::super::OrchestrationResult::new(
                response.to_string(),
                vec![],
                super::super::FinishReason::Stop,
            ))
        }

        fn supports_function_calling(&self) -> bool {
            false
        }

        fn provider_name(&self) -> &'static str {
            "mock"
        }
    }

    fn create_test_registry() -> AgentToolRegistry {
        let registry = AgentToolRegistry::new();
        // Note: In real tests, we'd need to set up agents properly
        // For now, this is a placeholder
        registry
    }

    #[tokio::test]
    async fn test_should_delegate_simple() {
        let registry = Arc::new(create_test_registry());
        let provider = Arc::new(MockProvider);
        let planner = Planner::new(registry, provider);

        let simple_request = "Review this file for bugs";
        let should_delegate = planner.should_delegate(simple_request).await.unwrap();
        assert!(!should_delegate, "Simple requests should not be delegated");
    }

    #[tokio::test]
    async fn test_should_delegate_complex() {
        let registry = Arc::new(create_test_registry());
        let provider = Arc::new(MockProvider);
        let planner = Planner::new(registry, provider);

        let complex_request = "Implement a REST API with authentication, user management, and file upload. Also create tests and documentation for all components.";
        let should_delegate = planner.should_delegate(complex_request).await.unwrap();
        assert!(should_delegate, "Complex requests should be delegated");
    }
}
