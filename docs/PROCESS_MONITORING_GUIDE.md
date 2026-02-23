# Process Monitoring in Watch Mode - User Guide

This guide explains how to use the new process monitoring, error detection, and automated fix proposal features in Radium TUI.

## Overview

REQ-244 introduces automated process monitoring with intelligent error detection:

1. **Spawn processes** in watch mode with centralized tracking
2. **Capture logs** via stdout/stderr and file monitoring
3. **Display processes** in a dedicated TUI panel
4. **Classify errors** using ML heuristics for severity
5. **Delegate to agents** for automated fix proposals
6. **Approve fixes** before application
7. **Verify fixes** by monitoring for error resolution

## Quick Start

### 1. Build and Run

```bash
# Build the release binary
cargo build --release

# Run the TUI
./dist/target/release/radium-tui
```

### 2. Enable Process Panel

Press **`Ctrl+Shift+P`** to toggle the process monitoring panel.

The panel will replace the orchestrator thinking panel when visible.

### 3. Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+P` | Toggle process panel visibility |
| `Ctrl+N` | Spawn new process (when panel visible) |
| `Ctrl+A` | Show pending fix approvals |
| `Tab` | Cycle focus between panels |
| `↑/↓` | Navigate process list |
| `Enter` | View process details (when focused) |

## Features

### Process Panel

The process panel displays all spawned processes with:

- **Status Icon**:
  - `●` (Green) - Running
  - `○` (Gray) - Stopped
  - `✗` (Red) - Crashed
  - `⟳` (Yellow) - Restarting

- **Process Info**:
  - PID (Process ID)
  - Command
  - Uptime
  - Restart count
  - Status details (exit codes, errors)

### Error Classification

When errors are detected in process logs, they are automatically classified:

**Severity Levels**:
- **Critical**: Panics, fatal errors, segmentation faults
- **High**: Uncaught exceptions, compilation failures (with exit code)
- **Medium**: Regular errors, warnings
- **Low**: Minor issues, deprecation warnings

**Classification Factors**:
- Keywords (`panic`, `fatal`, `error`, `warn`)
- Exit codes (non-zero indicates failure)
- Stack traces (increase severity)
- Frequency (repeated errors escalate)

### Agent Delegation

Based on error type, the system suggests specialized agents:

| Error Pattern | Suggested Agent |
|--------------|-----------------|
| `error[E...]` (Rust) | `rust-expert-agent` |
| `TypeError`, `SyntaxError` | `typescript-agent` |
| Build failures | `build-agent` |
| Generic errors | `code-agent` |

### Fix Approval Modal

When a high-severity error is detected, a fix proposal modal appears:

**Modal Contents**:
- Error summary
- Proposed fix with syntax highlighting
- Confidence score (0.0 - 1.0)
- Root cause analysis
- Expected impact
- Rollback strategy

**Actions**:
- `Enter` - Approve/Reject/View Details
- `↑/↓` - Navigate options
- `d` - Toggle detailed view
- `Esc` - Cancel/Close

**Border Color Indicators**:
- Green: High confidence (≥ 0.8)
- Yellow: Medium confidence (0.6 - 0.8)
- Red: Low confidence (< 0.6)

## Example Workflow

### Spawning a Process

```bash
# 1. Toggle process panel
Press: Ctrl+Shift+P

# 2. Spawn a development server
Press: Ctrl+N
Enter command: npm run dev

# The process will appear in the panel with status: Running
```

### Error Detection Flow

```
Process error occurs
    ↓
Log captured (stdout/stderr)
    ↓
Error classified (severity: High)
    ↓
Agent delegated (typescript-agent)
    ↓
Fix proposal generated
    ↓
Modal appears - awaiting approval
    ↓
User approves fix
    ↓
Fix applied to codebase
    ↓
Process monitored for verification
    ↓
Success: Error resolved ✓
```

### Handling Fix Proposals

1. **Review the proposal**: Read the error, proposed fix, and reasoning
2. **Check confidence**: Higher confidence = more reliable fix
3. **Approve or Reject**:
   - If confident: Select "Approve" and press `Enter`
   - If unsure: Select "View Details" to see more information
   - If incorrect: Select "Reject" and optionally provide reason
4. **Monitor verification**: After approval, the system monitors logs to confirm the fix worked

## Configuration

Add to your `.radiumrc` or `config.toml`:

```toml
[process_watch]
enabled = true
auto_restart = true
max_restarts = 3
restart_cooldown = 5  # seconds

[error_classification]
min_severity = "high"  # Only route High+ errors to agents
auto_approve_threshold = 0.0  # 0 = always ask user (recommended)
error_debounce = 30  # seconds between error notifications

[fix_approval]
require_approval = true  # Never auto-apply fixes
min_confidence = 0.5  # Minimum confidence to show proposal
approval_timeout = 300  # 5 minutes to approve/reject
```

## Testing Scenarios

### Test 1: Rust Compilation Error

