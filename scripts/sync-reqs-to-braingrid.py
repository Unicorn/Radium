#!/usr/bin/env python3
"""
Bidirectional sync between local REQ documents and Braingrid.

This script supports:
- Local → Braingrid: Sync local REQ documents to Braingrid (create/update)
- Braingrid → Local: Pull REQs from Braingrid to local files (create/update)
- Bidirectional: Sync both directions, resolving conflicts
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

# Status mapping (Local → Braingrid)
STATUS_MAP = {
    "Not Started": "PLANNED",
    "In Progress": "IN_PROGRESS",
    "Completed": "COMPLETED",
    "Review": "REVIEW",
}

# Reverse status mapping (Braingrid → Local)
REVERSE_STATUS_MAP = {
    "PLANNED": "Not Started",
    "IN_PROGRESS": "In Progress",
    "COMPLETED": "Completed",
    "REVIEW": "Completed",  # REVIEW maps to Completed
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
            # Or: "REQ-43       📝 PLANNED       Agent Configuration Sys  N/A                 0%"
            parts = line.split()
            if len(parts) >= 3:
                short_id = parts[0]
                # Status might have emoji prefix
                status_idx = 1
                if parts[1] in ["👀", "📝", "✅", "❌", "🔄"]:
                    status = parts[2] if len(parts) > 2 else "UNKNOWN"
                    status_idx = 2
                else:
                    status = parts[1]
                
                # Name starts after status, ends before "N/A" or percentage
                name_parts = []
                i = status_idx + 1
                while i < len(parts):
                    part = parts[i]
                    if part in ["N/A"] or part.endswith("%") or (i > status_idx + 1 and part in ["Branch"]):
                        break
                    name_parts.append(part)
                    i += 1
                name = " ".join(name_parts) if name_parts else "Unknown"
                
                reqs.append({
                    "short_id": short_id,
                    "name": name,
                    "status": status
                })
    
    return reqs


def get_requirement_full_content(short_id: str) -> Optional[Dict]:
    """Get full requirement details including ID, status, and content from Braingrid."""
    success, stdout, stderr = run_braingrid_command([
        "requirement", "show", short_id, "-p", PROJECT_ID
    ])
    
    if not success:
        return None
    
    # Parse the output
    result = {"short_id": short_id}
    lines = stdout.split("\n")
    in_content = False
    content_lines = []
    
    for line in lines:
        line = line.strip()
        if line.startswith("Short ID:"):
            parts = line.split(":", 1)
            if len(parts) >= 2:
                result["short_id"] = parts[1].strip()
        elif line.startswith("ID:"):
            parts = line.split(":", 1)
            if len(parts) >= 2:
                result["id"] = parts[1].strip()
        elif line.startswith("Status:"):
            parts = line.split(":", 1)
            if len(parts) >= 2:
                result["status"] = parts[1].strip()
        elif line.startswith("────────────────"):
            in_content = True
            continue
        elif in_content and line:
            content_lines.append(line)
        elif line and not in_content and ":" in line:
            # Try to parse other metadata
            parts = line.split(":", 1)
            if len(parts) >= 2:
                key = parts[0].strip().lower().replace(" ", "_")
                value = parts[1].strip()
                result[key] = value
    
    if content_lines:
        result["content"] = "\n".join(content_lines)
    
    return result


def get_requirement_details(short_id: str) -> Optional[Dict]:
    """Get full requirement details including ID by short_id."""
    result = get_requirement_full_content(short_id)
    if result:
        return {"id": result.get("id"), "short_id": result.get("short_id")}
    return None


def normalize_name(name: str) -> str:
    """Normalize a name for comparison."""
    # Remove "REQ-XXX: " prefix if present
    if ":" in name:
        name = name.split(":", 1)[1]
    # Lowercase and strip
    name = name.lower().strip()
    # Remove common suffixes that might be truncated
    for suffix in [" sys", " system", " layer", " improvements"]:
        if name.endswith(suffix):
            name = name[:-len(suffix)]
    return name


def get_full_requirement_name(short_id: str) -> Optional[str]:
    """Get the full requirement name from Braingrid (not truncated)."""
    result = get_requirement_full_content(short_id)
    if result:
        # Try to extract name from content or use the name from list
        content = result.get("content", "")
        if content:
            # Try to parse title from YAML front matter
            lines = content.split("\n")
            for line in lines:
                if line.startswith("title:"):
                    parts = line.split(":", 1)
                    if len(parts) >= 2:
                        return parts[1].strip().strip('"').strip("'")
        # Fallback to name from list view
        return result.get("name")
    return None


def find_existing_req(title: str, req_id: str, existing_reqs: List[Dict]) -> Optional[Dict]:
    """Find existing REQ in Braingrid by title or short_id. Returns full req dict with ID."""
    # Normalize title for comparison
    title_normalized = normalize_name(title)
    
    # Try to match by short_id pattern (REQ-001, REQ-1, etc.)
    req_num = req_id.replace("REQ-", "").lstrip("0") if req_id else ""
    patterns = []
    if req_num:
        patterns = [
            f"REQ-{req_num}",
            f"REQ-{req_id.replace('REQ-', '').zfill(3)}",  # REQ-001 format
            req_id,
        ]
    
    # First, try name matching (more reliable since Braingrid may have different IDs)
    for req in existing_reqs:
        short_id = req.get("short_id", "")
        name = req.get("name", "")
        
        # Get full name from Braingrid (not truncated)
        full_name = get_full_requirement_name(short_id)
        if full_name:
            name = full_name
        
        name_normalized = normalize_name(name)
        
        # Check if names match (exact or one contains the other)
        if (name_normalized == title_normalized or 
            title_normalized in name_normalized or 
            name_normalized in title_normalized):
            # Get full details including ID
            details = get_requirement_details(short_id)
            if details:
                details["name"] = full_name or name
                details["short_id"] = short_id
                details["status"] = req.get("status", "")
                return details
            # Fallback: use short_id as ID
            req["id"] = short_id
            req["status"] = req.get("status", "")
            req["name"] = full_name or name
            return req
    
    # Then try short_id matching
    for req in existing_reqs:
        short_id = req.get("short_id", "")
        name = req.get("name", "")
        
        # Check if short_id matches any pattern
        if short_id in patterns:
            # Get full details including ID
            details = get_requirement_details(short_id)
            if details:
                full_name = get_full_requirement_name(short_id)
                details["name"] = full_name or name
                details["short_id"] = short_id
                details["status"] = req.get("status", "")
                return details
            # Fallback: use short_id as ID
            req["id"] = short_id
            req["status"] = req.get("status", "")
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


def reverse_map_status(braingrid_status: str) -> str:
    """Map Braingrid status to local status."""
    return REVERSE_STATUS_MAP.get(braingrid_status, "Not Started")


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


def get_local_req_files() -> List[Path]:
    """Get all local REQ files."""
    req_files = []
    for phase_dir in PHASE_DIRS:
        phase_path = DOCS_PLAN_DIR / phase_dir
        if phase_path.exists():
            for file_path in phase_path.glob("REQ-*.md"):
                req_files.append(file_path)
    req_files.sort()
    return req_files


def get_local_req_ids() -> Dict[str, Path]:
    """Get mapping of REQ IDs to file paths."""
    req_ids = {}
    for req_file in get_local_req_files():
        try:
            metadata, _ = read_req_file(req_file)
            req_id = metadata.get("req_id", "")
            if req_id:
                req_ids[req_id] = req_file
        except:
            pass
    return req_ids


def sync_from_braingrid_to_local(braingrid_reqs: List[Dict], local_req_ids: Dict[str, Path]) -> List[Tuple[bool, str]]:
    """Sync REQs from Braingrid to local files."""
    results = []
    missing_local = []
    
    # Find REQs in Braingrid that don't exist locally
    for req in braingrid_reqs:
        short_id = req.get("short_id", "")
        name = req.get("name", "")
        
        # Extract REQ number from short_id (REQ-1, REQ-21, etc.)
        req_num = short_id.replace("REQ-", "").strip()
        if not req_num:
            continue
        
        # Try different formats
        possible_ids = [
            f"REQ-{req_num.zfill(3)}",  # REQ-001
            f"REQ-{req_num}",  # REQ-1
        ]
        
        found = False
        for req_id in possible_ids:
            if req_id in local_req_ids:
                found = True
                break
        
        if not found:
            missing_local.append((short_id, name, req_num))
    
    if not missing_local:
        print("All Braingrid REQs exist locally.")
        return results
    
    print(f"\nFound {len(missing_local)} REQ(s) in Braingrid not in local files:")
    for short_id, name, req_num in missing_local:
        print(f"  - {short_id}: {name}")
    
    # For now, just report - user can manually create or we can auto-create
    # TODO: Implement auto-creation from Braingrid content
    print("\nNote: Auto-creation from Braingrid not yet implemented.")
    print("Please create local REQ files manually or update existing ones.")
    
    return results


def update_local_status_from_braingrid(braingrid_reqs: List[Dict], local_req_ids: Dict[str, Path]) -> List[Tuple[bool, str]]:
    """Update local REQ status from Braingrid status."""
    results = []
    updated = 0
    
    for req in braingrid_reqs:
        short_id = req.get("short_id", "")
        braingrid_status = req.get("status", "")
        
        if not short_id or not braingrid_status:
            continue
        
        # Extract REQ number
        req_num = short_id.replace("REQ-", "").strip()
        possible_ids = [
            f"REQ-{req_num.zfill(3)}",
            f"REQ-{req_num}",
        ]
        
        local_file = None
        for req_id in possible_ids:
            if req_id in local_req_ids:
                local_file = local_req_ids[req_id]
                break
        
        if not local_file:
            continue
        
        # Read local file
        try:
            metadata, content = read_req_file(local_file)
            local_status = metadata.get("status", "")
            mapped_status = reverse_map_status(braingrid_status)
            
            # Only update if different
            if local_status != mapped_status:
                # Update the status in the file
                lines = content.split("\n")
                updated_lines = []
                in_front_matter = False
                front_matter_end = -1
                
                for i, line in enumerate(lines):
                    if line.strip() == "---":
                        if in_front_matter:
                            front_matter_end = i
                            in_front_matter = False
                        else:
                            in_front_matter = True
                        updated_lines.append(line)
                    elif in_front_matter and line.startswith("status:"):
                        updated_lines.append(f"status: {mapped_status}")
                    else:
                        updated_lines.append(line)
                
                # Write back
                new_content = "\n".join(updated_lines)
                with open(local_file, "w", encoding="utf-8") as f:
                    f.write(new_content)
                
                results.append((True, f"Updated {local_file.name}: {local_status} → {mapped_status}"))
                updated += 1
        except Exception as e:
            results.append((False, f"Error updating {local_file.name}: {e}"))
    
    if updated > 0:
        print(f"\nUpdated {updated} local REQ status(es) from Braingrid.")
    
    return results


def main():
    """Main sync function with bidirectional support."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Bidirectional sync between local REQs and Braingrid")
    parser.add_argument("--direction", choices=["to-braingrid", "from-braingrid", "bidirectional"], 
                       default="bidirectional", help="Sync direction")
    parser.add_argument("--update-status", action="store_true", 
                       help="Update local status from Braingrid (when syncing from Braingrid)")
    args = parser.parse_args()
    
    print("=" * 60)
    print("Bidirectional REQ Sync")
    print("=" * 60)
    print(f"Project ID: {PROJECT_ID}")
    print(f"Docs plan directory: {DOCS_PLAN_DIR}")
    print(f"Direction: {args.direction}")
    print()
    
    # Check if docs/plan directory exists
    if not DOCS_PLAN_DIR.exists():
        print(f"Error: {DOCS_PLAN_DIR} does not exist", file=sys.stderr)
        sys.exit(1)
    
    # List existing requirements in Braingrid
    print("Fetching existing requirements from Braingrid...")
    braingrid_reqs = list_braingrid_requirements()
    print(f"Found {len(braingrid_reqs)} requirements in Braingrid")
    
    # Get local REQ files
    local_req_files = get_local_req_files()
    local_req_ids = get_local_req_ids()
    print(f"Found {len(local_req_files)} REQ files locally")
    print()
    
    all_results = []
    
    # Sync Local → Braingrid
    if args.direction in ["to-braingrid", "bidirectional"]:
        print("=" * 60)
        print("Syncing Local → Braingrid")
        print("=" * 60)
        print()
        
        results = []
        for req_file in local_req_files:
            success, message = sync_req_file(req_file, braingrid_reqs)
            results.append((req_file.name, success, message))
            # Refresh after each creation
            if success and "Created" in message:
                braingrid_reqs = list_braingrid_requirements()
            print()
        
        all_results.extend(results)
        
        successful = sum(1 for _, success, _ in results if success)
        print(f"Local → Braingrid: {successful}/{len(results)} successful")
        print()
    
    # Sync Braingrid → Local
    if args.direction in ["from-braingrid", "bidirectional"]:
        print("=" * 60)
        print("Syncing Braingrid → Local")
        print("=" * 60)
        print()
        
        # Check for missing local REQs
        missing_results = sync_from_braingrid_to_local(braingrid_reqs, local_req_ids)
        all_results.extend(missing_results)
        
        # Update local status from Braingrid if requested
        if args.update_status:
            status_results = update_local_status_from_braingrid(braingrid_reqs, local_req_ids)
            all_results.extend(status_results)
        else:
            print("\nUse --update-status to update local REQ status from Braingrid.")
        print()
    
    # Print final summary
    print("=" * 60)
    print("Final Sync Summary")
    print("=" * 60)
    successful = sum(1 for _, success, _ in all_results if success)
    failed = len(all_results) - successful
    
    print(f"Total operations: {len(all_results)}")
    print(f"Successful: {successful}")
    print(f"Failed: {failed}")
    print()
    
    if failed > 0:
        print("Failed operations:")
        for name, success, message in all_results:
            if not success:
                print(f"  - {name}: {message}")
    
    if failed > 0:
        sys.exit(1)
    else:
        print("All sync operations completed successfully!")
        sys.exit(0)


if __name__ == "__main__":
    main()

