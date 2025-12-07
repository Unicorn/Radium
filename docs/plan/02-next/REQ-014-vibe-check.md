---
req_id: REQ-014
title: Vibe Check (Metacognitive Oversight)
phase: NEXT
status: Completed
priority: High
estimated_effort: 20-27 hours
dependencies: [REQ-004, REQ-005, REQ-006]
related_docs:
  - docs/project/02-now-next-later.md#step-66-metacognitive-oversight-system--ace-learning
  - docs/project/03-implementation-plan.md#step-66-metacognitive-oversight-system
  - docs/project/VIBE_CHECK_INTEGRATION.md
---

# Vibe Check (Metacognitive Oversight)

## Problem Statement

Agents can fall into reasoning lock-in, where they continue down a flawed path without recognizing errors. Without metacognitive oversight, agents cannot:
- Detect when they're making mistakes
- Recognize when they're overcomplicating solutions
- Identify misalignment with user intent
- Learn from past mistakes
- Apply successful strategies from previous work

Research shows that Chain-Pattern Interrupt (CPI) systems improve agent success rates by +27% and reduce harmful actions by -41%. Radium needs a metacognitive oversight system that provides phase-aware feedback and prevents reasoning lock-in.

## Solution Overview

Implement a comprehensive metacognitive oversight system that provides:
- VibeCheck workflow behavior for requesting oversight
- Metacognitive LLM service using second LLM for meta-feedback
- Phase-aware interrupt integration (planning/implementation/review)
- Session constitution system for per-session rules
- Learning from mistakes system with categorized tracking
- ACE skillbook system for strategy learning
- Integration with workflow execution for automatic oversight

The vibe check system enables agents to receive metacognitive feedback, learn from mistakes, and apply successful strategies, significantly improving alignment and success rates.

## Functional Requirements

### FR-1: VibeCheck Behavior

**Description**: Workflow behavior for requesting metacognitive oversight.

**Acceptance Criteria**:
- [x] VibeCheck added to BehaviorActionType enum
- [x] VibeCheckEvaluator implementation
- [x] VibeCheckDecision struct with risk scores, advice, traits, uncertainties
- [x] Integration with behavior.json system
- [x] Support for automatic triggers at workflow checkpoints
- [x] Phase-aware context support (planning/implementation/review)

**Implementation**: `crates/radium-core/src/workflow/behaviors/vibe_check.rs`

### FR-2: Metacognitive LLM Service

**Description**: Second LLM service for providing meta-feedback.

**Acceptance Criteria**:
- [x] MetacognitiveService using second LLM
- [x] Phase-aware system prompts (planning/implementation/review)
- [x] Support for multiple LLM providers via Model trait
- [x] Structured output: `{ advice, risk_score, traits, uncertainties }`
- [x] Risk score estimation from advice content
- [x] Trait extraction (Complex Solution Bias, Feature Creep, etc.)
- [x] Uncertainty extraction from questions
- [x] Fallback handling for API failures
- [x] Integration with history, learning context, and constitution rules

**Implementation**: 
- `crates/radium-core/src/oversight/mod.rs`
- `crates/radium-core/src/oversight/metacognitive.rs` (~450+ lines)

### FR-3: Session Constitution System

**Description**: Per-session rules and constraints for workflow execution.

**Acceptance Criteria**:
- [x] ConstitutionManager for session-scoped rules
- [x] TTL-based cleanup (1 hour) for stale sessions
- [x] Max 50 rules per session limit
- [x] Constitution tools (update_constitution, reset_constitution, get_constitution)
- [x] Integration with workflow execution context
- [x] Thread-safe implementation

**Implementation**: `crates/radium-core/src/policy/constitution.rs` (~200+ lines)

### FR-4: Learning from Mistakes System

**Description**: Track and learn from past mistakes and successes.

**Acceptance Criteria**:
- [x] LearningStore with categorized mistake tracking
- [x] Standard categories: "Complex Solution Bias", "Feature Creep", "Premature Implementation", "Misalignment", "Overtooling"
- [x] Similarity detection to prevent duplicate entries
- [x] Category normalization
- [x] Learning types: mistake, preference, success
- [x] Learning context generation for oversight prompts
- [x] File-based persistence

**Implementation**: 
- `crates/radium-core/src/learning/mod.rs`
- `crates/radium-core/src/learning/store.rs` (~400+ lines)

### FR-5: ACE Skillbook System

**Description**: Track and apply successful strategies from past work.

**Acceptance Criteria**:
- [x] Skill struct with helpful/harmful/neutral counts
- [x] Skill sections: task_guidance, tool_usage, error_handling, code_patterns, communication, general
- [x] SkillManager for generating skillbook updates
- [x] UpdateOperation enum (ADD, UPDATE, TAG, REMOVE)
- [x] Incremental updates to prevent context collapse
- [x] Pattern extraction from OversightResponse
- [x] Skillbook context injection into agent prompts

