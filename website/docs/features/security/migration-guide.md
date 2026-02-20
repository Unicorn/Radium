---
id: "migration-guide"
title: "Migration Guide: Plaintext to Encrypted Vault"
sidebar_label: "Migration Guide: Plaintext to Encrypted ..."
---

# Migration Guide: Plaintext to Encrypted Vault

This guide walks you through migrating your existing plaintext credentials to the encrypted vault.

## Overview

Radium previously stored credentials in plaintext at `~/.radium/auth/credentials.json`. The new secret management system uses an encrypted vault at `~/.radium/auth/secrets.vault`.

## Migration Process

### Step 1: Verify Current Credentials

Before migrating, verify your current credentials work:

```bash
radium auth status
```

This shows which providers are configured and their status.

### Step 2: Run Migration

Start the migration process:

```bash
radium secret migrate
```

You'll be prompted to:
1. Set a master password for the encrypted vault
2. Confirm the master password

### Step 3: Review Migration Report

After migration, you'll see a report showing:
- Total credentials found
- Number successfully migrated
- Number that failed (if any)
- Backup file location

### Step 4: Verify Migration

Verify your credentials still work:

```bash
radium auth status
radium secret list
```

### Step 5: Test Secret Access

Test that secrets are accessible:

```bash
# This should work if migration succeeded
radium secret list
```

## What Happens During Migration

1. **Backup Creation**: A timestamped backup of `credentials.json` is created
2. **Vault Creation**: New encrypted vault is created with your master password
3. **Credential Transfer**: All provider credentials are encrypted and stored
4. **File Deprecation**: Original file is marked as deprecated (not deleted)

## Backup Files

Backup files are created with the format:
```
credentials.json.backup-<timestamp>
```

Example: `credentials.json.backup-1702070400`

## Rollback Procedure

If migration fails or you need to rollback:

### Option 1: Manual Restore

1. Locate the backup file in `~/.radium/auth/`
2. Copy it back to `credentials.json`:
   ```bash
   cp ~/.radium/auth/credentials.json.backup-<timestamp> ~/.radium/auth/credentials.json
   ```
3. Remove the vault file (if created):
   ```bash
   rm ~/.radium/auth/secrets.vault
   ```

### Option 2: Use Migration Manager

The migration utility provides a rollback function (programmatic access):

```rust
use radium_core::security::MigrationManager;

let backup_path = PathBuf::from("~/.radium/auth/credentials.json.backup-<timestamp>");
MigrationManager::rollback(&backup_path)?;
```

## Troubleshooting

### Migration Fails with "No credentials to migrate"

**Cause**: No `credentials.json` file exists.

**Solution**: 
- Verify the file exists: `ls ~/.radium/auth/credentials.json`
- If missing, you may need to set up credentials first: `radium auth login`

### Migration Fails with "Invalid master password"

**Cause**: Master password doesn't meet requirements.

**Solution**:
- Password must be at least 12 characters
- Must contain at least one letter
- Must contain at least one number or special character

### Some Credentials Fail to Migrate

**Cause**: Individual credentials may be corrupted or inaccessible.

**Solution**:
1. Check the migration report for specific failures
2. Manually add failed credentials: `radium secret add <name>`
3. Verify the original credentials file is intact

### Vault Already Exists

**Cause**: Vault file already exists from a previous migration attempt.

**Solution**:
- If you want to start fresh, remove the vault: `rm ~/.radium/auth/secrets.vault`
- If you want to add to existing vault, use `radium secret add` instead

## Post-Migration

After successful migration:

1. **Original File**: The `credentials.json` file is marked as deprecated but preserved
2. **New Vault**: All credentials are now in `secrets.vault`
3. **Backup**: Original file is backed up with timestamp

### Deprecated File Notice

The original `credentials.json` file will have a deprecation notice added at the top:

```
// DEPRECATED: Credentials migrated to encrypted vault on <timestamp>
// This file is kept for rollback purposes only.
// Do not use this file - credentials are now stored in secrets.vault
// To rollback, restore from backup: credentials.json.backup-<timestamp>
```

### Next Steps

1. **Test Everything**: Verify all your workflows still work
2. **Update Documentation**: Update any scripts or docs referencing credentials.json
3. **Secure Backup**: Store your master password securely (password manager)
4. **Clean Up**: After confirming everything works, you can remove the deprecated file

## Idempotency

Migration is idempotent - you can run it multiple times safely:

- If credentials already exist in vault, they won't be duplicated
- New credentials in `credentials.json` will be added to vault
- Existing vault credentials are preserved

## Security Considerations

### Master Password

- **Never share** your master password
- **Store securely** in a password manager
- **Use a strong password** (12+ characters, mixed case, numbers, symbols)
- **Remember it** - there's no password recovery

### Backup Files

- Backup files contain **plaintext credentials**
- Store backups securely or encrypt them
- Delete backups after confirming migration success
- Don't commit backups to version control

### Vault File

- Vault file is encrypted but still sensitive
- Keep file permissions at 0600 (default)
- Don't commit vault file to version control
- Back up the vault file separately if needed

## See Also

- [Secret Management Guide](secret-management.md)
- [Configuration Guide](configuration.md)

