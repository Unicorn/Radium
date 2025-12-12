---
id: "shell-completion"
title: "Shell Completion"
sidebar_label: "Shell Completion"
---

# Shell Completion

Radium CLI supports tab completion for bash, zsh, and fish shells.

## Generating Completions

### Using the Generation Script

```bash
# Generate all completion scripts
./scripts/generate-completions.sh
```

This will create completion files in the `completions/` directory:
- `completions/rad.bash` - Bash completion
- `completions/rad.zsh` - Zsh completion
- `completions/rad.fish` - Fish completion

### Manual Generation

You can also generate completions manually using the CLI:

```bash
# Set environment variable and run CLI
RADIUM_GENERATE_COMPLETIONS=bash cargo run --release -p radium-cli > completions/rad.bash
RADIUM_GENERATE_COMPLETIONS=zsh cargo run --release -p radium-cli > completions/rad.zsh
RADIUM_GENERATE_COMPLETIONS=fish cargo run --release -p radium-cli > completions/rad.fish
RADIUM_GENERATE_COMPLETIONS=powershell cargo run --release -p radium-cli > completions/rad.ps1
RADIUM_GENERATE_COMPLETIONS=elvish cargo run --release -p radium-cli > completions/rad.elv
```

Or if you have the `rad` binary installed:

```bash
RADIUM_GENERATE_COMPLETIONS=bash rad > completions/rad.bash
RADIUM_GENERATE_COMPLETIONS=zsh rad > completions/rad.zsh
RADIUM_GENERATE_COMPLETIONS=fish rad > completions/rad.fish
RADIUM_GENERATE_COMPLETIONS=powershell rad > completions/rad.ps1
RADIUM_GENERATE_COMPLETIONS=elvish rad > completions/rad.elv
```

## Installation

### Bash

Add to your `~/.bashrc` or `~/.bash_profile`:

```bash
source /path/to/radium/completions/rad.bash
```

Or for system-wide installation:

```bash
sudo cp completions/rad.bash /etc/bash_completion.d/rad
```

### Zsh

Add to your `~/.zshrc`:

```bash
source /path/to/radium/completions/rad.zsh
```

Or use the zsh completions directory:

```bash
mkdir -p ~/.zsh/completions
cp completions/rad.zsh ~/.zsh/completions/_rad
```

Then add to `~/.zshrc`:

```zsh
fpath=(~/.zsh/completions $fpath)
autoload -U compinit
compinit
```

### Fish

Copy to fish completions directory:

```bash
mkdir -p ~/.config/fish/completions
cp completions/rad.fish ~/.config/fish/completions/rad.fish
```

Or create a symlink:

```bash
ln -s /path/to/radium/completions/rad.fish ~/.config/fish/completions/rad.fish
```

## Usage

After installation, tab completion will work automatically:

```bash
# Complete commands
rad <TAB>
# Shows: init, plan, craft, complete, step, run, chat, ...

# Complete subcommands
rad agents <TAB>
# Shows: list, search, info, validate, create

# Complete options
rad craft --<TAB>
# Shows: --iteration, --task, --resume, --dry-run, --json, --yolo, --engine
```

## Supported Shells

The CLI supports completion generation for all major shells:

- **Bash** - Full support for commands, subcommands, and options
- **Zsh** - Full support with enhanced completion features
- **Fish** - Full support with fish-specific enhancements
- **PowerShell** - Full support (Windows)
- **Elvish** - Full support

All 30+ commands and their subcommands are included in the completions. The completion system uses `clap_complete` to automatically generate completions from the command structure defined in `main.rs`.

## Troubleshooting

### Completions not working

1. **Reload shell configuration**:
   ```bash
   # Bash
   source ~/.bashrc

   # Zsh
   source ~/.zshrc

   # Fish
   # Restart fish shell
   ```

2. **Check file permissions**:
   ```bash
   chmod +x completions/rad.bash
   ```

3. **Verify installation path**:
   ```bash
   # Check if file exists
   ls -la ~/.config/fish/completions/rad.fish
   ```

### Regenerating completions

If you add new commands or options, regenerate completions:

```bash
./scripts/generate-completions.sh
```

Then reload your shell configuration.

