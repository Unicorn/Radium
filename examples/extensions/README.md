# Extension Examples

This directory contains example Radium extensions demonstrating different use cases and patterns. These examples serve as templates and learning resources for creating your own extensions.

## Quick Start

1. **Choose an example** based on your needs (see [Choosing an Example](#choosing-an-example))
2. **Install the example** to see it in action
3. **Study the structure** to understand how it's organized
4. **Customize it** to create your own extension

## Examples

### hello-world

**Purpose**: Minimal extension example  
**Use Case**: Learning extension structure, first extension  
**Components**: Prompts only  
**Complexity**: ⭐ Beginner

A minimal extension with a single agent prompt. Perfect for understanding the basic structure of Radium extensions.

**Installation:**
```bash
rad extension install ./examples/extensions/hello-world
```

**What it demonstrates:**
- Basic extension structure
- Minimal manifest configuration
- Single prompt component
- Simple directory layout

**See also:** [hello-world README](hello-world/README.md)

### code-review-agents

**Purpose**: Multi-agent extension  
**Use Case**: Language-specific code review agents  
**Components**: Multiple prompts (categorized)  
**Complexity**: ⭐⭐ Intermediate

Demonstrates how to create multiple specialized agents in a single extension. Includes Rust, TypeScript, and Python code reviewers.

**Installation:**
```bash
rad extension install ./examples/extensions/code-review-agents
```

**What it demonstrates:**
- Multiple prompts in one extension
- Categorized organization (subdirectories)
- Language-specific agent templates
- Glob patterns for component discovery

**Components:**
- `prompts/review/python-reviewer.md` - Python code review agent
- `prompts/review/rust-reviewer.md` - Rust code review agent
- `prompts/review/typescript-reviewer.md` - TypeScript code review agent

**See also:** [code-review-agents README](code-review-agents/README.md)

### github-integration

**Purpose**: MCP server integration  
**Use Case**: GitHub API integration with MCP  
**Components**: MCP servers, prompts  
**Complexity**: ⭐⭐ Intermediate

Shows how to package MCP server configurations and create agents that use MCP tools. Includes GitHub API MCP server and PR management agent.

**Installation:**
```bash
rad extension install ./examples/extensions/github-integration
```

**What it demonstrates:**
- MCP server configuration packaging
- Agent prompts that use MCP tools
- Integration between extensions and MCP
- Environment variable configuration

**Components:**
- `mcp/github-api.json` - GitHub MCP server configuration
- `prompts/github-pr-agent.md` - PR management agent

**See also:** [github-integration README](github-integration/README.md)

### custom-workflows

**Purpose**: Workflow templates  
**Use Case**: Reusable workflow templates  
**Components**: Workflow templates  
**Complexity**: ⭐⭐ Intermediate

Demonstrates creating reusable workflow templates for common tasks like code review and deployment.

**Installation:**
```bash
rad extension install ./examples/extensions/custom-workflows
```

**What it demonstrates:**
- Workflow template packaging
- Reusable workflow definitions
- Common development workflows

**See also:** [custom-workflows README](custom-workflows/README.md)

### complete-toolkit

**Purpose**: Full-featured extension  
**Use Case**: All component types, advanced patterns  
**Components**: All types (prompts, MCP, commands, hooks, workflows)  
**Complexity**: ⭐⭐⭐ Advanced

A comprehensive example showing all component types, categorized organization, nested commands, and dependency management.

**Installation:**
```bash
rad extension install ./examples/extensions/complete-toolkit --install-deps
```

**What it demonstrates:**
- All component types in one extension
- Categorized organization (subdirectories)
- Nested command structures
- Dependency declarations
- Advanced manifest configuration

**Components:**
- **Prompts**: Developer agent, React framework agent
- **MCP Servers**: Database tools configuration
- **Commands**: Build command, production deployment command
- **Hooks**: Metrics collection hook
- **Workflows**: Full-stack development workflow

**Dependencies:**
- `hello-world` (automatically installed with `--install-deps`)

**See also:** [complete-toolkit README](complete-toolkit/README.md)

## Choosing an Example

Choose an example based on your needs:

| If you want to... | Use this example |
|-------------------|------------------|
| Learn the basics | `hello-world` |
| Create multiple agents | `code-review-agents` |
| Integrate MCP servers | `github-integration` |
| Create workflow templates | `custom-workflows` |
| See all features | `complete-toolkit` |

### By Component Type

- **Prompts only**: `hello-world`, `code-review-agents`
- **MCP servers**: `github-integration`
- **Commands**: `complete-toolkit`
- **Hooks**: `complete-toolkit`
- **Workflows**: `custom-workflows`, `complete-toolkit`
- **All components**: `complete-toolkit`

### By Complexity

- **Beginner**: `hello-world`
- **Intermediate**: `code-review-agents`, `github-integration`, `custom-workflows`
- **Advanced**: `complete-toolkit`

## Testing Examples

All examples can be installed and tested:

```bash
# Install an example
rad extension install ./examples/extensions/hello-world

# List installed extensions
rad extension list

# Get extension info
rad extension info hello-world

# Verify extension structure
rad extension info hello-world --verbose

# Uninstall
rad extension uninstall hello-world
```

### Verifying Installation

After installing an example, verify it works:

```bash
# Check extension is installed
rad extension list | grep hello-world

# View extension details
rad extension info hello-world

# Check components are discoverable
# (depends on component type - prompts, MCP, commands, etc.)
```

## Customization Guide

### Using Examples as Templates

1. **Copy an example** that matches your needs:
   ```bash
   cp -r examples/extensions/hello-world my-extension
   ```

2. **Edit the manifest** (`radium-extension.json`):
   ```json
   {
     "name": "my-extension",
     "version": "1.0.0",
     "description": "My custom extension",
     "author": "Your Name",
     ...
   }
   ```

3. **Add your components**:
   - Add prompts to `prompts/`
   - Add MCP configs to `mcp/`
   - Add commands to `commands/`
   - Add hooks to `hooks/`

4. **Test locally**:
   ```bash
   rad extension install ./my-extension
   rad extension list
   ```

5. **Update version** when making changes:
   ```json
   {
     "version": "1.1.0"
   }
   ```

### Best Practices from Examples

- **Use descriptive names**: Clear, lowercase with dashes
- **Organize with subdirectories**: Group related components
- **Document everything**: Include comprehensive READMEs
- **Version properly**: Follow semantic versioning
- **Test before sharing**: Verify installation works
- **Sign extensions**: Use cryptographic signatures for distribution

## Extension Signing

Examples can be signed for distribution:

```bash
# Generate a keypair
rad extension sign ./examples/extensions/hello-world --generate-key

# This creates:
# - private.key (keep secure!)
# - public.key (share with users)

# Sign the extension
rad extension sign ./examples/extensions/hello-world --key-file ./private.key

# Verify signature
rad extension verify hello-world --key-file ./public.key
```

See the [Publishing Guide](../../docs/extensions/publishing-guide.md) for more details.

## Example Comparison

| Example | Prompts | MCP | Commands | Hooks | Workflows | Dependencies |
|---------|---------|-----|----------|-------|-----------|--------------|
| hello-world | ✅ | ❌ | ❌ | ❌ | ❌ | None |
| code-review-agents | ✅ (3) | ❌ | ❌ | ❌ | ❌ | None |
| github-integration | ✅ | ✅ | ❌ | ❌ | ❌ | None |
| custom-workflows | ❌ | ❌ | ❌ | ❌ | ✅ | None |
| complete-toolkit | ✅ | ✅ | ✅ | ✅ | ✅ | hello-world |

## Documentation

- [Extension System Guide](../../docs/extensions/README.md) - Overview and user guide
- [User Guide](../../docs/extensions/user-guide.md) - Complete usage documentation
- [Creating Extensions](../../docs/extensions/creating-extensions.md) - How to create extensions
- [Publishing Guide](../../docs/extensions/publishing-guide.md) - Publishing to marketplace
- [API Reference](../../docs/extensions/api-reference.md) - Developer API documentation
- [Integration Guide](../../docs/extensions/integration-guide.md) - Integration examples
- [Architecture Documentation](../../docs/extensions/architecture.md) - Technical architecture

## Contributing Examples

If you create a useful example extension, consider:

1. **Following the structure** of existing examples
2. **Including a comprehensive README** with:
   - Purpose and use case
   - Installation instructions
   - Component descriptions
   - Usage examples
   - Customization guide
3. **Testing thoroughly** before sharing
4. **Documenting dependencies** and requirements
5. **Following best practices** from existing examples

## Troubleshooting

### Extension Won't Install

- Check manifest is valid JSON
- Verify all required fields are present
- Ensure extension name is unique
- Check file permissions

### Components Not Loading

- Verify component directories exist
- Check glob patterns match file paths
- Ensure file formats are correct (`.md`, `.json`, `.toml`)
- Review component paths in manifest

### See Also

- [Troubleshooting Guide](../../docs/extensions/user-guide.md#troubleshooting)
- [Creating Extensions Guide](../../docs/extensions/creating-extensions.md#troubleshooting)

