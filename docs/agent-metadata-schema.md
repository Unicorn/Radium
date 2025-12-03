# Agent Metadata Schema Documentation

**Version**: 1.0
**Date**: 2025-12-02
**Status**: Implemented

---

## Overview

This document defines the enhanced agent metadata schema for Radium agents. The schema uses YAML frontmatter in markdown files to provide rich metadata including model recommendations, capabilities, and performance profiles.

---

## File Format

Agent files consist of two parts:
1. **YAML Frontmatter** - Metadata about the agent
2. **Markdown Content** - The agent's prompt template

```markdown
---
# YAML frontmatter here
---

# Agent Prompt Content Here
```

---

## Complete Schema

### Required Fields

```yaml
---
# Basic Identity (REQUIRED)
name: string                    # Unique agent identifier (kebab-case)
color: string                   # Display color (red, green, blue, cyan, etc.)
description: string             # Detailed agent description

# ... optional fields below ...
---
```

### Optional Fields

```yaml
---
# Basic Identity (Optional)
display_name: string            # Human-readable name (defaults to name)
category: string                # Agent category (engineering, design, etc.)

# Descriptions
summary: string                 # One-line summary (max 120 chars, auto-generated from description if not provided)

# Model Recommendations
recommended_models:
  primary:                      # Primary model for most tasks
    engine: string              # gemini | openai | anthropic | mock
    model: string               # Specific model ID
    reasoning: string           # Why this model is recommended
    priority: string            # speed | balanced | thinking | expert
    cost_tier: string           # low | medium | high | premium
    requires_approval: bool     # Optional: requires user approval

  fallback:                     # Fallback when primary unavailable
    engine: string
    model: string
    reasoning: string
    priority: string
    cost_tier: string

  premium:                      # Premium option for critical tasks
    engine: string
    model: string
    reasoning: string
    priority: string
    cost_tier: string

# Capabilities
capabilities: array<string>     # List of specific capabilities

# Performance Profile
performance_profile:
  thinking_depth: enum          # low | medium | high | expert
  iteration_speed: enum         # slow | medium | fast | instant
  context_requirements: enum    # low | medium | high | extensive
  output_volume: enum           # low | medium | high | extensive

# Quality & Integration
quality_gates: array<string>    # Required quality checks
works_well_with: array<string>  # Agent IDs that complement this agent
typical_workflows: array<string> # Common workflow patterns

# Tools & Constraints
tools: array<string>            # Specific tools this agent uses
constraints: object             # Additional constraints (flexible key-value pairs)
---
```

---

## Field Definitions

### Basic Identity

| Field | Type | Required | Description | Example |
|-------|------|----------|-------------|---------|
| `name` | string | ✅ Yes | Unique agent identifier (kebab-case) | `"architect-ux"` |
| `display_name` | string | ❌ No | Human-readable name | `"ArchitectUX"` |
| `category` | string | ❌ No | Agent category | `"design"` |
| `color` | string | ✅ Yes | Display color | `"purple"` |
| `summary` | string | ❌ No | One-line summary | `"Technical architecture specialist"` |
| `description` | string | ✅ Yes | Detailed description (can be multiline) | See examples below |

**Valid Colors**: `red`, `green`, `blue`, `cyan`, `magenta`, `yellow`, `white`, `purple`, `orange`, `indigo`

### Model Recommendations

#### Priority Levels

| Priority | Use Case | Models | Cost |
|----------|----------|--------|------|
| `speed` | Rapid prototyping, quick iterations | gemini-2.0-flash-exp, gpt-4o-mini | Low |
| `balanced` | Standard tasks, moderate complexity | gpt-4o, claude-3.5-sonnet | Medium |
| `thinking` | Complex reasoning, architectural decisions | o1-preview, o1-mini | Medium-High |
| `expert` | Critical decisions, security audits | o1-preview (full), claude-opus | High |

#### Cost Tiers

| Tier | Range | Description |
|------|-------|-------------|
| `low` | $0.00 - $0.10 per 1M tokens | Speed-optimized models |
| `medium` | $0.10 - $1.00 per 1M tokens | Balanced models |
| `high` | $1.00 - $10.00 per 1M tokens | Thinking-intensive models |
| `premium` | $10.00+ per 1M tokens | Expert-level models |

