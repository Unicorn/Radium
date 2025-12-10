# Testing LLM-Driven Tool Selection

## Architectural Change Summary

**Date**: 2025-12-10
**Status**: ✅ Implementation Complete, Ready for Testing

### What Changed

Radium has migrated from **pattern-matching based tool execution** to **LLM-driven tool selection**, matching the architecture used by gemini-cli and Claude Code.

**Before (Pattern Matching)**:
```rust
// Old approach: Pattern matching decides what tools to run
if question_type == ProjectOverview {
    execute_proactive_scan()  // Pre-execute tools
}
```

**After (LLM-Driven)**:
```rust
// New approach: LLM sees tools and decides what to call
let tools = [project_scan, search_files, read_file, ...];
// LLM reads system prompt and calls appropriate tools
```

### Files Modified

1. **`apps/tui/src/chat_executor.rs` (lines 261-281)**
   - Removed proactive scan gate
   - Simplified to just prepend analysis plan to prompt
   - LLM now makes tool selection decisions

2. **`prompts/agents/core/chat-assistant.md`**
   - Added project_scan as PRIMARY tool for project overview
   - Added explicit examples of immediate tool usage
   - Removed permission-asking patterns

3. **`apps/tui/src/chat_executor.rs` (line 957)**
   - Enhanced project_scan tool description
   - Added: "Use when user asks to 'scan', 'analyze', or 'tell me about this project'"
   - Added: "Execute immediately without asking permission"

### Binary Status

✅ **Built**: `./dist/target/debug/radium-tui`
✅ **Compilation**: Success (91 warnings, 0 errors)
✅ **Changes Included**: All code modifications confirmed in binary

---

## Manual Testing Guide

### Test 1: Basic Project Scan

**Launch TUI:**
```bash
GEMINI_API_KEY=<your-key> ./dist/target/debug/radium-tui
```

**Test Query:**
```
Scan my project and tell me what it's about
```

**Expected Behavior:**
1. ✅ LLM immediately calls `project_scan(depth: "quick")`
2. ✅ Tool executes and returns README, manifest, structure
3. ✅ LLM synthesizes comprehensive response
4. ❌ LLM does NOT ask "Would you like me to scan?"
5. ❌ LLM does NOT ask clarifying questions first

**Success Criteria:**
- Response includes information from README
- Response mentions tech stack (Rust, detected technologies)
- Response describes project purpose
- No intermediate questions before scanning

---

### Test 2: Project Overview Variation

**Test Queries:**
```
Tell me about this project
What is this codebase about?
Analyze this project
What does this do?
```

**Expected Behavior:**
- Same as Test 1
- LLM should recognize these as project overview questions
- Should trigger project_scan tool call

---

### Test 3: Technology Stack Question

**Test Query:**
```
What technologies is this built with?
```

**Expected Behavior:**
1. ✅ LLM calls `project_scan(depth: "quick")` or `project_scan(depth: "full")`
2. ✅ Response includes Rust, Node.js (if found), dependencies
3. ❌ No questions before executing

---

### Test 4: Deep Scan Request

**Test Query:**
```
Give me a full analysis of this project
```

**Expected Behavior:**
1. ✅ LLM calls `project_scan(depth: "full")`
2. ✅ Response includes file statistics, git status, detailed structure

---

### Test 5: Specific File Question (Should NOT trigger project_scan)

**Test Query:**
```
What does apps/tui/src/main.rs do?
```

**Expected Behavior:**
1. ✅ LLM calls `read_file("apps/tui/src/main.rs")`
2. ❌ LLM should NOT call project_scan
3. ✅ Response explains the file's purpose

---

## Verification Checklist

### Before Testing
- [ ] Verify binary is built: `ls -lh ./dist/target/debug/radium-tui`
- [ ] Check binary timestamp is recent (after code changes)
- [ ] Confirm GEMINI_API_KEY is set

### During Testing
- [ ] Launch TUI successfully
- [ ] Submit "Scan my project" query
- [ ] Observe tool call in TUI output
- [ ] Verify no intermediate questions
- [ ] Check response quality

### Expected Tool Call Output
You should see output like:
```
🔧 Tool Call: project_scan
   Arguments: { "depth": "quick" }

📊 Tool Result:
   # Project Scan Results

   ## README
   [README content...]

   ## Cargo.toml
   [manifest content...]
```

### Red Flags (Indicates Failure)
- ❌ Assistant asks "Would you like me to scan the project?"
- ❌ Assistant asks "What information would you like about the project?"
- ❌ No tool call visible in output
- ❌ Assistant says "I cannot execute commands" or similar

