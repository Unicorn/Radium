---
req_id: REQ-017
title: Agent Library
phase: LATER
status: Completed
priority: Medium
estimated_effort: 30-40 hours
dependencies: [REQ-002]
related_docs:
  - docs/project/02-now-next-later.md#step-9-enhanced-agent-library-72-agents
  - docs/project/03-implementation-plan.md#step-9-agent-library-70-agents
  - docs/legacy/legacy-system-feature-backlog.md#3-agent-system
---

# Agent Library

## Problem Statement

Users need access to a comprehensive library of specialized agents for different tasks. Without an agent library, users must:
- Create agents from scratch for every use case
- Duplicate agent configurations across projects
- Miss out on specialized agents for specific domains
- Lack access to battle-tested agent patterns

The legacy system had 70+ specialized agents. Radium needs an equivalent library that provides agents for various domains and use cases, with a foundation for future persona system enhancements.

## Solution Overview

Implement a comprehensive agent library that provides:
- Agent registry system for managing agents
- Agent template generator (`rad agents create`)
- Core example agents (arch, plan, code, review, doc)
- Comprehensive agent creation guide
- Agent discovery and validation
- CLI integration for agent management
- Foundation for future persona system with model recommendations

The agent library enables users to leverage specialized agents for different tasks, improving productivity and ensuring consistent agent behavior across projects.

## Functional Requirements

### FR-1: Agent Registry System

**Description**: System for managing and discovering agents.

**Acceptance Criteria**:
- [x] Agent registry for managing agents
- [x] Agent discovery from multiple directories
- [x] Agent validation and metadata extraction
- [x] Agent categorization and organization
- [x] Agent search and filtering

**Implementation**: `crates/radium-core/src/agents/registry.rs`

### FR-2: Agent Template Generator

**Description**: Tool for generating new agent configurations.

**Acceptance Criteria**:
- [x] `rad agents create` command
- [x] Agent TOML config generation
- [x] Prompt file structure generation
- [x] Agent metadata extraction
- [x] Template customization

**Implementation**: `apps/cli/src/commands/agents.rs`

### FR-3: Core Agent Library

**Description**: Core set of example agents for common tasks.

**Acceptance Criteria**:
- [x] arch-agent - Architecture and design
- [x] plan-agent - Planning and task breakdown
- [x] code-agent - Code generation
- [x] review-agent - Code review
- [x] doc-agent - Documentation

**Implementation**: `agents/core/*.toml` and prompts

### FR-4: Agent Creation Guide

**Description**: Comprehensive guide for creating new agents.

**Acceptance Criteria**:
- [x] Agent creation documentation (484 lines)
- [x] Best practices and patterns
- [x] Examples and templates
- [x] Integration guidelines

**Implementation**: `docs/guides/agent-creation-guide.md`

### FR-5: Future Enhancement - Persona System

**Description**: Enhanced agent metadata with model recommendations.

**Acceptance Criteria**:
- [ ] Enhanced YAML frontmatter with model recommendations
- [ ] Model selection engine (speed/balanced/thinking/expert)
- [ ] Cost estimation and budget tracking
- [ ] Fallback chain logic (primary → fallback → mock)
- [ ] Agent recommendation engine

**Status**: Future enhancement (not yet implemented)

## Technical Requirements

### TR-1: Agent Library Structure

**Description**: Directory structure for agent organization.

**Structure**:
```
agents/
├── core/           # Core agents (arch, plan, code, review, doc)
├── design/         # Design agents
├── testing/        # Testing agents
├── deployment/     # Deployment agents
└── custom/         # User-defined agents
```

### TR-2: Agent Metadata

**Description**: Enhanced metadata for future persona system.

**Future Format**:
```yaml
---
agent_id: arch-agent
name: Architecture Agent
recommended_models:
  primary: gemini-2.0-flash-thinking
  fallback: gemini-2.0-flash-exp
  premium: gemini-1.5-pro
capabilities: [architecture, design, planning]
performance_profile: thinking
---
```

## User Experience

### UX-1: Agent Discovery

**Description**: Users discover agents via CLI.

**Example**:
```bash
$ rad agents list
Found 5 agents:
  arch-agent (core) - Architecture Agent
  plan-agent (core) - Planning Agent
  code-agent (core) - Code Generation Agent
  review-agent (core) - Code Review Agent
  doc-agent (core) - Documentation Agent
```

### UX-2: Agent Creation

**Description**: Users create new agents with template generator.

**Example**:
```bash
$ rad agents create my-agent
Creating agent: my-agent
✓ Generated agent.toml
✓ Generated prompt template
✓ Agent created successfully
```

## Data Requirements

### DR-1: Agent Configurations

**Description**: TOML files containing agent configurations.

**Location**: `agents/<category>/<agent-id>.toml`

**Format**: See REQ-002 Agent Configuration format

## Dependencies

- **REQ-002**: Agent Configuration - Required for agent system

## Success Criteria

1. [x] Agent registry manages agents correctly
2. [x] Agent template generator creates valid configurations
3. [x] Core agents are available and functional
4. [x] Agent discovery works from multiple directories
5. [x] Agent validation catches configuration errors
6. [x] Agent creation guide provides comprehensive documentation
7. [x] Foundation is ready for future persona system enhancements

**Completion Metrics**:
- **Status**: ✅ Complete (Core)
- **Core Features**: Agent registry, template generator, 5 core agents
- **Documentation**: 484-line agent creation guide
- **Future Enhancement**: Persona system with model recommendations (planned)

## Out of Scope

- Complete porting of all 70+ legacy agents (future enhancement)
- Persona system implementation (future enhancement)
- Agent marketplace (future enhancement)
- Agent versioning (future enhancement)

## References

- [Now/Next/Later Roadmap](../project/02-now-next-later.md#step-9-enhanced-agent-library-72-agents)
- [Implementation Plan](../project/03-implementation-plan.md#step-9-agent-library-70-agents)
- [Feature Backlog](../legacy/legacy-system-feature-backlog.md#3-agent-system)
- [Agent Creation Guide](../../docs/guides/agent-creation-guide.md)
- [Agent Registry Implementation](../../crates/radium-core/src/agents/registry.rs)

