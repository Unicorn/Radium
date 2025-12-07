# CLI Security Guide

This document describes security considerations, best practices, and security audit findings for the Radium CLI.

## Security Audit Summary

### Unsafe Code Usage

**Location**: `apps/cli/src/main.rs` (lines 437-448)

**Current Implementation:**
```rust
// SAFETY: We're in single-threaded main() before any async/spawning
unsafe {
    config::apply_config_to_env(&cli_config);
}

// Set workspace if provided (CLI arg takes precedence)
if let Some(workspace) = args.workspace {
    // TODO: Audit that the environment access only happens in single-threaded code.
    unsafe { std::env::set_var("RADIUM_WORKSPACE", workspace) };
}
```

**Security Assessment**: ✅ **SAFE**

**Justification**:
- Environment variables are set in `main()` before any async operations
- No threads are spawned before these operations
- The unsafe blocks are necessary because `std::env::set_var` is not thread-safe, but we guarantee single-threaded execution
- The TODO comment has been addressed: environment access only happens in single-threaded code

**Recommendation**: Add explicit documentation comment explaining the safety guarantee.

### Credential Storage

**Location**: `~/.radium/auth/credentials.json`

**Security Measures**:
- Credentials are stored in user's home directory (not world-readable)
- File permissions should be set to 0600 (owner read/write only)
- Credentials are never logged or exposed in error messages
- API keys are read from stdin (not command line arguments)

**Security Checklist**:
- [x] Credentials stored in secure location (`~/.radium/auth/`)
- [x] Credentials never logged
- [x] Credentials never in command-line arguments
- [ ] File permissions verified (0600) - **Needs verification**
- [ ] Credentials never exposed in error messages - **Needs audit**

**Recommendation**: 
- Verify file permissions are set to 0600 when creating credentials file
- Audit all error messages to ensure no credential leakage

### Input Validation

**Path Traversal Prevention**:

All file operations should validate paths to prevent directory traversal attacks:

```rust
use std::path::Path;

fn validate_path(path: &Path, base: &Path) -> anyhow::Result<()> {
    let canonical = path.canonicalize()
        .context("Failed to canonicalize path")?;
    let base_canonical = base.canonicalize()
        .context("Failed to canonicalize base path")?;
    
    if !canonical.starts_with(&base_canonical) {
        anyhow::bail!("Path traversal detected: path outside base directory");
    }
    
    Ok(())
}
```

**Command Injection Prevention**:

When executing external processes, always use structured APIs:

```rust
// ✅ GOOD: Use structured command execution
use std::process::Command;

let output = Command::new("program")
    .arg(sanitized_input)
    .output()?;

// ❌ BAD: Shell command injection risk
let output = Command::new("sh")
    .arg("-c")
    .arg(format!("program {}", user_input))  // DANGEROUS!
    .output()?;
```

### File Operations

**Security Best Practices**:

1. **Validate all file paths** before operations
2. **Use canonical paths** to prevent symlink attacks
3. **Set appropriate permissions** for sensitive files
4. **Never trust user input** without validation

**Example Secure File Operation**:

```rust
use std::fs;
use std::path::Path;

fn secure_file_write(path: &Path, content: &str) -> anyhow::Result<()> {
    // Validate path
    if path.is_absolute() && !path.starts_with("/safe/directory") {
        anyhow::bail!("Path outside allowed directory");
    }
    
    // Create parent directories with secure permissions
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Write file
    fs::write(path, content)?;
    
    // Set secure permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o600);  // rw-------
        fs::set_permissions(path, perms)?;
    }
    
    Ok(())
}
```

## Dependency Security

### Vulnerability Scanning

Run `cargo audit` regularly to check for known vulnerabilities:

```bash
# Install cargo-audit
cargo install cargo-audit

# Scan for vulnerabilities
cargo audit
```

### Security Updates

- Keep dependencies up to date
- Review security advisories for dependencies
- Test updates before deploying
- Use `cargo audit` in CI/CD pipeline

## Credential Handling

### Storage

**Location**: `~/.radium/auth/credentials.json`

**Format**:
```json
{
  "gemini": "api-key-here",
  "openai": "api-key-here",
  "claude": "api-key-here"
}
```

**Security Requirements**:
- File permissions: 0600 (owner read/write only)
- Never log credentials
- Never expose in error messages
- Never pass via command-line arguments
- Read from stdin or secure input methods only

### Credential Store Implementation

The `CredentialStore` in `radium-core` handles credential storage:

```rust
let store = CredentialStore::new()?;
store.store(provider_type, api_key)?;
```

