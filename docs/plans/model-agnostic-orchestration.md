# Model-Agnostic Orchestration Implementation Plan

**Status**: Planning
**Priority**: High
**Estimated Effort**: 3-4 days
**Owner**: TBD
**Created**: 2024-12-07

## Overview

Implement a model-agnostic orchestration system that allows users to interact with Radium naturally without manually selecting agents. The orchestrator automatically routes tasks to appropriate specialist agents and synthesizes results, working seamlessly across different AI providers (Gemini, Claude, OpenAI, local models).

## Goals

### Primary Goals
1. **Natural Interaction** - Users can type freely without `/chat` or `/agents` commands
2. **Intelligent Routing** - Orchestrator automatically selects the right agent(s) for each task
3. **Provider Agnostic** - Works with any AI model, no vendor lock-in
4. **Graceful Degradation** - Falls back to prompt-based routing if function calling unavailable

### Success Criteria
- ✅ User can chat naturally in TUI without manual agent selection
- ✅ Orchestrator correctly routes 90%+ of tasks to appropriate agents
- ✅ System works with Gemini, Claude, OpenAI, and prompt-based fallback
- ✅ Can switch orchestration models via configuration
- ✅ Performance: Orchestration overhead < 500ms

## Architecture

### High-Level Design

```
User Input
    ↓
Orchestrator (configurable model)
    ↓
[Analyzes intent + selects tools]
    ↓
Tool Execution Loop
    ↓
Agent Invocations (parallel/sequential)
    ↓
Result Synthesis
    ↓
Response to User
```

### Core Components

#### 1. Orchestration Abstraction Layer
**File**: `crates/radium-orchestrator/src/orchestration/mod.rs`

```rust
/// Model-agnostic orchestration provider
pub trait OrchestrationProvider: Send + Sync {
    /// Execute user input with available tools
    async fn execute_with_tools(
        &self,
        input: &str,
        tools: &[Tool],
        context: &OrchestrationContext,
    ) -> Result<OrchestrationResult>;

    /// Check if provider supports native function calling
    fn supports_function_calling(&self) -> bool;
}
```

#### 2. Tool Definition
**File**: `crates/radium-orchestrator/src/orchestration/tool.rs`

```rust
/// Model-agnostic tool definition
pub struct Tool {
    pub id: String,
    pub name: String,
    pub description: String,
    pub parameters: ToolParameters,
    pub handler: Arc<dyn ToolHandler>,
}

pub trait ToolHandler: Send + Sync {
    async fn execute(&self, args: &ToolArguments) -> Result<ToolResult>;
}
```

#### 3. Provider Implementations

**3a. Gemini Orchestrator**
**File**: `crates/radium-orchestrator/src/orchestration/gemini.rs`
- Implements `OrchestrationProvider`
- Converts tools to Gemini `function_declarations`
- Handles function call parsing and execution loop

**3b. Claude Orchestrator**
**File**: `crates/radium-orchestrator/src/orchestration/claude.rs`
- Implements `OrchestrationProvider`
- Converts tools to Claude tool format
- Handles tool use parsing and execution loop

**3c. OpenAI Orchestrator**
**File**: `crates/radium-orchestrator/src/orchestration/openai.rs`
- Implements `OrchestrationProvider`
- Converts tools to OpenAI function calling format
- Handles function call parsing and execution loop

**3d. Prompt-Based Orchestrator (Fallback)**
**File**: `crates/radium-orchestrator/src/orchestration/prompt_based.rs`
- Implements `OrchestrationProvider`
- Works with any model via structured prompting
- Parses text-based tool invocations

#### 4. Agent Tool Registry
**File**: `crates/radium-orchestrator/src/orchestration/agent_tools.rs`

```rust
/// Converts agents to tools for orchestration
pub struct AgentToolRegistry {
    agents: HashMap<String, AgentMetadata>,
}

impl AgentToolRegistry {
    pub fn build_tools(&self) -> Vec<Tool> {
        // Convert each agent to a tool definition
    }
}
```

#### 5. Orchestration Engine
**File**: `crates/radium-orchestrator/src/orchestration/engine.rs`

```rust
pub struct OrchestrationEngine {
    provider: Box<dyn OrchestrationProvider>,
    tool_registry: AgentToolRegistry,
    context: OrchestrationContext,
}

impl OrchestrationEngine {
    pub async fn execute(&mut self, user_input: &str) -> Result<String> {
        let tools = self.tool_registry.build_tools();
        let result = self.provider.execute_with_tools(
            user_input,
            &tools,
            &self.context,
        ).await?;

        // Handle multi-turn tool calling
        self.execute_tools(&result.tool_calls).await
    }
}
```