### Performance Profile

#### Thinking Depth

- `low` - Minimal thinking required (template generation, simple tasks)
- `medium` - Moderate thinking (standard development tasks)
- `high` - Deep thinking (architecture, complex algorithms)
- `expert` - Expert-level reasoning (security, critical systems)

#### Iteration Speed

- `slow` - Complex processing requiring time
- `medium` - Standard iteration speed
- `fast` - Quick iteration cycles
- `instant` - Near-instant responses

#### Context Requirements

- `low` - Minimal context needed
- `medium` - Moderate context (current file, basic project info)
- `high` - High context (multiple files, project history)
- `extensive` - Extensive context (full codebase, comprehensive history)

#### Output Volume

- `low` - Minimal output (short responses, targeted changes)
- `medium` - Moderate output (standard function/component)
- `high` - High output (multiple files, comprehensive)
- `extensive` - Extensive output (complete systems, detailed documentation)

---

## Example Schemas

### Minimal Example

```yaml
---
name: simple-agent
color: blue
description: A simple agent with minimal configuration
---

# Simple Agent Prompt
You are a simple agent...
```

### Standard Example

```yaml
---
name: frontend-developer
display_name: Frontend Developer
category: engineering
color: cyan
summary: Expert frontend developer specializing in React
description: |
  Expert frontend developer who specializes in modern web technologies,
  UI frameworks, and performance optimization.

recommended_models:
  primary:
    engine: gemini
    model: gemini-2.0-flash-exp
    reasoning: Fast iteration for component generation
    priority: speed
    cost_tier: low

  fallback:
    engine: openai
    model: gpt-4o-mini
    reasoning: Balanced fallback option
    priority: balanced
    cost_tier: low

capabilities:
  - react_development
  - css_styling
  - performance_optimization

performance_profile:
  thinking_depth: medium
  iteration_speed: fast
  context_requirements: medium
  output_volume: high
---

# Frontend Developer Prompt
You are an expert frontend developer...
```

### Complete Example

```yaml
---
name: security-audit-lead
display_name: Security Audit Lead
category: security
color: red
summary: Expert security audit orchestrator
description: |
  Expert security audit orchestrator who coordinates comprehensive security
  assessments across all domains. Manages vulnerability triage, ensures
  complete audit coverage, and delivers executive-ready security reports.

recommended_models:
  primary:
    engine: openai
    model: o1-preview
    reasoning: Deep security reasoning and threat modeling
    priority: expert
    cost_tier: high

  fallback:
    engine: anthropic
    model: claude-3.5-sonnet
    reasoning: Comprehensive analysis capabilities
    priority: thinking
    cost_tier: medium

  premium:
    engine: openai
    model: o1-preview
    reasoning: Maximum reasoning for critical security decisions
    priority: expert
    cost_tier: premium
    requires_approval: true

capabilities:
  - threat_modeling
  - vulnerability_assessment
  - security_reporting
  - compliance_audit

performance_profile:
  thinking_depth: expert
  iteration_speed: slow
  context_requirements: extensive
  output_volume: extensive

quality_gates:
  - security_validation
  - compliance_check
  - executive_review

works_well_with:
  - security-pentest-specialist
  - security-auth-specialist
  - security-privacy-auditor

typical_workflows:
  - comprehensive_security_audit
  - compliance_assessment
  - vulnerability_remediation
---

# Security Audit Lead Prompt
You are the Security Audit Lead...
```

---

## Rust API Usage

### Parsing Agent Metadata

```rust
use radium_core::agents::metadata::AgentMetadata;

// Parse from markdown string
let content = std::fs::read_to_string("agent.md")?;
let (metadata, prompt) = AgentMetadata::from_markdown(&content)?;

// Parse from file
let (metadata, prompt) = AgentMetadata::from_file("path/to/agent.md")?;

// Access metadata
println!("Agent: {}", metadata.get_display_name());
println!("Summary: {}", metadata.get_summary());

// Check model recommendations
if let Some(models) = &metadata.recommended_models {
    println!("Primary model: {} ({})",
        models.primary.model,
        models.primary.priority
    );
}
```

