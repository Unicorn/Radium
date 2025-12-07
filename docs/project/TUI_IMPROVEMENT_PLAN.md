# Radium TUI Improvement Plan

> **Goal:** Transform the Radium TUI from a basic prompt interface into a vibrant, intuitive, and robust CLI experience inspired by CodeMachine's polished UX.

## Current Pain Points

### 1. Poor First-Time Experience
- ❌ Cryptic error: "Unsupported Model Provider: GEMINI_API_KEY environment variable not set"
- ❌ User doesn't know they need to authenticate
- ❌ No guidance on which providers are supported
- ❌ No automatic workspace initialization
- ❌ Confusing whether to use env vars or `rad auth login`

### 2. Lack of Visual Polish
- ❌ Plain monochrome interface
- ❌ No splash screen or branding
- ❌ No loading states or progress indicators
- ❌ No use of colors for status (success/error/warning)
- ❌ No visual hierarchy

### 3. Poor Error Handling
- ❌ Raw technical errors shown to users
- ❌ No actionable guidance in error messages
- ❌ No error recovery flows
- ❌ Errors don't suggest next steps

### 4. Missing Features
- ❌ No model selection UI (user stuck with defaults)
- ❌ No agent discovery/preview
- ❌ No session history browser
- ❌ No interactive setup wizard
- ❌ No status indicators for running tasks

---

## Improvement Roadmap

### Phase 1: Foundation & First-Time Experience (Priority: Critical)

#### 1.1 Interactive Setup Wizard
**When:** First run or when no auth configured

**Flow:**
```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│                    Welcome to Radium! 🚀                        │
│                                                                 │
│     Transform your terminal into an AI-powered workspace       │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘

No AI providers configured yet. Let's get you set up!

Select providers to configure (Space to select, Enter to continue):

  [x] Gemini (Google)      - Best for: Planning & Research
  [ ] OpenAI (GPT-4)       - Best for: Code Generation
  [ ] Anthropic (Claude)   - Best for: Complex Reasoning

Continue? (y/n): _
```

**Implementation:**
- Detect if `CredentialStore` has any configured providers
- Show interactive checkbox list (using `crossterm` input handling)
- For each selected provider:
  - Prompt for API key with masked input
  - Validate key format
  - Test connection (optional, with timeout)
  - Store using `CredentialStore::store()`
- Create default workspace structure (`~/.radium/`)
- Set default model preferences

**Files to modify:**
- `apps/tui/src/setup.rs` (new module)
- `apps/tui/src/app.rs` (call setup wizard on first run)

#### 1.2 Better Error Messages
**Current:**
```
Error: Unsupported Model Provider: GEMINI_API_KEY environment variable not set
```

**Improved:**
```
┌─────────────────────────────────────────────────────────────────┐
│ ⚠️  Authentication Required                                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ No Gemini API key found. You need to authenticate before       │
│ chatting with agents.                                           │
│                                                                 │
│ Quick fix:                                                      │
│   rad auth login gemini                                         │
│                                                                 │
│ Or set environment variable:                                    │
│   export GEMINI_API_KEY='your-key-here'                        │
│                                                                 │
│ Press 'a' to authenticate now, or Esc to continue              │
└─────────────────────────────────────────────────────────────────┘
```

**Implementation:**
- Create error wrapper types with context
- Add "press 'a' to authenticate" shortcut
- Show formatted error boxes instead of raw messages
- Include actionable next steps

**Files to modify:**
- `apps/tui/src/errors.rs` (new module)
- `apps/tui/src/views/prompt.rs` (render error boxes)
- `apps/tui/src/chat_executor.rs` (better error handling)

#### 1.3 Automatic Workspace Initialization
**Current:** No automatic setup

**Improved:**
- On first run, create `~/.radium/` structure:
  ```
  ~/.radium/
  ├── auth/
  │   └── credentials.json
  ├── agents/
  │   └── (discovered agents)
  ├── sessions/
  │   └── (chat history)
  └── config.toml
  ```
