#!/usr/bin/env python3
"""
Convert Radium agent definitions (TOML + prompts) to Ai-Agent-Skills format (SKILL.md).

Usage:
    python scripts/convert_agents_to_skills.py
"""

import os
import tomllib
from pathlib import Path

def convert_agent_to_skill(agent_path: Path, output_dir: Path):
    """Convert a single agent TOML + prompt to SKILL.md format."""

    # Read TOML config
    with open(agent_path, 'rb') as f:
        config = tomllib.load(f)

    agent = config['agent']
    agent_id = agent['id']
    name = agent['name']
    description = agent['description']
    prompt_path = agent.get('prompt_path', '')
    engine = agent.get('engine', 'gemini')
    model = agent.get('model', '')

    # Determine category from path
    category = agent_path.parent.name

    # Read prompt content if available
    prompt_content = ""
    if prompt_path:
        full_prompt_path = Path(prompt_path)
        if full_prompt_path.exists():
            with open(full_prompt_path, 'r') as f:
                prompt_content = f.read()

    # Create skill name (lowercase-hyphen format)
    skill_name = agent_id

    # Create SKILL.md content
    frontmatter = f"""---
name: {skill_name}
description: {description}
license: Apache-2.0
metadata:
  category: {category}
  author: radium
  engine: {engine}
  model: {model}
  original_id: {agent_id}
---
"""

    # Combine frontmatter and prompt content
    skill_content = frontmatter + "\n" + prompt_content

    # Write to output directory
    skill_dir = output_dir / category / skill_name
    skill_dir.mkdir(parents=True, exist_ok=True)

    skill_file = skill_dir / "SKILL.md"
    with open(skill_file, 'w') as f:
        f.write(skill_content)

    print(f"✓ Converted {agent_id} → {skill_file}")
    return skill_name

def main():
    """Convert all Radium agents to skills."""

    base_dir = Path(__file__).parent.parent
    agents_dir = base_dir / "agents"
    skills_dir = base_dir / "skills"

    # Find all agent TOML files
    agent_files = list(agents_dir.rglob("*.toml"))

    # Exclude test agents
    agent_files = [f for f in agent_files if 'test' not in f.parts]

    converted_count = 0

    for agent_path in agent_files:
        try:
            convert_agent_to_skill(agent_path, skills_dir)
            converted_count += 1
        except Exception as e:
            print(f"✗ Failed to convert {agent_path}: {e}")

    print(f"\nConversion complete: {converted_count} agents → skills")
    print(f"Output directory: {skills_dir}")

if __name__ == "__main__":
    main()
