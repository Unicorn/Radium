# ACE Learning Integration

**Last Updated**: 2025-01-XX  
**Status**: ✅ Complete  
**Reference**: ACE paper (arXiv:2510.04618) and implementation in `old/agentic-context-engine/`

## Overview

Radium now includes ACE (Agentic Context Engineering) learning capabilities, enabling agents to learn from execution feedback and improve over time without fine-tuning. This extends the existing `LearningStore` with skillbook functionality, creating a comprehensive learning system that tracks both mistakes and successful strategies.

## Architecture

### Three-Component System

1. **LearningStore** (Extended)
   - Original: Tracks mistakes, preferences, and successes by category
   - New: Tracks skills with helpful/harmful/neutral counts
   - Both coexist in the same store

2. **MetacognitiveService** (Enhanced)
   - Original: Provides phase-aware oversight feedback
   - New: Extracts helpful/harmful patterns from oversight analysis
   - Patterns feed into SkillManager for skillbook updates

3. **SkillManager** (New)
   - Analyzes oversight feedback
   - Generates structured update operations
   - Prevents context collapse through incremental updates

### Learning Loop

```
Agent Execution
    ↓
Oversight Feedback (MetacognitiveService)
    ↓
Pattern Extraction (helpful/harmful patterns)
    ↓
SkillManager generates UpdateBatch
    ↓
LearningStore applies updates
    ↓
Context Injection (ContextManager)
    ↓
Next Agent Execution (with learned strategies)
```

## Key Components

### Skill Structure

```rust
pub struct Skill {
    pub id: String,
    pub section: String,  // task_guidance, tool_usage, error_handling, etc.
    pub content: String,
    pub helpful: u32,
    pub harmful: u32,
    pub neutral: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: SkillStatus,  // Active or Invalid
}
```

### Update Operations

Four operation types prevent context collapse:

- **ADD**: Add new strategy skills
- **UPDATE**: Modify existing skills
- **TAG**: Update helpful/harmful counts
- **REMOVE**: Soft-delete unhelpful skills

### Skill Sections

Skills are organized into sections for better context injection:

- `task_guidance` - General task execution strategies
- `tool_usage` - Tool and API usage patterns
- `error_handling` - Error prevention and recovery
- `code_patterns` - Code structure and patterns
- `communication` - User interaction patterns
- `general` - Other strategies

## Usage

### Adding Skills

```rust
let mut learning_store = LearningStore::new(&workspace_root)?;
let skill = learning_store.add_skill(
    "task_guidance".to_string(),
    "When querying financial data, filter by date range first to reduce result set size".to_string(),
    None,  // Auto-generate ID
)?;
```

### Tagging Skills

```rust
learning_store.tag_skill("skill-00001", "helpful", 1)?;
learning_store.tag_skill("skill-00001", "harmful", 0)?;
```

### Generating Updates from Oversight

```rust
let skill_manager = SkillManager::new(model);
let update_batch = skill_manager.generate_updates(
    &oversight_response,
    &learning_store,
    "Financial data analysis",
    "5/10 tasks completed",
).await?;

learning_store.apply_update(&update_batch)?;
```

### Context Injection

```rust
let context_manager = ContextManager::for_plan(&workspace, requirement_id)?;

// Automatically includes:
// - Learning context (mistakes/preferences)
// - Skillbook context (strategies)
let context = context_manager.build_context("agent[input:file.md]", Some(requirement_id))?;
```

## Integration Points

### ContextManager

- `gather_learning_context()` - Retrieves mistake/preference context
- `gather_skillbook_context()` - Retrieves strategy context
- Both automatically included in `build_context()`

### MetacognitiveService

- Extracts `helpful_patterns` and `harmful_patterns` from oversight analysis
- Patterns feed into SkillManager for skillbook updates
- Learning context included in oversight prompts

### Workflow Integration

The learning loop can be integrated into workflow execution:

```rust
// After task execution
let oversight = metacognitive_service.generate_oversight(&request).await?;
let updates = skill_manager.generate_updates(&oversight, &learning_store, context, progress).await?;
learning_store.apply_update(&updates)?;
```

## Benefits

1. **Prevents Context Collapse**: Incremental updates (ADD/UPDATE/TAG/REMOVE) instead of full regeneration
2. **Self-Improving Agents**: Agents learn from feedback without fine-tuning
3. **Pattern Recognition**: Both mistakes and successful strategies are tracked
4. **Contextual Learning**: Skills organized by section for targeted injection
5. **Backwards Compatible**: Extends existing LearningStore without breaking changes

## Technical Details

### Update Operations Format

```json
{
  "reasoning": "Extracted helpful pattern from successful task",
  "operations": [
    {
      "type": "ADD",
      "section": "task_guidance",
      "content": "Strategy description",
      "skill_id": null
    },
    {
      "type": "TAG",
      "section": null,
      "content": null,
      "skill_id": "skill-00001",
      "metadata": {"helpful": 1}
    }
  ]
}
```

### Skillbook Context Format

```
# Skillbook Strategies

## task_guidance

- [skill-00001] When querying financial data, filter by date range first (helpful=5, harmful=0)
- [skill-00002] Always validate user input before processing (helpful=3, harmful=1)

## tool_usage

- [skill-00003] Use batch API calls when processing multiple items (helpful=8, harmful=0)
```

## Workflow Integration

The `LearningIntegration` helper provides a clean API for integrating learning into workflow execution:

```rust
use radium_core::learning::{LearningConfig, LearningIntegration};
use radium_core::oversight::MetacognitiveService;
use radium_core::learning::{SkillManager, LearningStore};
use radium_core::workflow::behaviors::vibe_check::WorkflowPhase;

// Create integration helper
let integration = LearningIntegration::new(
    LearningConfig::default(),
    Arc::new(metacognitive_service),
    Arc::new(skill_manager),
    Arc::new(Mutex::new(learning_store)),
);

// After task execution
let oversight = integration.process_task_learning(
    WorkflowPhase::Implementation,
    "Complete the feature",
    "Use React and TypeScript",
    "50% complete",
    "Task output context",
    "Frontend development",
).await?;
```

This automatically:
1. Generates oversight feedback
2. Extracts helpful/harmful patterns
3. Generates skillbook updates
4. Applies updates to the learning store

## Future Enhancements

- **Semantic Deduplication**: Detect and merge similar skills automatically
- **Async Learning**: Process updates asynchronously for better performance
- **Embedding Support**: Add embeddings for semantic similarity detection
- **Skill Validation**: LLM-based validation of skill quality before adding
- **Automatic Workflow Integration**: Automatically call learning integration after each workflow step

## References

- ACE Paper: arXiv:2510.04618
- Implementation: `old/agentic-context-engine/`
- Integration Guide: `old/agentic-context-engine/docs/INTEGRATION_GUIDE.md`