- Check for existing workspace on startup
- Auto-discover agents in standard locations

**Implementation:**
- Check if `~/.radium/` exists on startup
- If not, create directory structure
- Copy default config
- Discover agents from:
  - `~/.radium/agents/`
  - `./agents/` (project-local)
  - Built-in agents

**Files to modify:**
- `apps/tui/src/workspace.rs` (new module)
- `apps/tui/src/app.rs` (call workspace init)

---

### Phase 2: Visual Polish & Theming (Priority: High)

#### 2.1 Splash Screen
**Implementation:**
```rust
// Show on startup, before main UI loads
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│                          Radium                                 │
│                       ━━━━━━━━━━━━                             │
│                                                                 │
│                    Loading workspace...                         │
└─────────────────────────────────────────────────────────────────┘
```

**Display for:** 500-1000ms or until workspace loads

**Files to create:**
- `apps/tui/src/views/splash.rs`

#### 2.2 Color Theme System
**Inspired by CodeMachine theme:**

```rust
pub struct RadiumTheme {
    // Primary colors
    pub primary: Color,           // Cyan: #00D9FF
    pub secondary: Color,         // Purple: #A78BFA

    // Status colors
    pub success: Color,           // Green: #10b981
    pub warning: Color,           // Yellow: #f59e0b
    pub error: Color,             // Red: #ef4444
    pub info: Color,              // Blue: #06b6d4

    // Text colors
    pub text: Color,              // White: #eeeeee
    pub text_muted: Color,        // Gray: #808080
    pub text_dim: Color,          // Dark Gray: #606060

    // Background colors
    pub bg_primary: Color,        // Dark: #181D27
    pub bg_panel: Color,          // Darker: #141414
    pub bg_element: Color,        // Dark Gray: #1e1e1e

    // Border colors
    pub border: Color,            // Gray: #484848
    pub border_active: Color,     // Lighter: #606060
}
```

**Apply to:**
- Command suggestions: Secondary color (purple)
- Success messages: Green
- Error messages: Red
- Input prompt: Primary color (cyan)
- Agent names: Info color (blue)
- Session info: Text muted

**Files to create:**
- `apps/tui/src/theme.rs`
- `apps/tui/src/views/styled.rs` (styled component helpers)

#### 2.3 Status Indicators & Icons
**Add visual feedback:**

```
Status Icons:
  ✓  Success / Completed
  ⚠  Warning
  ✗  Error / Failed
  ⏳ Loading / In Progress
  💬 Chat message
  🤖 Agent response
  📝 Session
  🔑 Authentication
  ⚙️  Settings
```

**Usage:**
- Agent status (idle, running, completed, failed)
- Auth status (configured, missing)
- Message types (user vs agent)
- Command execution status

**Files to modify:**
- `apps/tui/src/icons.rs` (new module)
- All view files (use icons for status)

#### 2.4 Branded Header
**Always visible at top:**

```
┌─────────────────────────────────────────────────────────────────┐
│ Radium                                    [Session: 20251207]   │
│ ━━━━━━                                    [Agent: assistant]    │
└─────────────────────────────────────────────────────────────────┘
```

**Shows:**
- Branding
- Current session ID
- Current agent
- Auth status indicator

**Files to modify:**
- `apps/tui/src/views/header.rs` (new module)
- `apps/tui/src/views/prompt.rs` (include header)

---

### Phase 3: Enhanced Features (Priority: Medium)

#### 3.1 Model Selection UI
**Command:** `/models`

**Display:**
```
Available Models:

Gemini:
  [ ] gemini-2.0-flash-exp       (Fast, experimental)
  [x] gemini-2.0-flash-thinking  (Default - Reasoning optimized)
  [ ] gemini-1.5-pro             (Most capable)

OpenAI:
  [ ] gpt-4o                     (Multimodal)
  [ ] gpt-4o-mini                (Fast, efficient)

Press number to select, Enter to confirm
```

