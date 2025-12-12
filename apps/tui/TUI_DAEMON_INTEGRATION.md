# TUI Daemon Integration Status

## Current State

TUI currently uses `OrchestrationService` from `radium-orchestrator` for session management. This service:
- Has its own session management (`ensure_session_initialized`, `handle_input`)
- Manages sessions in-memory with `SessionState`
- Handles event streaming via `subscribe_events()`
- Supports approval requests through `ApprovalRequired` events

## Daemon Session Management

The new daemon session management API (REQ-235) provides:
- `CreateSession`, `ListSessions`, `AttachSession` RPCs
- `SessionEventsStream` for bidirectional event streaming
- Persistent session storage across process restarts
- Multi-client session sharing

## Integration Path

To fully integrate TUI with daemon session management:

1. **Option A: Use daemon session RPCs directly**
   - Replace OrchestrationService session management with daemon RPCs
   - Connect to daemon via gRPC client
   - Use `SessionEventsStream` for event handling

2. **Option B: Enhance OrchestrationService**
   - Add daemon backend to OrchestrationService
   - Keep existing TUI interface, route to daemon internally
   - Maintain backward compatibility

## Current Status

✅ TUI's OrchestrationService integration works for local sessions
⚠️ Daemon session integration is a future enhancement
✅ Event streaming and approval handling already supported
✅ Session persistence exists at OrchestrationService level

## Next Steps

When implementing full daemon integration:
1. Add daemon connection configuration to TUI
2. Update OrchestrationService to support daemon backend
3. Add session list/attach UI in TUI
4. Connect event streams to daemon's SessionEventsStream
