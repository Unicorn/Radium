# Event Streaming Architecture

## Overview

The RAD-Radium server supports real-time event streaming from orchestration engine to gRPC clients. This allows clients to receive live updates about tool execution, assistant messages, and approval requests during agent execution.

## Architecture

```
OrchestrationEngine
    ↓ emit(OrchestrationEvent)
broadcast::Sender<OrchestrationEvent>
    ↓ subscribe
EventBridge
    ↓ convert & route by session_id
mpsc::Sender<SessionEvent>
    ↓ stream
gRPC Client (session_events_stream)
```

## Components

### EventBridge

Located in `crates/radium-core/src/server/event_bridge.rs`

**Responsibilities**:
- Manage session-to-sender mappings
- Subscribe to orchestration event broadcast
- Convert `OrchestrationEvent` → `SessionEvent` (protobuf)
- Route events to appropriate sessions by correlation_id

**Key Methods**:
- `register_session(session_id, sender)` - Register a new session stream
- `unregister_session(session_id)` - Clean up when session ends
- `start_forwarding(event_rx)` - Begin listening to orchestration events

### RadiumService Integration

Located in `crates/radium-core/src/server/radium_service.rs`

**Fields** (feature-gated with `#[cfg(feature = "workflow")]`):
```rust
event_bridge: Arc<EventBridge>      // Routes events to sessions
event_tx: broadcast::Sender<OrchestrationEvent>  // Broadcasts events
```

**Initialization**:
1. Create broadcast channel with 1000-event buffer
2. Initialize EventBridge
3. Start event forwarding with receiver

**session_events_stream RPC**:
1. Generate unique session ID
2. Create mpsc channel for server-to-client streaming
3. Register session with EventBridge
4. Spawn cleanup task for session unregister
5. Return bidirectional stream

## Event Types

### Orchestration Events → Session Events

| OrchestrationEvent | SessionEvent | Description |
|-------------------|--------------|-------------|
| `ToolCallRequested` | `ToolCallEvent` | Agent requests tool execution |
| `ToolCallFinished` | `ToolResultEvent` | Tool execution completed |
| `ApprovalRequired` | `ApprovalRequestEvent` | Tool requires user approval |
| `AssistantMessage` | `MessageEvent` | Assistant response |
| `Error` | `MessageEvent` (system) | Error occurred |

### Event Fields

**ToolCallEvent**:
- `session_id` - Session identifier
- `tool_name` - Name of tool being called
- `arguments_json` - JSON-encoded arguments
- `call_id` - Unique call identifier
- `timestamp` - Event timestamp (ms since epoch)

**ToolResultEvent**:
- `session_id` - Session identifier
- `tool_name` - Name of tool executed
- `result_json` - JSON-encoded result
- `success` - Whether execution succeeded
- `error` - Error message (if failed)
- `timestamp` - Event timestamp (ms since epoch)

**ApprovalRequestEvent**:
- `session_id` - Session identifier
- `tool_name` - Tool requiring approval
- `arguments_json` - JSON-encoded arguments
- `policy_rule` - Which policy triggered approval
- `request_id` - Unique request identifier
- `timestamp` - Event timestamp (ms since epoch)

**MessageEvent**:
- `session_id` - Session identifier
- `message` - Message content
- `role` - Message role (assistant/system/user)
- `timestamp` - Event timestamp (ms since epoch)

## Usage

### Server-Side

**Building with event streaming**:
```bash
cargo build --features workflow
```

**Starting the server**:
```bash
cargo run --bin radium-core --features workflow
```

### Client-Side

**Connecting to event stream** (pseudo-code):
```rust
use radium::radium_client::RadiumClient;
use radium::SessionEvent;

async fn stream_events() {
    let mut client = RadiumClient::connect("http://localhost:50051").await?;

    // Create bidirectional stream
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    let outbound = ReceiverStream::new(rx);

    let mut response = client
        .session_events_stream(outbound)
        .await?
        .into_inner();

    // Receive events
    while let Some(event) = response.next().await {
        match event?.event {
            Some(Event::ToolCall(tool_call)) => {
                println!("Tool called: {}", tool_call.tool_name);
            }
            Some(Event::ToolResult(result)) => {
                println!("Tool result: {}", result.result_json);
            }
            Some(Event::ApprovalRequest(req)) => {
                println!("Approval required: {}", req.tool_name);

                // Send approval response
                let response = SessionEvent {
                    approval_response: Some(ApprovalResponseEvent {
                        session_id: req.session_id,
                        request_id: req.request_id,
                        approved: true,
                        reason: Some("User approved".to_string()),
                        timestamp: chrono::Utc::now().timestamp_millis(),
                    }),
                    ..Default::default()
                };
                tx.send(response).await?;
            }
            Some(Event::Message(msg)) => {
                println!("[{}] {}", msg.role, msg.message);
            }
            None => {}
        }
    }

    Ok(())
}
```

