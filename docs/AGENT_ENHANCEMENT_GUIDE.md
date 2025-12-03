# Agent Enhancement Guide

Complete guide for enhancing agents with model recommendations, performance profiles, and intelligent routing capabilities.

---

## Table of Contents

1. [Overview](#overview)
2. [Enhanced Metadata Schema](#enhanced-metadata-schema)
3. [Model Recommendations](#model-recommendations)
4. [Performance Profiles](#performance-profiles)
5. [Capabilities](#capabilities)
6. [Enhancement Workflow](#enhancement-workflow)
7. [Examples](#examples)
8. [Best Practices](#best-practices)

---

## Overview

Enhanced agents include metadata that enables intelligent model selection, cost optimization, and better routing. This guide shows you how to transform a basic agent into a fully-enhanced agent with optimal model recommendations.

### Benefits of Enhancement

- **Intelligent Model Selection**: Automatic routing to optimal models
- **Cost Optimization**: Choose models based on task complexity
- **Performance Tuning**: Match model capabilities to agent needs
- **Fallback Chains**: Automatic failover for high availability
- **Budget Control**: Prevent cost overruns with budget limits

---

## Enhanced Metadata Schema

### Basic Agent (Before)

```yaml
---
name: my-agent
description: A simple agent
color: blue
---
```

### Enhanced Agent (After)

```yaml
---
name: my-agent
display_name: My Agent
category: engineering
color: blue
summary: One-line description for quick reference
description: |
  Detailed multi-line description of what
  this agent does and when to use it.

recommended_models:
  primary:
    engine: gemini
    model: gemini-2.0-flash-exp
    reasoning: Why this model is optimal
    priority: speed
    cost_tier: low
  fallback:
    engine: openai
    model: gpt-4o-mini
    reasoning: Backup option reasoning
    priority: balanced
    cost_tier: low

capabilities:
  - capability_1
  - capability_2
  - capability_3

performance_profile:
  thinking_depth: medium
  iteration_speed: fast
  context_requirements: medium
  output_volume: medium
---
```

---

## Model Recommendations

### Structure

```yaml
recommended_models:
  primary:      # Main model for this agent
    engine: gemini | openai | anthropic | mock
    model: model-name
    reasoning: Why this model is chosen
    priority: speed | balanced | thinking | expert
    cost_tier: low | medium | high | premium
    requires_approval: true | false  # Optional, defaults to false

  fallback:     # Optional backup model
    # Same structure as primary

  premium:      # Optional premium model for critical tasks
    # Same structure as primary
    requires_approval: true  # Usually true for premium
```

### Priority Levels

**speed** - Fast iteration, low latency
- Use for: UI generation, rapid prototyping, high-frequency tasks
- Models: gemini-2.0-flash-exp, gpt-4o-mini

**balanced** - Good quality at reasonable cost
- Use for: General development, most tasks
- Models: gpt-4o, gemini-1.5-pro, claude-3-sonnet

**thinking** - Deep reasoning for complex problems
- Use for: Architecture decisions, complex algorithms, security analysis
- Models: o1-preview, claude-3-opus

**expert** - Maximum capability for critical decisions
- Use for: Critical security, major architecture, high-stakes decisions
- Models: o1, claude-3-opus-20240229

### Cost Tiers

**low** - $0.00 - $0.10 per 1M tokens
- Ideal for: High-frequency tasks, prototyping, iteration
- Examples: gemini-flash, gpt-4o-mini

**medium** - $0.10 - $1.00 per 1M tokens
- Ideal for: General development, balanced tasks
- Examples: gpt-4o, gemini-pro

**high** - $1.00 - $10.00 per 1M tokens
- Ideal for: Complex analysis, deep thinking
- Examples: o1-preview, claude-opus

**premium** - $10.00+ per 1M tokens
- Ideal for: Critical decisions, highest quality
- Examples: o1, specialized models

---

## Performance Profiles

Define the agent's characteristics to help with selection and routing.

```yaml
performance_profile:
  thinking_depth: low | medium | high | expert
  iteration_speed: instant | fast | medium | slow
  context_requirements: low | medium | high | expert
  output_volume: low | medium | high | extreme
```

### Thinking Depth

- **low**: Simple, straightforward tasks
- **medium**: Moderate complexity, some reasoning
- **high**: Complex problems requiring deep analysis
- **expert**: Critical decisions, maximum reasoning

### Iteration Speed

- **instant**: Real-time responses needed
- **fast**: Quick turnaround preferred
- **medium**: Standard development pace
- **slow**: Can wait for quality

### Context Requirements

- **low**: Minimal context needed (< 4K tokens)
- **medium**: Moderate context (4K - 32K tokens)
- **high**: Large context (32K - 128K tokens)
- **expert**: Maximum context (128K+ tokens)

### Output Volume

- **low**: Brief responses
- **medium**: Standard responses
- **high**: Extensive documentation/code
- **extreme**: Very large outputs (documentation, multiple files)

---

## Capabilities

List the agent's specific capabilities to enable capability-based routing.

```yaml
capabilities:
  - frontend_development
  - react_components
  - typescript
  - css_architecture
  - responsive_design
  - accessibility
```

### Capability Categories

**Development**:
- `frontend_development`, `backend_development`, `fullstack_development`
- `api_design`, `database_modeling`, `testing`, `debugging`

**Design**:
- `ui_design`, `ux_design`, `visual_design`, `interaction_design`
- `css_architecture`, `design_systems`, `responsive_design`

**Security**:
- `vulnerability_analysis`, `threat_modeling`, `code_review`
- `penetration_testing`, `compliance_auditing`

**Documentation**:
- `technical_writing`, `api_documentation`, `user_guides`
- `tutorial_creation`, `code_examples`

**Architecture**:
- `system_architecture`, `component_design`, `data_modeling`
- `performance_optimization`, `scalability_planning`

---

## Enhancement Workflow

### Step 1: Analyze Agent Purpose

1. What is the primary function of this agent?
2. What tasks will it perform most frequently?
3. What level of quality is required?
4. How important is speed vs. quality?
5. What is the expected output volume?

### Step 2: Choose Model Priority

```
Quick Tasks + High Frequency = speed priority
General Development = balanced priority
Complex Analysis = thinking priority
Critical Decisions = expert priority
```

### Step 3: Select Primary Model

Based on priority, choose from:

**Speed Priority**:
- Primary: `gemini-2.0-flash-exp` (instant, $0.05/1M)
- Fallback: `gpt-4o-mini` (fast, $0.15/1M)

**Balanced Priority**:
- Primary: `gpt-4o` (balanced, $2.50/1M)
- Fallback: `gemini-1.5-pro` (balanced, $1.25/1M)

**Thinking Priority**:
- Primary: `o1-preview` (deep, $15/1M)
- Fallback: `claude-3-opus` (strong, $15/1M)

**Expert Priority**:
- Primary: `o1` (maximum, $60/1M)
- Premium: `claude-3-opus-20240229` (expert, $15/1M)

### Step 4: Define Performance Profile

Match the profile to expected usage:

```yaml
# Fast iteration agent (UI, prototyping)
performance_profile:
  thinking_depth: low
  iteration_speed: instant
  context_requirements: low
  output_volume: high

# Balanced development agent
performance_profile:
  thinking_depth: medium
  iteration_speed: medium
  context_requirements: medium
  output_volume: medium

# Deep analysis agent (security, architecture)
performance_profile:
  thinking_depth: expert
  iteration_speed: slow
  context_requirements: high
  output_volume: medium
```

### Step 5: List Capabilities

Add specific capabilities this agent provides:

```yaml
capabilities:
  - primary_capability_1
  - primary_capability_2
  - secondary_capability_1
  - tool_or_framework_expertise
```

### Step 6: Test and Validate

1. Parse the agent metadata to ensure valid YAML
2. Test with ModelSelector to verify selection works
3. Validate cost estimates match expectations
4. Confirm fallback chain works as intended

---

## Examples

### Speed-Optimized Design Agent

```yaml
---
name: ui-specialist
display_name: UI Specialist
category: design
color: purple
summary: Fast-iteration UI designer for rapid prototyping
description: |
  Creates UI components and layouts with lightning-fast iteration.
  Optimized for high output volume and quick turnaround.

recommended_models:
  primary:
    engine: gemini
    model: gemini-2.0-flash-exp
    reasoning: Instant iteration for UI component generation
    priority: speed
    cost_tier: low
  fallback:
    engine: openai
    model: gpt-4o-mini
    reasoning: Fast backup for UI tasks
    priority: speed
    cost_tier: low

capabilities:
  - ui_component_design
  - css_generation
  - rapid_prototyping

performance_profile:
  thinking_depth: low
  iteration_speed: instant
  context_requirements: low
  output_volume: high
---
```

### Thinking-Optimized Security Agent

```yaml
---
name: security-auditor
display_name: Security Auditor
category: security
color: red
summary: Deep-thinking security specialist for comprehensive audits
description: |
  Performs thorough security assessments with deep analysis.
  Optimized for comprehensive vulnerability detection.

recommended_models:
  primary:
    engine: openai
    model: o1-preview
    reasoning: Deep reasoning for complex security analysis
    priority: thinking
    cost_tier: high
  fallback:
    engine: openai
    model: gpt-4
    reasoning: Strong analytical backup
    priority: balanced
    cost_tier: medium
  premium:
    engine: anthropic
    model: claude-3-opus
    reasoning: Expert-level for critical security
    priority: expert
    cost_tier: premium
    requires_approval: true

capabilities:
  - vulnerability_analysis
  - threat_modeling
  - security_architecture

performance_profile:
  thinking_depth: expert
  iteration_speed: slow
  context_requirements: high
  output_volume: medium
---
```

---

## Best Practices

### 1. Start with Agent's Primary Use Case

Design recommendations around the most common tasks this agent will perform.

### 2. Consider Cost vs. Quality Trade-offs

Not every task needs the most expensive model. Match cost to criticality:
- **Prototyping**: Use speed/low-cost
- **Production code**: Use balanced/medium-cost
- **Security/critical**: Use thinking/high-cost

### 3. Always Include Fallback

Ensure high availability with a fallback model:
```yaml
recommended_models:
  primary: # Your optimal choice
  fallback: # Always include this
```

### 4. Be Specific in Reasoning

Help users understand why you chose each model:
```yaml
reasoning: "Fast CSS generation with instant iteration for rapid prototyping cycles"
```

### 5. Match Priority to Task Complexity

```
Simple, frequent tasks → speed
General development → balanced
Complex analysis → thinking
Critical decisions → expert
```

### 6. Use Premium Models Sparingly

Reserve premium models for truly critical tasks:
```yaml
premium:
  # Only for critical security, major architecture, high-stakes decisions
  requires_approval: true  # Always require approval for premium
```

### 7. Test with Real Workloads

Validate your choices:
1. Create test scenarios
2. Use ModelSelector to select models
3. Measure actual costs
4. Adjust based on results

### 8. Document Your Decisions

Add clear reasoning to help others understand your choices:
```yaml
# Good
reasoning: "Fast iteration for CSS with 10ms response time requirement"

# Bad
reasoning: "It's fast"
```

### 9. Keep Performance Profile Realistic

Match profile to actual agent behavior:
- Don't mark as `instant` if agent needs to think
- Don't mark as `expert` thinking if tasks are simple
- Be honest about output volume

### 10. Update Based on Usage

Monitor actual usage and adjust:
- If costs are too high → lower priority or tier
- If quality is insufficient → higher priority or tier
- If fallback triggers often → reconsider primary choice

---

## Migration Guide

### Enhancing Existing Agents

1. **Start Simple**: Add just primary model first
2. **Test**: Verify it works with ModelSelector
3. **Add Fallback**: Ensure high availability
4. **Add Profile**: Define performance characteristics
5. **Add Capabilities**: Enable capability-based routing
6. **Consider Premium**: Only if truly needed

### Gradual Enhancement

You don't need to enhance all agents at once:

**Priority 1**: High-frequency agents (UI, development)
**Priority 2**: Critical agents (security, architecture)
**Priority 3**: Specialized agents (documentation, testing)
**Priority 4**: Occasional-use agents

### Testing Enhanced Agents

```rust
use radium_core::agents::metadata::AgentMetadata;
use radium_core::models::{ModelSelector, SelectionOptions};

// Load enhanced agent
let agent = AgentMetadata::from_file("agents/ui-specialist.md")?;

// Test selection
let mut selector = ModelSelector::new()
    .with_budget_limit(1.0);

let options = SelectionOptions::new(&agent)
    .with_token_estimate(1000, 500);

let result = selector.select_model(options)?;

println!("Selected: {}", result.selected);
println!("Model: {}", result.model.model_id());
println!("Cost: ${:.4}", result.estimated_cost.unwrap_or(0.0));
```

---

## Summary

Enhancing agents with model recommendations enables:
- ✅ Intelligent, automatic model selection
- ✅ Cost optimization based on task complexity
- ✅ High availability through fallback chains
- ✅ Budget control and cost tracking
- ✅ Better performance through optimal routing

Follow this guide to transform basic agents into intelligent, cost-optimized agents that automatically select the best model for each task.
