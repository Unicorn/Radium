#!/bin/bash
# Generate Rust API documentation for all public crates
# This script generates documentation using cargo doc and copies it to the static directory

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Script directory (where this script is located)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Project root (parent of website directory)
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
# Website directory
WEBSITE_DIR="$SCRIPT_DIR/.."
# Static API directory
STATIC_API_DIR="$WEBSITE_DIR/static/api"

echo -e "${GREEN}Generating Rust API documentation...${NC}"

# Change to project root
cd "$PROJECT_ROOT"

# Ensure we have Rust toolchain
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: cargo not found. Please install Rust toolchain.${NC}" >&2
    exit 1
fi

# Public crates to document
CRATES=(
    "radium-core"
    "radium-abstraction"
    "radium-models"
    "radium-orchestrator"
)

# Generate documentation for all crates
echo -e "${YELLOW}Running cargo doc for all public crates...${NC}"
cargo doc --no-deps \
    --package radium-core \
    --package radium-abstraction \
    --package radium-models \
    --package radium-orchestrator || {
    echo -e "${RED}Error: Failed to generate Rust documentation${NC}" >&2
    exit 1
}

# Create static API directory if it doesn't exist
mkdir -p "$STATIC_API_DIR"

# Copy documentation for each crate
# Cargo doc generates docs in target/doc/ by default (or dist/target/doc if configured)
# Try to find the actual doc directory
if [ -d "$PROJECT_ROOT/dist/target/doc" ]; then
    DOC_ROOT="$PROJECT_ROOT/dist/target/doc"
elif [ -d "$PROJECT_ROOT/target/doc" ]; then
    DOC_ROOT="$PROJECT_ROOT/target/doc"
else
    # Try to find it anywhere
    DOC_ROOT=$(find "$PROJECT_ROOT" -type d -name "doc" -path "*/target/doc" 2>/dev/null | head -1)
    if [ -z "$DOC_ROOT" ]; then
        echo -e "${RED}Error: Could not find generated documentation directory${NC}" >&2
        exit 1
    fi
fi
echo -e "${GREEN}Using documentation directory: $DOC_ROOT${NC}"
for crate in "${CRATES[@]}"; do
    crate_name="${crate//-/_}"  # Convert kebab-case to snake_case
    doc_source="$DOC_ROOT/$crate_name"
    doc_dest="$STATIC_API_DIR/$crate_name"
    
    if [ ! -d "$doc_source" ]; then
        echo -e "${YELLOW}Warning: Documentation not found for $crate at $doc_source${NC}" >&2
        continue
    fi
    
    echo -e "${GREEN}Copying documentation for $crate...${NC}"
    
    # Remove existing destination if it exists
    if [ -d "$doc_dest" ]; then
        rm -rf "$doc_dest"
    fi
    
    # Copy the documentation
    cp -r "$doc_source" "$doc_dest"
    
    # Verify the copy was successful
    if [ ! -d "$doc_dest" ] || [ ! -f "$doc_dest/index.html" ]; then
        echo -e "${RED}Error: Failed to copy documentation for $crate${NC}" >&2
        exit 1
    fi
    
    echo -e "${GREEN}✓ Documentation for $crate copied to $doc_dest${NC}"
done

# Copy the main doc index and assets if they exist
if [ -d "$DOC_ROOT" ]; then
    # Copy shared assets (CSS, JS, fonts, etc.)
    if [ -d "$DOC_ROOT/static.files" ]; then
        echo -e "${GREEN}Copying shared documentation assets...${NC}"
        for crate in "${CRATES[@]}"; do
            crate_name="${crate//-/_}"
            doc_dest="$STATIC_API_DIR/$crate_name"
            if [ -d "$doc_dest" ]; then
                # Copy static files to each crate's directory
                if [ ! -d "$doc_dest/static.files" ]; then
                    cp -r "$DOC_ROOT/static.files" "$doc_dest/"
                fi
            fi
        done
    fi
fi

echo -e "${GREEN}✓ Rust API documentation generation complete!${NC}"
echo -e "${GREEN}Documentation available at:${NC}"
for crate in "${CRATES[@]}"; do
    crate_name="${crate//-/_}"
    echo -e "  - /api/$crate_name/"
done

