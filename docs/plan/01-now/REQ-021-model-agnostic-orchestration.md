---
req_id: REQ-021
title: Model-Agnostic Orchestration System
phase: NOW
status: Not Started
priority: High
estimated_effort: 24-32 hours
dependencies: []
related_docs:
  - docs/plans/model-agnostic-orchestration.md
  - docs/project/00-project-overview.md#orchestration
  - docs/project/03-implementation-plan.md
---

# Model-Agnostic Orchestration System

## Problem Statement

Users currently must manually select agents and invoke commands (`/chat`, `/agents`) to interact with Radium. This creates friction and requires users to understand which specialist agent to use for each task. Additionally, the orchestration approach should not lock users into a single AI provider - the system must work seamlessly across Gemini, Claude, OpenAI, and local models.

The current workflow requires:
- Explicit `/chat` or `/agents` commands before interaction
- Manual agent selection from 67+ available agents
- Understanding of agent capabilities and specializations
- Switching between agents for multi-step workflows

This creates a poor user experience compared to natural conversation with intelligent routing.

## Solution Overview

Implement an intelligent orchestration system that allows users to interact naturally in the TUI without manual agent selection. The orchestrator automatically analyzes user input, routes tasks to appropriate specialist agents, coordinates multi-agent workflows, and synthesizes results - all while remaining completely model-agnostic through trait-based abstraction.

The orchestrator becomes the default interaction mode, handling:
- Intent analysis and task routing
- Agent selection and invocation
- Multi-turn tool execution
- Result synthesis and presentation
- Graceful degradation when advanced features unavailable

## Functional Requirements

### FR-1: Natural Conversation Interface

**Description**: Users can type freely in the TUI without explicit commands. The orchestrator intercepts input and handles routing automatically.

**Acceptance Criteria**:
- [ ] TUI accepts input without requiring `/chat` or `/agents` prefix
- [ ] Orchestrator analyzes input and determines intent
- [ ] User sees orchestrator thinking process ("🤔 Analyzing...")
- [ ] Clear feedback when agents are being invoked
- [ ] Streaming results displayed as they arrive

### FR-2: Intelligent Agent Routing

**Description**: Orchestrator automatically selects the appropriate specialist agent(s) for each task based on agent descriptions and capabilities.

**Acceptance Criteria**:
- [ ] 90%+ routing accuracy for common tasks
- [ ] Support for single-agent tasks
- [ ] Support for multi-agent workflows
- [ ] Parallel execution for independent tasks
- [ ] Sequential execution for dependent tasks
- [ ] Clear explanation of routing decisions

### FR-3: Multi-Provider Support

**Description**: Orchestration works seamlessly across all AI providers without vendor lock-in.

**Acceptance Criteria**:
- [ ] Support for Gemini function calling
- [ ] Support for Claude tool use
- [ ] Support for OpenAI function calling
- [ ] Prompt-based fallback for models without native tool support
- [ ] Consistent behavior across providers
- [ ] Provider selection via configuration

### FR-4: Tool Execution Loop

**Description**: Orchestrator handles multi-turn tool calling, executing agent invocations and processing results.

**Acceptance Criteria**:
- [ ] Parse tool/function calls from model responses
- [ ] Execute agent invocations with proper parameters
- [ ] Handle tool execution errors gracefully
- [ ] Support up to 5 tool iterations per request
- [ ] Return results to orchestrator for synthesis
- [ ] Prevent infinite loops

### FR-5: Configuration Management

**Description**: Users can configure orchestrator behavior, model selection, and fallback strategies.

**Acceptance Criteria**:
- [ ] Select orchestration provider (gemini, claude, openai, prompt-based)
- [ ] Configure model per provider
- [ ] Set temperature and generation parameters
- [ ] Configure max tool iterations
- [ ] Enable/disable orchestration globally
- [ ] Set fallback preferences

### FR-6: User Control and Transparency

**Description**: Users can monitor orchestrator decisions and override when needed.

