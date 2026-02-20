#!/bin/bash

# fix-mdx-errors.sh
# Fixes MDX compilation errors by escaping angle brackets in markdown text

set -e

DOCS_DIR="./docs"

echo "Fixing MDX compilation errors..."

# Fix angle brackets in text (not in code blocks)
# Replace <number with &lt;number
find "$DOCS_DIR" -name "*.md" -type f -exec sed -i '' \
  -e 's/\([^`]\)<\([0-9]\)/\1\&lt;\2/g' \
  -e 's/^\([^`]*\)<\([0-9]\)/\1\&lt;\2/g' \
  {} \;

# Fix specific tag-like patterns that aren't meant to be tags
find "$DOCS_DIR" -name "*.md" -type f -exec sed -i '' \
  -e 's/<source>/<source\>/g' \
  -e 's/<Client>/<Client\>/g' \
  -e 's/<server>/<server\>/g' \
  {} \;

echo "✓ MDX errors fixed!"
echo ""
echo "Files that were modified:"
find "$DOCS_DIR" -name "*.md" -type f -exec grep -l '&lt;' {} \; || echo "  (none with &lt;)"
