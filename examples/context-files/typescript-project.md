# TypeScript Project Context

This example shows a language-specific context file for a TypeScript/Node.js project.

## Project Type

This is a TypeScript project using modern JavaScript practices and tooling.

## TypeScript-Specific Guidelines

### Code Formatting

- Use Prettier with project configuration
- Format on save in your editor
- Enforce formatting in CI/CD

### Type Safety

- Enable strict mode in `tsconfig.json`
- Avoid `any` types - use `unknown` if needed
- Use type guards for runtime checks
- Leverage TypeScript's type inference

### Linting

- Use ESLint with TypeScript plugin
- Enable recommended rules
- Fix all linting errors before merging
- Use `eslint --fix` for auto-fixable issues

### Dependencies

- Use npm or yarn for package management
- Prefer well-maintained packages
- Lock dependency versions
- Review security advisories with `npm audit`

## Error Handling

- Use try/catch for async operations
- Create custom error classes when needed
- Provide clear error messages
- Log errors with appropriate context

## Testing

- Use Jest or Vitest for testing
- Write unit tests for all functions
- Use integration tests for workflows
- Aim for >80% code coverage
- Mock external dependencies

## Documentation

- Use JSDoc comments for public APIs
- Include type information in comments
- Document complex algorithms
- Keep README up to date

## Performance

- Use async/await for I/O operations
- Avoid blocking the event loop
- Profile with Node.js profiler
- Use appropriate data structures

## Modern JavaScript

- Use ES6+ features (const/let, arrow functions, destructuring)
- Prefer async/await over promises
- Use optional chaining (`?.`) and nullish coalescing (`??`)
- Leverage template literals for strings

## Package Management

- Use semantic versioning
- Keep dependencies up to date
- Use `npm ci` in CI/CD
- Document breaking changes