**Acceptance Criteria**:
- [ ] `/orchestrator` command shows current configuration
- [ ] `/orchestrator switch <provider>` changes orchestration model
- [ ] `/orchestrator toggle` enables/disables orchestration
- [ ] Orchestrator thinking process visible in UI
- [ ] Agent invocations clearly displayed
- [ ] Ability to cancel long-running orchestrations

## Technical Requirements

### TR-1: Orchestration Provider Abstraction

**Description**: Define a trait-based abstraction that allows any AI model to provide orchestration capabilities.

**Data Models**:
```rust
/// Model-agnostic orchestration provider
pub trait OrchestrationProvider: Send + Sync {
    async fn execute_with_tools(
        &self,
        input: &str,
        tools: &[Tool],
        context: &OrchestrationContext,
    ) -> Result<OrchestrationResult>;

    fn supports_function_calling(&self) -> bool;
}

/// Orchestration result with tool calls
pub struct OrchestrationResult {
    pub response: String,
    pub tool_calls: Vec<ToolCall>,
    pub finish_reason: FinishReason,
}

/// Tool definition
pub struct Tool {
    pub id: String,
    pub name: String,
    pub description: String,
    pub parameters: ToolParameters,
    pub handler: Arc<dyn ToolHandler>,
}
```

**APIs**:
- `OrchestrationProvider::execute_with_tools(input, tools, context) -> Result<OrchestrationResult>`
- `OrchestrationProvider::supports_function_calling() -> bool`

### TR-2: Tool Handler Interface

**Description**: Define how tools (agents) are invoked by the orchestrator.

**Data Models**:
```rust
pub trait ToolHandler: Send + Sync {
    async fn execute(&self, args: &ToolArguments) -> Result<ToolResult>;
}

pub struct ToolArguments {
    pub agent_id: String,
    pub task: String,
    pub context: HashMap<String, String>,
}

pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub metadata: HashMap<String, String>,
}
```

**APIs**:
- `ToolHandler::execute(args) -> Result<ToolResult>`

### TR-3: Agent Tool Registry

**Description**: Convert discovered agents into tool definitions that orchestrators can invoke.

**Data Models**:
```rust
pub struct AgentToolRegistry {
    agents: HashMap<String, AgentMetadata>,
    tools: Vec<Tool>,
}
```

**APIs**:
- `AgentToolRegistry::build_tools() -> Vec<Tool>`
- `AgentToolRegistry::refresh()` - Reload agent definitions

### TR-4: Provider Implementations

**Description**: Implement OrchestrationProvider for each AI provider.

**Integration Points**:
- **GeminiOrchestrator**: Convert tools to `function_declarations`, parse function calls
- **ClaudeOrchestrator**: Convert tools to Claude tool format, parse tool use
- **OpenAIOrchestrator**: Convert tools to OpenAI functions, parse function calls
- **PromptBasedOrchestrator**: Generate structured prompts, parse text-based tool invocations

**Performance Constraints**:
- Orchestration overhead: < 500ms
- Tool execution: parallel when possible
- Response streaming: supported for all providers

### TR-5: Orchestration Context

**Description**: Maintain conversation history and session state across orchestration calls.

**Data Models**:
```rust
pub struct OrchestrationContext {
    pub conversation_history: Vec<Message>,
    pub user_preferences: UserPreferences,
    pub session_state: HashMap<String, Value>,
}
```

## User Experience

### UX-1: Default TUI Interaction

**Description**: Orchestration is the default mode when users type in TUI.

**Example**:
```
You: I need to refactor the authentication module

🤔 Analyzing task...
📋 Invoking: senior-developer
   Task: Refactor authentication module

[Senior Developer working...]

✅ Analysis complete

The authentication module has been refactored with:
- Improved error handling
- Better separation of concerns
- Added unit tests

Files modified:
- crates/radium-core/src/auth/mod.rs
- crates/radium-core/src/auth/credentials.rs
```

### UX-2: Multi-Agent Workflows

**Description**: Orchestrator coordinates multiple agents for complex tasks.

