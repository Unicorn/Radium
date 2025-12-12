---
id: "tui-architecture"
title: "TUI Architecture Documentation"
sidebar_label: "TUI Architecture Documentation"
---

# TUI Architecture Documentation

## Overview

The Radium TUI (Terminal User Interface) is a unified prompt-based interface built with `ratatui` that provides an interactive CLI experience for working with AI agents, orchestration, and MCP integrations. This document describes the architecture, component structure, and design patterns used throughout the TUI implementation.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        main.rs                                │
│  - Terminal initialization (crossterm)                       │
│  - Event loop (keyboard input polling)                      │
│  - Render loop (ratatui frame drawing)                      │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                         App                                   │
│  - Main application state                                     │
│  - Keyboard event handling                                    │
│  - Command execution                                          │
│  - Orchestration service integration                          │
│  - MCP integration                                             │
└──────────────┬───────────────────────────────┬──────────────┘
               │                               │
               ▼                               ▼
┌──────────────────────────┐    ┌──────────────────────────────┐
│      PromptData           │    │      SetupWizard              │
│  - Input buffer           │    │  - State machine              │
│  - Output buffer          │    │  - Provider selection         │
│  - Conversation history   │    │  - API key input              │
│  - Command suggestions    │    │  - Credential storage         │
│  - Command palette        │    └──────────────────────────────┘
└───────────┬───────────────┘
            │
            ▼
┌─────────────────────────────────────────────────────────────┐
│                      Views Layer                              │
│  - prompt.rs      - Main unified prompt interface           │
│  - header.rs      - Header with session info                 │
│  - splash.rs      - Startup splash screen                    │
│  - loading.rs     - Loading indicators                        │
│  - markdown.rs    - Markdown rendering                       │
│  - sessions.rs    - Session list view                        │
│  - model_selector.rs - Model selection UI                     │
│  - split.rs       - Split view for complex workflows         │
└───────────┬──────────────────────────────────────────────────┘
            │
            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Components Layer                           │
│  - output_window.rs    - Output display with scrolling       │
│  - agent_timeline.rs  - Agent execution timeline             │
│  - telemetry_bar.rs   - Token/metrics display                │
│  - checkpoint_modal.rs - Checkpoint management UI            │
│  - log_viewer.rs      - Log viewing component                │
│  - loop_indicator.rs  - Loop execution indicator             │
│  - status_footer.rs   - Status footer                        │
└───────────┬──────────────────────────────────────────────────┘
            │
            ▼
┌─────────────────────────────────────────────────────────────┐
│                     State Layer                               │
│  - workflow_state.rs  - Workflow execution tracking         │
│  - agent_state.rs     - Individual agent state                │
│  - telemetry_state.rs - Token/metrics tracking               │
│  - checkpoint_state.rs - Checkpoint management               │
│  - OutputBuffer       - Bounded output buffer                 │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Application Entry Point (`main.rs`)

The entry point initializes the terminal, sets up logging, and runs the main event loop.

**Key Responsibilities:**
- Terminal setup (raw mode, alternate screen)
- Splash screen rendering
- Event polling and routing to `App`
- Frame rendering coordination

**Event Flow:**
```
Terminal Event → crossterm → main.rs → App::handle_key() → State Update → Render
```

### 2. Application Core (`app.rs`)

The `App` struct is the central coordinator for all TUI functionality.

**State Management:**
- `PromptData` - Unified prompt interface state
- `SetupWizard` - Optional setup wizard state
- `WorkspaceStatus` - Workspace initialization state
- `OrchestrationService` - AI orchestration integration
- `McpIntegration` - MCP tool integration

**Key Methods:**
- `handle_key()` - Routes keyboard events to appropriate handlers
- `execute_command()` - Executes slash commands (`/help`, `/chat`, etc.)
- `handle_orchestrated_input()` - Routes natural language to orchestration
- `send_chat_message()` - Sends messages to AI agents

**Event Routing:**
```rust
Keyboard Event → App::handle_key()
  ├─ Setup wizard active? → SetupWizard::handle_key()
  ├─ Command palette active? → Update palette suggestions
  ├─ Slash command? → execute_command()
  └─ Regular input → handle_orchestrated_input() or send_chat_message()
```

### 3. Unified Prompt Interface (`views/prompt.rs`)

