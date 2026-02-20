# Testing Thinking & Recommendations Panels Integration

This document describes how to test the transparent thinking and interactive recommendations features in the Radium TUI.

## Prerequisites

1. **API Key configured**: You need either ANTHROPIC_API_KEY or GEMINI_API_KEY set
2. **Workspace initialized**: Run in a directory with `.radium/` folder
3. **Orchestration enabled**: The TUI should have orchestration mode enabled

## Setup

```bash
# Set API key (choose one)
export ANTHROPIC_API_KEY="your-key-here"
# OR
export GEMINI_API_KEY="your-key-here"

# Navigate to a Radium workspace
cd /path/to/your/project

# Run the TUI
cargo run --package radium-tui --release
```

## Test Scenarios

### Test 1: Basic Thinking Panel Display

**Objective**: Verify thinking panel shows agent reasoning in real-time

**Steps**:
1. Launch TUI and ensure orchestration is enabled
2. Type a simple query: `help check test coverage`
3. Press Enter

**Expected Behavior**:
- Bottom-left panel should appear titled "💭 Thinking"
- Should show thinking steps in real-time:
  ```
  Context: Processing request: help check test coverage

  1. ⠋ Analyzing request (iteration 1)     (0ms)
     → Found 2 tool(s): search_code, glob_file_search

  2. ⠋ Executing 2 tool(s)                 (0ms)
     → All 2 tool(s) executed successfully

  3. ✓ Analyzing request (iteration 2)     (150ms)
  ```
- Panel remains visible until response is complete
- Final status should show ✓ (completed) or ● (with findings)

**Keyboard Shortcuts**:
- Press `t` to toggle expand/collapse of thinking panel
- When collapsed, should show: "3 steps • Processing request..."

### Test 2: Multi-Iteration Thinking

**Objective**: Verify thinking panel handles multi-turn orchestration

**Steps**:
1. Ask a complex question requiring multiple tool calls:
   `Find all test files, analyze their coverage, and suggest improvements`
2. Observe thinking panel updates

**Expected Behavior**:
- Multiple iterations shown (iteration 1, 2, 3, etc.)
- Each iteration shows:
  - Analysis step (what tools to use)
  - Execution step (running the tools)
  - Results (success/failure counts)
- Steps accumulate in the panel (don't replace each other)
- Timestamps show elapsed time for each step

### Test 3: Error Handling

**Objective**: Verify thinking panel shows errors appropriately

**Steps**:
1. Ask the orchestrator to read a non-existent file:
   `read the file /path/to/nonexistent.txt and summarize it`
2. Observe error handling

**Expected Behavior**:
- Thinking step shows: "Executing 1 tool(s)"
- Updated with: "0 succeeded, 1 failed"
- Status indicator shows appropriate symbol (✗ or ●)
- Panel doesn't crash or hide on errors

### Test 4: Panel Visibility Toggle

**Objective**: Verify keyboard shortcuts work correctly

**Steps**:
1. Start orchestration with any query
2. While thinking panel is visible, press `t`
3. Press `t` again

**Expected Behavior**:
- First `t` press: Panel collapses to summary view
  - Shows: "3 steps • Processing request..."
- Second `t` press: Panel expands back to full view
- Thinking continues updating in background when collapsed

### Test 5: Thinking Panel + Chat History

**Objective**: Verify panels work alongside other UI elements

**Steps**:
1. Have some chat history in the conversation
2. Ask a new question
3. Observe panel positioning

**Expected Behavior**:
- Thinking panel appears in bottom-left quadrant
- Does not overlap with chat history
- Does not interfere with input prompt
- Toast notifications still appear on top

## Event Flow Verification

### Check Event Emission

To verify events are being emitted from the orchestration engine, enable debug logging:

```bash
RUST_LOG=radium_orchestrator=debug cargo run --package radium-tui
```

Look for log entries like:
```
DEBUG radium_orchestrator::orchestration::engine: Emitting ThinkingSessionStarted
DEBUG radium_orchestrator::orchestration::engine: Emitting ThinkingStepAdded
DEBUG radium_orchestrator::orchestration::engine: Emitting ThinkingStepUpdated
```

### Check Event Reception

To verify events are being received by the TUI, enable TUI logging:

```bash
RUST_LOG_TUI=debug cargo run --package radium-tui 2> tui.log
```

Then check `tui.log` for event processing.

## Current Limitations

### Known Gaps

1. **Recommendations Not Yet Generated**: The recommendations panel exists but isn't populated yet. This requires:
   - Analyzing task results
   - Generating actionable suggestions
   - (Planned for next phase)

2. **Recommendation Execution Not Wired**: The Y/N confirmation is displayed but doesn't execute commands yet
   - TODO: Wire up actual command execution (line 1187 in app.rs)
   - (Planned for next phase)

3. **No Persistence**: Thinking steps are lost when app restarts
   - Consider adding session history persistence in future

## Troubleshooting

### Thinking Panel Not Showing

**Symptom**: Orchestration runs but no thinking panel appears

**Possible Causes**:
1. Orchestration events not subscribed
   - Check: `app.orchestration_event_rx` should be Some
   - Fix: Ensure service initialization subscribes to events

2. Events not being emitted
   - Check: Enable debug logging to verify emission
   - Fix: Ensure `engine.set_event_sender()` was called

3. Panel visibility check failing
   - Check: `thinking_panel.is_visible()` returns true
   - Fix: Ensure `start_session()` was called

### Events Out of Order

**Symptom**: Thinking steps appear in wrong sequence

**Possible Causes**:
1. Broadcast channel lag
   - Check logs for "events lagged" warnings
   - Fix: Increase channel buffer size (currently 1024)

2. Concurrent event processing
   - This is expected - events may arrive slightly out of order
   - Fix: Add sequence numbers if ordering is critical

### Panel Position Issues

**Symptom**: Panel overlaps other UI elements

**Possible Causes**:
1. Screen size too small
   - Min recommended: 120x40 characters
   - Fix: Resize terminal or adjust panel size calculation

2. Layout calculation error
   - Check: Panel area calculation in main.rs (lines 747-752, 759-764)
   - Fix: Adjust Rect calculations for different screen sizes

## Success Criteria

The integration is working correctly if:

✅ Thinking panel appears when orchestration starts
✅ Steps are added in real-time as execution progresses
✅ Status symbols update correctly (⠋ → ✓/●/✗)
✅ Elapsed times are shown and increase
✅ Toggle with 't' key works (expand/collapse)
✅ Panel disappears when orchestration completes
✅ No crashes or UI glitches during execution
✅ Panel works alongside other TUI features (chat, toasts, etc.)

## Next Steps

After verifying thinking panel integration:

1. **Implement Recommendations Generation**
   - Analyze completed tasks
   - Generate actionable suggestions
   - Emit RecommendationAdded events

2. **Wire Up Command Execution**
   - Implement actual execution when user confirms (Y)
   - Handle errors and display results
   - Update recommendation status in real-time

3. **Add Polish**
   - Smooth animations for panel appearance
   - Better error messages
   - Persistence across sessions
   - Customizable panel positioning