## Implementation Phases

### Phase 1: Core Abstraction (Days 1-2)
**Goal**: Build model-agnostic foundation

#### Tasks
- [ ] **ORCH-001**: Create `OrchestrationProvider` trait
  - Define trait interface
  - Add error types
  - Document trait requirements
  - **Files**: `crates/radium-orchestrator/src/orchestration/mod.rs`

- [ ] **ORCH-002**: Implement `Tool` and `ToolHandler` abstractions
  - Define tool structure
  - Create handler trait
  - Add parameter validation
  - **Files**: `crates/radium-orchestrator/src/orchestration/tool.rs`

- [ ] **ORCH-003**: Create `OrchestrationContext`
  - Conversation history
  - User preferences
  - Session state
  - **Files**: `crates/radium-orchestrator/src/orchestration/context.rs`

- [ ] **ORCH-004**: Build `AgentToolRegistry`
  - Load all agents
  - Convert to tool definitions
  - Cache tool metadata
  - **Files**: `crates/radium-orchestrator/src/orchestration/agent_tools.rs`

### Phase 2: Provider Implementations (Days 2-3)
**Goal**: Implement orchestration for each AI provider

#### Tasks
- [ ] **ORCH-005**: Implement `GeminiOrchestrator`
  - Tool → function_declaration conversion
  - Function call parsing
  - Multi-turn execution loop
  - **Files**: `crates/radium-orchestrator/src/orchestration/gemini.rs`
  - **Dependencies**: Gemini Function Calling API

- [ ] **ORCH-006**: Implement `ClaudeOrchestrator`
  - Tool → Claude tool format conversion
  - Tool use parsing
  - Multi-turn execution loop
  - **Files**: `crates/radium-orchestrator/src/orchestration/claude.rs`
  - **Dependencies**: Claude Tool Use API

- [ ] **ORCH-007**: Implement `OpenAIOrchestrator`
  - Tool → OpenAI function format conversion
  - Function call parsing
  - Multi-turn execution loop
  - **Files**: `crates/radium-orchestrator/src/orchestration/openai.rs`
  - **Dependencies**: OpenAI Function Calling API

- [ ] **ORCH-008**: Implement `PromptBasedOrchestrator` (Fallback)
  - Generate structured prompts with tool descriptions
  - Parse text-based tool invocations
  - Handle errors gracefully
  - **Files**: `crates/radium-orchestrator/src/orchestration/prompt_based.rs`
  - **Dependencies**: None (works with any model)

### Phase 3: Engine Integration (Day 3)
**Goal**: Wire orchestration into the system

#### Tasks
- [ ] **ORCH-009**: Create `OrchestrationEngine`
  - Provider factory
  - Tool execution loop
  - Result synthesis
  - **Files**: `crates/radium-orchestrator/src/orchestration/engine.rs`

- [ ] **ORCH-010**: Add configuration support
  - Orchestrator settings in config
  - Model selection
  - Fallback preferences
  - **Files**: `crates/radium-core/src/config/mod.rs`

- [ ] **ORCH-011**: Create agent tool handlers
  - Implement `ToolHandler` for agent invocation
  - Handle async execution
  - Stream results if supported
  - **Files**: `crates/radium-orchestrator/src/orchestration/handlers.rs`

### Phase 4: TUI Integration (Day 4)
**Goal**: Make orchestration the default TUI experience

#### Tasks
- [ ] **ORCH-012**: Update TUI chat executor
  - Use `OrchestrationEngine` by default
  - Show orchestrator thinking process
  - Display agent invocations
  - **Files**: `apps/tui/src/chat_executor.rs`

- [ ] **ORCH-013**: Add orchestration UI feedback
  - Show "🤔 Analyzing..." state
  - Display agent being invoked
  - Stream results as they arrive
  - **Files**: `apps/tui/src/views/prompt.rs`

- [ ] **ORCH-014**: Update TUI app initialization
  - Initialize orchestration engine
  - Load default orchestrator config
  - Make orchestrator the default chat target
  - **Files**: `apps/tui/src/app.rs`

- [ ] **ORCH-015**: Add `/orchestrator` command
  - Switch orchestration model
  - View current orchestrator
  - Toggle orchestration on/off
  - **Files**: `apps/tui/src/commands.rs`

### Phase 5: Testing & Documentation (Day 4)
**Goal**: Ensure reliability and usability

