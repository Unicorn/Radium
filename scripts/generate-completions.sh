#!/bin/bash
# Generate shell completion scripts for rad CLI

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
COMPLETIONS_DIR="$PROJECT_ROOT/completions"
BIN_NAME="radium-cli"

# Create completions directory
mkdir -p "$COMPLETIONS_DIR"

# Build the CLI if needed
if [ ! -f "$PROJECT_ROOT/target/release/$BIN_NAME" ]; then
    echo "Building CLI..."
    cd "$PROJECT_ROOT"
    cargo build --release -p radium-cli
fi

BIN_PATH="$PROJECT_ROOT/target/release/$BIN_NAME"

# Generate completions
echo "Generating shell completions..."

# Bash
echo "  Generating bash completion..."
"$BIN_PATH" --generate-completion bash > "$COMPLETIONS_DIR/rad.bash" || {
    # Fallback: use clap_complete directly if --generate-completion not available
    cd "$PROJECT_ROOT"
    cargo run --release -p radium-cli -- --generate-completion bash > "$COMPLETIONS_DIR/rad.bash" 2>/dev/null || {
        echo "    Note: Bash completion generation requires --generate-completion flag"
    }
}

# Zsh
echo "  Generating zsh completion..."
"$BIN_PATH" --generate-completion zsh > "$COMPLETIONS_DIR/rad.zsh" || {
    cd "$PROJECT_ROOT"
    cargo run --release -p radium-cli -- --generate-completion zsh > "$COMPLETIONS_DIR/rad.zsh" 2>/dev/null || {
        echo "    Note: Zsh completion generation requires --generate-completion flag"
    }
}

# Fish
echo "  Generating fish completion..."
"$BIN_PATH" --generate-completion fish > "$COMPLETIONS_DIR/rad.fish" || {
    cd "$PROJECT_ROOT"
    cargo run --release -p radium-cli -- --generate-completion fish > "$COMPLETIONS_DIR/rad.fish" 2>/dev/null || {
        echo "    Note: Fish completion generation requires --generate-completion flag"
    }
}

echo "Completions generated in $COMPLETIONS_DIR"
echo ""
echo "To install:"
echo "  Bash: source $COMPLETIONS_DIR/rad.bash"
echo "  Zsh:  source $COMPLETIONS_DIR/rad.zsh"
echo "  Fish: cp $COMPLETIONS_DIR/rad.fish ~/.config/fish/completions/"

