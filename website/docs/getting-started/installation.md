---
id: "installation"
title: "Installing Radium"
sidebar_label: "Installing Radium"
---

# Installing Radium

This guide will help you install Radium on your system.

## Prerequisites

- **Operating System**: Linux, macOS, or Windows (via WSL2)
- **Rust**: 1.70 or later (for building from source)
- **API Keys**: At least one of:
  - Anthropic API key (for Claude models)
  - Google AI API key (for Gemini models)
  - OpenAI API key (for GPT models)

## Quick Install (Recommended)

The fastest way to install Radium is using the install script:

```bash
curl -sSf https://raw.githubusercontent.com/clay-curry/RAD/main/install.sh | sh
```

This will:
1. Download the latest Radium release
2. Install the `rad` CLI tool
3. Add Radium to your PATH
4. Verify the installation

## Manual Installation

### From Pre-built Binaries

1. Download the latest release for your platform from [GitHub Releases](https://github.com/clay-curry/RAD/releases)

2. Extract the archive:
   ```bash
   tar -xzf radium-*.tar.gz
   ```

3. Move the binary to your PATH:
   ```bash
   sudo mv radium-*/rad /usr/local/bin/
   ```

4. Verify installation:
   ```bash
   rad --version
   ```

### From Source

1. Clone the repository:
   ```bash
   git clone https://github.com/clay-curry/RAD.git
   cd RAD
   ```

2. Build with Cargo:
   ```bash
   cargo build --release
   ```

3. The compiled binary will be at `./target/release/rad`

4. Optionally install system-wide:
   ```bash
   cargo install --path apps/cli
   ```

## Configuration

After installation, configure your API keys:

```bash
# Set your preferred AI provider API key
export ANTHROPIC_API_KEY="sk-ant-..."
# or
export GOOGLE_AI_API_KEY="..."
# or
export OPENAI_API_KEY="sk-..."
```

Add these to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.) to persist them.

## Verify Installation

Run the following command to verify everything is working:

```bash
rad --version
rad auth status
```

You should see the Radium version and your authentication status.

## Next Steps

- [Quick Start](./quick-start.md) - Create your first agent
- [Configuration](./configuration.md) - Customize Radium settings
- [Core Concepts](./core-concepts.md) - Understand how Radium works

## Troubleshooting

### Command not found

If you get "rad: command not found", ensure `/usr/local/bin` is in your PATH:

```bash
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Permission denied

If you get permission errors, you may need to make the binary executable:

```bash
chmod +x /usr/local/bin/rad
```

### Missing API key

If you see authentication errors, ensure your API key is correctly set:

```bash
echo $ANTHROPIC_API_KEY  # Should output your API key
```

For more help, see the [Troubleshooting Guide](../user-guide/troubleshooting.md) or [open an issue](https://github.com/clay-curry/RAD/issues).
