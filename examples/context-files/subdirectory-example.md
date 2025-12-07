# Subdirectory-Specific Context

This context file is located in a subdirectory and extends or overrides project-level context.

## Purpose

This file demonstrates how subdirectory context files can provide specialized instructions for specific parts of the codebase.

## Module-Specific Guidelines

### API Development

- All endpoints must have OpenAPI documentation
- Use appropriate HTTP status codes
- Implement request/response validation
- Include error handling for all failure cases

### Database Interactions

- Use transactions for multi-step operations
- Implement proper connection pooling
- Handle connection errors gracefully
- Use migrations for schema changes

### Testing

- Mock external dependencies
- Test both success and failure paths
- Use realistic test data
- Clean up test artifacts

## Overriding Project Context

When this subdirectory context conflicts with project-level context, this file takes precedence. For example:

- If project context says "use async/await", but this module requires synchronous operations for compatibility reasons, document the exception here
- If project testing standards don't apply to integration tests in this directory, specify alternatives here

## Integration Notes

This module integrates with:
- External API service X
- Database schema Y
- Authentication system Z

Agents working in this directory should be aware of these dependencies and their constraints.

