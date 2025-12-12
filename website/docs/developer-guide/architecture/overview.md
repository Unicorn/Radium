---
id: "architecture-overview"
title: "Architecture Overview"
sidebar_label: "Architecture Overview"
---

# Architecture Overview

This document provides a high-level overview of Radium's architecture, including core components, data flows, and system design principles.

## System Architecture

Radium is built as a modular, Rust-based platform with multiple interfaces and a unified orchestration engine.

### High-Level Components

```
┌─────────────────────────────────────────────────────────┐
│                    Client Applications                   │
├──────────────┬──────────────┬──────────────┬───────────┤
│     CLI      │     TUI      │   Desktop    │    Web    │
└──────┬───────┴──────┬───────┴──────┬───────┴─────┬─────┘
       │              │               │             │
       └──────────────┼───────────────┼─────────────┘
                      │               │
              ┌───────▼───────────────▼───────┐
              │    Radium Core (gRPC Server)   │
              └───────┬───────────────────────┘
                      │
       ┌──────────────┼──────────────┐
       │              │              │
┌──────▼──────┐ ┌─────▼─────┐ ┌─────▼──────┐
│ Orchestrator│ │  Planning  │ │   Memory   │
│   Engine    │ │   System   │ │   System   │
└──────┬──────┘ └─────┬──────┘ └─────┬──────┘
       │              │              │
       └──────────────┼──────────────┘
                      │
              ┌───────▼────────┐
              │  Agent System  │
              └───────┬────────┘
                      │
       ┌──────────────┼──────────────┐
       │              │              │
┌──────▼──────┐ ┌─────▼─────┐ ┌─────▼──────┐
│   Gemini    │ │  Claude    │ │   OpenAI   │
│   Engine    │ │  Engine    │ │   Engine   │
└─────────────┘ └────────────┘ └────────────┘
```

## Core Components

### Radium Core

The core backend (`crates/radium-core`) provides:

- **gRPC Server**: Unified API for all client applications
- **Agent Orchestration**: Multi-agent coordination and task routing
- **Planning System**: Autonomous plan generation and execution
- **Memory System**: Plan-scoped memory storage and retrieval
- **Context Management**: Context gathering from multiple sources
- **Policy Engine**: Fine-grained tool execution control

### Orchestration Engine

The orchestration engine (`crates/radium-orchestrator`) provides:

- **Intelligent Routing**: Automatic agent selection based on task analysis
- **Multi-Agent Coordination**: Coordinate multiple agents for complex workflows
- **Model Agnostic**: Works with any AI provider
- **Result Synthesis**: Combine results from multiple agents

### Agent System

The agent system manages:

- **Agent Discovery**: Automatic discovery from configuration files
- **Agent Execution**: Execute agents with their configured models
- **Agent Memory**: Store and retrieve agent outputs
- **Agent Configuration**: TOML-based agent configuration

### Engine Abstraction

The engine abstraction layer provides:

- **Unified Interface**: Consistent API across AI providers
- **Provider Support**: Gemini, Claude, OpenAI, self-hosted models
- **Model Selection**: Intelligent model selection via persona system
- **Fallback Chains**: Automatic fallback to alternative models

## Data Flow

### Agent Execution Flow

```
User Request
    ↓
Orchestrator (analyzes request)
    ↓
Agent Selection
    ↓
Context Gathering (memory, files, sources)
    ↓
Agent Execution (via engine abstraction)
    ↓
Result Processing
    ↓
Memory Storage
    ↓
Response to User
```

### Planning Flow

```
Goal/Specification
    ↓
Plan Generator (AI-powered)
    ↓
Plan Validation (multi-stage)
    ↓
Dependency Graph (DAG construction)
    ↓
Workflow Generation
    ↓
Plan Execution
    ↓
Result Storage
```

## Architecture Patterns

### Modular Monorepo

Radium uses a modular monorepo structure:

- **Crates**: Rust libraries for core functionality
- **Apps**: Client applications (CLI, TUI, Desktop)
- **Packages**: TypeScript packages for web/desktop

### Extension System

The extension system enables:

- **Component Packaging**: Package prompts, MCP servers, commands, hooks
- **Distribution**: Share components via marketplace
- **Discovery**: Automatic discovery of installed extensions
- **Integration**: Seamless integration with core system

### Policy Engine

The policy engine provides:

