# Phase 1 Testing - CLI Chat with Tools & Context

**Test Date**: 2025-12-11
**Implementation**: Natural Conversation & Task Execution Enhancement (Phase 1)

## Test Environment Setup

### Prerequisites
1. Build the latest CLI:
   ```bash
   cargo build --release -p radium-cli
   ```

2. Ensure you have an API key set:
   ```bash
   # For Gemini (default)
   export GEMINI_API_KEY=your_api_key_here

   # Or for Claude
   export ANTHROPIC_API_KEY=your_api_key_here
   ```

3. Navigate to a test directory with some files to explore

## Test Suite

### Test 1: Basic Tool Calling ✅

**Objective**: Verify that the chat mode can call tools autonomously

**Steps**:
```bash
./target/release/radium-cli chat chat-gemini
```

**Test Prompts**:
1. `What files are in the apps/cli/src/commands directory?`
   - **Expected**: Model calls `list_dir` tool, shows file list
   - **Verify**: Progress spinner appears during execution
   - **Verify**: Tool result displays with byte count

2. `Read the apps/cli/src/main.rs file`
   - **Expected**: Model calls `read_file` tool
   - **Verify**: File contents are displayed
   - **Verify**: Spinner shows "Executing read_file..."

3. `What git commits mention "context"?`
   - **Expected**: Model calls `git_log` or `run_terminal_cmd` with git
   - **Verify**: Git history is retrieved and displayed

**Success Criteria**:
- [ ] Tools execute without errors
- [ ] Progress spinners display during execution
- [ ] Results are displayed correctly
- [ ] No crashes or hangs

---

### Test 2: Multi-Turn Conversation ✅

**Objective**: Verify context is maintained across multiple turns

**Steps**:
```bash
./target/release/radium-cli chat chat-gemini
```

**Test Conversation**:
```
You: List the files in apps/cli/src/commands
Assistant: [calls list_dir, shows files]

You: Now read the chat.rs file from that directory
Assistant: [should understand context, calls read_file on apps/cli/src/commands/chat.rs]

You: What function handles tool execution in that file?
Assistant: [should reference the previously read file, mentions execute_chat_turn_with_tools]

You: Show me the git history for that file
Assistant: [calls git_log or git_blame for apps/cli/src/commands/chat.rs]
```

**Success Criteria**:
- [ ] Assistant maintains context across turns
- [ ] File references from previous turns are understood
- [ ] No need to repeat full paths
- [ ] Conversation flows naturally

---

### Test 3: Conversation Context Tracking ✅

**Objective**: Verify that topics, decisions, and tasks are tracked

**Setup**: Add debug output to see context (optional)

**Test Prompts**:
1. `I'm working on testing the Rust CLI application`
   - **Expected**: Topic "rust" and "testing" detected

2. `Actually, let's focus on the database module instead`
   - **Expected**: Decision tracked ("changed approach")
   - **Expected**: Topic "database" added

3. `We need to add error handling to the chat module`
   - **Expected**: Task identified and tracked
   - **Expected**: Intent classified as "Implementation"

4. `Why does the tool execution fail sometimes?`
   - **Expected**: Intent classified as "Analysis"

**Verification**:
While you can't directly see the context tracking without adding debug prints, you can verify it's working by observing that:
- The assistant's responses become more context-aware over time
- Topics mentioned earlier influence later responses
- The assistant remembers decisions you made

**Success Criteria**:
- [ ] Assistant shows awareness of conversation history
- [ ] Topic changes are handled smoothly
- [ ] Task-related prompts are understood
- [ ] Intent affects response style (exploratory vs analytical)

---

### Test 4: Multi-Tool Operations ✅

**Objective**: Verify the model can chain multiple tool calls

**Test Prompts**:
```
You: Find all Rust files in apps/cli/src and tell me how many there are

Expected behavior:
1. Model calls glob_file_search or list_dir
2. Model counts the files
3. Model may call read_file on some to verify
4. Model returns count
```

```
You: Show me the most recent git commit and what files it changed

Expected behavior:
1. Model calls git_log to get latest commit
2. Model calls git_show to see changes
3. Model summarizes the commit
```

**Success Criteria**:
- [ ] Multiple tools can be called in sequence
- [ ] Tool results feed into subsequent calls
- [ ] Progress indicators show for each tool
- [ ] Final answer synthesizes all tool results

---

### Test 5: Error Handling ✅

**Objective**: Verify graceful error handling

**Test Prompts**:
1. `Read the file /nonexistent/path/file.txt`
   - **Expected**: Tool returns error, assistant explains file not found
   - **Expected**: No crash, conversation continues

