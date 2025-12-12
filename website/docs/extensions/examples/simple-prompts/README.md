---
id: "README"
title: "Simple Prompts Extension"
sidebar_label: "Simple Prompts Extension"
---

# Simple Prompts Extension

This is a minimal example extension that demonstrates how to package custom agent prompts.

## Structure

```
simple-prompts/
├── radium-extension.json
├── prompts/
│   └── code-reviewer.md
└── README.md
```

## Installation

```bash
rad extension install ./docs/extensions/examples/simple-prompts
```

## Components

### Prompts

- `code-reviewer.md` - A prompt template for code review agents

## Usage

After installation, the prompts will be discoverable by the agent system:

```bash
rad agents list
```

The prompts can be referenced in agent configurations using their filename (without extension).

