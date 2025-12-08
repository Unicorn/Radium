#!/bin/bash
# Code application hook for AfterTool
#
# This hook processes agent output after tool execution to extract code blocks
# and format them for editor application.
#
# Input: HookContext JSON from stdin with tool output
# Output: HookResult JSON with structured code blocks

set -euo pipefail

# Read HookContext JSON from stdin
input=$(cat)

# Extract tool output from context
if command -v jq >/dev/null 2>&1; then
    # Extract output from tool result
    tool_output=$(echo "$input" | jq -r '.data.result.output // .data.output // ""')
    
    if [ -z "$tool_output" ] || [ "$tool_output" == "null" ]; then
        # No output to process, pass through
        echo "$input" | jq '.should_continue = true'
        exit 0
    fi
    
    # Extract markdown code blocks using regex
    # Pattern: ```[lang]\ncode\n```
    code_blocks_json=$(echo "$tool_output" | awk '
        BEGIN { 
            in_block = 0
            current_lang = ""
            current_code = ""
            block_count = 0
            print "["
        }
        /^```/ {
            if (in_block) {
                # End of code block
                if (block_count > 0) print ","
                printf "  {\n"
                printf "    \"language\": \"%s\",\n", current_lang
                printf "    \"content\": %s,\n", current_code
                printf "    \"index\": %d\n", block_count
                printf "  }"
                in_block = 0
                current_lang = ""
                current_code = ""
                block_count++
            } else {
                # Start of code block
                in_block = 1
                current_lang = $0
                gsub(/```/, "", current_lang)
                gsub(/^[[:space:]]+|[[:space:]]+$/, "", current_lang)
                current_code = ""
            }
            next
        }
        in_block {
            if (current_code == "") {
                current_code = "\"" $0 "\""
            } else {
                current_code = current_code "\\n" "\"" $0 "\""
            }
        }
        END {
            if (in_block && block_count > 0) {
                # Handle unclosed block
                printf ","
                printf "  {\n"
                printf "    \"language\": \"%s\",\n", current_lang
                printf "    \"content\": %s,\n", current_code
                printf "    \"index\": %d\n", block_count
                printf "  }"
            }
            print "\n]"
        }
    ')
    
    # Build modified result with structured code blocks
    result=$(echo "$input" | jq --argjson blocks "$code_blocks_json" '
        . as $ctx |
        {
            should_continue: true,
            modified_data: {
                original_output: ($ctx.data.result.output // $ctx.data.output // ""),
                code_blocks: $blocks,
                block_count: ($blocks | length)
            }
        }
    ')
    
    echo "$result"
else
    # Fallback: pass through unchanged if jq not available
    echo "$input" | jq '.should_continue = true' 2>/dev/null || echo "$input"
fi

