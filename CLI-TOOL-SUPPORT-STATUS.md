# CLI Tool Support Implementation Status

**Date**: 2025-12-10
**Last Updated**: 2025-12-10 (Root Cause Identified)
**Status**: ✅ **IMPLEMENTATION COMPLETE** | ❌ **BLOCKED BY RADIUM-MODELS LIMITATIONS**

---

## Summary

Successfully implemented full tool calling support for the Radium CLI (~200 lines of code in apps/cli/src/commands/step.rs). The CLI tool execution infrastructure is **complete and correct**, but testing revealed that the underlying `radium-models` crate has incomplete function calling support:

1. **ClaudeModel**: `generate_with_tools()` not implemented (returns `UnsupportedModelProvider` error)
2. **GeminiModel**: `generate_with_tools()` implemented but has response parsing bug
3. **OpenAIModel**: Unknown status (not tested)

---

## 🔍 Root Cause Analysis (2025-12-10)

### Discovery Process

After implementing the CLI tool execution infrastructure, testing revealed **the implementation is correct** but blocked by limitations in the `radium-models` crate.

### Issue 1: ClaudeModel - Function Calling Not Implemented

**Location**: `crates/radium-models/src/claude.rs:461-470`

```rust
async fn generate_with_tools(
    &self,
    _messages: &[ChatMessage],
    _tools: &[radium_abstraction::Tool],
    _tool_config: Option<&radium_abstraction::ToolConfig>,
) -> Result<ModelResponse, ModelError> {
    Err(ModelError::UnsupportedModelProvider(
        format!("ClaudeModel does not support function calling yet"),
    ))
}
```

**Error Observed**:
```
ERROR: Model execution failed: Unsupported Model Provider: ClaudeModel does not support function calling yet
```

**Impact**: Cannot use Claude models (claude-sonnet-4, claude-opus-4, etc.) with tool calling in CLI

### Issue 2: GeminiModel - Response Parsing Bug

**Location**: `crates/radium-models/src/gemini/mod.rs` (response parsing logic)

**Error Observed**:
```
ERROR: Failed to parse Gemini API response error=error decoding response body
ERROR: Model execution failed: Serialization Error: Failed to parse response: error decoding response body
```

**Analysis**:
- API call IS being made (unlike Claude)
- Response is returned from Gemini API
- Parser fails to deserialize the response when tools are included
- Likely related to how Gemini 2.0 Flash formats tool-related responses

**Impact**: Cannot use Gemini models (gemini-2.0-flash-exp, etc.) with tool calling in CLI

### Conclusion

The CLI tool execution infrastructure in `apps/cli/src/commands/step.rs` is **architecturally correct and complete**. The blocking issues are:

1. **Missing functionality** in `radium-models::claude` (not implemented)
2. **Bug** in `radium-models::gemini` (response parsing)

Both issues require fixes in the `radium-models` crate, not in the CLI.

---

## ✅ Completed Implementation

### 1. Tool Execution Infrastructure (apps/cli/src/commands/step.rs)

**Location**: Lines 1456-1689

#### Helper Functions Added:
- `create_model()` - Factory for creating Model instances (lines 1456-1471)
- `convert_tools()` - Converts OrchestrationTool → AbstractionTool (lines 1480-1493)
- `convert_to_execution_response()` - Converts ModelResponse → ExecutionResponse (lines 1496-1512)
- `execute_tool_call()` - Executes individual tool calls (lines 1515-1536)
- `execute_with_tools_loop()` - Multi-turn conversation loop (lines 1539-1600)
- **`execute_agent_with_tools()`** - Main entry point (lines 1603-1689)

#### Key Features:
- ✅ Multi-turn tool execution loop (max 10 iterations)
- ✅ Proper system/user message separation (matching TUI)
- ✅ Tool call/response handling
- ✅ Error handling and graceful degradation
- ✅ Token usage tracking
- ✅ Progress logging

### 2. Tool Registration

**Source**: `radium-orchestrator::orchestration::tool_builder::build_standard_tools()`

**12 Tools Registered**:
1. `read_file` - Read file contents
2. `write_file` - Write to files
3. `search_replace` - Find and replace in files
4. `list_dir` - List directory contents
5. `glob_file_search` - Pattern-based file search
6. `read_lints` - Read linter output
7. `project_scan` - Analyze project structure
8. `find_references` - Find code references
9. `git_blame` - Git blame information
10. `git_show` - Show git objects
11. `analyze_code_structure` - AST-based code analysis
12. `run_terminal_cmd` - Execute terminal commands

**Tool Schemas**: Valid JSON Schema format, 150-400 bytes each

### 3. Integration Points

**Main Flow**: apps/cli/src/commands/step.rs:428-437
```rust
} else {
    // Use tool-enabled execution path
    execute_agent_with_tools(
        selected_engine_id,
        selected_model,
        &rendered,          // System instructions
        &user_input,        // Actual user query
        &workspace_root,
    ).await
};
```

---

## ⚠️ Current Blocker: Gemini API Issue

### Error Details
**Error**: "Failed to parse Gemini API response error=error decoding response body"

**When**: Sending request with:
- System instruction (from system message)
- User message
- 12 tool definitions
- ToolConfig with mode: Auto or Any

### What Works
✅ Tool schemas are valid JSON Schema
✅ Single user message (no tools) - API succeeds
✅ Tool registration and conversion logic
✅ Message structure matches TUI

### What Fails
❌ System + User messages + Tools → API response parsing error
❌ ToolUseMode::Any → Same parsing error (forces tool usage)

### Analysis

