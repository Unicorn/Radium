#!/usr/bin/env python3
"""
Sync local REQ documents to Braingrid.

This script parses REQ documents from /docs/plan and syncs them to Braingrid,
creating new requirements or updating existing ones.
"""

import os
import sys
import subprocess
import re
import json
from pathlib import Path
from typing import Dict, List, Optional, Tuple

# Project configuration
PROJECT_ID = "PROJ-14"
DOCS_PLAN_DIR = Path(__file__).parent.parent / "docs" / "plan"
PHASE_DIRS = ["01-now", "02-next", "03-later"]

# Status mapping
STATUS_MAP = {
    "Not Started": "PLANNED",
    "In Progress": "IN_PROGRESS",
    "Completed": "COMPLETED",
}


def parse_yaml_front_matter(content: str) -> Tuple[Dict, str]:
    """Parse YAML front matter from markdown content."""
    if not content.startswith("---"):
        return {}, content
    
    # Find the end of front matter
    lines = content.split("\n")
    if len(lines) < 2:
        return {}, content
    
    end_idx = None
    for i in range(1, len(lines)):
        if lines[i].strip() == "---":
            end_idx = i
            break
    
    if end_idx is None:
        return {}, content
    
    # Extract front matter
    front_matter_lines = lines[1:end_idx]
    
    # Parse YAML (simple parser for our use case)
    metadata = {}
    i = 0
    while i < len(front_matter_lines):
        line = front_matter_lines[i].rstrip()
        if not line or line.startswith("#"):
            i += 1
            continue
        
        if ":" in line:
            key, value = line.split(":", 1)
            key = key.strip()
            value = value.strip()
            
            # Remove quotes if present
            if (value.startswith('"') and value.endswith('"')) or (value.startswith("'") and value.endswith("'")):
                value = value[1:-1]
            
            # Handle list values
            if value.startswith("[") and value.endswith("]"):
                # Simple list parsing
                value = value[1:-1].strip()
                if value:
                    metadata[key] = [v.strip().strip('"').strip("'") for v in value.split(",") if v.strip()]
                else:
                    metadata[key] = []
            # Handle multi-line list values (YAML list format)
            elif value == "" and i + 1 < len(front_matter_lines):
                # Check if next line starts with "-"
                if front_matter_lines[i + 1].strip().startswith("-"):
                    items = []
                    i += 1
                    while i < len(front_matter_lines) and front_matter_lines[i].strip().startswith("-"):
                        item = front_matter_lines[i].strip()[1:].strip()
                        # Remove quotes
                        if (item.startswith('"') and item.endswith('"')) or (item.startswith("'") and item.endswith("'")):
                            item = item[1:-1]
                        items.append(item)
                        i += 1
                    metadata[key] = items
                    continue
                else:
                    metadata[key] = value
            else:
                metadata[key] = value
        
        i += 1
    
    # Extract markdown content (after front matter)
    markdown_content = "\n".join(lines[end_idx + 1:])
    
    return metadata, markdown_content


def read_req_file(file_path: Path) -> Tuple[Dict, str]:
    """Read REQ file and return metadata and full content."""
    with open(file_path, "r", encoding="utf-8") as f:
        content = f.read()
    
    metadata, markdown_content = parse_yaml_front_matter(content)
    
    # Return full content (with front matter) for Braingrid
    return metadata, content


def run_braingrid_command(cmd: List[str]) -> Tuple[bool, str, str]:
    """Run a braingrid CLI command and return success, stdout, stderr."""
    try:
        result = subprocess.run(
            ["braingrid"] + cmd,
            capture_output=True,
            text=True,
            check=False
        )
        return result.returncode == 0, result.stdout, result.stderr
    except Exception as e:
        return False, "", str(e)


