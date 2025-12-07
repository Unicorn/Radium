---
req_id: REQ-015
title: Engine Abstraction Layer
phase: LATER
status: Completed
priority: Medium
estimated_effort: 15-20 hours
dependencies: [REQ-002]
related_docs:
  - docs/project/02-now-next-later.md#step-7-engine-abstraction-layer
  - docs/project/03-implementation-plan.md#step-7-engine-abstraction-layer
  - docs/legacy/legacy-system-feature-backlog.md#4-engineprovider-system
---

# Engine Abstraction Layer

## Problem Statement

Users need support for multiple AI providers beyond Gemini. Without engine abstraction, users cannot:
- Use different LLM providers (OpenAI, Claude, etc.)
- Switch between providers based on task requirements
- Leverage provider-specific features and optimizations
- Support multiple authentication methods
- Test with mock providers

The legacy system supported multiple engines (Codex, Claude, Cursor, CCR, OpenCode, Auggie). Radium needs an equivalent abstraction layer that supports pluggable providers with unified interfaces.

## Solution Overview

Implement a comprehensive engine abstraction layer that provides:
- Engine trait abstraction for pluggable providers
- Engine registry and factory system
- CLI binary detection and validation
- Authentication system per engine
- Execution request/response structures
- Token usage tracking
- Mock engine provider for testing

The engine abstraction layer enables multi-provider LLM support, allowing users to choose the best provider for each task while maintaining a unified interface.

## Functional Requirements

### FR-1: Engine Trait Abstraction

**Description**: Pluggable engine trait for multiple provider implementations.

**Acceptance Criteria**:
- [x] Engine trait definition with unified interface
- [x] Engine metadata (name, description, CLI command)
- [x] Execution request/response structures
- [x] Error handling and provider-specific errors
- [x] Token usage tracking interface

**Implementation**: 
- `crates/radium-core/src/engines/mod.rs`
- `crates/radium-core/src/engines/trait.rs`

### FR-2: Engine Registry

**Description**: Dynamic engine registration and lookup system.

**Acceptance Criteria**:
- [x] Engine registry for managing providers
- [x] Dynamic engine registration
- [x] Engine metadata storage
- [x] Default engine selection
- [x] Engine lookup by ID

**Implementation**: `crates/radium-core/src/engines/registry.rs`

### FR-3: CLI Binary Detection

**Description**: Detect and validate engine CLI binaries.

**Acceptance Criteria**:
- [x] Path checking for CLI binaries
- [x] Version command execution
- [x] Timeout handling
- [x] Error detection
- [x] Binary availability checking

**Implementation**: `crates/radium-core/src/engines/detection.rs`

### FR-4: Authentication System

**Description**: Per-engine authentication management.

**Acceptance Criteria**:
- [x] Authentication status checking per engine
- [x] Authentication methods per engine
- [x] Auth state persistence
- [x] Multi-provider authentication support
- [x] API key management

**Implementation**: `crates/radium-core/src/engines/auth.rs`

### FR-5: Engine Providers

**Description**: Implementations for various LLM providers.

**Acceptance Criteria**:
- [x] Gemini provider (primary)
- [x] Mock provider for testing
- [ ] Codex provider (partial)
- [ ] Claude provider (partial)
- [ ] Cursor provider (partial)
- [ ] CCR provider (partial)
- [ ] OpenCode provider (partial)
- [ ] Auggie provider (partial)

**Implementation**: `crates/radium-core/src/engines/providers/*.rs`

## Technical Requirements

### TR-1: Engine Trait

**Description**: Trait definition for engine implementations.

**APIs**:
```rust
pub trait Engine: Send + Sync {
    fn name(&self) -> &str;
    fn execute(&self, request: &ExecutionRequest) -> Result<ExecutionResponse>;
    fn check_auth(&self) -> Result<AuthStatus>;
    fn detect_binary(&self) -> Result<bool>;
}
```

### TR-2: Engine Registry

**Description**: Registry for managing engine instances.

**APIs**:
```rust
pub struct EngineRegistry {
    engines: HashMap<String, Box<dyn Engine>>,
}

impl EngineRegistry {
    pub fn register(&mut self, engine: Box<dyn Engine>);
    pub fn get(&self, name: &str) -> Option<&dyn Engine>;
    pub fn list(&self) -> Vec<&dyn Engine>;
}
```

## User Experience

### UX-1: Engine Selection

**Description**: Users select engines via configuration or CLI.

**Example**:
```bash
$ rad status
Available Engines:
  gemini (default) - ✓ Authenticated
  openai - ✗ Not authenticated
  claude - ✗ Not authenticated
```

## Data Requirements

### DR-1: Engine Configuration

**Description**: Configuration for engine settings.

**Location**: `.radium/config.toml` or workspace configuration

**Format**: TOML with engine-specific settings

## Dependencies

- **REQ-002**: Agent Configuration - Required for agent system

## Success Criteria

1. [x] Engine abstraction supports multiple providers
2. [x] Engine registry manages providers correctly
3. [x] CLI binary detection works for available engines
4. [x] Authentication system supports multiple providers
5. [x] Mock engine provider works for testing
6. [x] All engine operations have comprehensive test coverage (23+ tests)

**Completion Metrics**:
- **Status**: ✅ Complete (Core)
- **Test Coverage**: 23+ passing tests
- **Implementation**: Engine abstraction layer fully implemented
- **Files**: 
  - `crates/radium-core/src/engines/` (trait, registry, detection, auth, providers)

## Out of Scope

- Complete provider implementations for all legacy engines (future enhancement)
- Advanced provider-specific features (future enhancement)
- Provider marketplace (future enhancement)

## References

- [Now/Next/Later Roadmap](../project/02-now-next-later.md#step-7-engine-abstraction-layer)
- [Implementation Plan](../project/03-implementation-plan.md#step-7-engine-abstraction-layer)
- [Feature Backlog](../legacy/legacy-system-feature-backlog.md#4-engineprovider-system)
- [Engine Abstraction Implementation](../../crates/radium-core/src/engines/)

