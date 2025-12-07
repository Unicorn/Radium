# Team Collaboration Context

This example demonstrates how to use context files effectively in a team environment.

## Team Context Strategy

When working in a team, context files should be:
- **Shared via version control** - Project root `GEMINI.md` goes in git
- **Personal overrides** - Use global context (`~/.radium/GEMINI.md`) for personal preferences
- **Module-specific** - Use subdirectory context for specialized areas

## Project Root Context (Shared in Git)

```markdown
# Team Project Context

This file is committed to version control and shared by all team members.

## Team Standards

- All code must be reviewed before merging
- Write tests for all new features
- Follow the team's coding standards
- Update documentation with changes

## Project Guidelines

- Use TypeScript for all new code
- Follow the project's architecture patterns
- Keep functions under 50 lines
- Document complex logic

## Workflow

1. Create feature branch from `main`
2. Implement with tests
3. Update documentation
4. Submit PR for review
5. Address review feedback
6. Merge after approval
```

## Personal Global Context (`~/.radium/GEMINI.md`)

Each team member can have their own global context file that doesn't affect others:

```markdown
# My Personal Preferences

These are my personal development preferences that apply to all projects.

## Editor Settings

- Use 2 spaces for indentation
- Enable format on save
- Use specific code snippets

## Personal Workflow

- I prefer TDD approach
- I like to write documentation first
- I use specific git aliases

## Learning Notes

- Remember to check error handling
- Always consider edge cases
- Think about performance implications
```

## Subdirectory Context (Team-Specific Areas)

For areas that need specialized context, create subdirectory context files:

```markdown
# API Module Team Guidelines

This module has specific requirements that differ from the project standard.

## API Standards

- All endpoints must have OpenAPI docs
- Use specific error response format
- Implement rate limiting

## Team-Specific Notes

- This module is maintained by the API team
- Contact @api-team for questions
- Follow the API design guide in docs/
```

## Best Practices for Teams

### 1. Keep Project Context Focused

Only include project-wide standards in the project root `GEMINI.md`. Personal preferences belong in global context.

### 2. Use Imports for Organization

Break large context files into smaller, focused files:

```markdown
# Project Context

@docs/coding-standards.md
@docs/architecture.md
@docs/workflow.md

## Project-Specific Notes

Additional project context here...
```

### 3. Document Team Decisions

Use context files to document architectural decisions and team agreements:

```markdown
## Architecture Decisions

- We use PostgreSQL for all data storage
- We use Redis for caching
- We use Docker for deployment

## Team Agreements

- Code reviews require 2 approvals
- All PRs must pass CI before merging
- We deploy on Tuesdays and Thursdays
```

### 4. Version Control Strategy

- ✅ Commit project root `GEMINI.md` to git
- ✅ Commit subdirectory context files to git
- ❌ Don't commit global context (`~/.radium/GEMINI.md`)
- ✅ Document context file changes in commit messages

### 5. Onboarding New Team Members

New team members should:
1. Clone the repository (gets project context)
2. Create their own global context file if desired
3. Review existing context files to understand team standards
4. Use `rad context list` to see all context files

## Example Workflow

```bash
# Team member checks what context applies
rad context show src/api

# Team member validates context files
rad context validate

# Team member creates new context file from template
rad context init --template coding-standards
```

## Troubleshooting Team Context

**Problem**: Team member's context differs from others

**Solution**: 
- Check if they have a global context file overriding project context
- Use `rad context show <path>` to see what's loaded
- Ensure project context is committed to git

**Problem**: Context conflicts between team members

**Solution**:
- Keep project context focused on shared standards
- Use personal global context for individual preferences
- Document team agreements in project context