**Implementation:**
- Parse available models from provider APIs
- Show interactive list with current selection
- Save preference to `~/.radium/config.toml`
- Update model for current session

**Files to create:**
- `apps/tui/src/commands/models.rs`
- `apps/tui/src/views/model_selector.rs`

#### 3.2 Agent Browser
**Command:** `/agents`

**Improved display:**
```
┌─────────────────────────────────────────────────────────────────┐
│ Available Agents                                                │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. assistant       General-purpose AI assistant               │
│                     Model: gemini-2.0-flash-thinking           │
│                                                                 │
│  2. doc-agent       Documentation specialist                   │
│                     Model: gemini-1.5-pro                       │
│                                                                 │
│  3. plan-agent      Planning and architecture                  │
│                     Model: gemini-2.0-flash-thinking           │
│                                                                 │
│ Press number to chat, 'i' for info, '/' for commands           │
└─────────────────────────────────────────────────────────────────┘
```

**Features:**
- Show agent metadata (name, description, model)
- Quick action keys (number to select, 'i' for details)
- Visual grouping by type/category

**Files to modify:**
- `apps/tui/src/views/agents.rs` (new module)
- `apps/tui/src/app.rs` (handle agent browser)

#### 3.3 Session History
**Command:** `/sessions`

**Display:**
```
┌─────────────────────────────────────────────────────────────────┐
│ Recent Sessions                                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Today                                                          │
│    ● session_20251207_030618  (assistant)  15 messages         │
│    ● session_20251207_025432  (doc-agent)   8 messages         │
│                                                                 │
│  Yesterday                                                      │
│    ○ session_20251206_184523  (plan-agent) 23 messages         │
│                                                                 │
│ Press number to resume, 'd' to delete, '/' for commands        │
└─────────────────────────────────────────────────────────────────┘
```

**Features:**
- Group by date (Today, Yesterday, This Week, etc.)
- Show message count
- Resume or delete sessions
- Persist to `~/.radium/sessions/`

**Files to create:**
- `apps/tui/src/session_manager.rs`
- `apps/tui/src/views/sessions.rs`

#### 3.4 Loading States
**Show progress for async operations:**

```
┌─────────────────────────────────────────────────────────────────┐
│ Agent is thinking...                                            │
│ ⏳ Generating response                                          │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ 100%                 │
└─────────────────────────────────────────────────────────────────┘
```

**For:**
- Agent response generation
- Model switching
- Agent discovery
- Session loading

**Files to create:**
- `apps/tui/src/views/loading.rs`

---

### Phase 4: Advanced UX (Priority: Low)

#### 4.1 Command Palette
**Trigger:** `/` then start typing

**Display:**
```
> /ag_

Matching commands:
  /agents      - List all available agents
  /agents info - Show detailed agent information

Press ↑↓ to navigate, Enter to select, Esc to cancel
```

**Features:**
- Fuzzy search through commands
- Show command descriptions
- Arrow key navigation
- Auto-complete with Tab

#### 4.2 Markdown Rendering
**For agent responses:**
- Syntax highlighting for code blocks
- Bold/italic formatting
- List rendering
- Table support (basic)

**Library:** Consider `termimad` or custom implementation

#### 4.3 Scrollback Buffer
**Features:**
- Scroll through conversation history (PgUp/PgDn)
- Search in history (Ctrl+F)
- Jump to top/bottom (Home/End)
- Visual scrollbar indicator

#### 4.4 Split View
**For complex workflows:**
```
┌─────────────────────┬───────────────────────────────────────────┐
│ Agents              │ Chat with: assistant                      │
│                     │                                           │
│ 1. assistant        │ You: Hello                                │
│ 2. doc-agent        │                                           │
│ 3. plan-agent       │ Agent: Hi! How can I help you today?      │
│                     │                                           │
│                     │                                           │
│ Press Tab to switch │ > _                                       │
└─────────────────────┴───────────────────────────────────────────┘
```

