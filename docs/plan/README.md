# Radium Feature Requirements

> **Purpose**: Structured feature requirements extracted from roadmap and implementation plans for Braingrid-compatible task breakdown and implementation planning.

## Overview

This directory contains structured feature requirements (REQs) organized by priority phases. Each REQ document is self-contained and optimized for AI agent consumption, providing detailed requirements that balance WHAT needs to be built with necessary technical context, while deferring HOW to implement to the task breakdown phase.

## Directory Structure

```
/docs/plan/
├── README.md                          # This file - overview and navigation
├── 01-now/                            # Priority 1: Critical foundation features
│   ├── REQ-001-workspace-system.md
│   ├── REQ-002-agent-configuration.md
│   ├── REQ-003-core-cli-commands.md
│   └── REQ-004-workflow-behaviors.md
├── 02-next/                           # Priority 2: High-value features
│   ├── REQ-005-plan-generation.md
│   ├── REQ-006-memory-context.md
│   ├── REQ-007-monitoring-telemetry.md
│   ├── REQ-008-sandboxing.md
│   ├── REQ-009-mcp-integration.md
│   ├── REQ-010-policy-engine.md
│   ├── REQ-011-context-files.md
│   ├── REQ-012-custom-commands.md
│   ├── REQ-013-checkpointing.md
│   └── REQ-014-vibe-check.md
├── 03-later/                          # Priority 3: Advanced features
│   ├── REQ-015-engine-abstraction.md
│   ├── REQ-016-tui-improvements.md
│   ├── REQ-017-agent-library.md
│   ├── REQ-018-extension-system.md
│   ├── REQ-019-hooks-system.md
│   └── REQ-020-session-analytics.md
└── templates/
    └── req-template.md                # Standard template for new REQs
```

## Phase Organization

### NOW Phase (Priority 1: Critical Foundation)

Foundation features required for all other functionality:

- **REQ-001**: Workspace System - Directory structure, RequirementId, Plan discovery
- **REQ-002**: Agent Configuration System - TOML configs, prompt templates, agent discovery
- **REQ-003**: Core CLI Commands - All rad commands (init, status, clean, plan, craft, etc.)
- **REQ-004**: Workflow Behaviors - Loop, trigger, checkpoint, vibe_check, Policy Engine

### NEXT Phase (Priority 2: High-Value Features)

High-value features that enhance core functionality:

- **REQ-005**: Plan Generation & Execution - AI-powered planning, rad plan/craft
- **REQ-006**: Memory & Context System - Plan-scoped memory, context gathering, file injection
- **REQ-007**: Monitoring & Telemetry - SQLite monitoring, telemetry tracking, logs
- **REQ-008**: Sandboxing - Docker/Podman, macOS Seatbelt isolation
- **REQ-009**: MCP Integration - Model Context Protocol for external tools
- **REQ-010**: Policy Engine - Rule-based tool execution control
- **REQ-011**: Context Files - Hierarchical GEMINI.md system
- **REQ-012**: Custom Commands - TOML-based command definitions
- **REQ-013**: Checkpointing - Git snapshots and restore
- **REQ-014**: Vibe Check - Metacognitive oversight system

### LATER Phase (Priority 3: Advanced Features)

Advanced features for future implementation:

- **REQ-015**: Engine Abstraction - Multi-provider LLM support
- **REQ-016**: TUI Improvements - Enhanced terminal UI
- **REQ-017**: Agent Library - Port 70+ agents from legacy system
- **REQ-018**: Extension System - Installable extension packages
- **REQ-019**: Hooks System - Execution flow interception
- **REQ-020**: Session Analytics - Comprehensive session reporting

## Feature Matrix

