# Deep Analysis Improvements

This document describes the improvements made to Radium's AI analysis capabilities to ensure agents perform comprehensive, multi-file analysis instead of shallow, single-file responses.

## Problem

Previously, when asked general questions like "Tell me about this project", agents would:
- Read only one file (often just GEMINI.md or a single doc file)
- Give surface-level answers without deep understanding
- Skip comprehensive analysis
- Not synthesize information from multiple sources

## Solution

### 1. Enhanced Agent Prompts

All key agents now have **mandatory** deep analysis protocols:

- **Research Agent**: 5-phase analysis protocol with mandatory file reading
- **Code Agent**: Pre-implementation analysis workflow
- **Analyzer Agent**: Comprehensive analysis with multi-tool coordination

Key improvements:
- **MANDATORY** language - agents are explicitly told they MUST follow protocols
- Parallel file reading instructions
- Introspection checklists that must be completed before answering
- Explicit prohibition of single-file answers

### 2. Question-Type Detection

Created `QuestionType` enum and `AnalysisPlan` system that:
- Detects question types (ProjectOverview, TechnologyStack, Architecture, etc.)
- Recommends specific files to read for each question type
- Suggests semantic search queries
- Provides synthesis guidance

### 3. Analysis Plan Integration

Enhanced `execute_chat_message` in TUI to:
- Automatically create analysis plans for user questions
- Inject analysis plans into agent prompts
- Prepend analysis guidance to prompt content

### 4. Context Manager Enhancements

Added to `ContextManager`:
- `create_analysis_plan()` - Creates analysis plans from user input
- `build_context_with_analysis()` - Builds context with analysis plan included

## Usage

### Automatic (TUI Chat)

When using TUI chat (`/chat research-agent` or orchestration), analysis plans are automatically:
1. Created from user input
2. Injected into the prompt
3. Enforced by agent instructions

### Manual (CLI)

You can use analysis plans programmatically:

```rust
use radium_core::context::{ContextManager, Workspace};

let workspace = Workspace::discover()?;
let manager = ContextManager::new(&workspace);
let plan = manager.create_analysis_plan("Tell me about this project");

// Use plan.recommended_files, plan.suggested_searches, etc.
```

## Expected Behavior

When asked "Tell me about this project", agents should now:

1. **Read multiple files in parallel**:
   - README.md
   - package.json / Cargo.toml
   - nx.json / rust-toolchain.toml
   - Architecture docs
   - GEMINI.md

2. **Perform semantic searches**:
   - "What is the main purpose and architecture of this project?"
   - Related architecture queries

3. **Synthesize information**:
   - Combine findings from all sources
   - Provide comprehensive overview
   - Include specific examples with file paths

4. **Verify completeness**:
   - Complete introspection checklist
   - Ensure all aspects covered
   - Show deep understanding

## Files Modified

- `prompts/agents/specialized/research-agent.md` - Enhanced with mandatory protocols
- `prompts/agents/core/code-agent.md` - Added analysis workflows
- `prompts/agents/specialized/analyzer-agent.md` - Added introspection
- `apps/tui/src/chat_executor.rs` - Integrated analysis plans
- `crates/radium-core/src/context/analysis.rs` - Question type detection
- `crates/radium-core/src/context/manager.rs` - Analysis plan methods
- `crates/radium-orchestrator/src/routing/question_type.rs` - Orchestrator question types

## Testing

To verify the improvements work:

1. Ask a general question: "Tell me about this project"
2. Check that the agent reads multiple files (check tool calls)
3. Verify the answer is comprehensive and includes:
   - Technology stack details
   - Architecture information
   - Specific file references
   - Multiple sources synthesized

## Future Enhancements

- Pre-execution file reading phase (force file reads before agent execution)
- Agent routing based on question type (route general questions to research-agent)
- Analysis plan caching for similar questions
- Metrics tracking for analysis depth