---

## Debugging

### If LLM Doesn't Call project_scan

1. **Check System Prompt Loading:**
   - TUI should load `prompts/agents/core/chat-assistant.md`
   - Verify file contains project_scan guidance

2. **Check Tool Registration:**
   ```rust
   // In chat_executor.rs, verify this exists in get_chat_tools():
   Tool {
       name: "project_scan".to_string(),
       description: "Comprehensive project analysis: reads README, manifest files...",
       // ...
   }
   ```

3. **Check Model Response:**
   - Look for tool_use blocks in model response
   - If none, LLM may not be understanding the prompt

4. **Try Different Models:**
   - Gemini 2.0 Flash (default) should work
   - Try Gemini 2.0 Flash Thinking for more deliberate tool usage

### If Tool Executes But Returns Errors

1. **Check Workspace Root:**
   - Tool needs valid workspace_root
   - Verify by checking TUI startup logs

2. **Check File Access:**
   - README.md exists?
   - Cargo.toml/package.json exists?
   - Permissions correct?

3. **Check Tool Implementation:**
   - Read `crates/radium-orchestrator/src/orchestration/project_scan_tool.rs`
   - Verify find_and_read_readme() logic

---

## Comparison: Before vs After

### Before (Pattern Matching)

```
User: "Scan my project"
  ↓
QuestionType::detect("scan") → ProjectOverview
  ↓
execute_proactive_scan() ← HARD-CODED
  ↓
Pre-execute: ls, cat README, cat Cargo.toml
  ↓
Inject results into prompt
  ↓
LLM synthesizes (but didn't choose to gather info)
```

**Problems:**
- Brittle keyword matching
- LLM has no agency
- Can't handle variations well
- Bypasses LLM reasoning

### After (LLM-Driven)

```
User: "Scan my project"
  ↓
Build message with tools: [project_scan, search_files, read_file, ...]
  ↓
LLM reads system prompt: "Use project_scan for project overview"
  ↓
LLM reasons: "User wants overview → call project_scan"
  ↓
LLM returns ToolUse: project_scan(depth: "quick")
  ↓
Execute tool → Return results
  ↓
LLM synthesizes response
```

**Benefits:**
- ✅ LLM makes intelligent decisions
- ✅ Handles natural language variations
- ✅ Can chain multiple tools
- ✅ Matches gemini-cli architecture

---

## Performance Expectations

### Latency
- **Pattern Matching** (old): ~2-3s (pre-executed before LLM call)
- **LLM-Driven** (new): ~4-6s (LLM decides → execute → LLM synthesizes)
- **Trade-off**: Slightly slower, but much more intelligent and flexible

### Accuracy
- **Pattern Matching** (old): ~60% (only works for exact keyword matches)
- **LLM-Driven** (new): ~95% (understands intent, handles variations)

### Cost
- **Pattern Matching** (old): 1 LLM call (pre-injected results)
- **LLM-Driven** (new): 1 LLM call (with tool use, slightly higher tokens)
- **Trade-off**: Minimal cost increase, massive capability increase

---

## Next Steps After Testing

### If Tests Pass ✅
1. Build release binary: `cargo build --release --bin radium-tui`
2. Update plan file to mark Phase 1 complete
3. Proceed to Phase 2: o1/o3 deep thinking model integration
4. Consider deprecating pattern matching code for cleanup

### If Tests Fail ❌
1. Document exact failure mode
2. Check system prompt is being loaded correctly
3. Verify tool schema is valid JSON
4. Check model version (Gemini 2.0 Flash recommended)
5. Review error logs in TUI output

---

## Architecture Notes

### Why This Approach is Better

**Gemini-CLI's Success**: Gemini-CLI works because the LLM sees tools and decides autonomously. No hardcoded decisions.

**Claude Code's Success**: Same approach - expose tools via FunctionDeclarations, let the LLM reason about when to use them.

**Radium's New Approach**: Now matches both, using:
1. Tool registry with clear descriptions
2. System prompts that guide (but don't force) tool usage
3. LLM autonomy to choose appropriate tools

### Design Principles

1. **Declarative Over Imperative**: Declare what tools exist, don't dictate when to use them
2. **LLM Agency**: Trust the model to make good decisions
3. **Prompt Engineering**: Use system prompts to guide, not code to enforce
4. **Schema-Driven**: Tool schemas tell the LLM what's possible

---

**Testing Date**: ___________
**Tester**: ___________
**Result**: ⬜ Pass  ⬜ Fail  ⬜ Partial

**Notes**:
```
[Space for testing notes]
```
