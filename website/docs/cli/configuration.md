---
id: "configuration"
title: "CLI Configuration"
sidebar_label: "CLI Configuration"
---

# CLI Configuration

Radium CLI supports configuration files to set default values for common options, reducing repetitive command-line arguments.

## Configuration File Locations

Radium looks for configuration files in the following order (later files override earlier ones):

1. **Global config**: `~/.radium/config.toml`
2. **Local config**: `./.radiumrc` (in current directory)

## Configuration Precedence

Configuration values are resolved in this order (highest to lowest priority):

1. **CLI arguments** - Command-line flags always take precedence
2. **Environment variables** - `RADIUM_ENGINE`, `RADIUM_MODEL`, etc.
3. **Local config** - `./.radiumrc`
4. **Global config** - `~/.radium/config.toml`
5. **Defaults** - Built-in defaults

## Configuration Options

### Basic Options

```toml
# Default engine to use
engine = "claude"

# Default model to use
model = "claude-3-opus"

# Default workspace path
workspace = "/path/to/workspace"

# Log level
log_level = "info"
```

### Output Configuration

```toml
[output]
# Default output format: "human" or "json"
format = "human"

# Always use JSON output
always_json = false
```

### Command Aliases

```toml
[aliases]
# Short aliases for common commands
c = "craft"
p = "plan"
s = "status"
a = "agents"
```

## Example Configuration

### Global Configuration (`~/.radium/config.toml`)

```toml
# Global defaults
engine = "claude"
model = "claude-3-opus"
log_level = "info"

[output]
format = "human"

[aliases]
c = "craft"
p = "plan"
```

### Local Configuration (`./.radiumrc`)

```toml
# Project-specific overrides
engine = "openai"
model = "gpt-4"

[output]
always_json = true
```

## Usage Examples

### With Configuration

```bash
# Uses engine from config
rad step code-agent

# CLI flag overrides config
rad step code-agent --engine gemini

# Environment variable overrides config
RADIUM_ENGINE=openai rad step code-agent
```

### Command Aliases

With aliases configured:

```bash
# Instead of: rad craft REQ-001
rad c REQ-001

# Instead of: rad plan spec.md
rad p spec.md

# Instead of: rad status
rad s
```

## Creating Configuration Files

### Copy Example File

```bash
cp .radiumrc.example .radiumrc
# Edit .radiumrc with your preferences
```

### Create Manually

```bash
# Create global config
mkdir -p ~/.radium
cat > ~/.radium/config.toml << EOF
engine = "claude"
model = "claude-3-opus"
log_level = "info"
EOF

# Create local config
cat > .radiumrc << EOF
engine = "mock"
[output]
format = "json"
EOF
```

## Configuration Validation

Configuration files are validated on load. Invalid configurations will produce clear error messages:

```bash
# Invalid config will show error
rad status
# Error: Failed to parse configuration file: .radiumrc: invalid value
```

## Best Practices

1. **Use global config for personal defaults**
   - Set your preferred engine/model in `~/.radium/config.toml`

2. **Use local config for project-specific settings**
   - Set project-specific engines or output formats in `./.radiumrc`

3. **Keep sensitive data out of config files**
   - Use environment variables or `rad auth login` for API keys

4. **Version control local configs carefully**
   - Consider adding `.radiumrc` to `.gitignore` if it contains personal preferences

5. **Use aliases for frequently used commands**
   - Create short aliases for commands you use often

## Troubleshooting

### Configuration not loading

```bash
# Check if config file exists
ls -la ~/.radium/config.toml
ls -la .radiumrc

# Check file permissions
chmod 644 ~/.radium/config.toml
```

### Configuration not taking effect

Remember the precedence order:
- CLI flags always win
- Environment variables override config
- Local config overrides global config

```bash
# Check what's being used
rad status  # Shows current configuration

# Override with CLI flag
rad step agent-id --engine mock
```

### Invalid configuration

```bash
# Validate TOML syntax
toml validate .radiumrc

# Check for common errors:
# - Missing quotes around strings
# - Invalid section names
# - Type mismatches
```

