# Extension Examples

This directory contains example Radium extensions demonstrating different use cases and patterns.

## Examples

### hello-world

**Purpose**: Minimal extension example  
**Use Case**: Learning extension structure, first extension  
**Components**: Prompts only

A minimal extension with a single agent prompt. Perfect for understanding the basic structure of Radium extensions.

```bash
rad extension install ./examples/extensions/hello-world
```

### code-review-agents

**Purpose**: Multi-agent extension  
**Use Case**: Language-specific code review agents  
**Components**: Multiple prompts (categorized)

Demonstrates how to create multiple specialized agents in a single extension. Includes Rust, TypeScript, and Python code reviewers.

```bash
rad extension install ./examples/extensions/code-review-agents
```

### github-integration

**Purpose**: MCP server integration  
**Use Case**: GitHub API integration with MCP  
**Components**: MCP servers, prompts

Shows how to package MCP server configurations and create agents that use MCP tools. Includes GitHub API MCP server and PR management agent.

```bash
rad extension install ./examples/extensions/github-integration
```

### custom-workflows

**Purpose**: Workflow templates  
**Use Case**: Reusable workflow templates  
**Components**: Workflow templates

Demonstrates creating reusable workflow templates for common tasks like code review and deployment.

```bash
rad extension install ./examples/extensions/custom-workflows
```

### complete-toolkit

**Purpose**: Full-featured extension  
**Use Case**: All component types, advanced patterns  
**Components**: All types (prompts, MCP, commands, hooks, workflows)

A comprehensive example showing all component types, categorized organization, nested commands, and dependency management.

```bash
rad extension install ./examples/extensions/complete-toolkit --install-deps
```

## Choosing an Example

- **New to extensions?** Start with `hello-world`
- **Creating multiple agents?** See `code-review-agents`
- **Integrating MCP servers?** Check `github-integration`
- **Creating workflows?** Look at `custom-workflows`
- **Advanced patterns?** Study `complete-toolkit`

## Testing Examples

All examples can be installed and tested:

```bash
# Install an example
rad extension install ./examples/extensions/hello-world

# List installed extensions
rad extension list

# Get extension info
rad extension info hello-world

# Uninstall
rad extension uninstall hello-world
```

## Documentation

- [Extension System Guide](../../docs/extensions/README.md)
- [Creating Extensions](../../docs/extensions/creating-extensions.md)
- [Architecture Documentation](../../docs/extensions/architecture.md)

