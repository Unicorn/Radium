# Project Context

This project uses Rust and follows these guidelines for all development work.

## Code Style

- Use `cargo fmt` for code formatting
- Follow Rust naming conventions (snake_case for functions, PascalCase for types)
- Keep functions focused and single-purpose
- Prefer explicit error handling over panics

## Testing Requirements

- Write comprehensive tests for all public APIs
- Aim for >80% code coverage
- Use integration tests for complex workflows
- Include test documentation explaining test scenarios

## Documentation Standards

- Document all public types, functions, and modules
- Use rustdoc comments (`///`) for public items
- Include examples in documentation when helpful
- Keep documentation in sync with code changes

## Error Handling

- Use `anyhow::Result` for application code
- Use custom error types for library code
- Provide clear, actionable error messages
- Log errors with appropriate context

## Performance Considerations

- Profile before optimizing
- Use appropriate data structures
- Consider memory usage for large datasets
- Benchmark critical paths