```bash
# 1. Spawn cargo watch
Ctrl+N → cargo watch -x check

# 2. Introduce a type error in your code
# 3. Observe error classification (should be High severity)
# 4. Review fix proposal from rust-expert-agent
# 5. Approve and verify fix
```

### Test 2: TypeScript Runtime Error

```bash
# 1. Spawn development server
Ctrl+N → npm run dev

# 2. Introduce a TypeError
# 3. Observe error classification and agent delegation
# 4. Review and approve fix
```

### Test 3: Process Crash and Restart

```bash
# 1. Spawn a process that will crash
Ctrl+N → node script-that-crashes.js

# 2. Observe status change: Running → Crashed
# 3. System automatically restarts (if auto_restart = true)
# 4. Status changes to: Restarting → Running
```

## Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────┐
│                  TUI (radium-tui)                │
│  ┌──────────────────┐  ┌──────────────────────┐ │
│  │  ProcessPanel    │  │ FixApprovalModal     │ │
│  │  - Status view   │  │ - Fix display        │ │
│  │  - Process list  │  │ - Approval UI        │ │
│  └──────────────────┘  └──────────────────────┘ │
└─────────────────────────────────────────────────┘
                    │
                    ↓
┌─────────────────────────────────────────────────┐
│            Orchestrator (radium-orchestrator)    │
│  ┌──────────────────────────────────────────┐   │
│  │  ErrorRouter                              │   │
│  │  - Route errors to agents                 │   │
│  │  - Manage fix proposals                   │   │
│  │  - Apply approved fixes                   │   │
│  │  - Verify fix success                     │   │
│  └──────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
                    │
                    ↓
┌─────────────────────────────────────────────────┐
│              Core (radium-core)                  │
│  ┌──────────────────┐  ┌──────────────────────┐ │
│  │ ProcessRegistry  │  │ ErrorClassifier      │ │
│  │ - Spawn/kill     │  │ - Severity scoring   │ │
│  │ - Track PIDs     │  │ - Agent suggestion   │ │
│  │ - Capture logs   │  │ - Frequency tracking │ │
│  └──────────────────┘  └──────────────────────┘ │
│  ┌──────────────────┐  ┌──────────────────────┐ │
│  │ LogWatcher       │  │ MonitoringService    │ │
│  │ - File monitor   │  │ - Telemetry          │ │
│  │ - Stream logs    │  │ - Persistence        │ │
│  └──────────────────┘  └──────────────────────┘ │
└─────────────────────────────────────────────────┘
```

### Data Flow

```
Process spawns
    ↓
Logs captured (stdout/stderr + file watch)
    ↓
Lines streamed to ErrorClassifier
    ↓
High severity detected → ErrorRouter
    ↓
Agent task spawned with error context
    ↓
Agent generates fix proposal
    ↓
Proposal queued for approval
    ↓
User approves via FixApprovalModal
    ↓
ErrorRouter applies fix
    ↓
LogWatcher monitors for error resolution
    ↓
Verification complete
```

## Troubleshooting

### Process not appearing in panel

- Ensure process panel is visible (`Ctrl+Shift+P`)
- Check that ProcessRegistry initialized correctly
- Verify working directory permissions

### Errors not being detected

- Check `min_severity` configuration (may be filtering errors)
- Verify error appears in process logs
- Check error classification rules in ErrorClassifier

### Fix proposals not appearing

- Ensure `require_approval = true` in config
- Check that error severity is High or above
- Verify `min_confidence` threshold allows proposal

### Fixes failing to apply

- Review error logs for permission issues
- Check that proposed fix is valid syntax
- Verify working directory is correct

## Development

### Running Tests

```bash
# Unit tests
cargo test --package radium-core process::
cargo test --package radium-core monitoring::error_classifier
cargo test --package radium-orchestrator error_router

# Integration tests
cargo test --package radium-orchestrator --test process_monitoring_integration_test

# All tests
cargo test --workspace
```

### Adding Custom Error Patterns

Edit `crates/radium-core/src/monitoring/error_classifier.rs`:

```rust
fn classify_error_type(&self, log_line: &str) -> (ErrorType, String) {
    // Add your custom pattern
    if log_line.contains("MyCustomError") {
        return (ErrorType::Custom, "custom-agent".to_string());
    }
    // ... existing patterns
}
```

## Future Enhancements

Planned improvements (not yet implemented):

1. **ML-Based Classification**: Replace heuristics with lightweight BERT model
2. **Fix History**: Track fix success rates and learn from past fixes
3. **Auto-Approval**: Allow high-confidence fixes to auto-apply
4. **Process Templates**: Pre-configured spawn commands
5. **Log Search**: Full-text search and filtering in process logs
6. **Multi-Process Graphs**: Visual dependency graphs
7. **Resource Monitoring**: CPU/memory usage per process
8. **Custom Hooks**: User-defined scripts on process events

## Support

For issues or questions:
- GitHub: https://github.com/anthropics/radium/issues
- Documentation: `/docs/process-monitoring.md`
- Examples: `/examples/watch-mode/`

---

**Version**: 0.1.0 (REQ-244)
**Last Updated**: 2025-12-30
