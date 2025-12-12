---
id: "extension-best-practices"
title: "Extension Best Practices"
sidebar_label: "Extension Best Practices"
---

# Extension Best Practices

This guide provides best practices for creating high-quality Radium extensions.

## Naming Conventions

### Extension Names

- **Use kebab-case**: `my-extension`, `code-review-agents`
- **Be descriptive**: Names should indicate the extension's purpose
- **Avoid conflicts**: Consider prefixing with your username or organization
- **Start with a letter**: Must start with a letter (not a number)
- **No spaces**: Use dashes or underscores instead

**Good examples**:
- `github-integration`
- `rust-code-reviewer`
- `custom-workflows`

**Bad examples**:
- `My Extension` (spaces, uppercase)
- `123-extension` (starts with number)
- `extension` (too generic)

### Component Names

- **Prompts**: Use descriptive names like `code-review-agent.md`
- **MCP Servers**: Use server purpose, e.g., `github-api.json`
- **Commands**: Use action verbs, e.g., `deploy-production.toml`
- **Hooks**: Use descriptive names, e.g., `metrics-hook.toml`

## Component Organization

### Directory Structure

Organize components logically:

```
my-extension/
├── radium-extension.json
├── prompts/
│   ├── agents/          # Agent prompts
│   │   └── reviewer.md
│   └── frameworks/      # Framework-specific prompts
│       └── react.md
├── mcp/
│   └── api-server.json
├── commands/
│   ├── build.toml
│   └── deploy/
│       └── production.toml
└── hooks/
    └── logging.toml
```

### Categorization

- **Group related components**: Use subdirectories for organization
- **Consistent naming**: Follow a consistent naming pattern
- **Clear hierarchy**: Keep directory depth reasonable (2-3 levels max)

## Manifest Design

### Required Fields

Always provide:
- **name**: Clear, descriptive extension name
- **version**: Semantic version (start with 1.0.0)
- **description**: Brief but informative description
- **author**: Your name or organization

### Component Paths

- **Use glob patterns**: `prompts/*.md` instead of listing every file
- **Be specific**: `prompts/agents/*.md` is better than `prompts/**/*`
- **Document patterns**: Explain glob patterns in README if complex

### Dependencies

- **Declare dependencies**: List all required extensions
- **Version constraints**: Specify version requirements if needed (future)
- **Minimize dependencies**: Only declare truly required dependencies

## Dependency Management

### When to Declare Dependencies

Declare a dependency when:
- Your extension requires another extension to function
- Components reference or extend another extension's components
- Your extension provides enhancements to another extension

### Dependency Best Practices

- **Minimize dependencies**: Fewer dependencies = easier installation
- **Document why**: Explain why each dependency is needed
- **Test with dependencies**: Ensure installation works with `--install-deps`
- **Handle missing dependencies**: Provide clear error messages

## Backward Compatibility

### Versioning Strategy

- **Semantic versioning**: Follow semver (MAJOR.MINOR.PATCH)
- **Breaking changes**: Increment MAJOR version
- **New features**: Increment MINOR version
- **Bug fixes**: Increment PATCH version

### Maintaining Compatibility

- **Don't remove components**: Mark as deprecated instead
- **Additive changes**: Prefer adding new components over modifying existing
- **Document changes**: Include changelog for version updates
- **Migration guides**: Provide migration instructions for breaking changes

## Security Best Practices

### No Hardcoded Secrets

**Never include**:
- API keys or tokens
- Passwords or credentials
- Private keys or certificates
- Personal information

**Instead**:
- Use environment variables
- Document required configuration
- Provide setup instructions

### Path Validation

- **No path traversal**: Validate all file paths
- **Relative paths**: Use relative paths in manifests
- **Safe globs**: Avoid overly permissive glob patterns

### Input Validation

If your extension processes user input:
- Validate all inputs
- Sanitize file paths
- Handle errors gracefully
- Don't execute arbitrary code

## Performance Considerations

### Lazy Loading

- **Minimal dependencies**: Only include what's needed
- **Efficient globs**: Use specific glob patterns
- **Avoid large files**: Keep individual files reasonable in size

### Resource Usage

- **Document requirements**: List any system requirements
- **Optimize components**: Keep prompts and configs concise
- **Test performance**: Ensure extension doesn't slow down Radium

## Documentation

### README Requirements

Your README should include:

1. **Clear title and description**
2. **Installation instructions**
3. **Usage examples**
4. **Component documentation**
5. **Configuration options**
6. **Troubleshooting section**

### Code Comments

- **Document complex logic**: Add comments for non-obvious code
- **Explain decisions**: Document why certain approaches were chosen
- **Provide examples**: Include usage examples in comments

## Testing

### Local Testing

Before distribution:
- [ ] Test installation from local directory
- [ ] Verify all components are discoverable
- [ ] Test with different Radium versions (if possible)
- [ ] Check for conflicts with other extensions
- [ ] Validate manifest and structure

### Test Scenarios

Test these scenarios:
- Fresh installation
- Installation with dependencies
- Overwriting existing extension
- Uninstallation
- Component discovery and usage

## Error Handling

### User-Friendly Errors

- **Clear messages**: Provide actionable error messages
- **Context**: Include relevant context in errors
- **Suggestions**: Offer solutions or next steps

### Validation

- **Validate early**: Check manifest and structure before installation
- **Comprehensive checks**: Validate all components
- **Helpful errors**: Point to specific issues

## Examples and Templates

### Provide Examples

Include example usage:
- Example agent configurations
- Example workflow usage
- Example command invocations
- Example MCP server setup

### Template Extensions

Consider creating template extensions:
- Minimal template for quick starts
- Full-featured template showing all patterns
- Language-specific templates

## Community Guidelines

### Code of Conduct

- **Be respectful**: Treat all users with respect
- **Be helpful**: Provide support and answer questions
- **Be open**: Accept feedback and contributions

### Contribution Guidelines

If accepting contributions:
- Document contribution process
- Set clear expectations
- Review contributions carefully
- Maintain code quality

## See Also

- [Extension Distribution Guide](extension-distribution.md)
- [Extension Testing Guide](extension-testing.md)
- [Extension Versioning Guide](extension-versioning.md)
- [Creating Extensions](../extensions/creating-extensions.md)

