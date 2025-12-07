---
req_id: REQ-016
title: TUI Improvements
phase: LATER
status: Completed
priority: Medium
estimated_effort: 15-20 hours
dependencies: [REQ-003, REQ-007]
related_docs:
  - docs/project/02-now-next-later.md#step-8-enhanced-tui
  - docs/project/03-implementation-plan.md#step-8-enhanced-tui
  - docs/project/TUI_IMPROVEMENT_PLAN.md
---

# TUI Improvements

## Problem Statement

The TUI needs significant improvements to provide a polished, intuitive user experience. Without TUI improvements, users face:
- Poor first-time experience with cryptic errors
- Lack of visual polish and branding
- Poor error handling without actionable guidance
- Missing features (model selection, agent discovery, session history)
- No loading states or progress indicators

The legacy system and modern tools (like CodeMachine) provide polished TUI experiences. Radium needs an equivalent system with comprehensive improvements across four phases.

## Solution Overview

Implement comprehensive TUI improvements across four phases:
- **Phase 1**: Foundation & First-Time Experience (Critical)
- **Phase 2**: Visual Polish & Theming (High)
- **Phase 3**: Enhanced Features (Medium)
- **Phase 4**: Advanced UX (Low)

The TUI improvements transform Radium from a basic prompt interface into a vibrant, intuitive, and robust CLI experience.

## Functional Requirements

### FR-1: Phase 1 - Foundation & First-Time Experience

**Description**: Critical fixes for first-time user experience.

**Acceptance Criteria**:
- [x] Interactive setup wizard for first-time users
- [x] Better error messages with actionable guidance
- [x] Automatic workspace initialization
- [x] Basic theming (colors and status indicators)

**Implementation**: 
- `apps/tui/src/setup.rs`
- `apps/tui/src/errors.rs`
- `apps/tui/src/workspace.rs`

### FR-2: Phase 2 - Visual Polish & Theming

**Description**: Visual improvements and branding.

**Acceptance Criteria**:
- [x] Splash screen on startup
- [x] Color theme system (primary, secondary, status colors)
- [x] Status indicators and icons
- [x] Branded header with session info

**Implementation**: 
- `apps/tui/src/views/splash.rs`
- `apps/tui/src/theme.rs`
- `apps/tui/src/icons.rs`
- `apps/tui/src/views/header.rs`

### FR-3: Phase 3 - Enhanced Features

**Description**: Additional features for improved usability.

**Acceptance Criteria**:
- [x] Model selection UI (`/models` command)
- [x] Enhanced agent browser with metadata
- [x] Session history (`/sessions` command)
- [x] Loading states for async operations

**Implementation**: 
- `apps/tui/src/commands/models.rs`
- `apps/tui/src/views/model_selector.rs`
- `apps/tui/src/views/agents.rs`
- `apps/tui/src/session_manager.rs`
- `apps/tui/src/views/sessions.rs`
- `apps/tui/src/views/loading.rs`

### FR-4: Phase 4 - Advanced UX

**Description**: Advanced user experience features.

**Acceptance Criteria**:
- [x] Command palette (Ctrl+P) with fuzzy search
- [x] Markdown rendering for agent responses
- [x] Scrollback buffer (PgUp/PgDn)
- [x] Split view for complex workflows

**Implementation**: 
- `apps/tui/src/views/markdown.rs`
- `apps/tui/src/views/split.rs`

## Technical Requirements

### TR-1: TUI Theme System

**Description**: Color theme system for consistent styling.

**Data Models**:
```rust
pub struct RadiumTheme {
    pub primary: Color,           // Cyan: #00D9FF
    pub secondary: Color,         // Purple: #A78BFA
    pub success: Color,           // Green: #10b981
    pub warning: Color,           // Yellow: #f59e0b
    pub error: Color,            // Red: #ef4444
    pub info: Color,             // Blue: #06b6d4
    pub text: Color,             // White: #eeeeee
    pub text_muted: Color,       // Gray: #808080
    pub bg_primary: Color,       // Dark: #181D27
    pub bg_panel: Color,         // Darker: #141414
    pub border: Color,           // Gray: #484848
}
```

### TR-2: TUI State Management

**Description**: State management for TUI components.

**Data Models**:
```rust
pub struct WorkflowUIState {
    pub agents: Vec<AgentState>,
    pub current_agent: Option<String>,
    pub output_buffer: Vec<String>,
    pub telemetry: TelemetryState,
    pub checkpoint: CheckpointState,
}
```

## User Experience

### UX-1: Setup Wizard

**Description**: Interactive wizard for first-time setup.

**Example**:
```
Welcome to Radium! 🚀

No AI providers configured yet. Let's get you set up!

Select providers to configure:
  [x] Gemini (Google)
  [ ] OpenAI (GPT-4)
  [ ] Anthropic (Claude)
```

### UX-2: Model Selection

**Description**: Users select models via `/models` command.

**Example**:
```
Available Models:

Gemini:
  [x] gemini-2.0-flash-thinking  (Default)
  [ ] gemini-1.5-pro

Press number to select, Enter to confirm
```

## Data Requirements

### DR-1: TUI Configuration

**Description**: Configuration for TUI settings.

**Location**: `~/.radium/config.toml`

**Format**: TOML with theme and UI preferences

## Dependencies

- **REQ-003**: Core CLI Commands - Required for CLI integration
- **REQ-007**: Monitoring & Telemetry - Required for telemetry display

## Success Criteria

1. [x] First-time users can start chatting within 60 seconds
2. [x] Error messages are actionable and friendly
3. [x] Visual feedback for all async operations
4. [x] Consistent color usage throughout
5. [x] All status changes have visual indicators
6. [x] Professional, modern appearance
7. [x] All TUI improvements have comprehensive test coverage (36+ tests)

**Completion Metrics**:
- **Status**: ✅ Complete
- **Test Coverage**: 36+ passing tests
- **Implementation**: Enhanced TUI fully implemented
- **Files**: 
  - `apps/tui/src/` (setup, errors, workspace, theme, icons, views, commands)

## Out of Scope

- Custom theme support via config file (future enhancement)
- Multi-language support (i18n) (future enhancement)
- Plugin/extensions system (future enhancement)

## References

- [Now/Next/Later Roadmap](../project/02-now-next-later.md#step-8-enhanced-tui)
- [Implementation Plan](../project/03-implementation-plan.md#step-8-enhanced-tui)
- [TUI Improvement Plan](../project/TUI_IMPROVEMENT_PLAN.md)
- [TUI Implementation](../../apps/tui/src/)

