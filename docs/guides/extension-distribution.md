# Extension Distribution Guide

This guide explains how to package, host, and distribute Radium extensions to the community.

## Packaging Extensions

### Creating Archives

Extensions can be distributed as compressed archives. Supported formats:
- `.tar.gz` (recommended)
- `.zip`

#### Using tar (Linux/macOS)

```bash
# Create a tar.gz archive
tar -czf my-extension.tar.gz my-extension/

# Verify the archive
tar -tzf my-extension.tar.gz
```

#### Using zip (All platforms)

```bash
# Create a zip archive
zip -r my-extension.zip my-extension/

# Verify the archive
unzip -l my-extension.zip
```

### Archive Contents

Ensure your archive contains:
- `radium-extension.json` - Extension manifest (required)
- Component directories (`prompts/`, `mcp/`, `commands/`, `hooks/`)
- `README.md` - Extension documentation (recommended)
- Any other files referenced in the manifest

**Do not include:**
- `.git/` directory
- `node_modules/` or build artifacts
- Temporary files
- IDE-specific files (`.vscode/`, `.idea/`)

### Pre-Distribution Checklist

Before distributing your extension:

- [ ] Manifest is valid JSON
- [ ] All required fields are present (name, version, description, author)
- [ ] Version follows semantic versioning
- [ ] All component paths in manifest match actual files
- [ ] Extension passes validation: `rad extension install ./my-extension`
- [ ] README.md explains the extension's purpose and usage
- [ ] No hardcoded secrets or sensitive information
- [ ] Archive can be installed successfully

## Hosting Options

### GitHub Releases

GitHub Releases is a popular option for distributing extensions:

1. **Create a release**:
   ```bash
   # Tag your release
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. **Upload archive**:
   - Go to your repository's Releases page
   - Create a new release with the tag
   - Upload your extension archive (`.tar.gz` or `.zip`)

3. **Share the URL**:
   ```
   https://github.com/username/repo/releases/download/v1.0.0/my-extension.tar.gz
   ```

**Installation**:
```bash
rad extension install https://github.com/username/repo/releases/download/v1.0.0/my-extension.tar.gz
```

### Personal Website or CDN

You can host extensions on any web server:

1. Upload your archive to your web server
2. Ensure the file is accessible via HTTPS
3. Share the direct download URL

**Example**:
```bash
rad extension install https://example.com/extensions/my-extension.tar.gz
```

### File Sharing Services

For quick sharing during development:
- Dropbox (with direct link)
- Google Drive (with direct link)
- Any service that provides direct download URLs

**Note**: Ensure the URL points directly to the file, not a download page.

## Distribution Checklist

Before publishing your extension:

### Required

- [ ] Valid `radium-extension.json` manifest
- [ ] Semantic version number
- [ ] Clear description and author information
- [ ] README.md with installation instructions
- [ ] Extension installs without errors
- [ ] All components are discoverable after installation

### Recommended

- [ ] Example usage in README
- [ ] Screenshots or demos (if applicable)
- [ ] License file (LICENSE or LICENSE.txt)
- [ ] Changelog (CHANGELOG.md)
- [ ] Documentation for all components
- [ ] Tested on multiple platforms (if applicable)

### Optional

- [ ] Source code repository link
- [ ] Issue tracker link
- [ ] Contribution guidelines
- [ ] Code of conduct

## README Requirements

Your extension's README.md should include:

1. **Title and Description**: What the extension does
2. **Installation**: How to install the extension
3. **Usage**: How to use the extension's components
4. **Components**: What components are included
5. **Dependencies**: Any required dependencies
6. **Examples**: Usage examples
7. **License**: License information

### Example README Structure

```markdown
# My Extension

Brief description of what this extension does.

## Installation

```bash
rad extension install https://example.com/my-extension.tar.gz
```

## Components

- **Prompts**: Agent prompt templates for...
- **MCP Servers**: MCP server configurations for...
- **Commands**: Custom commands for...

## Usage

After installation, the extension components will be available...

## Examples

[Usage examples]

## License

MIT License
```

## URL-Based Installation Requirements

For URL-based installation to work:

1. **Direct download**: URL must point directly to the archive file
2. **HTTPS preferred**: Use HTTPS for security
3. **Archive format**: Must be `.tar.gz` or `.zip`
4. **File size**: Consider file size limits (recommended < 50MB)
5. **Content-Type**: Server should serve with correct `Content-Type` header

## Licensing Considerations

Choose an appropriate license for your extension:

- **MIT**: Permissive, allows commercial use
- **Apache 2.0**: Permissive with patent grant
- **GPL v3**: Copyleft, requires derivative works to be GPL
- **Proprietary**: All rights reserved

Include a LICENSE file in your extension root.

## Security Best Practices

- **No secrets**: Never include API keys, tokens, or passwords
- **Validate inputs**: If your extension processes user input, validate it
- **Document dependencies**: List all external dependencies
- **Use environment variables**: For configuration that varies by user
- **Review code**: Have others review your extension before distribution

## Troubleshooting Distribution Issues

### Installation from URL Fails

**Problem**: `rad extension install <url>` fails

**Solutions**:
- Verify URL is accessible and returns the file directly
- Check file format is `.tar.gz` or `.zip`
- Ensure HTTPS is used (if required)
- Verify file isn't corrupted

### Archive Too Large

**Problem**: Archive file is very large

**Solutions**:
- Remove unnecessary files (`.git/`, `node_modules/`, build artifacts)
- Use compression (`.tar.gz` is more efficient than `.zip`)
- Split into multiple extensions if appropriate
- Consider hosting large assets separately

### Manifest Not Found

**Problem**: Installation says manifest not found

**Solutions**:
- Ensure `radium-extension.json` is in the archive root
- Verify the archive structure matches expected format
- Check that the archive was created correctly

## See Also

- [Extension Best Practices](extension-best-practices.md)
- [Extension Testing Guide](extension-testing.md)
- [Extension Versioning Guide](extension-versioning.md)
- [Creating Extensions](../extensions/creating-extensions.md)

