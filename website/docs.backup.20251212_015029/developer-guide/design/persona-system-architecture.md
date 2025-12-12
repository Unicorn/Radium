# Persona System Architecture

This document describes the architecture for the future persona system that will provide intelligent model recommendations, cost optimization, and agent selection capabilities for Radium.

## Table of Contents

1. [Overview](#overview)
2. [Enhanced Agent Metadata Schema](#enhanced-agent-metadata-schema)
3. [Model Selection Engine](#model-selection-engine)
4. [Cost Estimation and Budget Tracking](#cost-estimation-and-budget-tracking)
5. [Agent Recommendation Engine](#agent-recommendation-engine)
6. [Backward Compatibility Strategy](#backward-compatibility-strategy)
7. [Integration Points](#integration-points)
8. [Implementation Roadmap](#implementation-roadmap)

## Overview

### Goals

The persona system extends the current agent configuration system to provide:

- **Intelligent Model Selection**: Automatically choose the best model based on task requirements and performance profiles
- **Cost Optimization**: Track and enforce cost budgets while maintaining quality
- **Fallback Chains**: Gracefully handle model unavailability with automatic fallbacks
- **Agent Recommendations**: Suggest the best agent for a given task based on capabilities

### Design Principles

1. **Backward Compatible**: Existing TOML-based agent configs continue to work
2. **Progressive Enhancement**: Persona features are optional and additive
3. **Performance First**: Model selection prioritizes speed when appropriate
4. **Cost Aware**: Budget tracking and enforcement prevent cost overruns
5. **Extensible**: Architecture supports future enhancements

## Enhanced Agent Metadata Schema

### YAML Frontmatter Format

Agents can optionally include enhanced metadata in YAML frontmatter at the top of their prompt files:

```yaml
---
agent_id: arch-agent
name: Architecture Agent
recommended_models:
  primary: gemini-2.0-flash-thinking
  fallback: gemini-2.0-flash-exp
  premium: gemini-1.5-pro
capabilities: [architecture, design, planning, system-design]
performance_profile: thinking
cost_budget:
  max_per_execution: 0.10
  max_daily: 5.00
  max_monthly: 100.00
---
```

### Schema Definition

#### `agent_id` (string, required)

Unique identifier matching the agent's TOML config ID. Used for validation and linking.

#### `name` (string, required)

Human-readable name matching the agent's TOML config name.

#### `recommended_models` (object, optional)

Model recommendations with fallback chain:

- **`primary`** (string, required): Primary model to use for this agent
- **`fallback`** (string, optional): Fallback model if primary is unavailable
- **`premium`** (string, optional): Premium model for high-priority tasks

**Model Selection Priority:**
1. Primary model (default)
2. Fallback model (if primary unavailable)
3. Premium model (if explicitly requested or primary/fallback unavailable)
4. Mock model (for testing/development)

#### `capabilities` (array of strings, optional)

List of agent capabilities for recommendation matching:

- Examples: `["architecture", "design", "planning", "code-generation", "testing", "documentation"]`
- Used by recommendation engine to match agents to tasks
- Case-insensitive matching
- Partial matching supported (e.g., "code" matches "code-generation")

#### `performance_profile` (string, optional)

Performance profile that guides model selection:

- **`speed`**: Prioritize fast responses (use fastest available model)
- **`balanced`**: Balance speed and quality (default)
- **`thinking`**: Prioritize quality and reasoning (use thinking models)
- **`expert`**: Maximum quality regardless of cost (use premium models)

**Default**: `balanced` if not specified

#### `cost_budget` (object, optional)

Cost budget constraints:

- **`max_per_execution`** (float, optional): Maximum cost per agent execution (in USD)
- **`max_daily`** (float, optional): Maximum daily cost for this agent (in USD)
- **`max_monthly`** (float, optional): Maximum monthly cost for this agent (in USD)

**Budget Enforcement:**
- Budgets are checked before model selection
- If budget exceeded, system falls back to cheaper models or mock
- Budgets reset at midnight (daily) and first of month (monthly)

### Example Configurations

#### Speed-Optimized Agent

```yaml
---
agent_id: quick-responder
name: Quick Response Agent
recommended_models:
  primary: gemini-2.0-flash-exp
  fallback: gemini-1.5-flash
performance_profile: speed
cost_budget:
  max_per_execution: 0.01
---
```

#### Quality-Optimized Agent

```yaml
---
agent_id: expert-architect
name: Expert Architecture Agent
recommended_models:
  primary: gemini-2.0-flash-thinking
  fallback: gemini-1.5-pro
  premium: gemini-1.5-pro
performance_profile: expert
capabilities: [architecture, system-design, technical-leadership]
cost_budget:
  max_per_execution: 0.50
  max_daily: 20.00
---
```

## Model Selection Engine

### Selection Algorithm

The model selection engine chooses the best model based on:

1. **Performance Profile**: Determines priority (speed vs quality)
2. **Model Availability**: Checks if model is available and accessible
3. **Cost Budget**: Ensures selection doesn't exceed budget constraints
4. **Fallback Chain**: Uses fallback models if primary is unavailable

### Selection Process

```
1. Load agent configuration (TOML + YAML frontmatter if present)
2. Determine performance profile (from persona or default: balanced)
3. Check cost budgets (per-execution, daily, monthly)
4. Select model based on profile:
   - speed: Use fastest available model from recommended_models
   - balanced: Use primary model, fallback to fallback if needed
   - thinking: Use thinking model (primary or premium)
   - expert: Use premium model, fallback to primary if unavailable
5. Verify model availability (check API connectivity)
6. If unavailable, try fallback chain:
   - primary → fallback → premium → mock
7. If budget exceeded, downgrade to cheaper model or use mock
8. Return selected model with metadata
```

### Model Availability Checking

**Caching Strategy:**
- Cache model availability status for 5 minutes
- Check availability on-demand if cache expired
- Use health check endpoints when available
- Fallback to trial execution for unknown models

**Availability States:**
- **Available**: Model is accessible and ready
- **Unavailable**: Model is down or rate-limited
- **Unknown**: Status not yet determined (will check on first use)

### Fallback Chain Execution

Fallback chains execute in this order:

1. **Primary Model**: Default choice
2. **Fallback Model**: Used if primary unavailable
3. **Premium Model**: Used if explicitly requested or both primary/fallback unavailable
4. **Mock Model**: Used for testing or if all models unavailable

**Fallback Triggers:**
- Model API returns error (4xx, 5xx)
- Model rate-limited (429)
- Model timeout (> 30 seconds)
- Budget constraint violation

## Cost Estimation and Budget Tracking

### Cost Calculation

Costs are calculated based on:

- **Input Tokens**: Number of tokens in prompt
- **Output Tokens**: Number of tokens in response
- **Model Pricing**: Per-token pricing from model provider
- **Execution Time**: Optional time-based costs for premium models

**Cost Formula:**
```
cost = (input_tokens * input_price_per_token) + (output_tokens * output_price_per_token)
```

### Budget Tracking

**Storage:**
- Budgets stored in `~/.radium/budgets/` directory
- One file per agent: `{agent_id}.json`
- Tracks daily and monthly spending

**Budget File Format:**
```json
{
  "agent_id": "arch-agent",
  "daily": {
    "date": "2025-12-07",
    "spent": 2.45,
    "limit": 5.00
  },
  "monthly": {
    "month": "2025-12",
    "spent": 45.20,
    "limit": 100.00
  },
  "last_reset_daily": "2025-12-07T00:00:00Z",
  "last_reset_monthly": "2025-12-01T00:00:00Z"
}
```

### Budget Enforcement

**Enforcement Policies:**

1. **Per-Execution Budget**: Checked before each execution
   - If exceeded, reject execution or downgrade model
   - User can override with `--force` flag

2. **Daily Budget**: Checked before each execution
   - If daily limit reached, reject execution
   - Resets at midnight (local timezone)

3. **Monthly Budget**: Checked before each execution
   - If monthly limit reached, reject execution
   - Resets on first day of month

**Budget Violation Handling:**
- **Reject**: Return error, don't execute (default)
- **Downgrade**: Use cheaper model if available
- **Mock**: Use mock model for testing
- **Warn**: Execute but log warning (requires `--allow-budget-exceed` flag)

### Cost Reporting

**CLI Command:**
```bash
rad agents budget [agent-id] [--daily] [--monthly] [--reset]
```

**Output Format:**
```
Agent: arch-agent
Daily Budget: $2.45 / $5.00 (49% used)
Monthly Budget: $45.20 / $100.00 (45% used)
Last Reset: Daily: 2025-12-07, Monthly: 2025-12-01
```

## Agent Recommendation Engine

### Recommendation Algorithm

The recommendation engine suggests the best agent for a task based on:

1. **Capability Matching**: Match task requirements to agent capabilities
2. **Performance Profile**: Consider task complexity and performance needs
3. **Cost Constraints**: Respect budget limitations
4. **Historical Performance**: Learn from past agent performance (future enhancement)

### Capability Matching

**Matching Process:**

1. Extract task requirements from user input or workflow context
2. Tokenize and normalize requirements (lowercase, stem words)
3. Match against agent capabilities (case-insensitive, partial matching)
4. Score agents based on:
   - **Exact Match**: +10 points
   - **Partial Match**: +5 points
   - **Related Match**: +2 points (semantic similarity, future enhancement)

**Example:**
```
Task: "Design a REST API for user management"
Requirements: [api, design, rest, user-management]

Agent Capabilities:
- api-design-agent: [api, design, rest, api-design] → Score: 30 (exact matches)
- arch-agent: [architecture, design, system-design] → Score: 5 (partial match)
- code-agent: [code, implementation] → Score: 0 (no match)
```

### Scoring System

**Score Components:**

1. **Capability Match Score** (0-100): Based on capability matching
2. **Performance Profile Match** (0-20): Task complexity vs agent profile
3. **Cost Efficiency** (0-10): Lower cost = higher score
4. **Availability** (0-10): Model availability bonus

**Final Score:**
```
final_score = capability_score + profile_match + cost_efficiency + availability
```

**Recommendation Threshold:**
- Score >= 50: Recommended
- Score >= 30: Acceptable
- Score < 30: Not recommended

### Recommendation API

**CLI Command:**
```bash
rad agents recommend "Design a REST API" [--profile speed|balanced|thinking|expert]
```

**Output:**
```
Recommended Agents:
1. api-design-agent (Score: 85)
   - Capabilities: api, design, rest, api-design
   - Performance: balanced
   - Estimated Cost: $0.05

2. arch-agent (Score: 45)
   - Capabilities: architecture, design, system-design
   - Performance: thinking
   - Estimated Cost: $0.15
```

## Backward Compatibility Strategy

### Dual-Format Support

The system supports both formats simultaneously:

1. **TOML-Only** (Current): Agents with only TOML config work as before
2. **TOML + YAML** (Enhanced): Agents with both formats get persona features
3. **YAML-Only** (Future): Eventually support YAML-only configs

### Default Values

When persona metadata is missing, system uses defaults:

- **Performance Profile**: `balanced`
- **Recommended Models**: Use TOML `engine` and `model` fields
- **Capabilities**: Empty array (no capability matching)
- **Cost Budget**: No limits (unlimited)

### Migration Path

**Phase 1: Additive (Current)**
- Persona features are optional
- Existing agents work unchanged
- New agents can use persona features

**Phase 2: Enhancement (Future)**
- Tools to add persona metadata to existing agents
- CLI command: `rad agents enhance <agent-id>`
- Auto-generate capabilities from agent descriptions

**Phase 3: Deprecation (Future)**
- TOML-only configs still supported but deprecated
- Migration guide provided
- Deprecation timeline: 2 major versions

### Compatibility Matrix

| Feature | TOML-Only | TOML + YAML | YAML-Only |
|---------|-----------|-------------|-----------|
| Basic Execution | ✅ | ✅ | ✅ (future) |
| Model Selection | ✅ (single) | ✅ (with fallback) | ✅ (with fallback) |
| Cost Tracking | ❌ | ✅ | ✅ |
| Agent Recommendations | ❌ | ✅ | ✅ |
| Performance Profiles | ❌ | ✅ | ✅ |

## Integration Points

### AgentConfig Struct Changes

**Current Structure:**
```rust
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub prompt_path: PathBuf,
    pub engine: Option<String>,
    pub model: Option<String>,
    // ... other fields
}
```

**Enhanced Structure (Additive):**
```rust
pub struct AgentConfig {
    // ... existing fields ...
    
    // Persona fields (optional)
    pub persona: Option<AgentPersona>,
}

pub struct AgentPersona {
    pub recommended_models: RecommendedModels,
    pub capabilities: Vec<String>,
    pub performance_profile: PerformanceProfile,
    pub cost_budget: Option<CostBudget>,
}

pub struct RecommendedModels {
    pub primary: String,
    pub fallback: Option<String>,
    pub premium: Option<String>,
}

pub enum PerformanceProfile {
    Speed,
    Balanced,
    Thinking,
    Expert,
}

pub struct CostBudget {
    pub max_per_execution: Option<f64>,
    pub max_daily: Option<f64>,
    pub max_monthly: Option<f64>,
}
```

### ModelSelector Service Interface

```rust
pub trait ModelSelector {
    /// Select the best model for an agent based on persona and context
    fn select_model(
        &self,
        agent: &AgentConfig,
        context: &ExecutionContext,
    ) -> Result<SelectedModel>;
    
    /// Check model availability
    fn check_availability(&self, model: &str) -> Result<AvailabilityStatus>;
    
    /// Get model metadata (pricing, capabilities)
    fn get_model_metadata(&self, model: &str) -> Result<ModelMetadata>;
}
```

### CostTracker Service Interface

```rust
pub trait CostTracker {
    /// Track cost for an agent execution
    fn track_execution(
        &self,
        agent_id: &str,
        model: &str,
        cost: f64,
    ) -> Result<()>;
    
    /// Check if budget allows execution
    fn check_budget(
        &self,
        agent_id: &str,
        estimated_cost: f64,
    ) -> Result<BudgetStatus>;
    
    /// Get budget status for an agent
    fn get_budget_status(&self, agent_id: &str) -> Result<BudgetStatus>;
    
    /// Reset budget (daily or monthly)
    fn reset_budget(&self, agent_id: &str, period: BudgetPeriod) -> Result<()>;
}
```

### RecommendationEngine Service Interface

```rust
pub trait RecommendationEngine {
    /// Recommend agents for a task
    fn recommend_agents(
        &self,
        task_description: &str,
        context: &RecommendationContext,
    ) -> Result<Vec<AgentRecommendation>>;
    
    /// Get recommendation score for an agent
    fn score_agent(
        &self,
        agent: &AgentConfig,
        task_requirements: &[String],
    ) -> Result<f64>;
}
```

## Implementation Roadmap

### Phase 1: Foundation (Estimated: 2-3 weeks)

**Goals:**
- YAML frontmatter parsing
- Basic persona metadata loading
- Dual-format support (TOML + YAML)

**Deliverables:**
- YAML parser for prompt files
- Persona metadata extraction
- AgentConfig persona field (optional)
- Unit tests for parsing

**Dependencies:**
- YAML parsing library (serde-yaml)
- Prompt file reading enhancement

### Phase 2: Model Selection (Estimated: 2-3 weeks)

**Goals:**
- Model selection engine
- Fallback chain execution
- Model availability checking

**Deliverables:**
- ModelSelector service implementation
- Fallback chain logic
- Model availability cache
- Integration with execution engine

**Dependencies:**
- Phase 1 complete
- Model metadata database

### Phase 3: Cost Tracking (Estimated: 2-3 weeks)

**Goals:**
- Cost calculation
- Budget tracking
- Budget enforcement

**Deliverables:**
- CostTracker service implementation
- Budget storage system
- Budget enforcement logic
- CLI budget commands

**Dependencies:**
- Phase 2 complete
- Model pricing data

### Phase 4: Recommendations (Estimated: 2-3 weeks)

**Goals:**
- Agent recommendation engine
- Capability matching
- Scoring system

**Deliverables:**
- RecommendationEngine service implementation
- Capability matching algorithm
- Scoring system
- CLI recommend command

**Dependencies:**
- Phase 1 complete (capabilities)
- Agent capability database

### Phase 5: Integration & Polish (Estimated: 1-2 weeks)

**Goals:**
- Full system integration
- Performance optimization
- Documentation

**Deliverables:**
- End-to-end integration tests
- Performance benchmarks
- User documentation
- Migration tools

**Dependencies:**
- All previous phases complete

### Total Estimated Effort

**Total: 9-14 weeks** (2-3.5 months)

**Risk Factors:**
- Model pricing data availability
- Performance requirements
- Backward compatibility testing
- User adoption curve

**Mitigation Strategies:**
- Incremental rollout (feature flags)
- Comprehensive testing at each phase
- User feedback loops
- Fallback to current system if issues arise

## Conclusion

The persona system architecture provides a comprehensive foundation for intelligent agent management, cost optimization, and enhanced user experience. The design prioritizes backward compatibility, progressive enhancement, and extensibility to support future requirements.

Key benefits:
- **Intelligent Model Selection**: Automatically choose optimal models
- **Cost Control**: Track and enforce budgets
- **Better Recommendations**: Match agents to tasks effectively
- **Future-Proof**: Extensible architecture for enhancements

The phased implementation approach allows for incremental delivery while maintaining system stability and user experience.

