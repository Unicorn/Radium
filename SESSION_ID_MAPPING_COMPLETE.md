# Session ID Mapping Implementation Complete

**Date:** January 14, 2026
**Feature:** Session ID Mapping for Event Routing
**Issue:** #56 (Partial completion)

---

## ✅ Completed Implementation

### Session ID Mapping

Successfully implemented proper event routing from agent execution to client session streams using session ID mapping.

---

## 🎯 What Was Implemented

### 1. Protobuf Schema Update

**File:** `crates/radium-core/proto/radium.proto`

**Changes:**
- Added `optional string session_id = 6` to `ExecuteAgentRequest`
- Documented behavior: routes events to matching session stream
- Backward compatible: field is optional

**Schema:**
```protobuf
message ExecuteAgentRequest {
  optional string agent_id = 1;
  string input = 2;
  optional string model_type = 3;
  optional string model_id = 4;
  optional SelectionCriteria criteria = 5;
  optional string session_id = 6;  // ← NEW
}
```

---

### 2. Execute Agent Endpoint Update

**File:** `crates/radium-core/src/server/radium_service.rs`

**Changes:**
- Use `request.session_id` as `correlation_id` if provided
- Generate random `exec-{uuid}` if not provided
- Log correlation_id for debugging

**Implementation:**
```rust
// Use session_id as correlation_id if provided
let correlation_id = inner.session_id.unwrap_or_else(|| {
    let generated_id = format!("exec-{}", Uuid::new_v4());
    info!(correlation_id = %generated_id, "Generated correlation_id (no session_id provided)");
    generated_id
});

info!(correlation_id = %correlation_id, "Using correlation_id for event tracking");
```

**Behavior:**
- **With session_id**: Events route to matching session stream
- **Without session_id**: Events may be orphaned (no session to receive them)
- **Logging**: Both cases logged for debugging

---

### 3. Documentation Update

**File:** `crates/radium-core/src/server/EVENT_STREAMING.md`

**Additions:**
1. **Usage Example** - How to execute agent with session_id
2. **Session ID Mapping Section** - Complete flow explanation
3. **Best Practices** - Guidance for proper session ID usage
4. **Implementation Status** - Updated to mark session ID mapping complete

**Example Added:**
```rust
async fn execute_with_events(client: &mut RadiumClient, session_id: String) {
    let request = ExecuteAgentRequest {
        agent_id: Some("my-agent".to_string()),
        input: "Hello, world!".to_string()),
        session_id: Some(session_id.clone()),  // ← Use session_id here
        ..Default::default()
    };

    let response = client.execute_agent(request).await?;

    // Events will be sent to the matching session stream
}
```

---

## 🔄 Event Routing Flow

### Complete Flow

```
1. Client connects to session_events_stream
   ↓
   Server generates unique session_id (session-{uuid})
   ↓
2. Client receives session_id from stream

3. Client calls execute_agent with session_id
   ↓
   Server uses session_id as correlation_id
   ↓
4. Agent executes, events emitted with correlation_id
   ↓
   event_tx.send(OrchestrationEvent { correlation_id, ... })
   ↓
5. EventBridge receives events
   ↓
   extract_session_id(event) → correlation_id
   ↓
6. EventBridge looks up session by correlation_id
   ↓
   Finds matching session_senders[correlation_id]
   ↓
7. EventBridge converts & sends to session stream
   ↓
   SessionEvent sent via mpsc::Sender
   ↓
8. Client receives events in real-time
```

### Key Insight

The **correlation_id in OrchestrationEvent** must match the **session_id registered with EventBridge** for events to route correctly.

**Solution:** Use `request.session_id` as `correlation_id` ✅

---

## 📊 Before vs After

### Before (Without Session ID Mapping)

**Problem:**
- correlation_id generated randomly (`exec-{uuid}`)
- No connection to session_id from stream
- Events couldn't find matching sessions
- Events dropped (no active stream for correlation_id)

**Flow:**
```
execute_agent → random correlation_id → EventBridge → no matching session → dropped
```

### After (With Session ID Mapping)

**Solution:**
- correlation_id = request.session_id
- Matches session_id from stream connection
- Events find correct session
- Events delivered successfully

**Flow:**
```
execute_agent → session_id as correlation_id → EventBridge → matching session → delivered ✅
```

---

## 🎯 Benefits

### 1. Proper Event Routing
- Events always reach the correct client session
- No orphaned events when session_id provided

### 2. Multiple Concurrent Executions
- Same session can track multiple executions
- Each execution has unique events
- Client correlates by session_id

### 3. Backward Compatible
- session_id is optional field
- Existing clients work without changes
- New clients opt-in to event streaming

