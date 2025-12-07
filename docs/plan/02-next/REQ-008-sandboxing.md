---
req_id: REQ-008
title: Sandboxing
phase: NEXT
status: Completed
priority: High
estimated_effort: 12-15 hours
dependencies: [REQ-001, REQ-002]
related_docs:
  - docs/project/02-now-next-later.md#step-65-sandboxing
  - docs/project/03-implementation-plan.md#step-65-sandboxing
  - docs/features/gemini-cli-enhancements.md#sandboxing
---

# Sandboxing

## Problem Statement

Agent execution of shell commands and file operations can be dangerous without proper isolation. Without sandboxing, agents can:
- Modify critical system files
- Execute malicious commands
- Access sensitive data
- Cause system instability
- Compromise security

The legacy system and modern AI tools (like gemini-cli) use sandboxing to isolate agent execution. Radium needs an equivalent system that supports multiple sandbox methods (Docker, Podman, macOS Seatbelt) with configurable profiles.

## Solution Overview

Implement a comprehensive sandboxing system that provides:
- Sandbox abstraction trait for pluggable implementations
- Docker/Podman container-based sandboxing with volume mounting
- macOS Seatbelt sandboxing with permissive/restrictive profiles
- Network mode configuration (open/closed/proxied)
- Custom sandbox flags and environment variable support
- No-op sandbox for direct execution when sandboxing is disabled

The sandboxing system enables safe agent execution, especially for shell commands and file operations, while maintaining flexibility for different execution environments.

## Functional Requirements

### FR-1: Sandbox Abstraction

**Description**: Pluggable sandbox trait for multiple implementations.

**Acceptance Criteria**:
- [x] Sandbox trait with initialize, execute, cleanup methods
- [x] Sandbox factory for creating sandbox instances
- [x] Sandbox configuration structure
- [x] Error handling for unavailable sandbox types
- [x] No-op sandbox for direct execution

**Implementation**: `crates/radium-core/src/sandbox/sandbox.rs`

### FR-2: Docker/Podman Sandbox

**Description**: Container-based sandboxing using Docker or Podman.

**Acceptance Criteria**:
- [x] Docker container sandboxing
- [x] Podman container sandboxing
- [x] Volume mounting for workspace access
- [x] Network configuration (open/closed/proxied)
- [x] Custom sandbox flags support
- [x] Environment variable configuration
- [x] Image selection and management

**Implementation**: 
- `crates/radium-core/src/sandbox/docker.rs`
- `crates/radium-core/src/sandbox/podman.rs`

### FR-3: macOS Seatbelt Sandbox

**Description**: macOS native sandboxing using sandbox-exec.

**Acceptance Criteria**:
- [x] macOS Seatbelt sandbox integration
- [x] Permissive sandbox profile
- [x] Restrictive sandbox profile
- [x] Custom profile file support
- [x] Network control (open/closed/proxied)
- [x] File system access restrictions
- [x] Platform detection (macOS only)

**Implementation**: `crates/radium-core/src/sandbox/seatbelt.rs`

### FR-4: Sandbox Configuration

**Description**: Configurable sandbox settings and profiles.

**Acceptance Criteria**:
- [x] Sandbox type selection (None, Docker, Podman, Seatbelt)
- [x] Network mode configuration (open/closed/proxied)
- [x] Sandbox profile selection (permissive/restrictive/custom)
- [x] Custom sandbox flags
- [x] Environment variable configuration
- [x] Image/container configuration

**Implementation**: `crates/radium-core/src/sandbox/config.rs`

## Technical Requirements

### TR-1: Sandbox Trait

**Description**: Trait definition for sandbox implementations.

**APIs**:
```rust
#[async_trait]
pub trait Sandbox: Send + Sync {
    async fn initialize(&mut self) -> Result<()>;
    async fn execute(&self, command: &str, args: &[String], cwd: Option<&Path>) -> Result<Output>;
    async fn cleanup(&mut self) -> Result<()>;
    fn sandbox_type(&self) -> SandboxType;
}
```

### TR-2: Sandbox Configuration

**Description**: Configuration structure for sandbox settings.

**Data Models**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub sandbox_type: SandboxType,
    pub network: NetworkMode,
    pub profile: SandboxProfile,
    pub image: Option<String>,
    pub volumes: Vec<VolumeMount>,
    pub env: HashMap<String, String>,
    pub custom_flags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxType {
    None,
    Docker,
    Podman,
    Seatbelt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkMode {
    Open,
    Closed,
    Proxied,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxProfile {
    Permissive,
    Restrictive,
    Custom(String),
}
```

### TR-3: Sandbox Factory

**Description**: Factory for creating sandbox instances.

**APIs**:
```rust
pub struct SandboxFactory;

impl SandboxFactory {
    pub fn create(config: &SandboxConfig) -> Result<Box<dyn Sandbox>>;
}
```

## User Experience

### UX-1: Sandbox Configuration

**Description**: Users configure sandboxing in workspace settings.

**Example**:
```toml
# .radium/config.toml
[sandbox]
type = "docker"
network = "closed"
profile = "restrictive"
image = "rust:latest"
```

### UX-2: Sandbox Execution

**Description**: Agents execute commands in sandboxed environments.

**Example**:
```rust
let config = SandboxConfig::new(SandboxType::Docker)
    .with_image("rust:latest".to_string());
let mut sandbox = SandboxFactory::create(&config)?;
sandbox.initialize().await?;
let output = sandbox.execute("cargo", &["--version"], None).await?;
```

## Data Requirements

### DR-1: Sandbox Configuration

**Description**: TOML configuration for sandbox settings.

**Location**: `.radium/config.toml` or workspace configuration

**Schema**: See TR-2 Sandbox Configuration

## Dependencies

- **REQ-001**: Workspace System - Required for workspace structure
- **REQ-002**: Agent Configuration - Required for agent execution

## Success Criteria

1. [x] Sandbox abstraction supports multiple implementations
2. [x] Docker/Podman sandboxing works with volume mounting
3. [x] macOS Seatbelt sandboxing works with profiles
4. [x] Network modes are configurable
5. [x] Custom sandbox flags are supported
6. [x] All sandbox operations have comprehensive test coverage (15+ tests)

**Completion Metrics**:
- **Status**: ✅ Complete
- **Test Coverage**: 15+ passing tests
- **Implementation**: Full sandboxing system implemented
- **Files**: 
  - `crates/radium-core/src/sandbox/` (sandbox, docker, podman, seatbelt, config, error)

## Out of Scope

- Advanced container orchestration (Kubernetes, future enhancement)
- Sandbox performance optimization (future enhancement)
- Sandbox monitoring and metrics (future enhancement)

## References

- [Now/Next/Later Roadmap](../project/02-now-next-later.md#step-65-sandboxing)
- [Implementation Plan](../project/03-implementation-plan.md#step-65-sandboxing)
- [Gemini CLI Enhancements](../features/gemini-cli-enhancements.md#sandboxing)
- [Sandbox Implementation](../../crates/radium-core/src/sandbox/)

