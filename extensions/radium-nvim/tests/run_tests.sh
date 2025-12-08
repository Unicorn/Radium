#!/bin/bash
# Test runner for Radium Neovim extension
# Requires plenary.nvim or similar test framework

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXTENSION_DIR="$(dirname "$SCRIPT_DIR")"

echo "Running Radium Neovim Extension Tests"
echo "======================================"
echo ""

# Check if nvim is available
if ! command -v nvim >/dev/null 2>&1; then
    echo "Error: nvim not found. Please install Neovim."
    exit 1
fi

# Check if plenary.nvim is available (for testing)
# If not available, we'll run basic validation tests
if [ ! -d "$HOME/.local/share/nvim/site/pack/*/start/plenary.nvim" ] && \
   [ ! -d "$EXTENSION_DIR/../plenary.nvim" ]; then
    echo "Warning: plenary.nvim not found. Running basic validation only."
    echo ""
    echo "To run full tests, install plenary.nvim:"
    echo "  git clone https://github.com/nvim-lua/plenary.nvim.git"
    echo ""
    
    # Run basic validation
    echo "Validating extension structure..."
    
    # Check required files exist
    local files=(
        "$EXTENSION_DIR/radium-extension.json"
        "$EXTENSION_DIR/plugin/radium.lua"
        "$EXTENSION_DIR/plugin/radium/commands.lua"
        "$EXTENSION_DIR/plugin/radium/utils.lua"
        "$EXTENSION_DIR/plugin/radium/diff.lua"
        "$EXTENSION_DIR/hooks/editor-context.toml"
        "$EXTENSION_DIR/hooks/code-apply.toml"
    )
    
    local failed=0
    for file in "${files[@]}"; do
        if [ -f "$file" ]; then
            echo "  ✓ $file"
        else
            echo "  ✗ $file (missing)"
            failed=1
        fi
    done
    
    # Validate manifest JSON
    if command -v jq >/dev/null 2>&1; then
        echo ""
        echo "Validating manifest..."
        if jq empty "$EXTENSION_DIR/radium-extension.json" 2>/dev/null; then
            echo "  ✓ Manifest JSON is valid"
        else
            echo "  ✗ Manifest JSON is invalid"
            failed=1
        fi
    fi
    
    if [ $failed -eq 0 ]; then
        echo ""
        echo "✓ Basic validation passed"
        exit 0
    else
        echo ""
        echo "✗ Validation failed"
        exit 1
    fi
else
    # Run full test suite with plenary.nvim
    echo "Running full test suite..."
    nvim --headless \
        --noplugin \
        -u NONE \
        -c "lua package.path = '$EXTENSION_DIR/plugin/?.lua;' .. package.path" \
        -c "lua require('plenary.test_harness').test_directory('$EXTENSION_DIR/tests')" \
        -c "qa"
fi