The `PromptData` struct and `render_prompt()` function provide a unified interface that adapts to different display contexts.

**Display Contexts:**
- `Chat` - Active chat session with an agent
- `AgentList` - List of available agents
- `SessionList` - List of chat sessions
- `Dashboard` - System dashboard
- `Help` - Help information
- `ModelSelector` - Model selection interface

**Key Features:**
- Command autocomplete with suggestions
- Command palette (Ctrl+P) with fuzzy search
- Scrollback buffer for conversation history
- Markdown rendering for agent responses

### 4. Setup Wizard (`setup.rs`)

State machine-based wizard for first-time user setup.

**States:**
1. `Welcome` - Initial welcome screen
2. `ProviderSelection` - Select AI providers (Gemini, OpenAI)
3. `ApiKeyInput` - Enter API key for selected provider
4. `Validating` - Validating API key (future)
5. `Complete` - Setup complete

**State Transitions:**
```
Welcome → ProviderSelection → ApiKeyInput → Complete
                ↑                    │
                └────────────────────┘ (Esc to go back)
```

### 5. State Management (`state/`)

Modular state management for workflow execution and agent tracking.

#### Workflow State (`workflow_state.rs`)

Tracks overall workflow execution:
- Workflow status (Idle, Running, Paused, Completed, Failed, Cancelled)
- Step progression
- Agent registration and state
- Output buffer management
- Telemetry tracking

#### Agent State (`agent_state.rs`)

Tracks individual agent execution:
- Agent status and sub-agent states
- Execution timeline
- Output per agent

#### Telemetry State (`telemetry_state.rs`)

Tracks token usage and metrics:
- Input/output token counts
- Cost estimation
- Request timing

#### Checkpoint State (`checkpoint_state.rs`)

Manages workflow checkpoints:
- Checkpoint creation and restoration
- Checkpoint metadata

#### Output Buffer (`state/mod.rs`)

Bounded buffer for output lines:
- Fixed capacity (default 1000 lines)
- Automatic oldest-line removal
- Scroll position management
- Viewport culling support

### 6. View Components (`views/`)

View modules handle rendering of different UI contexts.

#### Prompt View (`prompt.rs`)
- Main unified interface rendering
- Context-aware content display
- Command palette overlay
- Markdown rendering integration

#### Header View (`header.rs`)
- Session information display
- Status indicators
- Branding

#### Markdown View (`markdown.rs`)
- Converts markdown text to ratatui `Line` objects
- Supports bold, italic, code, lists
- Handles code blocks

#### Model Selector (`model_selector.rs`)
- Model selection interface
- Provider grouping
- Default model indication

#### Sessions View (`sessions.rs`)
- Session list display
- Session metadata (message count, last active)

#### Split View (`split.rs`)
- Side-by-side content display
- Useful for complex workflows

### 7. UI Components (`components/`)

Reusable UI components for specific functionality.

#### Output Window (`output_window.rs`)
- Displays `OutputBuffer` content
- Scroll indicators
- Status line with scroll position
- Split view support

#### Agent Timeline (`agent_timeline.rs`)
- Visual timeline of agent execution
- Status indicators per agent
- Sub-agent hierarchy

#### Telemetry Bar (`telemetry_bar.rs`)
- Token usage display
- Cost estimation
- Request metrics

#### Checkpoint Modal (`checkpoint_modal.rs`)
- Checkpoint creation UI
- Checkpoint restoration interface

#### Log Viewer (`log_viewer.rs`)
- Structured log display
- Log level filtering

#### Loop Indicator (`loop_indicator.rs`)
- Visual indicator for loop execution
- Iteration count

#### Status Footer (`status_footer.rs`)
- Bottom status bar
- Contextual information

## Event Flow

### Keyboard Input Flow

```
1. crossterm polls for events (main.rs)
2. Event received → App::handle_key()
3. Route based on current state:
   - Setup wizard active? → SetupWizard::handle_key()
   - Command palette active? → Update palette
   - Slash command? → Command::parse() → execute_command()
   - Regular input? → handle_orchestrated_input() or send_chat_message()
4. State updated in App/PromptData
5. Next frame render → views render updated state
```

### Command Execution Flow

```
1. User types "/chat agent-1"
2. Command::parse() creates Command struct
3. App::execute_command() matches command name
4. Command handler executes:
   - Loads agent configuration
   - Creates/retrieves session
   - Updates DisplayContext to Chat
   - Updates PromptData with agent info
5. Next render shows chat interface
```