### Accessing Fields

```rust
// Required fields
let name = &metadata.name;
let color = &metadata.color;
let description = &metadata.description;

// Optional fields with defaults
let display_name = metadata.get_display_name(); // Falls back to name
let summary = metadata.get_summary(); // Falls back to truncated description

// Optional fields
if let Some(category) = &metadata.category {
    println!("Category: {}", category);
}

if let Some(capabilities) = &metadata.capabilities {
    for cap in capabilities {
        println!("- {}", cap);
    }
}

if let Some(profile) = &metadata.performance_profile {
    println!("Thinking depth: {}", profile.thinking_depth);
    println!("Iteration speed: {}", profile.iteration_speed);
}
```

---

## Validation Rules

### Automatic Validation

The parser automatically validates:

1. **Required Fields**: `name`, `color`, `description` must be present and non-empty
2. **Model Recommendations**: If present, must have valid `engine`, `model`, and `reasoning`
3. **YAML Syntax**: Must be valid YAML format
4. **Frontmatter Delimiters**: Must start and end with `---`

### Validation Errors

```rust
// Example validation errors:

// Missing required field
MetadataError::MissingField("name")

// Invalid value
MetadataError::InvalidValue {
    field: "primary.engine",
    reason: "cannot be empty"
}

// Invalid YAML
MetadataError::Yaml(...)

// Invalid frontmatter
MetadataError::InvalidFrontmatter("no closing '---' delimiter found")
```

---

## Migration Guide

### From Basic to Enhanced

**Before** (basic agent):
```yaml
---
name: test-agent
color: blue
description: Test agent
---
```

**After** (enhanced):
```yaml
---
name: test-agent
color: blue
description: Test agent for demonstrations

recommended_models:
  primary:
    engine: gemini
    model: gemini-2.0-flash-exp
    reasoning: Fast iteration for testing
    priority: speed
    cost_tier: low

capabilities:
  - testing
  - validation
---
```

### Backward Compatibility

The parser supports both formats:
- ✅ Basic format (name, color, description) works as before
- ✅ Enhanced format (with model recommendations) provides additional features
- ✅ Gradual migration - add fields incrementally

---

## Best Practices

### 1. Choose Appropriate Model Priority

```yaml
# Speed priority - for rapid iteration
recommended_models:
  primary:
    priority: speed      # ← Choose based on agent's primary use case

# Thinking priority - for complex reasoning
recommended_models:
  primary:
    priority: thinking   # ← Security, architecture, critical decisions
```

### 2. Provide Clear Reasoning

```yaml
recommended_models:
  primary:
    reasoning: "Fast iteration for CSS generation"  # ✅ Specific
    # NOT: "Good model"  # ❌ Too vague
```

### 3. Include Fallback Options

```yaml
recommended_models:
  primary:
    engine: gemini
    model: gemini-2.0-flash-exp
  fallback:              # ← Always include a fallback
    engine: openai
    model: gpt-4o-mini
```

### 4. Use Descriptive Capabilities

```yaml
capabilities:
  - css_architecture     # ✅ Specific
  - responsive_design
  - accessibility
  # NOT: ["design"]       # ❌ Too generic
```

### 5. Accurate Performance Profiles

```yaml
performance_profile:
  thinking_depth: high      # Match agent's actual needs
  iteration_speed: fast     # Realistic expectation
  context_requirements: medium
  output_volume: high
```

---

## Schema Version History

### Version 1.0 (2025-12-02)
- Initial schema definition
- Model recommendations with priority and cost tiers
- Performance profiles
- Capabilities and quality gates
- Full YAML frontmatter support

---

## Related Documentation

- [Agent Library Enhancement Plan](../roadmap/AGENT_LIBRARY_ENHANCEMENT_PLAN.md)
- [Agent Enhancement Summary](../AGENT_ENHANCEMENT_SUMMARY.md)
- [Radium Architecture](../roadmap/03-implementation-plan.md)

---

**Questions or Issues?**
See the main documentation or open an issue in the Radium repository.