**Example**:
```
You: Create a new feature for task templates

🤔 Analyzing task...
📋 Multi-agent workflow planned:

1. 📐 product-manager - Define feature requirements
2. 🏗️ architect - Design implementation approach
3. 💻 senior-developer - Implement feature
4. 🧪 tester - Create test suite

Executing in sequence...

[Progress shown for each agent]

✅ Feature complete! 4 agents collaborated successfully.
```

### UX-3: Configuration Commands

**Description**: Users control orchestrator behavior via TUI commands.

**Example**:
```bash
# View current orchestrator
/orchestrator
# Output: Using Gemini (gemini-2.0-flash-thinking-exp)

# Switch to Claude
/orchestrator switch claude
# Output: Switched to Claude (claude-3-5-sonnet-20241022)

# Toggle orchestration off
/orchestrator toggle
# Output: Orchestration disabled. Use /chat or /agents for direct interaction.
```

## Data Requirements

### DR-1: Configuration Storage

**Description**: Orchestrator configuration persisted in Radium config file.

**Schema**:
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

[orchestration.openai]
model = "gpt-4-turbo-preview"
temperature = 0.7
max_tool_iterations = 5

[orchestration.fallback]
enabled = true
provider = "prompt-based"
```

### DR-2: Tool Definitions Cache

**Description**: Cache agent tool definitions to avoid rebuilding on every request.

**Schema**:
```json
{
  "tools": [
    {
      "id": "agent_senior-developer",
      "name": "senior_developer",
      "description": "Premium implementation specialist - Masters Laravel/Livewire/FluxUI...",
      "parameters": {
        "type": "object",
        "properties": {
          "task": {"type": "string", "description": "The development task to perform"}
        },
        "required": ["task"]
      }
    }
  ],
  "last_updated": "2025-12-07T12:00:00Z"
}
```

### DR-3: Orchestration Session State

**Description**: Track conversation history and context within orchestration sessions.

**Schema**:
```rust
pub struct SessionState {
    pub session_id: String,
    pub conversation_history: Vec<Message>,
    pub invoked_agents: Vec<String>,
    pub tool_results: HashMap<String, ToolResult>,
    pub created_at: DateTime<Utc>,
}
```

## Dependencies

- **Gemini Function Calling API**: Required for GeminiOrchestrator
- **Claude Tool Use API**: Required for ClaudeOrchestrator
- **OpenAI Function Calling API**: Required for OpenAIOrchestrator
- **Agent Discovery System**: Must be operational to populate tool registry
- **Model Abstraction Layer**: Used by prompt-based fallback

## Success Criteria

1. [ ] Users can chat naturally without `/chat` or `/agents` commands
2. [ ] Orchestrator achieves 90%+ routing accuracy for common tasks
3. [ ] Works seamlessly across Gemini, Claude, OpenAI, and prompt-based fallback
4. [ ] Orchestration overhead < 500ms average
5. [ ] Multi-agent workflows execute correctly with clear progress feedback
6. [ ] Configuration changes via `/orchestrator` commands work correctly
7. [ ] System gracefully degrades to prompt-based when function calling unavailable
8. [ ] All integration tests pass across all provider implementations

## Out of Scope

- Learning system that adapts to user preferences over time (deferred to future REQ)
- Custom tool definitions beyond agent invocation (deferred to future REQ)
- A/B testing framework for orchestration strategies (deferred to future REQ)
- Advanced observability and tracing (basic logging included, advanced deferred)
- Web/desktop UI for orchestration visualization (TUI only in this REQ)

## References

- [Model-Agnostic Orchestration Plan](../plans/model-agnostic-orchestration.md)
- [Gemini Function Calling](https://ai.google.dev/gemini-api/docs/function-calling)
- [Claude Tool Use](https://docs.anthropic.com/claude/docs/tool-use)
- [OpenAI Function Calling](https://platform.openai.com/docs/guides/function-calling)
- [Project Overview](../project/00-project-overview.md)
- [Implementation Plan](../project/03-implementation-plan.md)