## Implementation Status

### ✅ Completed

1. **EventBridge Infrastructure** - Full implementation with tests
2. **Session Registration** - Automatic registration on stream connect
3. **Event Conversion** - OrchestrationEvent → SessionEvent mapping
4. **Feature Gating** - Proper `#[cfg(feature = "workflow")]` guards
5. **Bidirectional Streaming** - Client can send approval responses

### 🔄 In Progress

1. **Event Emission** - Need to emit events from agent execution
2. **End-to-End Testing** - Integration tests with real event flow

### 📋 TODO

1. **Connect OrchestrationService** - Use OrchestrationService for agent execution
2. **Event Emission Points** - Ensure events emitted at all key points:
   - Tool call requests
   - Tool execution start/finish
   - Assistant message generation
   - Approval requirements
   - Errors
3. **Integration Tests** - Test full event flow
4. **Performance Testing** - High-throughput scenarios
5. **Client Examples** - Reference implementations in multiple languages

## Configuration

### Broadcast Channel Buffer

Default: 1000 events

Modify in `radium_service.rs::new()`:
```rust
let (tx, rx) = broadcast::channel(1000); // Adjust buffer size
```

### Session Stream Buffer

Default: 100 events per session

Modify in `radium_service.rs::session_events_stream()`:
```rust
let (tx, rx) = mpsc::channel(100); // Adjust per-session buffer
```

### Session Timeout

Default: 3600 seconds (1 hour)

Modify in `radium_service.rs::session_events_stream()`:
```rust
tokio::time::Duration::from_secs(3600) // Adjust timeout
```

## Performance Considerations

### Broadcast Channel

- **Buffer Size**: Larger buffers prevent event loss under high load
- **Lagged Receivers**: Slow clients may lag and skip events
- **Memory Usage**: Buffer size × event size × num_subscribers

### Session Streams

- **Per-Session Buffers**: Each session has independent buffer
- **Backpressure**: Slow clients block their own stream, not others
- **Memory Usage**: buffer_size × event_size × num_active_sessions

### Recommendations

- **Production**: Use 5000-10000 event broadcast buffer
- **High Concurrency**: Implement event persistence for disconnected clients
- **Rate Limiting**: Add per-session rate limits if needed
- **Monitoring**: Track lagged events and buffer utilization

## Debugging

### Enable Debug Logging

```bash
RUST_LOG=radium_core::server::event_bridge=debug cargo run --features workflow
```

### Log Events

Key log messages:
- `"Registered session for event streaming"` - Session connected
- `"Unregistered session from event streaming"` - Session disconnected
- `"Event bridge lagged, N events skipped"` - Slow subscriber warning
- `"No active stream for session, event dropped"` - Orphaned event

### Common Issues

**Events not reaching client**:
1. Verify session registered: check for "Registered session" log
2. Check session_id matches in events
3. Verify events being emitted to event_tx
4. Check for "Event bridge lagged" warnings

**Session cleanup not happening**:
1. Cleanup task has 1-hour timeout by default
2. Implement proper graceful shutdown
3. Add explicit unregister on disconnect

## Security Considerations

### Session Isolation

- Events routed by session_id
- Sessions cannot receive events from other sessions
- correlation_id must match session_id

### Approval Responses

- Validate request_id matches pending approval
- Implement timeout for approval requests
- Log all approval decisions

### Resource Limits

- Set reasonable buffer sizes to prevent memory exhaustion
- Implement per-client connection limits
- Add rate limiting for event emission

## Future Enhancements

1. **Event Filtering** - Allow clients to subscribe to specific event types
2. **Event Persistence** - Store events for disconnected clients
3. **Replay Support** - Allow clients to replay missed events
4. **Compression** - Compress events for bandwidth efficiency
5. **Metrics** - Expose event streaming metrics (throughput, lag, etc.)

## References

- [EventBridge Implementation](event_bridge.rs)
- [RadiumService Implementation](radium_service.rs)
- [Protocol Definitions](../proto/radium.proto)
- [GitHub Issue #56](https://github.com/Unicorn/Radium/issues/56)
