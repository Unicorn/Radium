---
id: "introduction"
title: "Welcome to Radium"
sidebar_label: "Welcome to Radium"
---

# Welcome to Radium

**Radium** is a next-generation agentic orchestration platform that enables developers to build, deploy, and manage multi-agent AI systems with ease.

## What is Radium?

Radium provides a comprehensive framework for:

- **Multi-Agent Orchestration** - Coordinate multiple AI agents working together toward complex goals
- **Policy-Driven Governance** - Apply context-aware policies to ensure safe and controlled agent behavior
- **Flexible Integration** - Support for Claude, Gemini, OpenAI, and self-hosted models (Ollama, vLLM, LocalAI)
- **Developer-Friendly CLI** - Powerful command-line tools for agent management, workflows, and debugging
- **Enterprise-Ready** - Built in Rust for performance, safety, and reliability

## Quick Links

### For Users
- [Installation](./getting-started/installation.md) - Get Radium up and running
- [Quick Start](./getting-started/quick-start.md) - Create your first agent in minutes
- [User Guide](./user-guide/overview.md) - Learn how to configure agents and workflows

### For Developers
- [Developer Guide](./developer-guide/overview.md) - Extend Radium's capabilities
- [API Reference](./api/overview.md) - Complete Rust API documentation
- [Architecture](./developer-guide/architecture/overview.md) - Understand how Radium works
- [Roadmap](./roadmap/index.md) - View our open-source roadmap and vision

## Key Features

### 🤖 Multi-Agent Orchestration
Coordinate multiple specialized agents, distribute tasks intelligently, and manage complex workflows with built-in swarm control.

### 🛡️ Policy Engine
Apply fine-grained policies to control agent behavior based on context, user permissions, and safety requirements.

### 🔌 Universal Model Support
- **Cloud Providers**: Claude (Anthropic), Gemini (Google), OpenAI
- **Self-Hosted**: Ollama, vLLM, LocalAI
- **Extensible**: Add custom model providers

### ⚡ Performance & Safety
Built in Rust for:
- Memory safety without garbage collection
- Concurrency without data races
- Zero-cost abstractions
- Blazing fast execution

### 🔧 Developer Tools
- Comprehensive CLI for all operations
- Interactive TUI for real-time monitoring
- Desktop app for visual workflow management
- MCP (Model Context Protocol) integration

## Getting Started

The fastest way to get started is:

1. **Install Radium**
   ```bash
   curl -sSf https://raw.githubusercontent.com/clay-curry/RAD/main/install.sh | sh
   ```

2. **Create your first agent**
   ```bash
   rad agent create my-first-agent
   ```

3. **Chat with your agent**
   ```bash
   rad chat my-first-agent
   ```

That's it! You're now ready to explore the full power of Radium.

## Community & Support

- **GitHub**: [clay-curry/RAD](https://github.com/clay-curry/RAD)
- **Issues**: [Report bugs or request features](https://github.com/clay-curry/RAD/issues)
- **Discussions**: [Ask questions and share ideas](https://github.com/clay-curry/RAD/discussions)

## Roadmap & Vision

Radium is evolving toward a comprehensive composable intelligence infrastructure. Check out our [open-source roadmap](./roadmap/index.md) to see our vision, technical architecture plans, and implementation milestones.

## What's Next?

Continue to [Installation](./getting-started/installation.md) to set up Radium on your system, or explore our [Roadmap](./roadmap/index.md) to see where we're heading.
