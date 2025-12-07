# Coding Standards

This file contains coding standards imported into the main project context.

## Rust Conventions

- Follow official Rust style guide
- Use `cargo clippy` to catch common issues
- Enable all lints in CI/CD pipeline
- Fix clippy warnings before merging

## Code Organization

- Group related functionality in modules
- Keep module files under 500 lines when possible
- Use `mod.rs` files for module organization
- Separate public and private APIs clearly

## Naming Conventions

- Functions: `snake_case`
- Types: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

## Comments and Documentation

- Use `//` for inline comments
- Use `///` for public API documentation
- Use `//!` for module-level documentation
- Explain "why" not "what" in comments

## Git Commit Messages

Follow the conventional commits format:

```
[REQ-XXX] [TASK-X] Brief description

Detailed explanation of changes and rationale.
```

- Start with REQ/TASK identifiers when applicable
- Keep first line under 72 characters
- Use imperative mood ("Add feature" not "Added feature")
- Explain why, not just what