#### Tasks
- [ ] **ORCH-016**: Unit tests for core abstractions
  - Test `OrchestrationProvider` trait
  - Test tool definitions
  - Test context management
  - **Files**: `crates/radium-orchestrator/src/orchestration/tests/`

- [ ] **ORCH-017**: Integration tests for providers
  - Test Gemini orchestrator
  - Test Claude orchestrator
  - Test OpenAI orchestrator
  - Test prompt-based fallback
  - **Files**: `crates/radium-orchestrator/tests/orchestration_integration_test.rs`

- [ ] **ORCH-018**: End-to-end TUI tests
  - Test natural conversation flow
  - Test agent routing
  - Test multi-agent workflows
  - **Files**: `apps/tui/tests/orchestration_e2e_test.rs`

- [ ] **ORCH-019**: Documentation
  - Update README with orchestration guide
  - Add configuration examples
  - Document provider differences
  - **Files**: `docs/orchestration.md`

## Configuration Example

```toml
[orchestration]
enabled = true
provider = "gemini"  # or "claude", "openai", "prompt-based"

[orchestration.gemini]
model = "gemini-2.0-flash-thinking-exp"
temperature = 0.7
max_tool_iterations = 5

[orchestration.claude]
model = "claude-3-5-sonnet-20241022"
temperature = 0.7
max_tool_iterations = 5

[orchestration.fallback]
enabled = true
provider = "prompt-based"
```

## Technical Decisions

### Tool Calling vs Prompt-Based Routing

**Use Function/Tool Calling When Available:**
- More reliable parsing
- Native model capability
- Better multi-turn handling

**Fallback to Prompt-Based When Needed:**
- Model doesn't support function calling
- Cost optimization (cheaper models)
- User preference

### Parallel vs Sequential Agent Execution

**Parallel** (default):
- Independent tasks can run simultaneously
- Faster overall execution
- Better resource utilization

**Sequential** (when needed):
- Tasks depend on previous results
- Complex workflows
- Explicit user request

### Error Handling Strategy

1. **Provider Errors**: Retry with fallback provider
2. **Agent Errors**: Return error to orchestrator for recovery
3. **Tool Parsing Errors**: Log and ask user for clarification
4. **Timeout**: Cancel long-running agents, return partial results

## Dependencies

### New Dependencies
```toml
# In crates/radium-orchestrator/Cargo.toml
[dependencies]
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
```

### Model API Requirements
- Gemini: Function calling API
- Claude: Tool use API
- OpenAI: Function calling API

## Performance Considerations

### Optimization Strategies
1. **Tool Caching**: Cache tool definitions, reload only on agent changes
2. **Parallel Execution**: Run independent agents concurrently
3. **Streaming**: Stream orchestrator thinking and agent results
4. **Smart Routing**: Use cheaper models for simple routing decisions

### Performance Targets
- Orchestration overhead: < 500ms
- Agent routing accuracy: > 90%
- Multi-agent coordination: < 2s for 3 agents

## Risk Mitigation

### Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Provider API changes | High | Abstract behind trait, maintain adapters |
| Function calling unreliable | Medium | Implement robust fallback to prompt-based |
| Cost escalation | Medium | Use cheaper models for orchestration |
| Complex workflows fail | Medium | Limit max tool iterations, clear error messages |
| User confusion | Low | Show orchestrator thinking, allow manual override |

## Success Metrics

### Quantitative
- [ ] 90%+ routing accuracy
- [ ] < 500ms orchestration overhead
- [ ] 95%+ uptime for orchestration
- [ ] Support 3+ providers

### Qualitative
- [ ] Users prefer orchestration over manual agent selection
- [ ] Clear feedback on orchestrator decision-making
- [ ] Easy to switch between orchestration providers
- [ ] Intuitive error messages

## Future Enhancements

### Post-MVP Features
1. **Learning System**: Orchestrator learns user preferences over time
2. **Multi-Agent Workflows**: Support complex agent choreography
3. **Custom Tool Definitions**: Users can define custom tools
4. **Observability**: Detailed logging and tracing of orchestration decisions
5. **A/B Testing**: Compare orchestration strategies

## References

- [Gemini Function Calling](https://ai.google.dev/gemini-api/docs/function-calling)
- [Claude Tool Use](https://docs.anthropic.com/claude/docs/tool-use)
- [OpenAI Function Calling](https://platform.openai.com/docs/guides/function-calling)
- [Radium Agent Architecture](../architecture/agents.md)

---

**Next Steps:**
1. Review and approve this plan
2. Import into Braingrid for tracking
3. Assign tasks to team members
4. Begin Phase 1 implementation