### Orchestration Flow

```
1. User types natural language (no "/")
2. App::handle_orchestrated_input() called
3. OrchestrationService processes input
4. Service determines actions/agents needed
5. Execution happens asynchronously
6. Results streamed to conversation buffer
7. UI updates with streaming output
```

## Integration Points

### radium_core Integration

**Authentication:**
- `CredentialStore` - API key storage and retrieval
- `ProviderType` - AI provider enumeration

**Workspace:**
- `Workspace` - Workspace initialization and management

**Agents:**
- Agent discovery from `./agents/` or `~/.radium/agents/`
- Agent configuration loading

**MCP:**
- `McpIntegration` - MCP server connection and tool execution
- `SlashCommandRegistry` - MCP-provided slash commands

### Orchestration Service Integration

**Initialization:**
- `OrchestrationService` created with workspace config
- Service handles natural language input
- Routes to appropriate agents/tools

**Event Handling:**
- Completion events streamed to UI
- Progress updates via telemetry
- Error handling and recovery

## Design Patterns

### 1. State Machine Pattern

Used in `SetupWizard` for managing setup flow:
- Clear state transitions
- State-specific behavior
- Error recovery

### 2. Component-Based UI

Views and components are modular:
- Reusable components
- Clear separation of concerns
- Easy to test and maintain

### 3. Event-Driven Architecture

Async/await for AI interactions:
- Non-blocking UI updates
- Streaming responses
- Concurrent operations

### 4. Unified Interface Pattern

Single `PromptData` struct adapts to multiple contexts:
- Reduces code duplication
- Consistent user experience
- Easy to add new contexts

## Configuration

### Theme System (`theme.rs`)

The `RadiumTheme` struct provides consistent colors:
- Primary/secondary brand colors
- Status colors (success, warning, error, info)
- Text and background colors
- Border colors

Currently hardcoded, but designed for future config file support.

### Workspace Configuration

Workspace initialized from:
- Current directory (if `.radium/` exists)
- `~/.radium/` (fallback)

## Testing Strategy

### Unit Tests

State modules have comprehensive unit tests:
- `workflow_state.rs` - Workflow lifecycle tests
- `state/mod.rs` - OutputBuffer tests

### Integration Tests

Located in `apps/tui/tests/`:
- `integration_test.rs` - Basic integration tests
- `orchestration_commands_test.rs` - Orchestration command tests

### Test Fixtures

Common test scenarios:
- Mock credential store
- Mock workspace
- Mock AI responses

## Performance Considerations

### Current Limitations

1. **Output Buffer**: Limited to 1000 lines (good)
2. **Conversation History**: Grows unbounded (needs optimization)
3. **Rendering**: Full re-render on every frame (needs viewport culling)
4. **Markdown**: Parsed on every render (could be cached)

### Optimization Opportunities

1. **Viewport Culling**: Only render visible lines
2. **Conversation Limits**: Archive old messages to disk
3. **Markdown Caching**: Cache parsed markdown
4. **Incremental Rendering**: Only re-render changed areas

## Future Enhancements

1. **Theme Customization**: Config file support for themes
2. **Keyboard Shortcuts Help**: In-app shortcut reference
3. **Performance Optimization**: Viewport culling and conversation limits
4. **Plugin System**: Extensible command system
5. **Multi-language Support**: i18n for internationalization

## File Structure

```
apps/tui/src/
├── main.rs              # Entry point, event loop
├── app.rs               # Main application logic
├── lib.rs               # Library exports
├── commands.rs          # Command parsing
├── setup.rs             # Setup wizard
├── theme.rs             # Theme system
├── icons.rs             # Status icons
├── errors.rs            # Error handling
├── workspace.rs         # Workspace initialization
├── session_manager.rs   # Session management
├── chat_executor.rs     # Chat execution
├── navigation.rs        # Navigation helpers
├── views/               # View layer
│   ├── prompt.rs       # Main prompt interface
│   ├── header.rs       # Header view
│   ├── splash.rs       # Splash screen
│   ├── loading.rs      # Loading indicators
│   ├── markdown.rs     # Markdown rendering
│   ├── sessions.rs     # Session list
│   ├── model_selector.rs # Model selection
│   └── split.rs        # Split view
├── components/         # Reusable components
│   ├── output_window.rs
│   ├── agent_timeline.rs
│   ├── telemetry_bar.rs
│   ├── checkpoint_modal.rs
│   ├── log_viewer.rs
│   ├── loop_indicator.rs
│   └── status_footer.rs
└── state/               # State management
    ├── workflow_state.rs
    ├── agent_state.rs
    ├── telemetry_state.rs
    ├── checkpoint_state.rs
    └── mod.rs
```