def list_braingrid_requirements() -> List[Dict]:
    """List all requirements in Braingrid."""
    # Try JSON format first, fall back to text parsing
    success, stdout, stderr = run_braingrid_command([
        "requirement", "list", "-p", PROJECT_ID
    ])
    
    if not success:
        print(f"Error listing requirements: {stderr}", file=sys.stderr)
        return []
    
    # Parse text output
    reqs = []
    lines = stdout.split("\n")
    in_table = False
    
    for line in lines:
        line = line.strip()
        if not line or line.startswith("📋") or line.startswith("Short ID"):
            continue
        if "──" in line:  # Table separator
            in_table = True
            continue
        if in_table and line:
            # Parse table row: "REQ-1        👀 REVIEW        Organize and Structure   N/A                 100%"
            parts = line.split()
            if len(parts) >= 3:
                short_id = parts[0]
                status = parts[1] if parts[1] not in ["👀", "📝", "✅", "❌"] else parts[2] if len(parts) > 2 else "UNKNOWN"
                # Name might be multiple words, find where it ends (before Branch column)
                # Simple heuristic: status is usually a single word, then name starts
                name_parts = []
                i = 2 if parts[1] in ["👀", "📝", "✅", "❌"] else 1
                while i < len(parts) and parts[i] not in ["N/A", "100%", "0%"] and not parts[i].endswith("%"):
                    name_parts.append(parts[i])
                    i += 1
                name = " ".join(name_parts) if name_parts else "Unknown"
                
                reqs.append({
                    "short_id": short_id,
                    "name": name,
                    "status": status
                })
    
    return reqs


def get_requirement_details(short_id: str) -> Optional[Dict]:
    """Get full requirement details including ID by short_id."""
    success, stdout, stderr = run_braingrid_command([
        "requirement", "show", short_id, "-p", PROJECT_ID
    ])
    
    if not success:
        return None
    
    # Parse the output to extract ID
    # The output format shows "ID: <uuid>" on a line
    for line in stdout.split("\n"):
        if line.startswith("ID:") or line.startswith("Short ID:"):
            # Try to extract UUID from the output
            # Format: "ID: 2edf1bd1-9edd-442d-84d2-f326c966bdc9"
            parts = line.split(":")
            if len(parts) >= 2:
                value = parts[1].strip()
                if "ID:" in line and len(value) > 10:  # UUID-like
                    return {"id": value, "short_id": short_id}
    
    return None


def find_existing_req(title: str, req_id: str, existing_reqs: List[Dict]) -> Optional[Dict]:
    """Find existing REQ in Braingrid by title or short_id. Returns full req dict with ID."""
    # Try to match by short_id pattern (REQ-001, REQ-1, etc.)
    req_num = req_id.replace("REQ-", "").lstrip("0") if req_id else ""
    patterns = []
    if req_num:
        patterns = [
            f"REQ-{req_num}",
            f"REQ-{req_id.replace('REQ-', '').zfill(3)}",  # REQ-001 format
            req_id,
        ]
    
    for req in existing_reqs:
        short_id = req.get("short_id", "")
        name = req.get("name", "")
        
        # Check short_id
        if short_id in patterns:
            # Get full details including ID
            details = get_requirement_details(short_id)
            if details:
                details["name"] = name
                return details
            # Fallback: use short_id as ID (some commands accept short_id)
            req["id"] = short_id
            return req
        
        # Check name similarity (fuzzy match)
        if name.lower() == title.lower():
            # Get full details including ID
            details = get_requirement_details(short_id)
            if details:
                details["name"] = name
                return details
            # Fallback
            req["id"] = short_id
            return req
    
    return None


def create_requirement(title: str, content: str, status: str) -> Tuple[bool, Optional[Dict]]:
    """Create a new requirement in Braingrid."""
    success, stdout, stderr = run_braingrid_command([
        "requirement", "create",
        "-p", PROJECT_ID,
        "-n", title,
        "-c", content
    ])
    
    if not success:
        print(f"Error creating requirement '{title}': {stderr}", file=sys.stderr)
        if stdout:
            print(f"  stdout: {stdout}", file=sys.stderr)
        return False, None
    
    # Get the created REQ by listing again
    reqs = list_braingrid_requirements()
    created_req = find_existing_req(title, "", reqs)
    
    if created_req:
        # Update status if needed (PLANNED is default, but we want to be explicit)
        if status and status != "PLANNED":
            req_id = created_req.get("id")
            if req_id:
                update_requirement_status(req_id, status)
        return True, created_req
    
    return True, None


def update_requirement(requirement_id: str, title: str, content: str, status: str) -> bool:
    """Update an existing requirement in Braingrid."""
    # Update status
    if status:
        success = update_requirement_status(requirement_id, status)
        if not success:
            return False
    
    # Update name if different (check if needed)
    # Note: Braingrid CLI may not support updating content directly
    # For now, we'll update status and name if different
    # Content updates may need to be done manually or via API
    
    return True


