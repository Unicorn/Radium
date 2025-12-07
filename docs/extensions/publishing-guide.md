# Extension Publishing Guide

Learn how to publish your extensions to the Radium marketplace for others to discover and use.

## Overview

Publishing an extension to the marketplace allows the community to:
- Discover your extension through search
- Install it easily with `rad extension install <name>`
- See download counts and ratings
- Trust your extension through cryptographic signatures

## Prerequisites

- A completed extension ready for distribution
- A marketplace API key (contact marketplace administrators)
- Optionally: A signing keypair for extension signing

## Step 1: Prepare Your Extension

Before publishing, ensure your extension is complete:

1. **Validate your extension structure:**
   ```bash
   rad extension install ./my-extension --overwrite
   ```

2. **Test all components work correctly**

3. **Update version in manifest:**
   ```json
   {
     "version": "1.0.0"
   }
   ```

4. **Write a clear description** in your manifest

## Step 2: Sign Your Extension (Recommended)

Signing your extension provides authenticity and security:

### Generate a Keypair

```bash
rad extension sign ./my-extension --generate-key
```

This creates:
- `private.key` - Keep this secure!
- `public.key` - Share this with users

### Sign the Extension

```bash
rad extension sign ./my-extension --key-file ./private.key
```

This creates `radium-extension.json.sig` in your extension directory.

## Step 3: Get a Marketplace API Key

Contact the marketplace administrators to obtain an API key. Store it securely:

```bash
export RADIUM_MARKETPLACE_API_KEY="your-api-key-here"
```

Or provide it via command line (see Step 4).

## Step 4: Publish Your Extension

Publish your extension to the marketplace:

```bash
# With API key from environment
rad extension publish ./my-extension

# Or provide API key directly
rad extension publish ./my-extension --api-key YOUR_API_KEY

# With automatic signing
rad extension publish ./my-extension --api-key YOUR_API_KEY --sign-with-key ./private.key
```

The publish command will:
1. Validate your extension structure
2. Check required manifest fields
3. Sign the extension (if key provided)
4. Create an archive
5. Upload to marketplace

## Step 5: Verify Publication

After publishing, verify your extension is available:

```bash
# Search for your extension
rad extension search my-extension

# Browse marketplace
rad extension browse

# Try installing by name
rad extension install my-extension
```

## Updating Your Extension

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

## Best Practices

### Versioning

- Follow [semantic versioning](https://semver.org/)
- Increment patch (1.0.1) for bug fixes
- Increment minor (1.1.0) for new features
- Increment major (2.0.0) for breaking changes

### Signing

- Always sign extensions before publishing
- Keep private keys secure and backed up
- Share public keys with users for verification
- Consider using a dedicated signing key for publishing

### Descriptions

- Write clear, concise descriptions
- Include use cases and examples
- List required dependencies
- Mention any special requirements

### Testing

- Test installation from archive
- Verify all components load correctly
- Test on clean Radium installations
- Check dependency resolution

## Troubleshooting

### Validation Errors

If publishing fails with validation errors:

- Check all required manifest fields are present
- Verify extension structure is correct
- Ensure component paths match actual files
- Test installation locally first

### API Key Issues

If you get authentication errors:

- Verify API key is correct
- Check API key hasn't expired
- Ensure you have publishing permissions
- Contact marketplace administrators

### Signing Errors

If signing fails:

- Verify private key file exists and is readable
- Check key format is correct (64 bytes for Ed25519)
- Ensure extension directory is writable
- Try generating a new keypair

## Security Considerations

- **Never commit private keys** to version control
- **Use environment variables** for API keys in CI/CD
- **Verify signatures** before installing extensions
- **Trust only verified publishers** for production use

## Next Steps

- Learn about [extension architecture](architecture.md)
- Read the [creating extensions guide](creating-extensions.md)
- Explore [marketplace features](../user-guide/marketplace.md)
- Join the community to share your extensions!

