# Agent Selection Patterns

Practical patterns for using the ModelSelector with different agent types and scenarios.

---

## Table of Contents

1. [Overview](#overview)
2. [Basic Patterns](#basic-patterns)
3. [Budget Management Patterns](#budget-management-patterns)
4. [Advanced Patterns](#advanced-patterns)
5. [Multi-Agent Workflows](#multi-agent-workflows)
6. [Error Handling](#error-handling)
7. [Real-World Scenarios](#real-world-scenarios)

---

## Overview

The ModelSelector automatically chooses the optimal model for each agent based on their metadata. This guide shows you how to use it effectively in different scenarios.

---

## Basic Patterns

### Simple Selection

```rust
use radium_core::agents::metadata::AgentMetadata;
use radium_core::models::{ModelSelector, SelectionOptions};

// Load agent
let agent = AgentMetadata::from_file("agents/ui-specialist.md")?;

// Create selector
let mut selector = ModelSelector::new();

// Select model
let options = SelectionOptions::new(&agent);
let result = selector.select_model(options)?;

// Use the model
let response = result.model
    .generate_text("Create a button component", None)
    .await?;
```

### With Cost Estimation

```rust
// Provide token estimates for cost calculation
let options = SelectionOptions::new(&agent)
    .with_token_estimate(
        1000,  // estimated prompt tokens
        500    // estimated completion tokens
    );

let result = selector.select_model(options)?;

// Check estimated cost
if let Some(cost) = result.estimated_cost {
    println!("Estimated cost: ${:.4}", cost);
}
```

---

## Budget Management Patterns

### Per-Operation Budget Limit

```rust
// Prevent any single operation from exceeding $1
let mut selector = ModelSelector::new()
    .with_budget_limit(1.0);

let options = SelectionOptions::new(&agent)
    .with_token_estimate(1000, 500);

match selector.select_model(options) {
    Ok(result) => {
        // Within budget, proceed
        let response = result.model.generate_text(prompt, None).await?;
    }
    Err(SelectionError::BudgetExceeded(cost, limit)) => {
        println!("Operation would cost ${:.2}, limit is ${:.2}", cost, limit);
        // Handle budget exceeded
    }
    Err(e) => {
        // Handle other errors
    }
}
```

### Session-Wide Budget Tracking

```rust
// Track total costs across multiple operations
let mut selector = ModelSelector::new()
    .with_total_budget_limit(10.0);  // Max $10 for entire session

for task in tasks {
    let agent = load_agent_for_task(&task)?;
    let options = SelectionOptions::new(&agent)
        .with_token_estimate(task.estimated_tokens(), task.estimated_completion());

    match selector.select_model(options) {
        Ok(result) => {
            // Process task
            process_with_model(task, result.model).await?;
        }
        Err(SelectionError::BudgetExceeded(_, _)) => {
            println!("Budget exhausted. Total spent: ${:.2}", selector.get_total_cost());
            break;
        }
        Err(e) => return Err(e.into()),
    }
}

println!("Session total: ${:.2}", selector.get_total_cost());
```

### Budget Reset Pattern

```rust
let mut selector = ModelSelector::new()
    .with_total_budget_limit(100.0);

for project in projects {
    // Reset budget for each project
    selector.reset_cost_tracking();

    for task in project.tasks {
        // Process tasks within project budget
        let result = selector.select_model(options)?;
        // ...
    }

    println!("Project cost: ${:.2}", selector.get_total_cost());
}
```

---

## Advanced Patterns

### Priority Override

```rust
use radium_core::agents::metadata::ModelPriority;

// Override agent's default priority for critical tasks
let selector = ModelSelector::new()
    .with_priority_override(ModelPriority::Expert);

// Will select expert/premium models regardless of agent defaults
let result = selector.select_model(options)?;
```

### Premium Model Approval

```rust
// By default, premium models are skipped
let options = SelectionOptions::new(&agent);
let result = selector.select_model(options)?;  // Uses primary/fallback

// Explicitly allow premium models
let options = SelectionOptions::new(&agent)
    .allow_premium();

let selector = selector.with_priority_override(ModelPriority::Expert);
let result = selector.select_model(options)?;  // May use premium if available
```

### Fallback Chain Inspection

```rust
let result = selector.select_model(options)?;

match result.selected {
    SelectedModel::Primary => {
        println!("Using primary model: {}", result.model.model_id());
    }
    SelectedModel::Fallback => {
        println!("Primary unavailable, using fallback: {}", result.model.model_id());
    }
    SelectedModel::Mock => {
        println!("All models unavailable, using mock");
    }
    SelectedModel::Premium => {
        println!("Using premium model: {}", result.model.model_id());
    }
}
```

---

## Multi-Agent Workflows

### Sequential Agent Chain

```rust
// Multiple agents in sequence with shared budget
let mut selector = ModelSelector::new()
    .with_total_budget_limit(5.0);

// Step 1: UI Designer
let ui_agent = AgentMetadata::from_file("agents/ui-specialist.md")?;
let options = SelectionOptions::new(&ui_agent).with_token_estimate(1000, 500);
let result = selector.select_model(options)?;
let ui_design = result.model.generate_text(ui_prompt, None).await?;

// Step 2: Engineer
let eng_agent = AgentMetadata::from_file("agents/fullstack-engineer.md")?;
let options = SelectionOptions::new(&eng_agent).with_token_estimate(2000, 1000);
let result = selector.select_model(options)?;
let implementation = result.model.generate_text(impl_prompt, None).await?;

// Step 3: Reviewer
let review_agent = AgentMetadata::from_file("agents/security-auditor.md")?;
let options = SelectionOptions::new(&review_agent).with_token_estimate(3000, 500);
let result = selector.select_model(options)?;
let review = result.model.generate_text(review_prompt, None).await?;

println!("Workflow total cost: ${:.2}", selector.get_total_cost());
```

### Parallel Agent Processing

```rust
use tokio::task;

// Different agents processing in parallel
let agents = vec![
    ("ui-specialist.md", ui_task),
    ("fullstack-engineer.md", backend_task),
    ("documentation-writer.md", docs_task),
];

let mut handles = vec![];

for (agent_file, task) in agents {
    let handle = task::spawn(async move {
        let agent = AgentMetadata::from_file(agent_file)?;
        let mut selector = ModelSelector::new();
        let options = SelectionOptions::new(&agent);
        let result = selector.select_model(options)?;

        result.model.generate_text(&task, None).await
    });

    handles.push(handle);
}

// Wait for all to complete
let results = futures::future::join_all(handles).await;
```

---

## Error Handling

### Comprehensive Error Handling

```rust
use radium_core::models::SelectionError;

match selector.select_model(options) {
    Ok(result) => {
        // Success - use the model
        Ok(result)
    }
    Err(SelectionError::BudgetExceeded(cost, limit)) => {
        eprintln!("Budget exceeded: ${:.2} > ${:.2}", cost, limit);
        // Could downgrade to cheaper agent or skip task
        Err("Budget exceeded".into())
    }
    Err(SelectionError::NoAvailableModels(msg)) => {
        eprintln!("No models available: {}", msg);
        // All models failed, including mock
        Err("No models available".into())
    }
    Err(SelectionError::InvalidConfiguration(msg)) => {
        eprintln!("Invalid agent configuration: {}", msg);
        // Agent metadata is invalid
        Err("Invalid configuration".into())
    }
    Err(SelectionError::ModelCreationFailed(e)) => {
        eprintln!("Failed to create model: {}", e);
        // Model creation error
        Err(e.into())
    }
    Err(e) => {
        eprintln!("Selection error: {}", e);
        Err(e.into())
    }
}
```

### Graceful Degradation

```rust
// Try premium agent first, fallback to cheaper if budget exceeded
let result = match selector.select_model(premium_options) {
    Ok(r) => r,
    Err(SelectionError::BudgetExceeded(_, _)) => {
        // Budget exceeded, use cheaper agent
        let cheap_agent = AgentMetadata::from_file("agents/ui-specialist.md")?;
        let options = SelectionOptions::new(&cheap_agent);
        selector.select_model(options)?
    }
    Err(e) => return Err(e.into()),
};
```

---

## Real-World Scenarios

### Rapid Prototyping

```rust
// High-frequency, low-cost for rapid iteration
let ui_agent = AgentMetadata::from_file("agents/ui-specialist.md")?;
let mut selector = ModelSelector::new()
    .with_budget_limit(0.01);  // Max $0.01 per component

for component in components {
    let options = SelectionOptions::new(&ui_agent)
        .with_token_estimate(500, 300);

    let result = selector.select_model(options)?;

    // Fast, cheap model for rapid prototyping
    let component_code = result.model
        .generate_text(&component.spec, None)
        .await?;

    save_component(&component_code)?;
}

println!("Prototyped {} components for ${:.2}",
    components.len(), selector.get_total_cost());
```

### Security Audit

```rust
// High-quality, deep analysis for security
let security_agent = AgentMetadata::from_file("agents/security-auditor.md")?;
let mut selector = ModelSelector::new()
    .with_priority_override(ModelPriority::Expert)  // Use best models
    .with_budget_limit(10.0);  // Higher budget for critical security

let options = SelectionOptions::new(&security_agent)
    .with_token_estimate(5000, 2000)
    .allow_premium();  // Allow premium models for security

let result = selector.select_model(options)?;

// High-quality security analysis
let audit_report = result.model
    .generate_text(&security_context, None)
    .await?;

println!("Security audit cost: ${:.2}",
    result.estimated_cost.unwrap_or(0.0));
```

### Documentation Generation

```rust
// High-output, low-cost for documentation
let docs_agent = AgentMetadata::from_file("agents/documentation-writer.md")?;
let mut selector = ModelSelector::new()
    .with_total_budget_limit(2.0);  // $2 for all documentation

for module in codebase.modules {
    let options = SelectionOptions::new(&docs_agent)
        .with_token_estimate(2000, 3000);  // High output

    let result = selector.select_model(options)?;

    // Fast, cheap model with high output volume
    let documentation = result.model
        .generate_text(&module.code, None)
        .await?;

    save_docs(&module.name, &documentation)?;
}

println!("Generated docs for {} modules using ${:.2}",
    codebase.modules.len(), selector.get_total_cost());
```

### Production Pipeline

```rust
// Full development pipeline with budget tracking
struct Pipeline {
    selector: ModelSelector,
    agents: HashMap<String, AgentMetadata>,
}

impl Pipeline {
    fn new(budget: f64) -> Result<Self> {
        let selector = ModelSelector::new()
            .with_total_budget_limit(budget);

        let agents = HashMap::from([
            ("ui", AgentMetadata::from_file("agents/ui-specialist.md")?),
            ("backend", AgentMetadata::from_file("agents/fullstack-engineer.md")?),
            ("security", AgentMetadata::from_file("agents/security-auditor.md")?),
            ("docs", AgentMetadata::from_file("agents/documentation-writer.md")?),
        ]);

        Ok(Self { selector, agents })
    }

    async fn run(&mut self, feature: &Feature) -> Result<FeatureResult> {
        // Phase 1: Design (fast, cheap)
        let ui = self.run_agent("ui", &feature.spec, 1000, 500).await?;

        // Phase 2: Implementation (balanced)
        let code = self.run_agent("backend", &ui, 2000, 1500).await?;

        // Phase 3: Security review (expensive, thorough)
        let audit = self.run_agent("security", &code, 3000, 1000).await?;

        // Phase 4: Documentation (high output)
        let docs = self.run_agent("docs", &code, 1500, 2000).await?;

        Ok(FeatureResult {
            ui, code, audit, docs,
            cost: self.selector.get_total_cost(),
        })
    }

    async fn run_agent(
        &mut self,
        agent_key: &str,
        input: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
    ) -> Result<String> {
        let agent = self.agents.get(agent_key)
            .ok_or("Agent not found")?;

        let options = SelectionOptions::new(agent)
            .with_token_estimate(prompt_tokens, completion_tokens);

        let result = self.selector.select_model(options)?;

        Ok(result.model.generate_text(input, None).await?.content)
    }
}

// Usage
let mut pipeline = Pipeline::new(20.0)?;  // $20 budget
let result = pipeline.run(&feature).await?;
println!("Feature cost: ${:.2}", result.cost);
```

---

## Summary

Key patterns for effective model selection:

1. **Start Simple**: Use basic selection, add features as needed
2. **Budget Early**: Set budgets to prevent cost surprises
3. **Track Costs**: Monitor total spending across operations
4. **Handle Errors**: Gracefully handle budget and availability errors
5. **Match Priority**: Use right model for right task (speed vs. quality)
6. **Test First**: Validate with small runs before production
7. **Monitor Usage**: Track which models get selected and costs

The ModelSelector automatically handles:
- ✅ Model selection based on agent metadata
- ✅ Cost estimation from token counts
- ✅ Budget enforcement (hard limits)
- ✅ Automatic fallback chains
- ✅ Premium model gating

You focus on:
- Setting appropriate budgets
- Providing token estimates (optional but recommended)
- Handling errors gracefully
- Monitoring total costs
