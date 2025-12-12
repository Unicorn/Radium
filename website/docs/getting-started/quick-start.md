---
id: "quick-start"
title: "Quick Start"
sidebar_label: "Quick Start"
---

# Quick Start Guide

Get up and running with Radium in minutes. This guide will walk you through creating your first agent and running your first orchestrated workflow.

## Prerequisites

- Radium installed ([Installation Guide](./installation.md))
- At least one AI provider API key:
  - Google AI API key (for Gemini)
  - Anthropic API key (for Claude)
  - OpenAI API key (for GPT models)

## Step 1: Set Up Your API Key

```bash
# Set your API key (choose one)
export GOOGLE_AI_API_KEY="your-key-here"
# or
export ANTHROPIC_API_KEY="your-key-here"
# or
export OPENAI_API_KEY="your-key-here"
```

Add this to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.) to persist it.

## Step 2: Initialize a Workspace

```bash
# Navigate to your project directory
cd my-project

# Initialize Radium workspace
rad init
```

This creates a `.radium/` directory with workspace configuration.

## Step 3: Create Your First Agent

Create a simple agent configuration:

```bash
# Create agent directory structure
mkdir -p agents/my-first-agent

# Create agent configuration
cat > agents/my-first-agent/agent.toml << 'EOF'
[agent]
id = "my-first-agent"
name = "My First Agent"
description = "A simple agent for getting started"
prompt_path = "prompts/my-first-agent.md"
engine = "gemini"
model = "gemini-2.0-flash-exp"
EOF

# Create a simple prompt
mkdir -p prompts
cat > prompts/my-first-agent.md << 'EOF'
# My First Agent

You are a helpful coding assistant. Help users write, debug, and understand code.

## Instructions
- Provide clear, concise explanations
- Write well-documented code
- Suggest best practices
EOF
```

## Step 4: Test Your Agent

```bash
# List your agents
rad agents list

# Run a simple command
rad step my-first-agent "Write a hello world function in Python"
```

## Step 5: Use Orchestration (TUI)

For a more interactive experience, use the TUI with orchestration:

```bash
# Start the TUI
rad tui

# In the TUI, just type naturally:
# "I need help writing a REST API endpoint"
```

The orchestrator will automatically:
- Analyze your request
- Select the appropriate agent
- Execute the task
- Return results

## Step 6: Create a Plan

Generate a structured plan from a natural language specification:

```bash
# Create a simple spec file
cat > my-feature.md << 'EOF'
# Build a REST API

Create a REST API with:
- User authentication
- CRUD operations for todos
- JSON responses
EOF

# Generate a plan
rad plan my-feature.md

# Execute the plan
rad craft REQ-001
```

## Next Steps

- **[Core Concepts](./core-concepts.md)** - Understand agents, orchestration, and workflows
- **[Configuration Guide](./configuration.md)** - Customize Radium settings
- **[User Guide](../user-guide/overview.md)** - Learn advanced features
- **[Agent Configuration](../user-guide/agent-configuration.md)** - Create sophisticated agents

## Common First Tasks

### Create a Code Review Agent

```bash
rad agents create code-reviewer "Code Reviewer" \
  --prompt "You are a senior code reviewer. Review code for bugs, security issues, and best practices."
```

### Set Up Orchestration

Orchestration is enabled by default in TUI. Configure it:

```bash
# View orchestration config
cat ~/.radium/orchestration.toml

# Or use TUI commands:
# /orchestrator - Show status
# /orchestrator switch gemini - Change provider
```

### Create an Extension

Package your agent as an extension:

```bash
rad extension create my-extension \
  --author "Your Name" \
  --description "My custom extension"
```

## Troubleshooting

### Agent Not Found

```bash
# Verify agent is discovered
rad agents list

# Check agent configuration
rad agents info my-first-agent

# Validate configuration
rad agents validate
```

### API Key Issues

```bash
# Check authentication status
rad auth status

# Test with a specific provider
rad engines health gemini
```

### TUI Not Starting

```bash
# Check if server is running
rad status

# Start server manually if needed
rad server start
```

## Learn More

- **[Installation](./installation.md)** - Detailed installation instructions
- **[Core Concepts](./core-concepts.md)** - Deep dive into Radium concepts
- **[User Guide](../user-guide/overview.md)** - Complete user documentation
- **[Roadmap](../roadmap/index.md)** - See where Radium is heading

---

**Ready for more?** Check out the [User Guide](../user-guide/overview.md) to explore advanced features like the persona system, vibe check, and learning system.

