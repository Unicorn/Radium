# Embedded Server Lifecycle Management

> **Status**: ✅ Implemented  
> **Last Updated**: 2025-01-XX

## Overview

Radium includes automatic server lifecycle management that embeds the `radium-core` gRPC server directly within client applications. This allows clients to automatically start and stop the server without requiring users to manually manage a separate server process.

## Architecture

The server is embedded as a library within each application:

- **Desktop App**: Server spawns in background task on app startup
- **CLI/TUI**: Server spawns in background when commands need it, auto-cleanup on exit
- **Standalone Server**: Still available as separate binary for advanced users

## Components

### EmbeddedServer Manager

Located in `crates/radium-core/src/server/manager.rs`, the `EmbeddedServer` struct provides:

- `start()` - Start server in background tokio task
- `wait_for_ready()` - Poll until server is accepting connections (uses gRPC Ping endpoint)
- `shutdown()` - Gracefully stop server
- `is_running()` - Check if server task is alive
- `address()` - Get the server's socket address

### ClientHelper

Located in `crates/radium-core/src/client.rs`, the `ClientHelper` provides:

- Automatic server lifecycle management
- gRPC client connection creation
- Environment variable support for disabling embedded server
- Auto-cleanup on drop (for CLI commands)

### Server Shutdown Support

The `server::run_with_shutdown()` function accepts a shutdown signal channel and gracefully stops the server when requested.

## Usage

### Desktop Application

The desktop app automatically starts the server on launch:

```rust
// In Tauri setup hook
let mut server = EmbeddedServer::new(config);
server.start().await?;
server.wait_for_ready(Duration::from_secs(10)).await?;
```

The server is stored in `AppState` and will automatically cleanup when the app exits.

### CLI Commands

For commands that need server access, use `ClientHelper`:

```rust
let mut helper = ClientHelper::new();
let client = helper.connect().await?;
// Use client for gRPC calls
// Server auto-cleanup on drop
```

### Environment Configuration

Disable embedded server to use an external server:

```bash
export RADIUM_DISABLE_EMBEDDED_SERVER=true
```

This is useful for:
- Development with a standalone server
- Production deployments with centralized server
- Testing against remote servers

## Configuration

The embedded server uses the default `Config` settings:

- Default address: `127.0.0.1:50051`
- gRPC-Web: Enabled by default
- Custom configuration via `Config::load()`

## Error Handling

The system includes comprehensive error handling:

- **Port conflicts**: Clear error messages with helpful suggestions
- **Startup failures**: Detailed error reporting
- **Connection timeouts**: Configurable timeout with retry logic
- **Graceful shutdown**: Timeout protection to prevent hanging

## Testing

Unit tests cover:

- Server lifecycle methods
- Health checking via Ping endpoint
- Graceful shutdown scenarios
- Port conflict detection
- Multiple server instances

Integration tests verify:

- Server startup and readiness
- Client connection establishment
- Graceful shutdown procedures

## Implementation Details

### Server Startup

1. Server task spawned in background
2. Binding to configured address
3. Health check via Ping endpoint
4. Ready signal when accepting connections

### Graceful Shutdown

1. Shutdown signal sent via oneshot channel
2. Server stops accepting new connections
3. Existing connections finish gracefully
4. Server task completes within timeout

### Health Checking

The system uses the gRPC `Ping` endpoint to verify server readiness:

```rust
let mut client = RadiumClient::new(channel);
let request = Request::new(PingRequest {
    message: "health-check".to_string(),
});
client.ping(request).await?;
```

## Future Enhancements

Potential improvements:

- Port auto-selection when default port is in use
- Server instance sharing between multiple CLI commands
- Health check retry with exponential backoff
- Server status monitoring dashboard
- Metrics collection for server performance

## Related Documentation

- [Backend Architecture](../architecture/architecture-backend.md)
- [Server Module Documentation](../../crates/radium-core/src/server/mod.rs)
- [Client Helper Documentation](../../crates/radium-core/src/client.rs)

