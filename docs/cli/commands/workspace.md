# Workspace Management Commands

Commands for initializing and managing Radium workspaces.

## `rad init`

Initialize a new Radium workspace.

### Usage

```bash
rad init [path] [options]
```

### Options

- `--use-defaults` - Use default values without prompting
- `--with-context` - Create a starter GEMINI.md context file

### Examples

```bash
# Initialize in current directory
rad init

# Initialize in specific path
rad init /path/to/project

# Initialize with defaults
rad init --use-defaults

# Initialize with context file
rad init --with-context
```

## `rad status`

Show workspace and engine status.

### Usage

```bash
rad status [--json]
```

### Options

- `--json` - Output as JSON

### Examples

```bash
# Show human-readable status
rad status

# Show JSON status
rad status --json
```

## `rad clean`

Clean workspace artifacts (temporary files, logs, cache).

### Usage

```bash
rad clean [options]
```

### Options

- `-v, --verbose` - Show detailed output
- `-d, --dir <path>` - Target workspace directory

### Examples

```bash
# Clean current workspace
rad clean

# Clean with verbose output
rad clean --verbose

# Clean specific directory
rad clean --dir /path/to/workspace
```

## `rad doctor`

Environment validation and diagnostics.

### Usage

```bash
rad doctor [--json]
```

### Options

- `--json` - Output as JSON

### Examples

```bash
# Run diagnostics
rad doctor

# Get JSON output
rad doctor --json
```

