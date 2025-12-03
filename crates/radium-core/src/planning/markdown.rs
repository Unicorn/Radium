//! Markdown file generation for plan documentation.

use super::parser::{ParsedIteration, ParsedPlan};
use std::fs;
use std::path::Path;

/// Generates all plan markdown files in the plan directory.
///
/// # Arguments
/// * `plan_dir` - Root directory of the plan
/// * `parsed_plan` - Parsed plan structure
///
/// # Errors
/// Returns an error if file creation fails
pub fn generate_plan_files(plan_dir: &Path, parsed_plan: &ParsedPlan) -> std::io::Result<()> {
    let plan_subdir = plan_dir.join("plan");
    fs::create_dir_all(&plan_subdir)?;

    // Generate 01_Plan_Overview_and_Setup.md
    let overview = generate_overview(parsed_plan);
    fs::write(plan_subdir.join("01_Plan_Overview_and_Setup.md"), overview)?;

    // Generate 02_Iteration_I*.md for each iteration
    for (idx, iteration) in parsed_plan.iterations.iter().enumerate() {
        let iteration_doc = generate_iteration_doc(iteration, idx + 1);
        let filename = format!("02_Iteration_I{}.md", idx + 1);
        fs::write(plan_subdir.join(filename), iteration_doc)?;
    }

    // Generate 03_Verification_and_Glossary.md
    let verification = generate_verification(parsed_plan);
    fs::write(plan_subdir.join("03_Verification_and_Glossary.md"), verification)?;

    // Generate coordinator-prompt.md
    let coordinator = generate_coordinator_prompt(parsed_plan);
    fs::write(plan_subdir.join("coordinator-prompt.md"), coordinator)?;

    Ok(())
}

