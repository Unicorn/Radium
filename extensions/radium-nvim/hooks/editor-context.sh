#!/bin/bash
# Editor context injection hook for BeforeTool
# 
# This hook injects editor context (file path, language, surrounding code)
# into tool execution context before Radium agents process code.
#
# Input: HookContext JSON from stdin
# Output: HookResult JSON to stdout with modified_data containing enriched context

set -euo pipefail

# Read HookContext JSON from stdin
input=$(cat)

# Extract editor context from environment variables or input
# Editor context should be provided via environment variables:
# - RADIUM_EDITOR_FILE_PATH
# - RADIUM_EDITOR_LANGUAGE
# - RADIUM_EDITOR_SELECTION
# - RADIUM_EDITOR_SURROUNDING_LINES

editor_context="{}"

# Check if editor context is available
if [ -n "${RADIUM_EDITOR_FILE_PATH:-}" ]; then
    # Build editor context JSON
    editor_context=$(jq -n \
        --arg file_path "${RADIUM_EDITOR_FILE_PATH}" \
        --arg language "${RADIUM_EDITOR_LANGUAGE:-}" \
        --arg selection "${RADIUM_EDITOR_SELECTION:-}" \
        --arg surrounding "${RADIUM_EDITOR_SURROUNDING_LINES:-}" \
        '{
            file_path: $file_path,
            language: $language,
            selection: $selection,
            surrounding_lines: $surrounding
        }')
fi

# Parse input context and merge editor context
if command -v jq >/dev/null 2>&1; then
    # Use jq to merge editor context into the hook context
    result=$(echo "$input" | jq --argjson editor "$editor_context" '
        . as $ctx |
        if $editor != {} then
            .modified_data = ($ctx.data // {} | . + {editor_context: $editor})
        else
            .modified_data = ($ctx.data // {})
        end |
        .should_continue = true
    ')
    echo "$result"
else
    # Fallback: pass through unchanged if jq not available
    echo "$input" | jq '.should_continue = true'
fi

