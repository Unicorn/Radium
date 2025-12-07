//! Parser for converting LLM-generated plan text into structured data.

use serde::{Deserialize, Serialize};

/// A parsed task from the LLM response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTask {
    /// Task number within the iteration.
    pub number: u32,
    /// Task title.
    pub title: String,
    /// Task description.
    pub description: Option<String>,
    /// Agent ID to assign to this task.
    pub agent_id: Option<String>,
    /// Task dependencies (references to other task IDs).
    pub dependencies: Vec<String>,
    /// Acceptance criteria for task completion.
    pub acceptance_criteria: Vec<String>,
}

/// A parsed iteration from the LLM response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedIteration {
    /// Iteration number.
    pub number: u32,
    /// Iteration name.
    pub name: String,
    /// Iteration description.
    pub description: Option<String>,
    /// Iteration goal.
    pub goal: Option<String>,
    /// Tasks in this iteration.
    pub tasks: Vec<ParsedTask>,
}

/// A complete parsed plan from the LLM response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedPlan {
    /// Project name.
    pub project_name: String,
    /// Project description.
    pub description: Option<String>,
    /// Tech stack detected.
    pub tech_stack: Vec<String>,
    /// Iterations in the plan.
    pub iterations: Vec<ParsedIteration>,
}

/// Parser for LLM-generated plan responses.
pub struct PlanParser;

impl PlanParser {
    /// Parses a plan from LLM response text.
    ///
    /// Expects a structured response with markdown headers and task lists.
    ///
    /// # Format Expected:
    /// ```markdown
    /// # Project Name
    ///
    /// Description...
    ///
    /// ## Iteration 1: Name
    ///
    /// Goal: ...
    ///
    /// ### Tasks
    /// 1. **Task Title** - Description
    ///    - Agent: agent-id
    ///    - Dependencies: I1.T2, I1.T3
    ///    - Acceptance: Criteria here
    /// ```
    pub fn parse(response: &str) -> Result<ParsedPlan, String> {
        let lines: Vec<&str> = response.lines().collect();

        // Extract project name (first # header)
        let project_name = Self::extract_project_name(&lines)?;

        // Extract description (text after project name, before first ##)
        let description = Self::extract_description(&lines);

        // Extract tech stack
        let tech_stack = Self::extract_tech_stack(&lines);

        // Parse iterations
        let iterations = Self::parse_iterations(&lines)?;

        Ok(ParsedPlan { project_name, description, tech_stack, iterations })
    }

