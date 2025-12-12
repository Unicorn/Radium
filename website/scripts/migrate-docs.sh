#!/bin/bash

# migrate-docs.sh
# Migrates documentation from /docs to /website/docs following the mapping in MIGRATION-MAPPING.md

set -e  # Exit on error

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Paths
SOURCE_DIR="../docs"
TARGET_DIR="./docs"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WEBSITE_DIR="$(dirname "$SCRIPT_DIR")"

cd "$WEBSITE_DIR"

echo -e "${BLUE}=== Radium Documentation Migration ===${NC}\n"

# Create backup of existing website/docs
if [ -d "$TARGET_DIR" ]; then
    echo -e "${YELLOW}Creating backup of existing docs...${NC}"
    BACKUP_DIR="./docs.backup.$(date +%Y%m%d_%H%M%S)"
    cp -r "$TARGET_DIR" "$BACKUP_DIR"
    echo -e "${GREEN}✓ Backup created at $BACKUP_DIR${NC}\n"
fi

# Clear existing docs (except the ones we manually created)
echo -e "${YELLOW}Clearing old docs...${NC}"
rm -rf "$TARGET_DIR"
mkdir -p "$TARGET_DIR"

# Keep our manually created files
echo -e "${BLUE}Preserving manually created introduction and installation...${NC}"
if [ -f "$BACKUP_DIR/introduction.md" ]; then
    cp "$BACKUP_DIR/introduction.md" "$TARGET_DIR/"
fi
if [ -d "$BACKUP_DIR/getting-started" ]; then
    mkdir -p "$TARGET_DIR/getting-started"
    cp -r "$BACKUP_DIR/getting-started/"* "$TARGET_DIR/getting-started/" 2>/dev/null || true
fi

echo -e "${GREEN}✓ Preserved existing docs${NC}\n"

# Function to copy directory
copy_category() {
    local source=$1
    local target=$2
    local label=$3

    if [ -d "$SOURCE_DIR/$source" ]; then
        echo -e "${BLUE}Migrating $label...${NC}"
        mkdir -p "$TARGET_DIR/$target"
        cp -r "$SOURCE_DIR/$source/"* "$TARGET_DIR/$target/" 2>/dev/null || true
        local count=$(find "$TARGET_DIR/$target" -name "*.md" -type f | wc -l)
        echo -e "${GREEN}✓ Copied $count markdown files from $source${NC}"
    else
        echo -e "${YELLOW}⚠ Skipping $source (not found)${NC}"
    fi
}

# Priority 1: Direct matches (keep as-is)
echo -e "\n${BLUE}=== Priority 1: Direct Matches ===${NC}"
copy_category "user-guide" "user-guide" "User Guide"
copy_category "features" "features" "Features"
copy_category "cli" "cli" "CLI Reference"
copy_category "examples" "examples" "Examples"
copy_category "developer-guide" "developer-guide" "Developer Guide"
copy_category "extensions" "extensions" "Extensions"
copy_category "hooks" "hooks" "Hooks"
copy_category "mcp" "mcp" "MCP Integration"
copy_category "api" "api" "API Reference"

# Priority 2: Reorganize into developer-guide
echo -e "\n${BLUE}=== Priority 2: Developer Guide Subcategories ===${NC}"
copy_category "architecture" "developer-guide/architecture" "Architecture"
copy_category "testing" "developer-guide/testing" "Testing"
copy_category "development" "developer-guide/development" "Development"
copy_category "design" "developer-guide/design" "Design"
copy_category "adr" "developer-guide/adr" "ADR (Architecture Decision Records)"

# Priority 3: Features subcategories
echo -e "\n${BLUE}=== Priority 3: Features Subcategories ===${NC}"
copy_category "monitoring" "features/monitoring" "Monitoring"
copy_category "planning" "features/planning" "Planning"
copy_category "security" "features/security" "Security"
copy_category "editor-integration" "features/editor-integration" "Editor Integration"
copy_category "yolo-mode" "features/yolo-mode" "YOLO Mode"