| REQ ID | Title | Phase | Status | Priority | Effort |
|--------|-------|-------|--------|----------|--------|
| REQ-001 | Workspace System | NOW | Completed | Critical | 10-14h | [View](01-now/REQ-001-workspace-system.md) |
| REQ-002 | Agent Configuration | NOW | Completed | Critical | 15-18h | [View](01-now/REQ-002-agent-configuration.md) |
| REQ-003 | Core CLI Commands | NOW | Completed | Critical | 11-14h | [View](01-now/REQ-003-core-cli-commands.md) |
| REQ-004 | Workflow Behaviors | NOW | Completed | High | 21-26h | [View](01-now/REQ-004-workflow-behaviors.md) |
| REQ-005 | Plan Generation | NEXT | Completed | High | 15-20h | [View](02-next/REQ-005-plan-generation.md) |
| REQ-006 | Memory & Context | NEXT | Completed | High | 19-23h | [View](02-next/REQ-006-memory-context.md) |
| REQ-007 | Monitoring & Telemetry | NEXT | Completed | High | 18-22h | [View](02-next/REQ-007-monitoring-telemetry.md) |
| REQ-008 | Sandboxing | NEXT | Completed | High | 12-15h | [View](02-next/REQ-008-sandboxing.md) |
| REQ-009 | MCP Integration | NEXT | Not Started | High | 4-5h | [View](02-next/REQ-009-mcp-integration.md) |
| REQ-010 | Policy Engine | NEXT | Completed | High | 6-7h | [View](02-next/REQ-010-policy-engine.md) |
| REQ-011 | Context Files | NEXT | Not Started | High | 3-4h | [View](02-next/REQ-011-context-files.md) |
| REQ-012 | Custom Commands | NEXT | Completed | High | 5-6h | [View](02-next/REQ-012-custom-commands.md) |
| REQ-013 | Checkpointing | NEXT | Completed | High | 6-7h | [View](02-next/REQ-013-checkpointing.md) |
| REQ-014 | Vibe Check | NEXT | Completed | High | 20-27h | [View](02-next/REQ-014-vibe-check.md) |
| REQ-015 | Engine Abstraction | LATER | Completed | Medium | 15-20h | [View](03-later/REQ-015-engine-abstraction.md) |
| REQ-016 | TUI Improvements | LATER | Completed | Medium | 15-20h | [View](03-later/REQ-016-tui-improvements.md) |
| REQ-017 | Agent Library | LATER | Completed | Medium | 30-40h | [View](03-later/REQ-017-agent-library.md) |
| REQ-018 | Extension System | LATER | Not Started | Low | TBD | [View](03-later/REQ-018-extension-system.md) |
| REQ-019 | Hooks System | LATER | Not Started | Low | TBD | [View](03-later/REQ-019-hooks-system.md) |
| REQ-020 | Session Analytics | LATER | Completed | Low | TBD | [View](03-later/REQ-020-session-analytics.md) |

## REQ Naming Convention

- Format: `REQ-XXX-feature-name.md`
- Sequential numbering across all phases (REQ-001, REQ-002, etc.)
- Kebab-case for feature names
- Prefix with phase number in directory structure

## Creating New REQs

1. Copy the template: `cp templates/req-template.md <phase-folder>/REQ-XXX-feature-name.md`
2. Fill in the YAML front matter with metadata
3. Complete all required sections:
   - Problem Statement
   - Solution Overview
   - Functional Requirements
   - Technical Requirements
   - User Experience
   - Data Requirements
   - Dependencies
   - Success Criteria
   - Out of Scope
   - References
4. Update this README's feature matrix
5. Add cross-references to original documentation

## REQ Document Structure

Each REQ document includes:

- **YAML Front Matter**: Metadata (req_id, title, phase, status, priority, effort, dependencies, related_docs)
- **Problem Statement**: Why this feature exists, user pain points
- **Solution Overview**: What the feature does at a high level
- **Functional Requirements**: Detailed WHAT with acceptance criteria
- **Technical Requirements**: Constraints, integration points, data models
- **User Experience**: How users interact with the feature
- **Data Requirements**: Data models, storage, APIs
- **Dependencies**: Other REQs or systems this depends on
- **Success Criteria**: Measurable outcomes and completion definition
- **Out of Scope**: Explicitly deferred items
- **References**: Links to original documentation

## Cross-References

Original documentation has been updated with cross-references to REQ documents:

- [Now/Next/Later Roadmap](../project/02-now-next-later.md) - Links to REQs for each step
- [Implementation Plan](../project/03-implementation-plan.md) - Links to REQs for detailed specs
- [Feature Enhancement Docs](../features/) - Links to relevant REQs

## Braingrid Integration

Each REQ is designed to be:
- Self-contained and understandable by AI agents
- Detailed enough for task breakdown without external context
- Structured for direct consumption by Braingrid
- Traceable back to original documentation

### Sync Status

**Last Synced**: 2025-12-07  
**Status**: ✅ All 20 REQs synced to Braingrid (PROJ-14)

All local REQ documents have been synced to Braingrid for initial parity. The sync script (`scripts/sync-reqs-to-braingrid.py`) can be used to keep local and Braingrid REQs in sync.

**Sync Summary**: See [SYNC_SUMMARY.md](SYNC_SUMMARY.md) for detailed sync results.

## Status Legend

- **Not Started**: Feature not yet implemented
- **In Progress**: Feature currently being implemented
- **Completed**: Feature fully implemented with tests

## Priority Legend

- **Critical**: Must be completed for basic functionality
- **High**: High-value features that significantly enhance the system
- **Medium**: Important features for complete functionality
- **Low**: Nice-to-have features for future enhancement

## Related Documentation

- [Project Overview](../project/00-project-overview.md)
- [Now/Next/Later Roadmap](../project/02-now-next-later.md)
- [Implementation Plan](../project/03-implementation-plan.md)
- [Completed Work](../project/01-completed.md)
- [Feature Backlog](../legacy/legacy-system-feature-backlog.md)