2. `Run the command "invalid_command_xyz"`
   - **Expected**: Terminal command fails gracefully
   - **Expected**: Error is reported to user

3. Ask for 15 consecutive tool operations (to test iteration limit)
   - **Expected**: Hits MAX_ITERATIONS (10) and returns error message
   - **Expected**: Clear error about exceeding max iterations

**Success Criteria**:
- [ ] Errors don't crash the chat session
- [ ] Error messages are user-friendly
- [ ] Can continue conversation after errors
- [ ] Iteration limit prevents infinite loops

---

### Test 6: Progress Indicators ✅

**Objective**: Verify visual feedback during operations

**Test Prompts**:
1. `Read a large file` (e.g., Cargo.lock or a large source file)
   - **Expected**: Spinner appears and animates
   - **Expected**: Success message shows file size

2. `Search for "async" in all Rust files`
   - **Expected**: Spinner shows "Executing glob_file_search..." or "Executing run_terminal_cmd..."
   - **Expected**: Progress visible during potentially long operation

**Visual Checks**:
- [ ] Spinner animates smoothly (⠁⠂⠄⡀⢀⠠⠐⠈)
- [ ] Tool name is displayed in cyan color
- [ ] Success checkmark (✓) appears when complete
- [ ] Byte count or result size is shown
- [ ] Multi-tool scenarios show iteration count

---

### Test 7: Session Persistence ✅

**Objective**: Verify conversation history is saved and can be resumed

**Steps**:
```bash
# Start a new session
./target/release/radium-cli chat chat-gemini

You: List files in apps/cli
You: What's in main.rs?
You: /quit

# Resume the session
./target/release/radium-cli chat chat-gemini --session chat-gemini_20XX... --resume

You: /history
```

**Success Criteria**:
- [ ] Previous interactions are visible with `/history`
- [ ] Can continue conversation from where it left off
- [ ] Session saves automatically
- [ ] `/save` command works

---

## Performance Benchmarks

### Expected Performance
- **Tool Execution**: < 2 seconds for file operations
- **Model Response**: 2-10 seconds depending on complexity
- **Spinner Responsiveness**: Updates at 100ms intervals
- **Memory Usage**: < 100MB for CLI chat session

### Measure
```bash
# Monitor during a chat session
top -pid $(pgrep -f "radium-cli chat")
```

---

## Known Limitations (Phase 1)

These are expected and will be addressed in Phase 2-3:

1. **No Safety Confirmations**: Dangerous operations (write_file, git_push) execute without confirmation
2. **No Session Management UI**: Must use command-line flags to resume
3. **No Agent Switching**: Locked to one agent per session
4. **Simple Context Tracking**: Pattern-matching only (no LLM-powered semantic analysis)
5. **No Rich Formatting**: Plain text output (tables/trees in Phase 3)

---

## Test Results Summary

| Test | Status | Notes |
|------|--------|-------|
| Basic Tool Calling | ⬜ | |
| Multi-Turn Conversation | ⬜ | |
| Context Tracking | ⬜ | |
| Multi-Tool Operations | ⬜ | |
| Error Handling | ⬜ | |
| Progress Indicators | ⬜ | |
| Session Persistence | ⬜ | |

**Overall Phase 1 Status**: ⬜ PENDING

---

## Troubleshooting

### Issue: Tools not being called
**Solution**:
- Verify agent has correct engine/model configured
- Check API key is set correctly
- Look for error messages in output

### Issue: Spinner not visible
**Solution**:
- Ensure terminal supports Unicode characters
- Check that indicatif is properly installed (`grep indicatif apps/cli/Cargo.toml`)

### Issue: Context not maintained
**Solution**:
- This is subtle - look for conversation awareness
- Try more explicit context references
- Check that ConversationContext is being updated (add debug prints if needed)

### Issue: Session won't resume
**Solution**:
- Check `.radium/_internals/history` directory exists
- Verify session name matches exactly
- Try `/history` in a new session to see available sessions

---

## Next Steps After Testing

Once Phase 1 tests pass:

1. **Document Issues**: Note any bugs or unexpected behavior
2. **Performance Tuning**: If slow, identify bottlenecks
3. **Phase 2 Planning**: Review policy engine and session management requirements
4. **User Feedback**: Gather real-world usage feedback

---

## Quick Test Script

For rapid verification, run this sequence:

```bash
./target/release/radium-cli chat chat-gemini << 'EOF'
List files in apps/cli/src
Read apps/cli/src/main.rs
What functions are exported?
/history
/quit
EOF
```

**Expected**: Should execute all commands, call tools autonomously, maintain context, show history, and exit cleanly.

---

**Testing Complete**: Mark this document with test results and timestamp when finished.