# Priority 4: Rename conventions
echo -e "\n${BLUE}=== Priority 4: Renames ===${NC}"
copy_category "self-hosted-models" "self-hosted" "Self-Hosted Models"

# Merge guides into user-guide
if [ -d "$SOURCE_DIR/guides" ]; then
    echo -e "\n${BLUE}Merging guides into user-guide...${NC}"
    mkdir -p "$TARGET_DIR/user-guide/guides"
    cp -r "$SOURCE_DIR/guides/"* "$TARGET_DIR/user-guide/guides/" 2>/dev/null || true
    local count=$(find "$TARGET_DIR/user-guide/guides" -name "*.md" -type f | wc -l)
    echo -e "${GREEN}✓ Merged $count guides${NC}"
fi

# Copy standalone root files to appropriate locations
echo -e "\n${BLUE}=== Processing Root Files ===${NC}"
if [ -f "$SOURCE_DIR/async-requirement-execution.md" ]; then
    mkdir -p "$TARGET_DIR/features/requirements"
    cp "$SOURCE_DIR/async-requirement-execution.md" "$TARGET_DIR/features/requirements/"
    echo -e "${GREEN}✓ Moved async-requirement-execution.md to features/requirements${NC}"
fi

if [ -f "$SOURCE_DIR/requirement-execution-ux.md" ]; then
    mkdir -p "$TARGET_DIR/features/requirements"
    cp "$SOURCE_DIR/requirement-execution-ux.md" "$TARGET_DIR/features/requirements/"
    echo -e "${GREEN}✓ Moved requirement-execution-ux.md to features/requirements${NC}"
fi

if [ -f "$SOURCE_DIR/json-schema-guide.md" ]; then
    mkdir -p "$TARGET_DIR/developer-guide/guides"
    cp "$SOURCE_DIR/json-schema-guide.md" "$TARGET_DIR/developer-guide/guides/"
    echo -e "${GREEN}✓ Moved json-schema-guide.md to developer-guide/guides${NC}"
fi

if [ -f "$SOURCE_DIR/mcp-proxy.md" ]; then
    cp "$SOURCE_DIR/mcp-proxy.md" "$TARGET_DIR/mcp/"
    echo -e "${GREEN}✓ Moved mcp-proxy.md to mcp${NC}"
fi

if [ -f "$SOURCE_DIR/mcp-proxy-config.md" ]; then
    cp "$SOURCE_DIR/mcp-proxy-config.md" "$TARGET_DIR/mcp/"
    echo -e "${GREEN}✓ Moved mcp-proxy-config.md to mcp${NC}"
fi

# Create getting-started directory structure
echo -e "\n${BLUE}=== Setting up Getting Started ===${NC}"
mkdir -p "$TARGET_DIR/getting-started"
# Note: installation.md and introduction.md already exist from backup

# Statistics
echo -e "\n${BLUE}=== Migration Statistics ===${NC}"
total_files=$(find "$TARGET_DIR" -name "*.md" -type f | wc -l)
total_dirs=$(find "$TARGET_DIR" -type d | wc -l)
echo -e "${GREEN}Total markdown files: $total_files${NC}"
echo -e "${GREEN}Total directories: $total_dirs${NC}"

# List all top-level categories
echo -e "\n${BLUE}Top-level documentation structure:${NC}"
ls -la "$TARGET_DIR" | grep "^d" | awk '{print "  " $9}' | grep -v "^\.$" | grep -v "^\.\.$ "

echo -e "\n${GREEN}=== Migration Complete! ===${NC}"
echo -e "${YELLOW}Next steps:${NC}"
echo -e "  1. Run: ${BLUE}bun run scripts/add-frontmatter.js${NC}"
echo -e "  2. Run: ${BLUE}bun run scripts/rewrite-links.js${NC}"
echo -e "  3. Review and test: ${BLUE}bun start${NC}"