fn generate_overview(plan: &ParsedPlan) -> String {
    let tech_stack_list =
        plan.tech_stack.iter().map(|tech| format!("- {}", tech)).collect::<Vec<_>>().join("\n");

    let iterations_summary = plan
        .iterations
        .iter()
        .enumerate()
        .map(|(idx, iter)| {
            format!(
                "{}. **{}** ({} tasks)\n   - Goal: {}",
                idx + 1,
                iter.name,
                iter.tasks.len(),
                iter.goal.as_deref().unwrap_or("No goal specified")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r"# Plan Overview and Setup

## Project: {project_name}

{description}

## Tech Stack

{tech_stack}

## Iterations Overview

{iterations}

## Getting Started

This plan is structured into iterations, each with specific goals and tasks. Execute iterations in order using:

```bash
rad craft <plan-id>
```

To execute a specific iteration:

```bash
rad craft <plan-id> --iteration I1
```

## Progress Tracking

Track progress using:

```bash
rad status
```

Progress is automatically saved after each task completion. You can resume execution at any time using the same `rad craft` command.

## Dependencies

Tasks may have dependencies on other tasks. The system will ensure tasks are executed in the correct order based on these dependencies.

## Success Criteria

Each task includes acceptance criteria. A task is considered complete when all its acceptance criteria are met.
",
        project_name = plan.project_name,
        description = plan.description.as_deref().unwrap_or("No description provided."),
        tech_stack = if tech_stack_list.is_empty() {
            "No tech stack specified.".to_string()
        } else {
            tech_stack_list
        },
        iterations = iterations_summary
    )
}

fn generate_iteration_doc(iteration: &ParsedIteration, iter_num: usize) -> String {
    let tasks_list = iteration
        .tasks
        .iter()
        .map(|task| {
            let deps = if task.dependencies.is_empty() {
                "None".to_string()
            } else {
                task.dependencies.join(", ")
            };

            let acceptance = task
                .acceptance_criteria
                .iter()
                .map(|criterion| format!("  - {}", criterion))
                .collect::<Vec<_>>()
                .join("\n");

            format!(
                r"### {num}. {title}

{description}

- **Agent**: {agent}
- **Dependencies**: {deps}
- **Status**: ⏳ Not Started

**Acceptance Criteria**:
{acceptance}
",
                num = task.number,
                title = task.title,
                description = task.description.as_deref().unwrap_or("No description provided."),
                agent = task.agent_id.as_deref().unwrap_or("auto"),
                deps = deps,
                acceptance = if acceptance.is_empty() {
                    "  - None specified".to_string()
                } else {
                    acceptance
                }
            )
        })
        .collect::<Vec<_>>()
        .join("\n---\n\n");

    format!(
        r"# Iteration {iter_num}: {name}

## Goal

{goal}

## Description

{description}

## Tasks

{tasks}

## Summary

This iteration contains **{task_count} tasks**. Complete all tasks to move to the next iteration.

## Execution

Execute this iteration with:

```bash
rad craft <plan-id> --iteration I{iter_num}
```
",
        iter_num = iter_num,
        name = iteration.name,
        goal = iteration.goal.as_deref().unwrap_or("No goal specified"),
        description = iteration.description.as_deref().unwrap_or("No description provided."),
        tasks = tasks_list,
        task_count = iteration.tasks.len()
    )
}

fn generate_verification(plan: &ParsedPlan) -> String {
    let total_tasks: usize = plan.iterations.iter().map(|i| i.tasks.len()).sum();

    format!(
        r"# Verification and Glossary

## Project Completion Checklist

- [ ] All {} iterations completed
- [ ] All {} tasks completed
- [ ] All acceptance criteria met
- [ ] Tests passing
- [ ] Documentation updated
- [ ] Code reviewed
- [ ] Deployment successful

## Glossary

### Terms

- **Iteration**: A phase of development with specific goals
- **Task**: A concrete unit of work within an iteration
- **Agent**: An AI assistant specialized for specific tasks
- **Dependency**: A task that must be completed before another can start
- **Acceptance Criteria**: Conditions that must be met for task completion

### Agent Types

Common agent types used in this plan:

- **setup-agent**: Initializes project structure and configuration
- **code-agent**: Implements features and functionality
- **test-agent**: Writes and executes tests
- **doc-agent**: Creates and updates documentation
- **review-agent**: Reviews code and provides feedback

### Commands Reference

- `rad plan <spec>` - Generate a plan from specification
- `rad craft <plan-id>` - Execute a plan
- `rad craft <plan-id> --iteration I1` - Execute specific iteration
- `rad craft <plan-id> --task I1.T1` - Execute specific task
- `rad craft <plan-id> --resume` - Resume from last checkpoint
- `rad status` - Show execution status

## Troubleshooting

### Task Fails

If a task fails:
1. Review the error message
2. Fix the underlying issue
3. Re-run with `rad craft <plan-id> --resume`

### Dependencies Not Met

If dependency errors occur:
1. Check which tasks are blocking
2. Complete blocking tasks first
3. Resume execution

### Agent Not Found

If an agent is not found:
1. Check agent name spelling
2. Verify agent is installed
3. Use `rad agents list` to see available agents

## Next Steps

After completing all iterations:
1. Review the completed work
2. Run integration tests
3. Update project documentation
4. Prepare for deployment
",
        plan.iterations.len(),
        total_tasks
    )
}

fn generate_coordinator_prompt(plan: &ParsedPlan) -> String {
    format!(
        r"# Coordinator Prompt for: {project_name}

## Role

You are the coordinator agent responsible for orchestrating the execution of this plan. Your job is to:

1. **Sequence Tasks**: Ensure tasks are executed in the correct order based on dependencies
2. **Monitor Progress**: Track completion status of all tasks
3. **Handle Failures**: Detect failures and recommend recovery actions
4. **Provide Context**: Give agents the context they need to complete tasks
5. **Verify Completion**: Check that acceptance criteria are met

## Project Context

**Project**: {project_name}
**Description**: {description}
**Iterations**: {iteration_count}
**Total Tasks**: {task_count}

## Execution Guidelines

### Before Starting
- Review all iterations and tasks
- Understand dependencies between tasks
- Verify all required agents are available

### During Execution
- Execute tasks in dependency order
- Provide task context to executing agents
- Monitor for errors and failures
- Save progress after each task

### After Each Task
- Verify acceptance criteria
- Update task status
- Check if next task can start
- Handle any blockers

### After Each Iteration
- Verify all tasks complete
- Review iteration goals
- Prepare for next iteration

## Communication Protocol

When coordinating with agents:
- Provide clear task description
- Include relevant context and dependencies
- Specify acceptance criteria
- Share progress and status updates

## Error Handling

If errors occur:
1. Capture error details
2. Determine if task should retry
3. Check if dependencies need re-execution
4. Report status to user

## Success Criteria

The plan is complete when:
- All tasks marked as completed
- All acceptance criteria met
- All iterations finished
- Final verification passed
",
        project_name = plan.project_name,
        description = plan.description.as_deref().unwrap_or("No description provided"),
        iteration_count = plan.iterations.len(),
        task_count = plan.iterations.iter().map(|i| i.tasks.len()).sum::<usize>()
    )
}
