---
id: "marketplace"
title: "Extension Marketplace"
sidebar_label: "Extension Marketplace"
---

# Extension Marketplace

Complete guide to using the Radium extension marketplace for discovering, installing, and publishing extensions.

## Table of Contents

- [Overview](#overview)
- [Discovering Extensions](#discovering-extensions)
- [Installing from Marketplace](#installing-from-marketplace)
- [Publishing Extensions](#publishing-extensions)
- [Ratings and Reviews](#ratings-and-reviews)
- [Marketplace Features](#marketplace-features)
- [Best Practices](#best-practices)

## Overview

The Radium extension marketplace is a centralized repository where you can:
- **Discover** community-contributed extensions
- **Install** extensions with a single command
- **Publish** your own extensions for others to use
- **Rate and review** extensions
- **Track** extension popularity and downloads

## Discovering Extensions

### Browse Popular Extensions

Browse the most popular extensions:

```bash
rad extension browse
```

This shows:
- Most downloaded extensions
- Highest rated extensions
- Recently published extensions
- Featured extensions

**Example Output:**

```bash
$ rad extension browse
Popular Extensions:
┌──────────────────────────┬─────────┬─────────┬──────────────┐
│ Name                     │ Version │ Rating │ Downloads    │
├──────────────────────────┼─────────┼─────────┼──────────────┤
│ github-integration       │ 2.1.0   │ ⭐ 4.8  │ 1,234        │
│ code-review-tools        │ 1.5.0   │ ⭐ 4.6  │ 987          │
│ database-mcp-server      │ 1.2.0   │ ⭐ 4.5  │ 756          │
└──────────────────────────┴─────────┴─────────┴──────────────┘
```

### Search Marketplace

Search for extensions by name, description, or tags:

```bash
# Search all sources
rad extension search "github"

# Search only marketplace
rad extension search "github" --marketplace-only

# JSON output
rad extension search "github" --marketplace-only --json
```

**Search Tips:**
- Use specific keywords for better results
- Search by category: "development", "testing", "deployment"
- Search by technology: "python", "rust", "typescript"
- Search by feature: "code-review", "linting", "formatting"

**Example:**

```bash
$ rad extension search "code review"
Found 5 extensions in marketplace:

1. code-review-tools (1.5.0) ⭐ 4.6
   Code review agent prompts and tools
   Downloads: 987
   Tags: code-review, development, quality

2. advanced-code-review (2.0.0) ⭐ 4.8
   Advanced code review tools with AI assistance
   Downloads: 1,234
   Tags: code-review, ai, automation

3. code-review-assistant (1.2.0) ⭐ 4.3
   AI-powered code review assistant
   Downloads: 456
   Tags: code-review, ai, assistant
```

## Installing from Marketplace

### Install by Name

Install an extension directly from the marketplace by name:

```bash
rad extension install extension-name
```

The CLI automatically detects if the name refers to a marketplace extension and downloads it.

**Example:**

```bash
$ rad extension install github-integration
Found extension 'github-integration' in marketplace
Downloading from: https://marketplace.radium.ai/extensions/github-integration.tar.gz
Installing extension from: https://marketplace.radium.ai/extensions/github-integration.tar.gz
Validating extension package...
✓ Extension 'github-integration' installed successfully
  Version: 2.1.0
  Description: GitHub API integration for Radium
```

### Install with Dependencies

Extensions can declare dependencies that are automatically installed:

```bash
rad extension install my-extension --install-deps
```

**Example:**

```bash
$ rad extension install advanced-code-review --install-deps
Installing extension 'advanced-code-review'...
Installing dependency 'code-review-base'...
Installing dependency 'ai-tools'...
✓ All dependencies installed successfully
✓ Extension 'advanced-code-review' installed successfully
```

## Publishing Extensions

### Prerequisites

Before publishing:

1. **Complete your extension:**
   - All components tested and working
   - Manifest properly configured
   - README documentation included

2. **Get a marketplace API key:**
   - Contact marketplace administrators
   - Or register at marketplace.radium.ai

3. **Sign your extension** (recommended):
   ```bash
   rad extension sign ./my-extension --generate-key
   ```

### Publish Your Extension

Publish to the marketplace:

```bash
# With API key from environment
export RADIUM_MARKETPLACE_API_KEY="your-api-key"
rad extension publish ./my-extension

# Or provide API key directly
rad extension publish ./my-extension --api-key YOUR_API_KEY

# With automatic signing
rad extension publish ./my-extension --api-key YOUR_API_KEY --sign-with-key ./private.key
```

**Publishing Process:**

1. **Validation**: Extension structure and manifest are validated
2. **Signing**: Extension is signed (if key provided)
3. **Packaging**: Extension is packaged as `.tar.gz`
4. **Upload**: Package is uploaded to marketplace
5. **Indexing**: Extension is indexed and made searchable

**Example:**

```bash
$ rad extension publish ./my-extension --api-key MY_API_KEY
Validating extension...
✓ Extension structure valid
✓ Manifest valid
Signing extension...
✓ Extension signed
Packaging extension...
✓ Extension packaged
Uploading to marketplace...
✓ Extension published successfully
  Name: my-extension
  Version: 1.0.0
  URL: https://marketplace.radium.ai/extensions/my-extension
```

### Update Published Extension

To publish an update:

1. **Increment version** in `radium-extension.json`:
   ```json
   {
     "version": "1.1.0"
   }
   ```

2. **Re-sign** if you signed the original:
   ```bash
   rad extension sign ./my-extension --key-file ./private.key
   ```

3. **Publish again:**
   ```bash
   rad extension publish ./my-extension --api-key YOUR_API_KEY
   ```

The marketplace will:
- Keep the previous version available
- Update the latest version
- Show version history
- Notify users of updates (if enabled)

## Ratings and Reviews

### View Extension Ratings

Ratings are shown when browsing or searching:

```bash
rad extension browse
rad extension search "query"
rad extension info extension-name
```

**Rating Display:**
- ⭐ 5.0: Excellent (5 stars)
- ⭐ 4.5: Very Good (4.5 stars)
- ⭐ 4.0: Good (4 stars)
- ⭐ 3.5: Average (3.5 stars)
- ⭐ 3.0: Below Average (3 stars)

### Rate an Extension

Rate extensions you've used (feature coming soon):

```bash
# Future command
rad extension rate extension-name --rating 5 --comment "Great extension!"
```

## Marketplace Features

### Extension Categories

Extensions are organized by category:

- **Development**: Code review, linting, formatting tools
- **Testing**: Test generation, test runners
- **Deployment**: CI/CD, deployment automation
- **Integration**: Third-party service integrations
- **Productivity**: Workflow automation, productivity tools
- **Custom**: Custom agent configurations

### Extension Tags

Extensions can have tags for better discoverability:

```json
{
  "metadata": {
    "tags": ["code-review", "development", "python", "quality"]
  }
}
```

Common tags:
- Technology: `python`, `rust`, `typescript`, `javascript`
- Category: `code-review`, `testing`, `deployment`, `integration`
- Feature: `ai`, `automation`, `linting`, `formatting`

### Download Statistics

View download statistics:

```bash
rad extension info extension-name
```

Shows:
- Total downloads
- Recent downloads
- Version distribution
- Popularity trends

### Extension Verification

Verified extensions are marked with a badge:

- ✅ **Verified**: Published by trusted authors
- 🔒 **Signed**: Cryptographically signed
- ⭐ **Popular**: High download count and ratings

## Best Practices

### For Extension Users

1. **Check ratings and reviews** before installing
2. **Verify signatures** for security
3. **Read documentation** in extension README
4. **Check dependencies** before installing
5. **Report issues** to extension authors

### For Extension Publishers

1. **Write clear descriptions** with use cases
2. **Include comprehensive README** files
3. **Tag appropriately** for discoverability
4. **Sign extensions** for authenticity
5. **Version properly** following semantic versioning
6. **Respond to reviews** and issues
7. **Keep extensions updated** with bug fixes

### Versioning Guidelines

Follow semantic versioning:

- **MAJOR** (2.0.0): Breaking changes
- **MINOR** (1.1.0): New features, backward compatible
- **PATCH** (1.0.1): Bug fixes, backward compatible

**Examples:**
- `1.0.0` → `1.0.1`: Bug fix
- `1.0.0` → `1.1.0`: New feature added
- `1.0.0` → `2.0.0`: Breaking change

### Description Best Practices

Write effective descriptions:

- **Be specific**: What does the extension do?
- **Include use cases**: When would someone use this?
- **List features**: What capabilities does it provide?
- **Mention requirements**: Dependencies, system requirements
- **Provide examples**: Show how to use it

**Good Example:**
```
Code review agent prompts and tools for Python, Rust, and TypeScript projects.
Includes automated review suggestions, best practice checks, and security scanning.
Perfect for teams wanting consistent code review standards.
```

**Bad Example:**
```
Code review stuff.
```

## Security Considerations

### Verify Signatures

Always verify extension signatures:

```bash
rad extension verify extension-name
```

### Trusted Publishers

Add trusted publisher keys:

```bash
rad extension trust-key add --name "Publisher Name" --key-file ./public.key
```

### Review Before Installing

- Check extension source code if available
- Review ratings and user feedback
- Verify publisher identity
- Check for security advisories

## Troubleshooting

### Cannot Connect to Marketplace

**Issue**: Cannot reach marketplace server

**Solutions:**
- Check internet connection
- Verify marketplace URL is accessible
- Check firewall/proxy settings
- Try again later (server may be down)

### Authentication Errors

**Issue**: API key authentication fails

**Solutions:**
- Verify API key is correct
- Check API key hasn't expired
- Ensure you have publishing permissions
- Contact marketplace administrators

### Extension Not Found

**Issue**: Extension not found in marketplace

**Solutions:**
- Verify extension name is correct
- Check if extension is published
- Try searching instead of installing by name
- Check if extension was removed

### Publishing Fails

**Issue**: Publishing fails with validation errors

**Solutions:**
- Check all required manifest fields are present
- Verify extension structure is correct
- Ensure component paths match actual files
- Test installation locally first

## Next Steps

- [User Guide](user-guide.md) - Complete extension usage guide
- [Publishing Guide](publishing-guide.md) - Detailed publishing instructions
- [Creating Extensions](creating-extensions.md) - Learn to create extensions
- [Architecture](architecture.md) - Technical details