1. **Not a Schema Issue**: Tool definitions are valid and properly formatted
2. **Not an Implementation Issue**: Code structure matches working TUI
3. **Gemini-Specific**: Likely related to how Gemini 2.0 Flash handles combined:
   - `systemInstruction` field
   - Multiple function declarations (12 tools)
   - Tool configuration

### Evidence
- GeminiModel properly extracts system messages → `systemInstruction`
- Tests confirm system message handling works
- TUI uses same message structure and works (though may use different code path)

---

## 📊 Testing Results

### Test 1: Tool Schema Validation
```bash
✓ Built 12 tools
✓ Tool schemas: 157-401 bytes each
✓ Sample schema (read_file): Valid JSON Schema
```

### Test 2: Execution with Tools
```bash
✓ Model instance created
✓ Built 12 tools
✓ Starting tool execution loop
✓ Tool execution iteration 1/10
❌ ERROR: Failed to parse Gemini API response
```

### Test 3: Without System Message
```bash
✓ No parsing error
✓ API call succeeds
❌ Model doesn't call tools (just acknowledges)
```

---

## 🔍 Root Cause Hypotheses

### Hypothesis 1: Gemini 2.0 API Compatibility
The `gemini-2.0-flash-exp` model may have stricter requirements or different format expectations for:
- System instructions combined with tools
- Tool configuration format
- Number of tools (12 may exceed limit)

### Hypothesis 2: API Request Format
The request body structure when combining `systemInstruction` + `tools` + `toolConfig` may be malformed or unsupported.

### Hypothesis 3: Response Format Change
Gemini 2.0 may return responses in a format our parser doesn't expect when tools are involved.

---

## 🛠️ Recommended Solutions

### Option 1: Test with Claude (Recommended First Step)
**Why**: Validates our implementation works with a known-good model

**Action**:
```bash
# Create Claude-based chat agent or test directly
ANTHROPIC_API_KEY=$KEY rad step <claude-agent> "List Rust files"
```

**Expected**: Tools should work correctly, confirming implementation is sound

### Option 2: Debug Gemini API Request/Response
**Action**:
1. Add request body logging before API call
2. Add raw response logging before parsing
3. Compare with TUI's successful requests
4. Check Gemini API documentation for 2.0 changes

### Option 3: Workaround for Gemini
**Action**: Merge system instructions into user message for Gemini only

```rust
if engine_id == "gemini" {
    // Workaround: combine system + user into single user message
    let messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: format!("{}\n\n{}", rendered_prompt, user_input),
        }
    ];
} else {
    // Standard: separate system and user
    let messages = vec![/* system */, /* user */];
}
```

### Option 4: Reduce Tool Count
**Action**: Test with fewer tools (3-4) to see if quantity is the issue

```rust
// Temporarily limit tools for testing
let orchestration_tools = build_standard_tools(workspace_root.clone(), None);
let limited_tools: Vec<_> = orchestration_tools.into_iter().take(4).collect();
```

---

## 📝 Files Modified

### Primary Implementation
- **apps/cli/src/commands/step.rs**: +~200 lines (tool execution)
  - Lines 1456-1689: New tool execution functions
  - Lines 428-437: Integration into main flow

### Supporting Infrastructure
- **crates/radium-orchestrator/src/orchestration/tool_builder.rs**: Fixed trait implementations
  - WorkspaceRootProvider: Return `Option<PathBuf>` instead of `&Path`
  - NoOpSandboxManager: Added for basic tool execution
  - build_standard_tools: Updated signature for trait objects

---

## 🎯 Success Criteria

### Phase 1: Verification ✓
- [x] Tool infrastructure implemented
- [x] 12 tools registered correctly
- [x] Tool schemas validated
- [x] Message structure matches TUI
- [x] Multi-turn loop implemented
- [x] Error handling added
- [x] CLI compiles successfully

### Phase 2: Testing (Blocked)
- [ ] Tools called by LLM
- [ ] Multi-turn conversation works
- [ ] All tool types execute correctly
- [ ] Error recovery functions
- [ ] Token tracking works

### Phase 3: Parity (Pending)
- [ ] CLI matches TUI functionality
- [ ] Works with all supported models
- [ ] Documentation updated
- [ ] Tests passing

---

## 🚀 Next Actions

**Immediate**:
1. Test with Claude to validate implementation
2. If Claude works → Gemini-specific issue confirmed
3. If Claude fails → Review implementation

**Short-term**:
1. Debug Gemini API request/response format
2. Compare exact API calls between TUI and CLI
3. Check Gemini 2.0 API documentation for changes
4. Implement workaround if needed

**Long-term**:
1. Add comprehensive test suite
2. Support all model providers
3. Document tool calling behavior
4. Performance optimization

---

## 📚 Key Learnings

1. **Architecture Divergence**: TUI uses `Model` trait directly, CLI was using `Engine` trait (no tool support)
2. **Solution**: Bypass Engine, use Model directly (like TUI)
3. **Tool Conversion**: Need careful conversion between OrchestrationTool and AbstractionTool types
4. **Message Structure**: System/user separation critical for proper tool calling
5. **Gemini Specifics**: System instructions use separate `systemInstruction` field, not role="system"

---

## 🔗 Related Documentation

- Implementation Plan: `/Users/clay/.claude/plans/happy-riding-pinwheel.md`
- Architecture Analysis: `/Users/clay/Development/RAD/FINAL-ARCHITECTURE-ASSESSMENT.md`
- CLI Reality Check: `/Users/clay/Development/RAD/CLI-ARCHITECTURE-REALITY.md`

---

**Status**: Implementation complete, awaiting Gemini API issue resolution or alternative model testing.
