---
id: "sandbox-setup"
title: "Sandbox Setup Guide"
sidebar_label: "Sandbox Setup Guide"
---

# Sandbox Setup Guide

This guide helps you set up and configure sandboxing for Radium agents.

## Prerequisites

### Docker

**Installation**:

- **macOS**: Install [Docker Desktop](https://docs.docker.com/desktop/install/mac-install/)
- **Linux**: Install via package manager (e.g., `apt install docker.io`)
- **Windows**: Install [Docker Desktop](https://docs.docker.com/desktop/install/windows-install/)

**Verification**:

```bash
docker --version
docker run hello-world
```

**Common Issues**:

- **Permission denied**: Add your user to the `docker` group:
  ```bash
  sudo usermod -aG docker $USER
  newgrp docker
  ```
- **Docker daemon not running**: Start Docker Desktop or Docker service
- **Image pull failed**: Check network connection and Docker registry access

### Podman

**Installation**:

- **macOS**: `brew install podman`
- **Linux**: Install via package manager (e.g., `apt install podman`)
- **Windows**: Install via WSL2 or use Podman Desktop

**Verification**:

```bash
podman --version
podman run hello-world
```

**Common Issues**:

- **Rootless mode**: Podman runs rootless by default, which is secure but may have limitations
- **Image pull failed**: Check network connection and registry access
- **Storage issues**: Podman uses different storage than Docker

### Seatbelt (macOS only)

**Availability**:

Seatbelt is built into macOS. Verify availability:

```bash
which sandbox-exec
```

**Requirements**:

- macOS 10.5 or later
- No additional installation needed

**Common Issues**:

- **Not available**: Ensure you're on macOS
- **Permission denied**: Check file permissions and profile syntax

## Configuration

### Basic Configuration

Add sandbox configuration to your agent TOML file:

```toml
[agent]
id = "my-agent"
name = "My Agent"
prompt_path = "prompts/my-agent.md"

[agent.sandbox]
sandbox_type = "docker"
network = "closed"
image = "alpine:latest"
```

### Advanced Configuration

```toml
[agent.sandbox]
sandbox_type = "docker"
network = "closed"
profile = "restrictive"
image = "rust:latest"
working_dir = "/app"
volumes = [
    "/host/path:/container/path",
    "/another/host:/another/container"
]
env = {
    "RUST_LOG" = "debug",
    "API_KEY" = "secret"
}
custom_flags = [
    "--cap-add=SYS_ADMIN",
    "--memory=512m"
]
```

## Testing Sandbox Configuration

Use the CLI to test your sandbox setup:

```bash
# Test Docker sandbox
rad sandbox test docker

# Test Podman sandbox
rad sandbox test podman

# Test Seatbelt sandbox (macOS)
rad sandbox test seatbelt
```

## Verification Steps

1. **Check prerequisites**:
   ```bash
   rad sandbox doctor
   ```

2. **List available sandboxes**:
   ```bash
   rad sandbox list
   ```

3. **Test sandbox execution**:
   ```bash
   rad sandbox test docker
   ```

4. **Verify agent configuration**:
   ```bash
   rad agents info my-agent
   ```

## Common Issues and Solutions

### Docker Issues

**Problem**: "Docker not found"

**Solution**: Install Docker and ensure it's in your PATH

**Problem**: "Permission denied"

**Solution**: Add user to docker group (see Prerequisites)

**Problem**: "Image pull failed"

**Solution**: 
- Check network connection
- Verify image name and tag
- Try pulling manually: `docker pull <image>`

### Podman Issues

**Problem**: "Podman not found"

**Solution**: Install Podman and ensure it's in your PATH

**Problem**: "Rootless container limitations"

**Solution**: 
- Use `podman machine` for full compatibility
- Or configure rootless mode properly

### Seatbelt Issues

**Problem**: "sandbox-exec not found"

**Solution**: Ensure you're on macOS

**Problem**: "Profile syntax error"

**Solution**: Check your custom profile file syntax

### Network Issues

**Problem**: "Network access blocked in closed mode"

**Solution**: This is expected. Use `network = "open"` if network access is needed.

**Problem**: "Network access fails in open mode"

**Solution**: 
- Check container network configuration
- Verify DNS resolution
- Check firewall settings

### Volume Mount Issues

**Problem**: "Volume mount failed"

**Solution**:
- Verify host path exists
- Check path permissions
- Ensure correct format: `/host:/container`

**Problem**: "Permission denied in mounted volume"

**Solution**:
- Check file permissions on host
- Use appropriate user in container
- Consider SELinux/AppArmor policies

## Best Practices

1. **Start with NoSandbox** for development
2. **Use Docker/Podman** for production
3. **Test sandbox configuration** before deploying
4. **Use minimal images** (alpine, distroless)
5. **Limit volume mounts** to necessary directories
6. **Close network** unless required
7. **Monitor sandbox execution** for errors

## Security Considerations

1. **Never mount sensitive directories** (e.g., `/etc`, `/home`)
2. **Use restrictive profiles** when possible
3. **Limit custom flags** to necessary capabilities
4. **Review environment variables** for secrets
5. **Test in isolated environment** first
6. **Keep container images updated**

## Next Steps

- See [Sandboxing Feature Documentation](../features/sandboxing.md) for detailed API reference
- Check [Example Configurations](../../examples/agents/) for working examples
- Review [Agent Configuration Guide](../user-guide/agent-configuration.md) for agent setup

