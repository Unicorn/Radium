# Security Configuration

This document describes all security configuration options available in Radium.

## Secret Management Configuration

Secret management settings control how credentials are stored, redacted, and injected.

### Configuration Options

```toml
[security.secrets]
# Enable secret redaction before sending to LLMs (default: true)
enable_secret_redaction = true

# Enable secret injection before tool execution (default: true)
enable_secret_injection = true

# Enable audit logging of secret operations (default: true)
enable_audit_logging = true

# Warn when hardcoded secrets are detected in workspace (default: true)
warn_on_hardcoded_secrets = true

# Path to the secret vault file (default: ~/.radium/auth/secrets.vault)
secret_vault_path = "~/.radium/auth/secrets.vault"

# Path to the audit log file (default: ~/.radium/auth/audit.log)
audit_log_path = "~/.radium/auth/audit.log"

# Minimum master password length (default: 12)
master_password_min_length = 12
```

### Default Values

All security features are enabled by default with secure settings:

- Secret redaction: **Enabled** - Prevents credential exposure in LLM context
- Secret injection: **Enabled** - Enables tools to use credentials
- Audit logging: **Enabled** - Records all secret operations
- Hardcoded secret warnings: **Enabled** - Alerts on credential detection

### Security Implications

#### enable_secret_redaction

- **Enabled (default)**: Credentials are replaced with placeholders before sending to LLMs
- **Disabled**: Real credentials may be exposed in agent context and responses

**Recommendation**: Always keep enabled unless debugging credential issues.

#### enable_secret_injection

- **Enabled (default)**: Placeholders are replaced with real values before tool execution
- **Disabled**: Tools will receive placeholders instead of real credentials

**Recommendation**: Always keep enabled for normal operation.

#### enable_audit_logging

- **Enabled (default)**: All secret operations are logged for compliance
- **Disabled**: No audit trail of secret access

**Recommendation**: Keep enabled for security monitoring and compliance.

#### warn_on_hardcoded_secrets

- **Enabled (default)**: Workspace scans warn about hardcoded credentials
- **Disabled**: No warnings about credential exposure risks

**Recommendation**: Keep enabled to catch security issues early.

## Privacy Configuration

Privacy settings control sensitive data redaction (separate from secret management).

```toml
[security.privacy]
# Enable privacy mode (default: true)
enable = true

# Privacy mode: "auto", "strict", or "off" (default: "auto")
mode = "auto"

# Redaction style: "full", "partial", or "hash" (default: "partial")
redaction_style = "partial"

# Enable audit logging of redactions (default: true)
audit_log = true

# Custom patterns for organization-specific sensitive data
[security.privacy.custom_patterns]
# Example custom pattern
# name = "custom_token"
# regex = "tok_[a-zA-Z0-9]{40}"
# replacement = "***TOKEN***"
```

## Configuration File Location

Radium looks for configuration in the following order:

1. `.radium/config.toml` (workspace-specific)
2. `~/.radium/config.toml` (user-specific)
3. Environment variables (if supported)
4. Default values

## Environment Variables

Some settings can be overridden via environment variables:

- `RADIUM_SECRET_VAULT_PATH`: Override vault file path
- `RADIUM_AUDIT_LOG_PATH`: Override audit log path

## Example Configuration

Complete example configuration file:

```toml
[security]
[security.secrets]
enable_secret_redaction = true
enable_secret_injection = true
enable_audit_logging = true
warn_on_hardcoded_secrets = true
secret_vault_path = "~/.radium/auth/secrets.vault"
audit_log_path = "~/.radium/auth/audit.log"
master_password_min_length = 12

[security.privacy]
enable = true
mode = "auto"
redaction_style = "partial"
audit_log = true
```

## Validation

Configuration is validated on load. Invalid values will result in errors:

- `master_password_min_length` must be at least 8
- Paths must be valid (expanded if they contain `~`)
- Boolean values must be `true` or `false`

## See Also

- [Secret Management Guide](secret-management.md)
- [Migration Guide](migration-guide.md)