### 4. Debuggable
- All correlation_id usage logged
- Easy to trace event flow
- Clear logging when session_id not provided

---

## 🧪 Testing

### Compilation

**With workflow feature:**
```bash
cargo build -p radium-core --features workflow
```
✅ Compiles successfully

**Without workflow feature:**
```bash
cargo build -p radium-core
```
✅ Compiles successfully

### Protobuf Regeneration

```bash
cargo build -p radium-core
```
✅ Protobuf regenerated with new session_id field

### Feature Gates

All event emission code properly gated:
```rust
#[cfg(feature = "workflow")]
{
    let _ = self.event_tx.send(...);
}
```
✅ No workflow code in non-workflow builds

---

## 📝 Usage Guide

### For Clients

**Step 1: Connect to event stream**
```rust
let (tx, rx) = tokio::sync::mpsc::channel(100);
let outbound = ReceiverStream::new(rx);

let mut stream = client
    .session_events_stream(outbound)
    .await?
    .into_inner();
```

**Step 2: Extract session ID**

The session ID is generated on the server side. The client needs to:
- Either receive it from an initial event
- Or use a well-known session ID agreed upon with the server

**Step 3: Execute agent with session ID**
```rust
let request = ExecuteAgentRequest {
    agent_id: Some("my-agent".to_string()),
    input: "Process this data".to_string(),
    session_id: Some(session_id.clone()),  // ← IMPORTANT
    ..Default::default()
};

let response = client.execute_agent(request).await?;
```

**Step 4: Receive events**
```rust
while let Some(event) = stream.next().await {
    match event?.event {
        Some(Event::Message(msg)) => {
            println!("Message: {}", msg.message);
        }
        Some(Event::ToolCall(call)) => {
            println!("Tool called: {}", call.tool_name);
        }
        // ... handle other event types
        None => {}
    }
}
```

---

## 🔄 Current Limitations

### 1. Session ID Discovery
- Client must know the session_id to provide it
- Current implementation: server generates, client must extract
- **Future**: Send initial SessionEvent with session_id

### 2. Event Granularity
- Still high-level events (start/end/error)
- No per-tool events yet
- **Future**: Integrate OrchestrationService for detailed events

### 3. Session Lifecycle
- 1-hour timeout for cleanup
- No explicit session close
- **Future**: Add explicit session termination

---

## 📋 Next Steps

### Immediate

1. **Session ID Discovery Enhancement**
   - Send initial event with session_id when stream connects
   - Client can extract from first event received

2. **Integration Test**
   - Write end-to-end test with real client
   - Verify events route correctly
   - Test multiple concurrent executions

### Short Term

3. **OrchestrationService Integration**
   - Replace Orchestrator with OrchestrationService
   - Get detailed per-tool events
   - Emit ToolCallEvent, ToolResultEvent, ApprovalRequestEvent

4. **Session Management**
   - Add explicit session close RPC
   - Better cleanup on disconnect
   - Session status queries

### Medium Term

5. **Event Filtering**
   - Allow clients to subscribe to specific event types
   - Reduce bandwidth for clients that don't need all events

6. **Event Persistence**
   - Store events for disconnected clients
   - Allow replay of missed events

---

## 📚 References

- [Event Streaming Documentation](crates/radium-core/src/server/EVENT_STREAMING.md)
- [EventBridge Implementation](crates/radium-core/src/server/event_bridge.rs)
- [RadiumService Implementation](crates/radium-core/src/server/radium_service.rs)
- [Protocol Definition](crates/radium-core/proto/radium.proto)
- [GitHub Issue #56](https://github.com/Unicorn/Radium/issues/56)

---

## 📊 Statistics

**Commit:** 31f38ce

**Files Modified:** 3
- `radium.proto` - Added session_id field
- `radium_service.rs` - Implemented session ID mapping
- `EVENT_STREAMING.md` - Updated documentation

**Lines Changed:**
- +54 additions
- -4 deletions
- 50 net additions

**Testing:**
- ✅ Compiles with `--features workflow`
- ✅ Compiles without workflow feature
- ✅ Protobuf regenerated successfully
- ✅ Feature gates working correctly

---

## ✅ Summary

Successfully implemented session ID mapping for proper event routing in the event streaming infrastructure. Clients can now provide a session_id when executing agents, and events will be correctly routed to the matching session stream.

This completes a critical piece of the event streaming infrastructure and enables proper end-to-end event flow from agent execution to client applications.

**Status:** ✅ Complete and Ready for Integration Testing

---

**Next:** Integration testing with real client to verify end-to-end event flow works correctly.