---

## Technical Implementation Details

### Dependencies to Add

```toml
[dependencies]
# Already have:
ratatui = "0.29"
crossterm = "0.28"

# Add for enhanced features:
tui-input = "0.8"           # Better input handling
tui-textarea = "0.4"        # Multi-line text editing
unicode-width = "0.1"       # Proper width calculations
textwrap = "0.16"           # Text wrapping
fuzzy-matcher = "0.3"       # Fuzzy search for commands

# Optional (for Phase 4):
termimad = "0.29"           # Markdown rendering
syntect = "5.0"             # Syntax highlighting
```

### Module Structure

```
apps/tui/src/
├── main.rs
├── lib.rs
├── app.rs                  # Main app state
├── theme.rs                # Color theme system
├── icons.rs                # Status icons
├── errors.rs               # Error formatting
├── setup.rs                # First-run wizard
├── workspace.rs            # Workspace initialization
├── session_manager.rs      # Session persistence
├── chat_executor.rs        # Existing
├── commands/
│   ├── mod.rs
│   ├── help.rs
│   ├── agents.rs
│   ├── models.rs           # New: Model selection
│   ├── sessions.rs
│   └── dashboard.rs
└── views/
    ├── mod.rs
    ├── splash.rs           # Splash screen
    ├── header.rs           # Branded header
    ├── prompt.rs           # Existing, enhanced
    ├── agents.rs           # Agent browser
    ├── sessions.rs         # Session history
    ├── loading.rs          # Loading states
    ├── model_selector.rs   # Model selection UI
    └── styled.rs           # Styled component helpers
```

---

## Success Metrics

### User Experience
- ✅ First-time users can start chatting within 60 seconds
- ✅ Error messages are actionable and friendly
- ✅ Zero raw technical errors shown to users
- ✅ Visual feedback for all async operations

### Visual Quality
- ✅ Consistent color usage throughout
- ✅ All status changes have visual indicators
- ✅ Professional, modern appearance
- ✅ Clear visual hierarchy

### Robustness
- ✅ Graceful degradation when providers unavailable
- ✅ Clear error recovery paths
- ✅ No crashes on invalid input
- ✅ Persistent state across sessions

---

## Implementation Priority

### Week 1: Critical Fixes
1. Interactive setup wizard
2. Better error messages
3. Automatic workspace init
4. Basic theming (colors)

### Week 2: Polish
5. Splash screen
6. Branded header
7. Status indicators
8. Loading states

### Week 3: Features
9. Model selection UI
10. Enhanced agent browser
11. Session history
12. Command autocomplete improvements

### Week 4+: Advanced (Optional)
13. Markdown rendering
14. Scrollback buffer
15. Split view
16. Advanced search

---

## Notes & Considerations

### Backwards Compatibility
- Maintain existing command structure (`/help`, `/agents`, etc.)
- Environment variables still work as fallback
- Existing chat executor logic preserved

### Testing Strategy
- Manual testing with different terminal sizes
- Test on macOS, Linux, Windows (via CI)
- Test with/without configured auth
- Test error scenarios (network failures, invalid keys, etc.)

### Documentation Updates Needed
- Update README with screenshots
- Document setup wizard flow
- Update CLI reference
- Add troubleshooting guide

---

## Open Questions

1. Should we support custom themes via config file?
2. Do we need session encryption for sensitive conversations?
3. Should we add telemetry for crash reporting?
4. Do we want multi-language support (i18n)?
5. Should we support plugins/extensions?

---

**Next Steps:**
1. Review and approve this plan
2. Create implementation tasks
3. Start with Phase 1 (Critical Fixes)
4. Iterate based on user feedback