def update_requirement_status(requirement_id: str, status: str) -> bool:
    """Update requirement status in Braingrid."""
    if not requirement_id:
        print(f"Error: No requirement ID provided for status update", file=sys.stderr)
        return False
    
    success, stdout, stderr = run_braingrid_command([
        "requirement", "update",
        requirement_id,
        "-p", PROJECT_ID,
        "--status", status
    ])
    
    if not success:
        print(f"Error updating requirement status: {stderr}", file=sys.stderr)
        if stdout:
            print(f"  stdout: {stdout}", file=sys.stderr)
        return False
    
    return True


def map_status(local_status: str) -> str:
    """Map local status to Braingrid status."""
    return STATUS_MAP.get(local_status, "PLANNED")


def sync_req_file(file_path: Path, existing_reqs: List[Dict]) -> Tuple[bool, str]:
    """Sync a single REQ file to Braingrid."""
    print(f"Processing {file_path.name}...")
    
    try:
        metadata, full_content = read_req_file(file_path)
        
        req_id = metadata.get("req_id", "")
        title = metadata.get("title", "")
        local_status = metadata.get("status", "Not Started")
        status = map_status(local_status)
        
        if not title:
            print(f"  Warning: No title found in {file_path.name}", file=sys.stderr)
            return False, "No title"
        
        # Find existing REQ
        existing_req = find_existing_req(title, req_id, existing_reqs)
        
        if existing_req:
            req_id_braingrid = existing_req.get("id")
            short_id = existing_req.get("short_id", "?")
            print(f"  Updating existing REQ: {title} ({short_id}, ID: {req_id_braingrid})")
            success = update_requirement(req_id_braingrid, title, full_content, status)
            if success:
                return True, f"Updated {title} ({short_id})"
            else:
                return False, f"Failed to update {title}"
        else:
            print(f"  Creating new REQ: {title}")
            success, new_req = create_requirement(title, full_content, status)
            if success:
                short_id = new_req.get("short_id", "?") if new_req else "?"
                return True, f"Created {title} ({short_id})"
            else:
                return False, f"Failed to create {title}"
    
    except Exception as e:
        print(f"  Error processing {file_path.name}: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        return False, str(e)


def main():
    """Main sync function."""
    print("Syncing local REQs to Braingrid...")
    print(f"Project ID: {PROJECT_ID}")
    print(f"Docs plan directory: {DOCS_PLAN_DIR}")
    print()
    
    # Check if docs/plan directory exists
    if not DOCS_PLAN_DIR.exists():
        print(f"Error: {DOCS_PLAN_DIR} does not exist", file=sys.stderr)
        sys.exit(1)
    
    # List existing requirements in Braingrid
    print("Fetching existing requirements from Braingrid...")
    existing_reqs = list_braingrid_requirements()
    print(f"Found {len(existing_reqs)} existing requirements")
    print()
    
    # Collect all REQ files
    req_files = []
    for phase_dir in PHASE_DIRS:
        phase_path = DOCS_PLAN_DIR / phase_dir
        if phase_path.exists():
            for file_path in phase_path.glob("REQ-*.md"):
                req_files.append(file_path)
    
    req_files.sort()  # Process in order
    
    print(f"Found {len(req_files)} REQ files to sync")
    print()
    
    # Sync each REQ
    results = []
    for req_file in req_files:
        success, message = sync_req_file(req_file, existing_reqs)
        results.append((req_file.name, success, message))
        # Refresh existing reqs list after each creation
        if success and "Created" in message:
            existing_reqs = list_braingrid_requirements()
        print()
    
    # Print summary
    print("=" * 60)
    print("Sync Summary")
    print("=" * 60)
    successful = sum(1 for _, success, _ in results if success)
    failed = len(results) - successful
    
    print(f"Total: {len(results)}")
    print(f"Successful: {successful}")
    print(f"Failed: {failed}")
    print()
    
    if failed > 0:
        print("Failed REQs:")
        for name, success, message in results:
            if not success:
                print(f"  - {name}: {message}")
        sys.exit(1)
    else:
        print("All REQs synced successfully!")
        sys.exit(0)


if __name__ == "__main__":
    main()

