# Rust Project Context

This example shows a language-specific context file for a Rust project.

## Project Type

This is a Rust project using modern Rust practices and tooling.

## Rust-Specific Guidelines

### Code Formatting

- Always run `cargo fmt` before committing
- Use default rustfmt configuration
- Format on save in your editor

### Linting

- Run `cargo clippy` with `-D warnings` in CI
- Fix all clippy warnings before merging
- Use clippy lints to improve code quality

### Dependencies

- Prefer standard library when possible
- Use well-maintained crates from crates.io
- Pin dependency versions in Cargo.lock
- Review security advisories regularly

## Error Handling

- Use `Result<T, E>` for fallible operations
- Use `anyhow::Result` for application code
- Use `thiserror` for library error types
- Provide clear, actionable error messages

## Testing

- Write unit tests in `#[cfg(test)]` modules
- Use integration tests in `tests/` directory
- Aim for >80% code coverage
- Test both success and error paths

## Documentation

- Use `///` for public API documentation
- Use `//!` for module-level documentation
- Include code examples in doc comments
- Run `cargo doc --open` to preview documentation

## Performance

- Profile before optimizing
- Use `cargo bench` for benchmarks
- Consider memory usage for large datasets
- Use appropriate data structures

## Safety

- Never use `unsafe` without justification
- Document why `unsafe` is necessary
- Prefer safe abstractions
- Use `#![deny(unsafe_code)]` when possible

