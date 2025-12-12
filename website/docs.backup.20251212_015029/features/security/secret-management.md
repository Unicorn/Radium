# Secret Management

Radium provides a comprehensive secret management system that prevents credential exposure in agent interactions, logs, and responses. This system uses encrypted storage and dynamic credential substitution to ensure credentials are never exposed to LLMs or in logs.

## Overview

The secret management system provides:

- **Encrypted Storage**: All secrets are stored in an encrypted vault using AES-256-GCM encryption
- **Dynamic Substitution**: Real credentials are replaced with placeholders before sending to LLMs
- **Automatic Injection**: Placeholders are replaced with real values just before tool execution
- **Workspace Scanning**: Detects hardcoded credentials in your workspace
- **Audit Logging**: Records all secret access operations for compliance

## How It Works

The secret management system operates in three stages:

1. **Storage**: Secrets are encrypted and stored in `~/.radium/auth/secrets.vault`
2. **Redaction**: Before sending context to LLMs, real credential values are replaced with `{{SECRET:name}}` placeholders
3. **Injection**: Just before tool execution, placeholders are replaced with real values

This ensures that:
- LLMs never see actual credential values
- Logs never contain plaintext credentials
- Tools receive real credentials when needed
- Credentials are never exposed in agent responses

## Quick Start

### 1. Create Your First Secret

```bash
radium secret add api_key
```

You'll be prompted to:
- Set a master password (if this is your first secret)
- Enter the secret value
- Confirm the secret value

### 2. List Your Secrets

```bash
radium secret list
```

This shows only secret names - values are never displayed.

### 3. Use Secrets in Your Work

Secrets are automatically redacted from context and injected into tools. You can reference them in your code or commands using placeholders:

- `{{SECRET:api_key}}` - Standard placeholder format
- `$SECRET_api_key` - Environment variable format

### 4. Scan for Hardcoded Credentials

```bash
radium secret scan
```

This scans your workspace for hardcoded credentials and reports any findings.

## Migration from Plaintext Credentials

If you have existing credentials in `~/.radium/auth/credentials.json`, you can migrate them to the encrypted vault:

```bash
radium secret migrate
```

This will:
- Create a timestamped backup of your credentials file
- Migrate all provider credentials to the encrypted vault
- Mark the original file as deprecated
- Provide a rollback path if needed

## Security Features

### Encryption

- **Algorithm**: AES-256-GCM (authenticated encryption)
- **Key Derivation**: PBKDF2 with 100,000 iterations
- **Master Password**: Minimum 12 characters with complexity requirements

### File Permissions

- Vault file: 0600 (owner read/write only)
- Audit log: 0600 (owner read/write only)
- Auth directory: 0700 (owner access only)

### Audit Logging

All secret operations are logged to `~/.radium/auth/audit.log`:
- Store operations
- Retrieve operations
- List operations
- Rotation operations
- Removal operations

Log entries include:
- Timestamp
- Operation type
- Secret name (never the value)
- Success/failure status
- Error messages (if failed)

## Best Practices

1. **Use Strong Master Passwords**: At least 12 characters with letters, numbers, and special characters
2. **Rotate Secrets Regularly**: Use `radium secret rotate <name>` to update secrets
3. **Scan Your Workspace**: Regularly run `radium secret scan` to find hardcoded credentials
4. **Review Audit Logs**: Periodically review `~/.radium/auth/audit.log` for suspicious activity
5. **Never Commit Secrets**: Use `.gitignore` to exclude credential files and vault files

## Configuration

Secret management can be configured in your Radium config file:

```toml
[security.secrets]
enable_secret_redaction = true
enable_secret_injection = true
enable_audit_logging = true
warn_on_hardcoded_secrets = true
secret_vault_path = "~/.radium/auth/secrets.vault"
audit_log_path = "~/.radium/auth/audit.log"
master_password_min_length = 12
```

## Troubleshooting

### Master Password Forgotten

If you forget your master password, you cannot recover secrets. You'll need to:
1. Restore from backup (if you have one)
2. Re-enter all secrets manually

### Vault Corruption

If the vault file becomes corrupted:
1. Check for backup files in `~/.radium/auth/`
2. Restore from the most recent backup
3. If no backup exists, you'll need to recreate secrets

### Migration Issues

If migration fails:
1. Check the backup file created during migration
2. Verify the original `credentials.json` file is intact
3. Review error messages for specific issues
4. Use the backup to rollback if needed

## API Reference

For programmatic access, see the Rust API documentation:

- `SecretManager`: Core secret storage and retrieval
- `SecretFilter`: Pre-LLM credential redaction
- `SecretInjector`: Pre-tool credential injection
- `SecretScanner`: Workspace credential detection
- `AuditLogger`: Operation logging

## See Also

- [Configuration Guide](configuration.md)
- [Migration Guide](migration-guide.md)

