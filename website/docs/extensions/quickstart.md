---
id: "quickstart"
title: "Extension System Quickstart"
sidebar_label: "Extension System Quickstart"
---

# Extension System Quickstart

Get started with Radium extensions in 10 minutes! This guide will walk you through creating, installing, and using your first extension.

## Prerequisites

- Radium CLI installed and configured
- Basic familiarity with command-line tools

## Step 1: Create Your First Extension

Let's create a simple extension with a custom prompt:

```bash
rad extension create my-first-extension --author "Your Name" --description "My first Radium extension"
```

This creates a directory structure like:

```
my-first-extension/
├── radium-extension.json
├── prompts/
├── mcp/
├── commands/
├── hooks/
└── README.md
```

## Step 2: Add a Component

Let's add a simple prompt to your extension:

```bash
# Create a prompt file
cat > my-first-extension/prompts/helper-agent.md << 'EOF'
# Helper Agent

A helpful assistant agent that provides guidance and answers questions.

## Capabilities

- Answer questions
- Provide guidance
- Help with tasks
EOF
```

## Step 3: Update the Manifest

Edit `my-first-extension/radium-extension.json` to include your prompt:

```json
{
  "name": "my-first-extension",
  "version": "1.0.0",
  "description": "My first Radium extension",
  "author": "Your Name",
  "components": {
    "prompts": ["prompts/*.md"]
  },
  "dependencies": []
}
```

## Step 4: Install Your Extension

Install the extension locally:

```bash
rad extension install ./my-first-extension
```

You should see:

```
✓ Extension 'my-first-extension' installed successfully
  Version: 1.0.0
  Description: My first Radium extension
```

## Step 5: Verify Installation

List your installed extensions:

```bash
rad extension list
```

You should see your extension in the list!

## Step 6: Use Your Extension

Your prompt is now available for use in Radium. The extension system automatically discovers and loads components from installed extensions.

## Next Steps

- Add more components (MCP servers, commands, hooks)
- Share your extension with others
- Browse the marketplace for community extensions
- Learn about [signing extensions](creating-extensions.md#signing-extensions)
- Read the [full documentation](README.md)

## Troubleshooting

### Extension Not Found

If your extension isn't appearing:

1. Check it's installed: `rad extension list`
2. Verify the manifest file exists: `radium-extension.json`
3. Ensure the extension name matches exactly (case-sensitive)

### Component Not Loading

If components aren't being discovered:

1. Check component directories exist
2. Verify glob patterns in manifest match file paths
3. Ensure file formats are correct (`.md` for prompts, `.json` for MCP, `.toml` for commands)

For more help, see the [troubleshooting guide](README.md#troubleshooting).

