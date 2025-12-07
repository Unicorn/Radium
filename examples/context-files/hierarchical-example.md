# Hierarchical Context Example

This example demonstrates how context files work together across multiple levels: global, project root, and subdirectory.

## How It Works

When you run an agent command in a subdirectory, Radium automatically loads context files in this order (lowest to highest precedence):

1. **Global context** (`~/.radium/GEMINI.md`) - Personal preferences, shared across all projects
2. **Project root context** (`GEMINI.md` in workspace root) - Project-specific guidelines
3. **Subdirectory context** (`<subdirectory>/GEMINI.md`) - Directory-specific overrides

Higher precedence files override lower precedence files, but all are merged together.

## Example Structure

```
~/.radium/
  └── GEMINI.md          # Global context (lowest precedence)

project-root/
  ├── GEMINI.md         # Project context (medium precedence)
  └── src/
      └── api/
          └── GEMINI.md # Subdirectory context (highest precedence)
```

## Global Context (`~/.radium/GEMINI.md`)

```markdown
# Global Development Guidelines

These are my personal preferences that apply to all projects.

## General Principles

- Always write tests
- Document public APIs
- Use meaningful variable names
- Keep functions small and focused

## Git Workflow

- Use feature branches
- Write clear commit messages
- Review code before merging
```

## Project Root Context (`GEMINI.md`)

```markdown
# Project Context

This is a Rust project with specific requirements.

## Language-Specific Guidelines

- Use `cargo fmt` for formatting
- Run `cargo clippy` before committing
- Follow Rust naming conventions

## Project Standards

- Minimum 80% test coverage
- All public APIs must be documented
- Use `anyhow::Result` for error handling
```

## Subdirectory Context (`src/api/GEMINI.md`)

```markdown
# API Module Context

This module handles HTTP API endpoints.

## API-Specific Guidelines

- All endpoints must have OpenAPI documentation
- Use appropriate HTTP status codes
- Implement request validation
- Handle errors gracefully

## Overrides

- This module uses synchronous code (not async) for compatibility
- Integration tests use real database connections (not mocks)
```

## Testing the Hierarchy

Use `rad context show` to see which files are loaded:

```bash
# From project root
rad context show .

# From subdirectory
rad context show src/api
```

## Expected Behavior

When running an agent in `src/api/`:
1. Global context is loaded first (personal preferences)
2. Project context is merged next (Rust-specific guidelines)
3. Subdirectory context is merged last (API-specific overrides)

The final context contains all three, with subdirectory context taking precedence where there are conflicts.

