---
id: "user-guide-overview"
title: "User Guide"
sidebar_label: "User Guide Overview"
---

# User Guide

Welcome to the Radium User Guide. This comprehensive guide covers everything you need to know to effectively use Radium for building and managing AI agent systems.

## Getting Started

New to Radium? Start here:

- **[Quick Start](../getting-started/quick-start.md)** - Get up and running in minutes
- **[Core Concepts](../getting-started/core-concepts.md)** - Understand fundamental concepts
- **[Installation](../getting-started/installation.md)** - Installation instructions

## Core Features

### Agent Configuration

Configure and manage AI agents for your specific needs.

- **[Agent Configuration Guide](./agent-configuration.md)** - Complete guide to configuring agents
  - TOML configuration format
  - Model selection and parameters
  - Prompt management
  - Persona system integration
  - Self-hosted model configuration

### Orchestration

Intelligent task routing that automatically selects and coordinates agents.

- **[Orchestration Guide](./orchestration.md)** - Natural language interaction with agents
- **[Orchestration Configuration](./orchestration-configuration.md)** - Configure orchestration behavior
- **[Orchestration Testing](./orchestration-testing.md)** - Test and validate orchestration
- **[Orchestration Troubleshooting](./orchestration-troubleshooting.md)** - Solve common issues

### Persona System

Intelligent model selection with cost optimization and automatic fallback.

- **[Persona System](./persona-system.md)** - Model selection and cost optimization
  - Performance profiles
  - Fallback chains
  - Cost optimization strategies

### Memory & Context

Maintain continuity across sessions and provide context to agents.

- **[Memory & Context System](./memory-and-context.md)** - Plan-scoped memory and context management
- **[Context Sources](./context-sources.md)** - File, HTTP, Jira, and BrainGrid sources

### Learning System

Track mistakes, preferences, and successes to build pattern recognition.

- **[Learning System](./learning-system.md)** - Learn from experience
  - Mistake tracking
  - Preference learning
  - ACE Skillbook

### Vibe Check

Metacognitive oversight to prevent reasoning lock-in and improve agent alignment.

- **[Vibe Check](./vibe-check.md)** - Chain-Pattern Interrupt system
  - Risk assessment
  - Pattern detection
  - Phase-aware feedback

### Constitution Rules

Session-scoped rules for workflow constraints.

- **[Constitution Rules](./constitution-rules.md)** - Temporary rules for specific sessions

### Custom Commands

Reusable command definitions with template substitution.

- **[Custom Commands](./custom-commands.md)** - Create and use custom commands

## Guides

Step-by-step guides for common tasks:

- **[Agent Creation Guide](./guides/agent-creation-guide.md)** - Create custom agents
- **[Reasoning Configuration](./guides/reasoning-configuration.md)** - Configure reasoning models
- **[Sandbox Setup](./guides/sandbox-setup.md)** - Set up sandboxed execution
- **[Optimizing Costs](./guides/optimizing-costs.md)** - Reduce API costs
- **[Extension Best Practices](./guides/extension-best-practices.md)** - Best practices for extensions
- **[Extension Distribution](./guides/extension-distribution.md)** - Share extensions
- **[Extension Versioning](./guides/extension-versioning.md)** - Version management
- **[Extension Testing](./guides/extension-testing.md)** - Test extensions
- **[Adding New Engine Provider](./guides/adding-new-engine-provider.md)** - Add custom AI providers

## Feature Documentation

### Planning & Execution

- **[Autonomous Planning](../features/planning/autonomous-planning.md)** - AI-powered plan generation
- **[Plan Execution](../features/plan-execution.md)** - Execute structured plans
- **[DAG Dependencies](../features/dag-dependencies.md)** - Dependency management

### Security & Policies

- **[Policy Engine](../features/policy-engine.md)** - Fine-grained tool execution control
- **[Constitution System](../features/constitution-system.md)** - Session-based rules
- **[Security Features](../features/security/)** - Security configuration and best practices

### Advanced Features

- **[Workflow Behaviors](../features/workflow-behaviors.md)** - Dynamic execution control
- **[Checkpointing](../features/checkpointing.md)** - Save and resume workflows
- **[Thinking Mode](../features/thinking-mode.md)** - Deep reasoning support
- **[Context Files](../features/context-files.md)** - Persistent agent instructions
- **[Context Caching](../features/context-caching.md)** - Performance optimization
- **[Session Analytics](../features/session-analytics.md)** - Track costs and performance
- **[Sandboxing](../features/sandboxing.md)** - Safe execution environment

## Integration Guides

### Extensions

- **[Extension System](../extensions/README.md)** - Package and share components
- **[Extension Quickstart](../extensions/quickstart.md)** - Get started with extensions
- **[Creating Extensions](../extensions/creating-extensions.md)** - Build your own extensions
- **[Extension Marketplace](../extensions/marketplace.md)** - Discover and share extensions

### MCP Integration

- **[MCP User Guide](../mcp/user-guide.md)** - Model Context Protocol integration
- **[MCP Configuration](../mcp/configuration.md)** - Configure MCP servers
- **[MCP Tools](../mcp/tools.md)** - Available MCP tools

### Self-Hosted Models

- **[Self-Hosted Models](../self-hosted/README.md)** - Run local AI models
- **[Ollama Setup](../self-hosted/setup/ollama.md)** - Set up Ollama
- **[vLLM Setup](../self-hosted/setup/vllm.md)** - Set up vLLM
- **[LocalAI Setup](../self-hosted/setup/localai.md)** - Set up LocalAI

## CLI Reference

- **[CLI Documentation](../cli/README.md)** - Complete CLI reference
- **[CLI Commands](../cli/commands/)** - All available commands
- **[CLI Configuration](../cli/configuration.md)** - Configure CLI behavior

## Roadmap & Vision

- **[Roadmap](../roadmap/index.md)** - Open-source roadmap and vision
- **[Vision & Innovation](../roadmap/vision.md)** - Long-term vision and innovations
- **[Technical Architecture](../roadmap/technical-architecture.md)** - Technical roadmap

## Best Practices

### Agent Design

- Use clear, specific prompts
- Define agent roles and responsibilities
- Configure appropriate models for tasks
- Use persona system for cost optimization

### Orchestration

- Write natural, descriptive requests
- Let orchestrator select agents automatically
- Use multi-agent workflows for complex tasks

### Memory & Context

- Leverage plan-scoped memory
- Use context sources for external information
- Build on previous agent outputs

### Security

- Configure policies for tool execution
- Use approval modes appropriately
- Set up session constitutions for sensitive tasks

## Troubleshooting

Common issues and solutions:

- **Agent not found**: Check agent discovery and configuration
- **Orchestration not working**: Verify API keys and configuration
- **Memory issues**: Check memory storage and retrieval
- **Performance problems**: Review session analytics and optimize

For detailed troubleshooting, see:
- **[Orchestration Troubleshooting](./orchestration-troubleshooting.md)**
- **[CLI Troubleshooting](../cli/troubleshooting.md)**
- **[Self-Hosted Troubleshooting](../self-hosted/troubleshooting.md)**

## Next Steps

- **[Developer Guide](../developer-guide/overview.md)** - Extend Radium's capabilities
- **[API Reference](../api/overview.md)** - Complete API documentation
- **[Examples](../examples/)** - Example workflows and configurations

---

**Ready to build?** Start with [Agent Configuration](./agent-configuration.md) or explore [Orchestration](./orchestration.md) for intelligent task routing.