- **Rule-Based Control**: TOML-based policy rules
- **Context Awareness**: Different policies for different contexts
- **Approval Modes**: Yolo, AutoEdit, Ask modes
- **Session Constitutions**: Temporary rules for specific sessions

## Future Architecture: Component Ecosystem

Radium is evolving toward a **composable intelligence infrastructure**:

### Component Foundry Pattern

- **Standardized Interfaces**: Consistent component patterns
- **Validation Framework**: Automated quality checks
- **Composition Rules**: Guidelines for combining components
- **Version Management**: Semantic versioning

**Status**: 📋 Planned  
**Learn more**: [Roadmap: Component Foundry](../roadmap/vision.md#1-component-foundry-pattern-cfp)

### Global Component Graph

- **Component Discovery**: Find components across ecosystem
- **Relationship Tracking**: Track component dependencies
- **Composition Engine**: Automatic component composition
- **Distributed Graph**: Support for federated graphs

**Status**: 📋 Planned  
**Learn more**: [Roadmap: Global Component Graph](../roadmap/technical-architecture.md#global-component-graph-design-t3)

### Autonomous Assembly

- **Goal-Driven Composition**: Systems compose based on goals
- **Constraint Satisfaction**: Respect technical and policy constraints
- **Dynamic Reconfiguration**: Adapt as needs change
- **Self-Healing**: Automatic remediation (DACR)

**Status**: 🔮 Future  
**Learn more**: [Roadmap: Vision](../roadmap/vision.md)

## Detailed Architecture Documents

### Core Systems

- **[Agent Configuration System](./agent-configuration-system.md)** - Agent configuration architecture
- **[Checkpoint System](./checkpoint-system.md)** - Checkpoint and resume architecture
- **[Engine Abstraction](./engine-abstraction.md)** - AI provider abstraction layer
- **[TUI Architecture](./tui-architecture.md)** - Terminal UI architecture

### System Design

- **[Agent System Architecture](../agent-system-architecture.md)** - Complete agent system architecture
- **[Extension System Architecture](../../extensions/architecture.md)** - Extension system architecture
- **[MCP Architecture](../../mcp/architecture.md)** - Model Context Protocol architecture

## Design Principles

### Performance

- **Rust-Based**: Memory safety and performance
- **Concurrent Execution**: Parallel agent execution
- **Efficient Caching**: Context and result caching
- **Optimized Models**: Cost-effective model selection

### Safety

- **Policy Engine**: Fine-grained control
- **Sandboxing**: Safe execution environment
- **Approval Modes**: User control over operations
- **Error Handling**: Graceful failure and recovery

### Extensibility

- **Extension System**: Easy component addition
- **Plugin Architecture**: Custom integrations
- **Engine Abstraction**: Add new AI providers
- **Hook System**: Custom behavior injection

### Composability

- **Component-Based**: Reusable components
- **Modular Design**: Independent modules
- **Standard Interfaces**: Consistent APIs
- **Version Management**: Compatibility tracking

## Technology Stack

### Backend

- **Rust**: Core language for performance and safety
- **gRPC**: Inter-service communication
- **Tokio**: Async runtime
- **Serde**: Serialization

### Frontend

- **TypeScript**: Web and desktop applications
- **React**: Web UI components
- **Tauri**: Desktop application framework
- **Ratatui**: Terminal UI library

### AI Integration

- **Google Gemini**: Via API
- **Anthropic Claude**: Via API
- **OpenAI GPT**: Via API
- **Self-Hosted**: Ollama, vLLM, LocalAI

## Roadmap & Evolution

Radium's architecture is evolving toward:

1. **Component Foundry**: Systematic component creation and validation
2. **Global Component Graph**: Ecosystem-wide component discovery
3. **Autonomous Assembly**: Self-composing systems
4. **KOR Protocol**: Component exchange protocol
5. **DAO Governance**: Community-driven development

**Learn more**: [Roadmap](../roadmap/index.md)

## Related Documentation

- **[Agent System Architecture](../agent-system-architecture.md)** - Detailed agent system
- **[Extension System Architecture](../../extensions/architecture.md)** - Extension architecture
- **[Technical Architecture Roadmap](../../roadmap/technical-architecture.md)** - Implementation roadmap
- **[Vision & Innovation](../../roadmap/vision.md)** - Long-term vision

---

**Want to dive deeper?** Explore the [detailed architecture documents](./) or check the [Technical Architecture Roadmap](../../roadmap/technical-architecture.md).