**Implementation**: 
- `crates/radium-core/src/learning/skill_manager.rs`
- `crates/radium-core/src/learning/updates.rs`

## Technical Requirements

### TR-1: VibeCheck Decision Structure

**Description**: Decision structure for vibe check results.

**Data Models**:
```rust
#[derive(Debug, Clone)]
pub struct VibeCheckDecision {
    pub should_trigger: bool,
    pub risk_score: f64,  // 0.0 to 1.0
    pub reason: String,
    pub advice: Vec<String>,
    pub traits: Vec<String>,  // e.g., "Complex Solution Bias"
    pub uncertainties: Vec<String>,
}
```

### TR-2: Oversight Response Structure

**Description**: Structured response from metacognitive service.

**Data Models**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OversightResponse {
    pub advice: String,
    pub risk_score: f64,
    pub traits: Vec<String>,
    pub uncertainties: Vec<String>,
    pub helpful_patterns: Vec<String>,
    pub harmful_patterns: Vec<String>,
}
```

### TR-3: Learning Store API

**Description**: APIs for learning from mistakes and successes.

**APIs**:
```rust
pub struct LearningStore {
    workspace_root: PathBuf,
}

impl LearningStore {
    pub fn add_mistake(&self, category: &str, description: &str) -> Result<()>;
    pub fn get_learning_context(&self, limit: usize) -> String;
    pub fn add_skill(&self, skill: Skill) -> Result<()>;
    pub fn tag_skill(&self, skill_id: &str, tag: SkillTag) -> Result<()>;
    pub fn get_skills_by_section(&self, section: &str) -> Vec<Skill>;
}
```

## User Experience

### UX-1: VibeCheck Request

**Description**: Agents request vibe check via behavior.json.

**Example**:
```json
// behavior.json
{
  "action": "vibecheck",
  "reason": "Uncertain about approach, need oversight"
}
```

### UX-2: Oversight Feedback

**Description**: Agents receive structured feedback from oversight LLM.

**Example**:
```
Oversight Feedback:
  Risk Score: 0.6 (Medium)
  Advice: Consider simplifying the approach
  Traits: Complex Solution Bias detected
  Uncertainties: Unclear about performance requirements
```

## Data Requirements

### DR-1: Learning Store

**Description**: File-based storage for learning entries and skills.

**Location**: `.radium/_internals/learning/`

**Format**: JSON files for mistakes and skills

### DR-2: Constitution Rules

**Description**: Session-scoped rules stored in memory.

**Storage**: In-memory with TTL-based cleanup

## Dependencies

- **REQ-004**: Workflow Behaviors - Required for workflow behavior system
- **REQ-005**: Plan Generation - Required for plan context
- **REQ-006**: Memory & Context System - Required for context gathering

## Success Criteria

1. [x] VibeCheck behavior can be triggered at workflow checkpoints
2. [x] Oversight LLM provides phase-aware feedback
3. [x] Session rules are enforced during workflow execution
4. [x] Mistakes and skills are logged and fed into oversight prompts
5. [x] Skillbook strategies are injected into agent context
6. [x] Risk scores can trigger automatic workflow behaviors
7. [x] Learning loop is complete: oversight → patterns → skillbook updates → context injection
8. [x] All oversight operations have comprehensive test coverage

**Completion Metrics**:
- **Status**: ✅ Complete
- **Test Coverage**: 29+ unit tests
- **Lines of Code**: ~1,800+ lines
- **Implementation**: Full metacognitive oversight system with ACE learning
- **Files**: 
  - `crates/radium-core/src/workflow/behaviors/vibe_check.rs`
  - `crates/radium-core/src/oversight/` (metacognitive)
  - `crates/radium-core/src/policy/constitution.rs`
  - `crates/radium-core/src/learning/` (store, skill_manager, updates)

## Out of Scope

- Advanced oversight analytics (future enhancement)
- Oversight model fine-tuning (future enhancement)
- Multi-agent oversight coordination (future enhancement)

## References

- [Now/Next/Later Roadmap](../project/02-now-next-later.md#step-66-metacognitive-oversight-system--ace-learning)
- [Implementation Plan](../project/03-implementation-plan.md#step-66-metacognitive-oversight-system)
- [Vibe Check Integration](../project/VIBE_CHECK_INTEGRATION.md)
- [VibeCheck Implementation](../../crates/radium-core/src/workflow/behaviors/vibe_check.rs)
- [Oversight Service Implementation](../../crates/radium-core/src/oversight/)