    fn extract_project_name(lines: &[&str]) -> Result<String, String> {
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("# ") && !trimmed.starts_with("## ") {
                return Ok(trimmed[2..].trim().to_string());
            }
        }
        Err("No project name found (expecting # header)".to_string())
    }

    fn extract_description(lines: &[&str]) -> Option<String> {
        let mut in_description = false;
        let mut description_lines = Vec::new();

        for line in lines {
            let trimmed = line.trim();

            if trimmed.starts_with("# ") && !trimmed.starts_with("## ") {
                in_description = true;
                continue;
            }

            if trimmed.starts_with("## ") {
                break;
            }

            if in_description && !trimmed.is_empty() {
                description_lines.push(trimmed);
            }
        }

        if description_lines.is_empty() { None } else { Some(description_lines.join(" ")) }
    }

    fn extract_tech_stack(lines: &[&str]) -> Vec<String> {
        let mut tech_stack = Vec::new();
        let mut in_tech_section = false;

        for line in lines {
            let trimmed = line.trim();

            if trimmed.to_lowercase().contains("tech stack")
                || trimmed.to_lowercase().contains("technologies")
            {
                in_tech_section = true;
                continue;
            }

            if in_tech_section {
                if trimmed.starts_with("## ") && !trimmed.to_lowercase().contains("tech") {
                    break;
                }

                if trimmed.starts_with('-') || trimmed.starts_with('*') {
                    let tech =
                        trimmed.trim_start_matches('-').trim_start_matches('*').trim().to_string();
                    if !tech.is_empty() {
                        tech_stack.push(tech);
                    }
                }
            }
        }

        tech_stack
    }

    fn parse_iterations(lines: &[&str]) -> Result<Vec<ParsedIteration>, String> {
        let mut iterations = Vec::new();
        let mut current_iteration: Option<ParsedIteration> = None;
        let mut current_task: Option<ParsedTask> = None;
        let mut task_number = 0;
        let mut iteration_number = 0;

        for line in lines {
            let trimmed = line.trim();

            // New iteration (## header with "Iteration" or "I[0-9]")
            if trimmed.starts_with("## ") {
                // Save previous iteration
                if let Some(mut iter) = current_iteration.take() {
                    if let Some(task) = current_task.take() {
                        iter.tasks.push(task);
                    }
                    iterations.push(iter);
                }

                iteration_number += 1;
                task_number = 0;

                let name = trimmed.strip_prefix("## ").unwrap_or(trimmed).trim().to_string();
                current_iteration = Some(ParsedIteration {
                    number: iteration_number,
                    name,
                    description: None,
                    goal: None,
                    tasks: Vec::new(),
                });
                continue;
            }

            // Goal line
            if trimmed.to_lowercase().starts_with("goal:") {
                if let Some(ref mut iter) = current_iteration {
                    iter.goal = Some(trimmed[5..].trim().to_string());
                }
                continue;
            }

            // Task line (numbered list starting with digit)
            if let Some(first_char) = trimmed.chars().next() {
                if first_char.is_ascii_digit() && trimmed.contains('.') {
                    // Save previous task
                    if let Some(ref mut iter) = current_iteration {
                        if let Some(task) = current_task.take() {
                            iter.tasks.push(task);
                        }
                    }

                    task_number += 1;

                    // Parse task title
                    let after_number = trimmed.split_once('.').map(|(_, rest)| rest.trim());
                    if let Some(title_text) = after_number {
                        let title = Self::extract_task_title(title_text);
                        let description = Self::extract_task_description(title_text);

                        current_task = Some(ParsedTask {
                            number: task_number,
                            title,
                            description,
                            agent_id: None,
                            dependencies: Vec::new(),
                            acceptance_criteria: Vec::new(),
                        });
                    }
                    continue;
                }
            }

            // Task metadata lines (indented with -)
            if trimmed.starts_with("- ") || trimmed.starts_with("  - ") {
                if let Some(ref mut task) = current_task {
                    let content =
                        trimmed.trim_start_matches("- ").trim_start_matches("  - ").trim();

                    if content.to_lowercase().starts_with("agent:") {
                        task.agent_id = Some(content[6..].trim().to_string());
                    } else if content.to_lowercase().starts_with("depends:")
                        || content.to_lowercase().starts_with("dependencies:")
                    {
                        let deps_str = if content.to_lowercase().starts_with("depends:") {
                            &content[8..]
                        } else {
                            &content[13..]
                        };

                        task.dependencies = deps_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    } else if content.to_lowercase().starts_with("acceptance:") {
                        task.acceptance_criteria.push(content[11..].trim().to_string());
                    } else if !content.is_empty() {
                        // Other acceptance criteria
                        task.acceptance_criteria.push(content.to_string());
                    }
                }
            }
        }

        // Save final iteration and task
        if let Some(mut iter) = current_iteration {
            if let Some(task) = current_task {
                iter.tasks.push(task);
            }
            iterations.push(iter);
        }

        if iterations.is_empty() {
            return Err("No iterations found in response".to_string());
        }

        Ok(iterations)
    }

    fn extract_task_title(text: &str) -> String {
        // Extract text between ** markers or up to first -
        if text.contains("**") {
            let parts: Vec<&str> = text.split("**").collect();
            if parts.len() >= 2 {
                return parts[1].trim().to_string();
            }
        }

        // Or take everything before first -
        if let Some(dash_pos) = text.find(" - ") {
            return text[..dash_pos].trim().to_string();
        }

        text.trim().to_string()
    }

    fn extract_task_description(text: &str) -> Option<String> {
        // Extract text after first -
        if let Some(dash_pos) = text.find(" - ") {
            let desc = text[dash_pos + 3..].trim();
            if !desc.is_empty() {
                return Some(desc.to_string());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_plan() {
        let response = r"# My Project

A simple project description.

## Iteration 1: Setup

Goal: Set up the project foundation

1. **Initialize repository** - Create git repo and add README
   - Agent: setup-agent
   - Acceptance: Git repo created with README.md

2. **Configure tooling** - Set up linters and formatters
   - Agent: config-agent
   - Dependencies: I1.T1
   - Acceptance: Linters configured
";

        let plan = PlanParser::parse(response).unwrap();

        assert_eq!(plan.project_name, "My Project");
        assert_eq!(plan.description, Some("A simple project description.".to_string()));
        assert_eq!(plan.iterations.len(), 1);

        let iter = &plan.iterations[0];
        assert_eq!(iter.number, 1);
        assert_eq!(iter.name, "Iteration 1: Setup");
        assert_eq!(iter.goal, Some("Set up the project foundation".to_string()));
        assert_eq!(iter.tasks.len(), 2);

        let task1 = &iter.tasks[0];
        assert_eq!(task1.number, 1);
        assert_eq!(task1.title, "Initialize repository");
        assert_eq!(task1.agent_id, Some("setup-agent".to_string()));
        assert_eq!(task1.acceptance_criteria.len(), 1);

        let task2 = &iter.tasks[1];
        assert_eq!(task2.number, 2);
        assert_eq!(task2.title, "Configure tooling");
        assert_eq!(task2.dependencies, vec!["I1.T1"]);
    }

    #[test]
    fn test_extract_tech_stack() {
        let response = r"# Project

## Tech Stack
- Rust
- PostgreSQL
- Docker

## Iteration 1
";

        let plan = PlanParser::parse(response).unwrap();
        assert_eq!(plan.tech_stack, vec!["Rust", "PostgreSQL", "Docker"]);
    }

    #[test]
    fn test_parse_multiple_iterations() {
        let response = r"# Multi-Iteration Project

Planning phase with multiple iterations.

## Iteration 1: Foundation

Goal: Build core infrastructure

1. **Setup project** - Initialize codebase
   - Agent: init-agent

## Iteration 2: Features

Goal: Implement main features

1. **Add feature A** - First feature
   - Agent: feature-agent
   - Dependencies: I1.T1

2. **Add feature B** - Second feature
   - Agent: feature-agent
   - Dependencies: I2.T1
";

        let plan = PlanParser::parse(response).unwrap();
        assert_eq!(plan.iterations.len(), 2);
        assert_eq!(plan.iterations[0].name, "Iteration 1: Foundation");
        assert_eq!(plan.iterations[1].name, "Iteration 2: Features");
        assert_eq!(plan.iterations[0].tasks.len(), 1);
        assert_eq!(plan.iterations[1].tasks.len(), 2);
    }

    #[test]
    fn test_parse_task_without_optional_fields() {
        let response = r"# Minimal Project

## Iteration 1: Start

1. **Basic task** - Just a simple task
";

        let plan = PlanParser::parse(response).unwrap();
        let task = &plan.iterations[0].tasks[0];
        assert_eq!(task.title, "Basic task");
        assert_eq!(task.agent_id, None);
        assert!(task.dependencies.is_empty());
        assert!(task.acceptance_criteria.is_empty());
    }

    #[test]
    fn test_parse_multiple_acceptance_criteria() {
        let response = r"# Project

## Iteration 1: Test

1. **Complex task** - Task with multiple criteria
   - Acceptance: First criterion
   - Acceptance: Second criterion
   - Acceptance: Third criterion
";

        let plan = PlanParser::parse(response).unwrap();
        let task = &plan.iterations[0].tasks[0];
        assert_eq!(task.acceptance_criteria.len(), 3);
        assert_eq!(task.acceptance_criteria[0], "First criterion");
        assert_eq!(task.acceptance_criteria[1], "Second criterion");
        assert_eq!(task.acceptance_criteria[2], "Third criterion");
    }

    #[test]
    fn test_parse_complex_dependencies() {
        let response = r"# Project

## Iteration 1: Test

1. **Task with deps** - Multiple dependencies
   - Dependencies: I1.T1, I1.T2, I2.T1
";

        let plan = PlanParser::parse(response).unwrap();
        let task = &plan.iterations[0].tasks[0];
        assert_eq!(task.dependencies, vec!["I1.T1", "I1.T2", "I2.T1"]);
    }

    #[test]
    fn test_parse_error_missing_project_name() {
        let response = r"## Iteration 1: Test

1. **Task** - Description
";

        let result = PlanParser::parse(response);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("project name"));
    }

    #[test]
    fn test_parse_empty_input() {
        let response = "";
        let result = PlanParser::parse(response);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_project_name_only() {
        let response = "# Solo Project";
        let result = PlanParser::parse(response);
        // Parser requires at least one iteration
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No iterations found"));
    }

    #[test]
    fn test_parse_tech_stack_with_asterisks() {
        let response = r"# Project

## Technologies
* Python
* Flask
* MongoDB

## Iteration 1: Start
1. **Task** - Description
";

        let plan = PlanParser::parse(response).unwrap();
        assert_eq!(plan.tech_stack, vec!["Python", "Flask", "MongoDB"]);
    }

    #[test]
    fn test_parse_task_with_depends_keyword() {
        let response = r"# Project

## Iteration 1: Test
1. **Task with depends** - Description
   - Depends: I1.T1, I1.T2
";

        let plan = PlanParser::parse(response).unwrap();
        let task = &plan.iterations[0].tasks[0];
        assert_eq!(task.dependencies, vec!["I1.T1", "I1.T2"]);
    }

    #[test]
    fn test_parse_iteration_without_goal() {
        let response = r"# Project

## Iteration 1: No Goal Iteration

1. **Task** - Do something
";

        let plan = PlanParser::parse(response).unwrap();
        let iter = &plan.iterations[0];
        assert_eq!(iter.goal, None);
    }

    #[test]
    fn test_parse_task_title_without_bold() {
        let response = r"# Project

## Iteration 1: Test
1. Plain Task Title - Description
";

        let plan = PlanParser::parse(response).unwrap();
        let task = &plan.iterations[0].tasks[0];
        assert_eq!(task.title, "Plain Task Title");
        assert_eq!(task.description, Some("Description".to_string()));
    }

    #[test]
    fn test_parse_multiple_empty_lines() {
        let response = r"# Project



## Iteration 1: Test


1. **Task** - Description


";

        let plan = PlanParser::parse(response).unwrap();
        assert_eq!(plan.iterations.len(), 1);
        assert_eq!(plan.iterations[0].tasks.len(), 1);
    }

    #[test]
    fn test_parse_project_with_whitespace() {
        let response = r"#    Project with Spaces

## Iteration 1: Test
1. **Task** - Description
";

        let plan = PlanParser::parse(response).unwrap();
        assert_eq!(plan.project_name, "Project with Spaces");
    }

    #[test]
    fn test_parse_empty_dependencies() {
        let response = r"# Project

## Iteration 1: Test
1. **Task** - Description
   - Dependencies:
";

        let plan = PlanParser::parse(response).unwrap();
        let task = &plan.iterations[0].tasks[0];
        assert!(task.dependencies.is_empty());
    }

    #[test]
    fn test_parse_task_numbers_non_sequential() {
        let response = r"# Project

## Iteration 1: Test
1. **First** - One
3. **Third** - Three
5. **Fifth** - Five
";

        let plan = PlanParser::parse(response).unwrap();
        // Parser assigns sequential numbers internally
        assert_eq!(plan.iterations[0].tasks.len(), 3);
        assert_eq!(plan.iterations[0].tasks[0].number, 1);
        assert_eq!(plan.iterations[0].tasks[1].number, 2);
        assert_eq!(plan.iterations[0].tasks[2].number, 3);
    }

    #[test]
    fn test_parse_no_description() {
        let response = r"# Minimal

## Iteration 1: Test
1. **Task**
";

        let plan = PlanParser::parse(response).unwrap();
        assert_eq!(plan.description, None);
        let task = &plan.iterations[0].tasks[0];
        assert_eq!(task.description, None);
    }
}
