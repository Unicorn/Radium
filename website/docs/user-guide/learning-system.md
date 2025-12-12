---
id: "learning-system"
title: "Learning System"
sidebar_label: "Learning System"
---

# Learning System

## Overview

The Learning System tracks mistakes, preferences, and successes to build pattern recognition for future improvement. It extends the original mistake-tracking system with ACE (Agentic Context Engineering) skillbook functionality.

## Components

### Learning Store

The Learning Store persists learning entries and skills in `.radium/_internals/learning/learning-log.json`:
- **Mistakes**: Errors made and corrected, with solutions
- **Preferences**: User preferences and constraints
- **Successes**: Successful patterns and approaches
- **Skills**: Strategies organized by section in the skillbook

### Skillbook

The skillbook contains learned strategies organized by section:
- `task_guidance`: How to approach and break down tasks
- `tool_usage`: Best practices for using tools
- `error_handling`: Error handling patterns
- `code_patterns`: Code structure and patterns
- `communication`: Communication strategies
- `general`: General strategies

Each skill tracks:
- Helpful count: Times this skill was helpful
- Harmful count: Times this skill was harmful
- Neutral count: Times this skill was neutral

## CLI Commands

### List Learning Entries

View all learning entries:

```bash
# List all entries
rad learning list

# Filter by category
rad learning list --category "Complex Solution Bias"

# JSON output
rad learning list --json
```

### Add Mistake

Record a mistake for future learning:

```bash
rad learning add-mistake \
  --category "Feature Creep" \
  --description "Added unnecessary features beyond requirements" \
  --solution "Stick to core requirements and avoid scope expansion"
```

### Add Skill

Add a skill to the skillbook:

```bash
rad learning add-skill \
  --section "task_guidance" \
  --content "Break complex tasks into smaller, manageable steps"
```

### Tag Skill

Tag a skill as helpful, harmful, or neutral:

```bash
# Tag as helpful
rad learning tag-skill --skill-id "skill-00001" --tag "helpful"

# Tag as harmful
rad learning tag-skill --skill-id "skill-00001" --tag "harmful" --increment 1

# Tag as neutral
rad learning tag-skill --skill-id "skill-00001" --tag "neutral"
```

### Show Skillbook

Display skills from the skillbook:

```bash
# Show all sections
rad learning show-skillbook

# Show specific section
rad learning show-skillbook --section "task_guidance"

# JSON output
rad learning show-skillbook --json
```

## Standard Categories

Mistakes are organized into standard categories:

- **Complex Solution Bias**: Over-engineering solutions
- **Feature Creep**: Adding unnecessary features
- **Premature Implementation**: Jumping to code too quickly
- **Misalignment**: Wrong direction or misunderstanding
- **Overtooling**: Using too many tools unnecessarily
- **Preference**: User preferences
- **Success**: Successful patterns
- **Other**: Uncategorized entries

Categories are automatically normalized (e.g., "complex" → "Complex Solution Bias").

## Duplicate Detection

The learning system prevents duplicate entries by detecting similar mistakes (60%+ word overlap). This prevents the skillbook from being cluttered with redundant information.

## Integration with Vibe Check

The learning system automatically integrates with Vibe Check:

1. **Mistake Logging**: When oversight detects a mistake, it's logged to the learning store
2. **Pattern Extraction**: Helpful and harmful patterns are extracted from oversight feedback
3. **Skillbook Updates**: Patterns are converted to skills and added to the skillbook
4. **Context Injection**: Learning context is injected into future oversight requests

## Examples

### Example 1: Adding a Mistake

```bash
$ rad learning add-mistake \
  --category "Complex Solution Bias" \
  --description "Created overly complex solution with unnecessary abstractions" \
  --solution "Simplify by removing unnecessary layers and using direct approach"

Mistake added successfully
  Category: Complex Solution Bias
  Description: Created overly complex solution with unnecessary abstractions
  Solution: Simplify by removing unnecessary layers and using direct approach
```

### Example 2: Viewing Skills

```bash
$ rad learning show-skillbook --section "task_guidance"

Skillbook

  task_guidance
    • [skill-00001]
      Break complex tasks into smaller, manageable steps
      Stats: Helpful: 5 | Harmful: 0 | Neutral: 1

    • [skill-00002]
      Start with core requirements before adding features
      Stats: Helpful: 3 | Harmful: 0 | Neutral: 0
```

### Example 3: Tagging a Skill

```bash
$ rad learning tag-skill --skill-id "skill-00001" --tag "helpful"

Skill skill-00001 tagged as helpful (increment: 1)
```

## Best Practices

1. **Be specific**: Provide clear descriptions and solutions for mistakes
2. **Use standard categories**: Stick to standard categories for better organization
3. **Tag skills regularly**: Tag skills as helpful/harmful based on outcomes
4. **Review skillbook**: Periodically review the skillbook to identify useful patterns
5. **Avoid duplicates**: The system detects duplicates, but be mindful when adding entries

## Troubleshooting

### Learning entries not persisting

- Check that `.radium/_internals/learning/` directory exists
- Verify file permissions for `learning-log.json`
- Ensure workspace is properly initialized (`rad init`)

### Skills not appearing in skillbook

- Verify the section name matches a standard section
- Check that skills haven't been soft-deleted (use `--include-invalid` if needed)
- Ensure the skillbook context is being generated correctly

### Duplicate detection too strict/loose

- The 60% word overlap threshold is fixed
- Use more specific descriptions to avoid false duplicates
- Use less specific descriptions if legitimate entries are being rejected

## References

- [Vibe Check Documentation](./vibe-check.md)
- [Constitution Rules Documentation](./constitution-rules.md)
- [Learning Store Implementation](../../crates/radium-core/src/learning/store.rs)
- [Skill Manager Implementation](../../crates/radium-core/src/learning/skill_manager.rs)

