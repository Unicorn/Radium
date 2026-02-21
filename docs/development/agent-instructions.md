# Agent Development Instructions

Guidelines for AI-assisted development on the Radium project.

## Core Principles

### Leave It Better Than You Found It

When working on any task, if you discover broken code, missing tests, lint issues, or rough edges -- fix them. Prioritize accordingly, but everything discovered during the course of work must be addressed before calling the work complete. We always leave the system better than we found it.

### Testing Policy

- If we own it or write it, we test it directly -- we do NOT mock it in tests
- Do NOT mock the database or the internal API
- Use a real database instance and real HTTP calls
- Mocks are only allowed for external third-party services
- Tables, code, configuration, and definitions that we own and that affect our code inside of 3rd party systems should be tested because we own it
- All tests must pass 100%
- Tests must run without `--watch` so they stop on their own

### Security

- Secrets must never be stored in anything other than `.env*` files
- Production environments must never use `.env` files or `dotenv` packages -- only system environment variables
- Software must hard fail if necessary environment variables are not available at boot time
- Linting errors should never be ignored or bypassed

## Row Level Security (RLS) Guidelines

All tables with user-owned data must have Row Level Security enabled. When adding new tables:

1. **Enable RLS**: `ALTER TABLE public.new_table ENABLE ROW LEVEL SECURITY;`
2. **Add ownership policies**: Use `current_user_id() = created_by` for tables with direct ownership, or `user_owns_workflow(workflow_id)` / `user_owns_project(project_id)` for child tables
3. **Add service_role bypass**: Always include `auth.role() = 'service_role'` policy for server-side access
4. **Use WITH CHECK on UPDATE**: Prevents ownership transfer via direct SQL
5. **No blanket anon grants**: Only grant `SELECT` to `anon` on lookup/reference tables

### RLS Testing

All tables with Row Level Security must have corresponding tests that verify:
- Owners can CRUD their own rows
- Non-owners cannot access other users' rows
- Anonymous role access matches expectations (lookup tables only)
- Service role can access all rows

### Key Functions

- `current_user_id()` -- Bridges `auth.uid()` (GoTrue UUID) to `public.users.id` (internal UUID)
- `user_owns_workflow(workflow_id)` -- Checks if current user owns the parent workflow
- `user_owns_project(project_id)` -- Checks if current user owns the parent project
- `user_owns_connector(connector_id)` -- Checks if current user owns the parent connector

## Git Workflow

- Work on feature branches, never directly on `main`
- Commit frequently with descriptive messages
- Run all tests before merging

## Braingrid Integration

When a user references REQ-XXX:
1. Check Braingrid for the requirement and tasks
2. Ensure tasks are well-defined (break down if needed)
3. Set REQ status to IN_PROGRESS
4. Update task status in real-time as you work
5. Mark REQ as REVIEW when complete and tests pass
