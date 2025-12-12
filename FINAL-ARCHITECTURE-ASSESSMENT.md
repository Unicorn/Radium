# Final Architecture Assessment: CLI Tool Support

**Date**: 2025-12-10
**Status**: 🔴 **ARCHITECTURAL DIVERGENCE DEEPER THAN ANTICIPATED**

---

## The Complete Picture

### TUI Architecture (HAS TOOLS)

```
User Input
    ↓
chat_executor.rs
    ↓
radium-models::Model trait
    ├→ model.generate_with_tools(&messages, &tools, &config) ← TOOLS SUPPORTED!
    ├→ Returns ModelResponse with tool_calls
    └→ Tool execution loop handled in TUI
    ↓
execute_with_tools() function
    ├→ Calls tools as needed
    ├→ Tool implementations from TUI (hardcoded)
    └→ Multi-turn conversation until done
```

**Key**: TUI uses `radium-models` crate directly with full tool calling support.

---

### CLI Architecture (NO TOOL SUPPORT)

```
User Input
    ↓
step.rs
    ↓
radium-core::engines::Engine trait
    ├→ engine.execute(ExecutionRequest) ← NO TOOLS!
    └→ ExecutionRequest has no tools field
    ↓
Single turn, no tool support
```

**Key**: CLI uses `radium-core::engines` abstraction which has NO tool support.

---

## Why Engine Trait Has No Tools

Looking at `radium-core/src/engines/engine_trait.rs`:

```rust
pub struct ExecutionRequest {
    pub model: String,
    pub prompt: String,
    pub system: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
    pub params: HashMap<String, Value>,
    // NO tools field!
}

pub trait Engine {
    async fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResponse>;
    // NO generate_with_tools method!
}
```

The `Engine` trait is a **simplified abstraction** for basic model execution without tool calling.

---

## The Real Options Now

### Option 1: Bypass Engine, Use Model Directly (Like TUI)

**Changes Required**:
1. Modify `step.rs` to use `radium-models::Model` instead of `radium-core::engines::Engine`
2. Build tool list using our `tool_builder` module
3. Call `model.generate_with_tools()` instead of `engine.execute()`
4. Implement tool execution loop (copy from TUI's `execute_with_tools`)

**Pros**:
- Full tool support
- Matches TUI architecture
- Proven to work

**Cons**:
- Major refactor of step.rs
- Bypasses the Engine abstraction entirely
- Duplicates TUI's tool execution logic

**Estimated Effort**: 4-6 hours

---

### Option 2: Add Tool Support to Engine Trait

**Changes Required**:
1. Add `tools` field to `ExecutionRequest`
2. Add `generate_with_tools` method to `Engine` trait
3. Implement in all engine providers (Claude, Gemini, OpenAI, Mock)
4. Update step.rs to use new tool-enabled API

**Pros**:
- Keeps Engine abstraction
- Clean API design
- Future-proof for other Engine users

**Cons**:
- Large refactor across multiple crates
- Need to implement for 4 different engines
- May break existing Engine users

**Estimated Effort**: 8-12 hours

---

### Option 3: Hybrid - Engine Delegates to Model

**Changes Required**:
1. Make Engine implementations delegate to Model for tool calls
2. Add optional `with_tools()` method to Engine trait
3. Keep ExecutionRequest simple, add ToolExecutionRequest variant

**Pros**:
- Backwards compatible
- Gradual migration path
- Reuses Model implementations

**Cons**:
- Added complexity
- Two code paths to maintain

**Estimated Effort**: 6-8 hours

---

### Option 4: Quick Hack - Just Copy TUI's Code

**Changes Required**:
1. Copy `execute_with_tools` from TUI to CLI
2. Copy `get_chat_tools` from TUI to CLI
3. Use Model directly in step.rs (ignore Engine)
4. Hardcode tool list like TUI does

**Pros**:
- Fastest solution (2-3 hours)
- Proven to work
- Gets CLI functional immediately

**Cons**:
- Code duplication
- Not "proper fix"
- Still have two divergent architectures

**Estimated Effort**: 2-3 hours

---

## Recommendation

Given the complexity discovered, I recommend **Option 4 (Quick Hack)** followed by **Option 2 (Proper Fix)** as separate phases:

### Phase 1: Quick Hack (Today)
- Copy TUI's tool execution logic to CLI
- Get CLI working with tools immediately
- Achieve feature parity

### Phase 2: Proper Fix (Next Week)
- Design and implement tool support in Engine trait
- Migrate both CLI and TUI to use unified Engine with tools
- Remove code duplication

This gives you immediate functionality while planning the proper architecture.

---

## What "Proper Fix" Actually Means

When you asked for "the proper fix", I initially thought you meant using OrchestrationService. Now I understand the real "proper fix" would be:

**Adding native tool support to the Engine abstraction** so both CLI and TUI can use the same high-level API.

This is a much larger undertaking than anticipated because it requires:
- Designing the tool calling API for Engine trait
- Implementing in 4 different model providers
- Ensuring backwards compatibility
- Testing across all providers

---

## My Mistake

I initially missed that:
1. Engine and Model are different abstractions
2. Engine has no tool support by design
3. TUI bypasses Engine entirely for chat

This led to underestimating the complexity of achieving true parity.

---

## Decision Point

**What would you like me to do?**

A. **Quick Hack** - Copy TUI's code to CLI (2-3 hours, gets tools working today)
B. **Proper Fix** - Add tools to Engine trait (8-12 hours, proper architecture)
C. **Hybrid** - Quick hack now, proper fix later (recommended)
D. **Different approach** - Your suggestion

I'm ready to proceed with whichever you choose.
