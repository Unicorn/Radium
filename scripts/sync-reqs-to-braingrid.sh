#!/bin/bash
# Bidirectional sync between local REQ documents and Braingrid using CLI

set -euo pipefail

PROJECT_ID="PROJ-14"
DOCS_PLAN_DIR="docs/plan"
PHASE_DIRS=("01-now" "02-next" "03-later")

# Status mapping
map_status() {
    case "$1" in
        "Not Started") echo "PLANNED" ;;
        "In Progress") echo "IN_PROGRESS" ;;
        "Completed") echo "COMPLETED" ;;
        *) echo "PLANNED" ;;
    esac
}

# Extract YAML front matter value
extract_yaml_value() {
    local file="$1"
    local key="$2"
    grep "^${key}:" "$file" | head -1 | sed "s/^${key}:[[:space:]]*//" | sed 's/^"\(.*\)"$/\1/' | sed "s/^'\(.*\)'$/\1/"
}

# Get REQ ID from short_id
get_req_id() {
    local short_id="$1"
    braingrid requirement show "$short_id" -p "$PROJECT_ID" 2>/dev/null | grep "^ID:" | awk '{print $2}' || echo ""
}

# Find existing REQ by name
find_req_by_name() {
    local title="$1"
    braingrid requirement list -p "$PROJECT_ID" 2>/dev/null | \
        grep -i "$title" | \
        awk '{print $1}' | \
        head -1 || echo ""
}

# Sync a single REQ file
sync_req_file() {
    local file="$1"
    local filename=$(basename "$file")
    
    echo "Processing $filename..."
    
    # Extract metadata
    local req_id=$(extract_yaml_value "$file" "req_id")
    local title=$(extract_yaml_value "$file" "title")
    local local_status=$(extract_yaml_value "$file" "status")
    local status=$(map_status "$local_status")
    
    if [ -z "$title" ]; then
        echo "  ⚠️  Warning: No title found in $filename"
        return 1
    fi
    
    # Read full content
    local content=$(cat "$file")
    
    # Find existing REQ
    local short_id=""
    
    # Try to find by REQ ID pattern
    if [ -n "$req_id" ]; then
        local req_num=$(echo "$req_id" | sed 's/REQ-//' | sed 's/^0*//')
        if [ -n "$req_num" ]; then
            # Try REQ-1, REQ-01, REQ-001 formats
            for pattern in "REQ-$req_num" "REQ-$(printf "%02d" "$req_num")" "REQ-$(printf "%03d" "$req_num")"; do
                if braingrid requirement show "$pattern" -p "$PROJECT_ID" >/dev/null 2>&1; then
                    short_id="$pattern"
                    break
                fi
            done
        fi
    fi
    
    # If not found by ID, try by name
    if [ -z "$short_id" ]; then
        short_id=$(find_req_by_name "$title")
    fi
    
    if [ -n "$short_id" ]; then
        echo "  📝 Updating existing REQ: $title ($short_id)"
        
        # Get the actual REQ ID (UUID)
        local req_uuid=$(get_req_id "$short_id")
        
        if [ -z "$req_uuid" ]; then
            echo "  ❌ Error: Could not get REQ ID for $short_id"
            return 1
        fi
        
        # Update status
        if braingrid requirement update "$req_uuid" -p "$PROJECT_ID" --status "$status" >/dev/null 2>&1; then
            echo "  ✅ Updated status to $status"
        else
            echo "  ⚠️  Warning: Could not update status"
        fi
        
        # Update name if different (check current name first)
        local current_name=$(braingrid requirement show "$short_id" -p "$PROJECT_ID" 2>/dev/null | grep -v "^$" | head -1 | sed 's/^[^a-zA-Z]*//')
        if [ "$current_name" != "$title" ]; then
            if braingrid requirement update "$req_uuid" -p "$PROJECT_ID" --name "$title" >/dev/null 2>&1; then
                echo "  ✅ Updated name"
            else
                echo "  ⚠️  Warning: Could not update name"
            fi
        fi
        
        echo "  ✅ Updated $title ($short_id)"
        return 0
    else
        echo "  ➕ Creating new REQ: $title"
        
        if braingrid requirement create -p "$PROJECT_ID" -n "$title" -c "$content" >/dev/null 2>&1; then
            echo "  ✅ Created $title"
            return 0
        else
            echo "  ❌ Error: Failed to create $title"
            return 1
        fi
    fi
}

# Main sync function
main() {
    local direction="${1:-bidirectional}"
    
    echo "============================================================"
    echo "Bidirectional REQ Sync (CLI-based)"
    echo "============================================================"
    echo "Project ID: $PROJECT_ID"
    echo "Docs plan directory: $DOCS_PLAN_DIR"
    echo "Direction: $direction"
    echo ""
    
    # Check if docs/plan directory exists
    if [ ! -d "$DOCS_PLAN_DIR" ]; then
        echo "❌ Error: $DOCS_PLAN_DIR does not exist"
        exit 1
    fi
    
    # Collect all REQ files
    local req_files=()
    for phase_dir in "${PHASE_DIRS[@]}"; do
        local phase_path="$DOCS_PLAN_DIR/$phase_dir"
        if [ -d "$phase_path" ]; then
            while IFS= read -r -d '' file; do
                req_files+=("$file")
            done < <(find "$phase_path" -name "REQ-*.md" -type f -print0 | sort -z)
        fi
    done
    
    echo "Found ${#req_files[@]} REQ files to sync"
    echo ""
    
    # Sync Local → Braingrid
    if [[ "$direction" == "to-braingrid" || "$direction" == "bidirectional" ]]; then
        echo "============================================================"
        echo "Syncing Local → Braingrid"
        echo "============================================================"
        echo ""
        
        local success=0
        local failed=0
        
        for file in "${req_files[@]}"; do
            if sync_req_file "$file"; then
                ((success++))
            else
                ((failed++))
            fi
            echo ""
        done
        
        echo "Local → Braingrid: $success successful, $failed failed"
        echo ""
    fi
    
    # TODO: Braingrid → Local sync (update local status from Braingrid)
    if [[ "$direction" == "from-braingrid" || "$direction" == "bidirectional" ]]; then
        echo "============================================================"
        echo "Syncing Braingrid → Local"
        echo "============================================================"
        echo ""
        echo "⚠️  Note: Braingrid → Local sync not yet implemented"
        echo "   Use --update-status flag with Python script for status updates"
        echo ""
    fi
    
    echo "============================================================"
    echo "Sync Complete"
    echo "============================================================"
}

# Run main function
main "${@:-bidirectional}"

