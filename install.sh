#!/usr/bin/env bash
set -euo pipefail

# Radium Installation Script
# This script builds and installs Radium from source

echo "🚀 Installing Radium..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Rust and Cargo are required but not installed."
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

# Check if git is installed
if ! command -v git &> /dev/null; then
    echo "❌ Error: Git is required but not installed."
    exit 1
fi

# Create temporary directory
TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

echo "📦 Cloning repository..."
cd "$TMPDIR"
git clone https://github.com/Unicorn/Radium.git
cd RAD

echo "🔨 Building Radium (this may take a few minutes)..."
cargo build --release

echo "📥 Installing Radium..."
cargo install --path apps/cli

# Check if cargo bin is in PATH
CARGO_BIN="$HOME/.cargo/bin"
if [[ ":$PATH:" != *":$CARGO_BIN:"* ]]; then
    echo ""
    echo "⚠️  Warning: $CARGO_BIN is not in your PATH."
    echo "Add this to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
    echo "  export PATH=\"\$HOME/.cargo/bin:\$PATH\""
    echo ""
fi

echo "✅ Radium installed successfully!"
echo ""
echo "Verify installation with:"
echo "  rad --version"
echo "  rad auth status"