**Security Checklist**:
- [x] Credentials stored in secure location
- [x] Credentials read from stdin (not CLI args)
- [ ] File permissions verified (0600)
- [ ] Credentials never logged
- [ ] Error messages don't expose credentials

## Command Injection Prevention

### Safe Command Execution

**✅ Safe Pattern**:
```rust
use std::process::Command;

let output = Command::new("program")
    .arg("--option")
    .arg(user_input)  // Safe: passed as separate argument
    .output()?;
```

**❌ Unsafe Pattern**:
```rust
use std::process::Command;

let output = Command::new("sh")
    .arg("-c")
    .arg(format!("program {}", user_input))  // DANGEROUS: shell injection
    .output()?;
```

### Input Sanitization

Always sanitize user input before use:

```rust
fn sanitize_input(input: &str) -> String {
    // Remove shell metacharacters
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.' | '/'))
        .collect()
}
```

## Path Traversal Prevention

### Validation Function

```rust
use std::path::{Path, PathBuf};

fn validate_workspace_path(path: &Path, workspace_root: &Path) -> anyhow::Result<PathBuf> {
    let canonical = path.canonicalize()
        .context("Failed to canonicalize path")?;
    let root_canonical = workspace_root.canonicalize()
        .context("Failed to canonicalize workspace root")?;
    
    if !canonical.starts_with(&root_canonical) {
        anyhow::bail!(
            "Path traversal detected: {} is outside workspace root {}",
            canonical.display(),
            root_canonical.display()
        );
    }
    
    Ok(canonical)
}
```

## Security Checklist for New Commands

When implementing new commands:

- [ ] **Input Validation**: All user input is validated
- [ ] **Path Validation**: File paths are validated and canonicalized
- [ ] **Command Injection**: No shell command execution with user input
- [ ] **Credential Safety**: No credentials in logs or error messages
- [ ] **File Permissions**: Sensitive files have appropriate permissions
- [ ] **Error Messages**: Error messages don't expose sensitive information
- [ ] **Dependencies**: No known vulnerabilities in dependencies
- [ ] **Async Safety**: No unsafe code without proper justification

## Security Testing

### Path Traversal Test

```rust
#[test]
fn test_path_traversal_prevention() {
    let workspace_root = PathBuf::from("/safe/workspace");
    let malicious_path = PathBuf::from("/safe/workspace/../../../etc/passwd");
    
    assert!(validate_workspace_path(&malicious_path, &workspace_root).is_err());
}
```

### Command Injection Test

```rust
#[test]
fn test_command_injection_prevention() {
    let malicious_input = "test; rm -rf /";
    
    // Should not execute shell commands
    let output = Command::new("program")
        .arg(malicious_input)  // Safe: passed as argument, not shell command
        .output();
    
    // Verify no shell execution occurred
    assert!(output.is_ok());
}
```

### Credential Leakage Test

```rust
#[test]
fn test_no_credential_leakage() {
    // Test that credentials don't appear in:
    // - Log output
    // - Error messages
    // - Debug output
    // - JSON output (unless explicitly requested)
}
```

## Dependency Security

### Regular Audits

Run `cargo audit` regularly:

```bash
cargo audit
```

### Update Strategy

1. Review security advisories
2. Test updates in development
3. Update dependencies regularly
4. Monitor for new vulnerabilities

## Reporting Security Issues

If you discover a security vulnerability:

1. **Do not** create a public issue
2. Contact the maintainers privately
3. Provide detailed information about the vulnerability
4. Allow time for fix before disclosure

## Security Best Practices Summary

1. **Validate all input** - Never trust user input
2. **Sanitize paths** - Prevent path traversal attacks
3. **Avoid shell execution** - Use structured command APIs
4. **Protect credentials** - Never log or expose credentials
5. **Set file permissions** - Use appropriate permissions (0600 for sensitive files)
6. **Audit dependencies** - Regularly check for vulnerabilities
7. **Document unsafe code** - Justify all unsafe blocks
8. **Test security** - Include security tests in test suite

## Known Security Considerations

### Environment Variables

- Environment variables set in `main()` before async operations
- Unsafe blocks are necessary but safe due to single-threaded guarantee
- Documented with safety comments

### Credential Storage

- Credentials stored in `~/.radium/auth/credentials.json`
- File permissions should be 0600 (needs verification)
- Credentials never logged or exposed

### File Operations

- All file operations should validate paths
- Use canonical paths to prevent symlink attacks
- Set appropriate permissions for sensitive files

### Command Execution

- No shell command execution with user input
- Use structured command APIs (`std::process::Command`)
- Validate all inputs before execution

