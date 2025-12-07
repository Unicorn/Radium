# Sandboxing

Radium provides a comprehensive sandboxing system for safe agent execution. Sandboxing isolates agent commands and file operations to prevent unauthorized access to system resources and protect against malicious code execution.

## Quick Start

```bash
# Initialize workspace with Docker sandbox
rad init --sandbox docker

# Or set sandbox configuration later
rad sandbox set docker --network closed

# Test sandbox availability
rad sandbox test docker
```

## Overview

The sandboxing system enables safe execution of shell commands and file operations by agents. It supports multiple sandbox types, each with different isolation mechanisms:

- **NoSandbox**: Direct execution without isolation (default, for trusted environments)
- **Docker**: Container-based sandboxing using Docker
- **Podman**: Container-based sandboxing using Podman (Docker-compatible alternative)
- **Seatbelt**: macOS native sandboxing using sandbox-exec

## Why Sandboxing?

Agent execution can be dangerous without proper isolation. Without sandboxing, agents can:

- Modify critical system files
- Execute malicious commands
- Access sensitive data
- Cause system instability
- Compromise security

Sandboxing provides isolation that prevents these risks while maintaining flexibility for legitimate operations.

## Sandbox Types

### NoSandbox

**Use when**: You trust the agent and the environment is already secure.

- No isolation
- Direct command execution
- Fastest performance
- Suitable for development and trusted agents

### Docker

**Use when**: You need strong isolation on Linux/macOS/Windows with Docker installed.

- Full container isolation
- Volume mounting support
- Network mode configuration
- Image-based execution environment
- Automatic cleanup with `--rm`

**Requirements**: Docker installed and running

### Podman

**Use when**: You prefer Podman over Docker or need rootless containers.

- Docker-compatible CLI
- Rootless container support
- Same features as Docker
- Better for environments without Docker daemon

**Requirements**: Podman installed

### Seatbelt (macOS only)

**Use when**: You're on macOS and want native sandboxing without containers.

- macOS native sandboxing
- Profile-based restrictions
- Permissive or restrictive profiles
- Custom profile support
- No container overhead

**Requirements**: macOS with `sandbox-exec` available

## Configuration

Sandbox configuration is specified in agent TOML files:

```toml
[agent]
id = "my-agent"
name = "My Agent"
description = "An agent with sandboxing"
prompt_path = "prompts/my-agent.md"

[agent.sandbox]
sandbox_type = "docker"
network = "closed"
profile = "restrictive"
image = "rust:latest"
working_dir = "/app"
volumes = ["/host:/container"]
env = { "KEY" = "value" }
custom_flags = ["--cap-add=SYS_ADMIN"]
```

### Configuration Options

- **sandbox_type**: `none`, `docker`, `podman`, or `seatbelt`
- **network**: `open`, `closed`, or `proxied`
- **profile**: `permissive`, `restrictive`, or `custom(path)`
- **image**: Container image for Docker/Podman (default: `rust:latest`)
- **working_dir**: Working directory inside sandbox
- **volumes**: Volume mounts in `host:container` format
- **env**: Environment variables as key-value pairs
- **custom_flags**: Additional flags for container execution

## Network Modes

### Open

Full network access. Use when the agent needs to make external API calls or access the internet.

### Closed

No network access. Use for maximum security when network access is not required.

### Proxied

Network access through host. Use when you need controlled network access.

## Profiles (Seatbelt only)

### Permissive

Minimal restrictions. Allows most operations while still providing basic isolation.

### Restrictive

Maximum restrictions. Blocks most operations except those explicitly allowed.

### Custom

Use a custom sandbox profile file. Specify the path to your `.sb` profile file.

## CLI Commands

Radium provides CLI commands for managing sandboxes:

```bash
# List available sandbox types
rad sandbox list

# Test a specific sandbox type
rad sandbox test docker

# Show current configuration
rad sandbox config

# Set default sandbox configuration for workspace
rad sandbox set docker --network closed --image alpine:latest

# Check prerequisites
rad sandbox doctor
```

### Setting Sandbox Configuration

You can set the default sandbox configuration for your workspace:

```bash
# Set Docker sandbox with closed network
rad sandbox set docker --network closed

# Set Podman sandbox with custom image
rad sandbox set podman --network open --image rust:latest

# Set sandbox with volume mounts
rad sandbox set docker --network closed --volumes /host:/container
```

The configuration is saved to `.radium/config.toml` and will be used as the default for agents that don't specify their own sandbox configuration.

## Architecture

The sandboxing system uses a trait-based architecture:

```
SandboxFactory
    ├── NoSandbox (direct execution)
    ├── DockerSandbox (Docker containers)
    ├── PodmanSandbox (Podman containers)
    └── SeatbeltSandbox (macOS sandbox-exec)
```

All sandboxes implement the `Sandbox` trait with three main methods:

- `initialize()`: Set up the sandbox environment
- `execute()`: Run a command in the sandbox
- `cleanup()`: Clean up resources

## Integration with Agent Execution

When an agent has a sandbox configuration:

1. Sandbox configuration is registered with the agent executor
2. Sandbox is created from agent config during agent initialization
3. Sandbox is initialized before agent execution
4. Shell commands execute through the sandbox (via `execute_with_sandbox()`)
5. Sandbox is cleaned up after execution (even on errors)

If a sandbox is not available (e.g., Docker not installed), the system gracefully falls back to NoSandbox with a warning logged.

## Examples

See the [examples directory](../../examples/agents/) for example agent configurations with sandboxing:

- `docker-sandboxed-agent.toml`: Agent using Docker sandboxing
- `seatbelt-sandboxed-agent.toml`: Agent using macOS Seatbelt sandboxing

## Security Best Practices

1. **Use restrictive profiles** when possible
2. **Close network access** unless required
3. **Limit volume mounts** to necessary directories
4. **Use minimal container images** (alpine, distroless)
5. **Review custom flags** for security implications
6. **Test sandbox configuration** before production use

## Troubleshooting

See [Sandbox Setup Guide](../guides/sandbox-setup.md) for detailed troubleshooting information.

## References

- [Sandbox Setup Guide](../guides/sandbox-setup.md)
- [Agent Configuration](../user-guide/agent-configuration.md)
- [Sandbox Implementation](../../crates/radium-core/src/sandbox/)

