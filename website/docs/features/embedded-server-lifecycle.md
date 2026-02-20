---
id: "embedded-server-lifecycle"
title: "Embedded Server Lifecycle"
sidebar_label: "Embedded Server Lifecycle"
---

# Embedded Server Lifecycle

Radium includes automatic server lifecycle management that embeds the gRPC server within client applications, eliminating the need for manual server management.

## Overview

The embedded server automatically starts and stops as needed, providing a seamless experience across all Radium interfaces:

- **Desktop App**: Server automatically starts when the app launches
- **CLI/TUI**: Server starts on-demand when commands require it
- **Standalone**: Still available as a separate binary for advanced use cases

## How It Works

### Automatic Lifecycle Management

The embedded server manager (`EmbeddedServer`) handles:

1. **Automatic Startup**: Server starts in a background task when needed
2. **Readiness Detection**: Waits for server to be ready before accepting connections
3. **Graceful Shutdown**: Properly shuts down when the application exits
4. **Health Monitoring**: Monitors server health and handles failures

### Server Configuration

The embedded server uses the same configuration as the standalone server:

```toml
[server]
address = "127.0.0.1:50051"
grpc_web_enabled = true
grpc_web_address = "127.0.0.1:50052"
```

### Client Integration

Clients automatically connect to the embedded server:

- **Desktop App**: Connects to embedded server on startup
- **CLI/TUI**: Server starts automatically when first command is executed
- **External Clients**: Can connect to embedded server if address is known

## Benefits

### Simplified Deployment

- No separate server process to manage
- No manual server startup required
- Automatic resource cleanup on exit

### Better User Experience

- Faster startup times (server starts in background)
- Seamless integration with client applications
- No configuration needed for basic usage

### Development Flexibility

- Can still run standalone server for testing
- Embedded server can be disabled if needed
- Same configuration works for both modes

## Architecture

The embedded server runs in a separate Tokio task:

```rust
let mut server = EmbeddedServer::new(config);
server.start().await?;
server.wait_for_ready(timeout).await?;
// Server is now ready to accept connections
```

### Lifecycle States

1. **Initialized**: Server created but not started
2. **Starting**: Server task spawned, binding to address
3. **Ready**: Server accepting connections
4. **Shutting Down**: Graceful shutdown in progress
5. **Stopped**: Server task completed

## Standalone Mode

For advanced use cases, you can still run the server standalone:

```bash
# Run standalone server
cargo run --bin radium-core

# Or via npm
npm run server
```

The standalone server provides:
- Manual control over server lifecycle
- Easier debugging and monitoring
- Integration with external process managers

## Troubleshooting

### Server Fails to Start

If the embedded server fails to start:

1. Check if the port is already in use
2. Verify server configuration
3. Check application logs for errors

### Server Not Ready

If the server doesn't become ready:

1. Increase the readiness timeout
2. Check network configuration
3. Verify server logs for binding errors

### Connection Issues

If clients can't connect:

1. Verify server address configuration
2. Check firewall settings
3. Ensure server is in "Ready" state

## Related Documentation

- [Architecture Overview](../developer-guide/architecture/overview.md) - System architecture
- [Configuration Guide](../getting-started/configuration.md) - Server configuration
- [CLI Reference](../cli/README.md) - CLI usage and commands