## API Reference

### App

```rust
pub struct App {
    pub should_quit: bool,
    pub prompt_data: PromptData,
    pub current_agent: Option<String>,
    pub current_session: Option<String>,
    pub setup_complete: bool,
    pub available_commands: Vec<(&'static str, &'static str)>,
    pub setup_wizard: Option<SetupWizard>,
    pub workspace_status: Option<WorkspaceStatus>,
    pub orchestration_service: Option<Arc<OrchestrationService>>,
    pub orchestration_enabled: bool,
    pub mcp_integration: Option<Arc<Mutex<McpIntegration>>>,
    pub mcp_slash_registry: SlashCommandRegistry,
}

impl App {
    pub fn new() -> Self;
    pub async fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<()>;
    pub async fn execute_command(&mut self, cmd: Command) -> Result<()>;
    pub async fn handle_orchestrated_input(&mut self, input: String) -> Result<()>;
    pub async fn send_chat_message(&mut self, message: String) -> Result<()>;
}
```

### PromptData

```rust
pub struct PromptData {
    pub context: DisplayContext,
    pub input: String,
    pub output: Vec<String>,
    pub conversation: Vec<String>,
    pub agents: Vec<(String, String)>,
    pub sessions: Vec<(String, usize)>,
    pub selected_index: usize,
    pub command_suggestions: Vec<String>,
    pub selected_suggestion_index: usize,
    pub scrollback_offset: usize,
    pub command_palette_active: bool,
    pub command_palette_query: String,
}

impl PromptData {
    pub fn new() -> Self;
    pub fn push_char(&mut self, c: char);
    pub fn pop_char(&mut self);
    pub fn clear_input(&mut self);
    pub fn add_output(&mut self, line: String);
    pub fn clear_output(&mut self);
}
```

### SetupWizard

```rust
pub enum SetupState {
    Welcome,
    ProviderSelection { selected_providers: Vec<ProviderType>, cursor: usize },
    ApiKeyInput { provider: ProviderType, input: String },
    Validating { provider: ProviderType },
    Complete,
}

pub struct SetupWizard {
    pub state: SetupState,
    pub error_message: Option<String>,
}

impl SetupWizard {
    pub fn new() -> Self;
    pub fn new_skip_welcome() -> Self;
    pub fn is_needed() -> bool;
    pub async fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<bool>;
    pub fn display_lines(&self) -> Vec<String>;
    pub fn title(&self) -> String;
}
```

### WorkflowUIState

```rust
pub enum WorkflowStatus {
    Idle,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

pub struct WorkflowUIState {
    pub workflow_id: String,
    pub workflow_name: String,
    pub status: WorkflowStatus,
    pub start_time: Option<Instant>,
    pub end_time: Option<Instant>,
    pub current_step: usize,
    pub total_steps: usize,
    pub agents: HashMap<String, AgentState>,
    pub output_buffer: OutputBuffer,
    pub telemetry: TelemetryState,
    pub checkpoint: CheckpointState,
    pub error_message: Option<String>,
}

impl WorkflowUIState {
    pub fn new(workflow_id: String, workflow_name: String, total_steps: usize) -> Self;
    pub fn start(&mut self);
    pub fn pause(&mut self);
    pub fn resume(&mut self);
    pub fn complete(&mut self);
    pub fn fail(&mut self, error: String);
    pub fn cancel(&mut self);
    pub fn next_step(&mut self);
    pub fn register_agent(&mut self, agent_id: String, agent_name: String);
    pub fn progress_percentage(&self) -> u8;
}
```

## Conclusion

The TUI architecture follows a clear separation of concerns with modular components, state management, and view layers. The unified prompt interface provides a consistent user experience while supporting multiple display contexts. The event-driven, async architecture ensures responsive UI updates during AI interactions.

For questions or contributions, refer to the main project documentation and development guidelines.

