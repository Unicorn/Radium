# GitHub Integration Extension

GitHub MCP server configurations and agents for managing GitHub repositories and pull requests.

## Installation

```bash
rad extension install ./examples/extensions/github-integration
```

## Components

### MCP Servers

- **github-api** - GitHub API MCP server for repository management
  - Requires `GITHUB_TOKEN` environment variable
  - Provides tools for PR management, issue tracking, and repository operations

### Agents

- **github-pr-agent** - Agent for managing GitHub pull requests
  - Uses GitHub MCP tools to interact with repositories
  - Helps create, review, and manage pull requests

## Setup

Before using this extension, set up your GitHub token:

```bash
export GITHUB_TOKEN=your_github_personal_access_token
```

Or add it to your shell configuration file (`.bashrc`, `.zshrc`, etc.).

## Usage

After installation, the MCP server configuration will be available to agents:

```bash
# List MCP servers
rad mcp list

# The github-pr-agent can use GitHub MCP tools
# to interact with GitHub repositories
```

## MCP Server Configuration

The extension includes a GitHub API MCP server configuration that uses the official `@modelcontextprotocol/server-github` package.

## Example

The `github-pr-agent` can be used to:
- Create pull requests with descriptions
- Review pull request changes
- Comment on pull requests
- Manage PR labels and assignees

## See Also

- [Extension System Guide](../../../docs/extensions/README.md)
- [MCP Configuration](../../../docs/mcp/configuration.md)

