# Claude Code Agent Rules

This document outlines the guidelines for Claude Code to work effectively in parallel with other AI agents (Cline, Cursor) on the Radium project.

## Core Principles

1. **Clear Communication**: Update documentation before starting work
2. **Atomic Commits**: Each task = commit with clear message
3. **Task Ownership**: Update progress files immediately upon starting
4. **Quality Assurance**: Tests must pass, linter must pass, build must succeed
5. **Transparency**: Document implementation decisions and reasoning

## Pre-Task Checklist

Before picking up ANY task from the roadmap:

- [ ] Read `roadmap/PROGRESS.md` to understand current status
- [ ] Check `.clinerules` and `.cursor/rules` for any active work
- [ ] Update the relevant task status in `PROGRESS.md` to **"In Progress - Claude"**
- [ ] Document the task details in the progress file with implementation approach
- [ ] Create/update a todo list in this session to track substeps

## During Development

### Code Quality Standards

- **Linting**: Must pass `cargo clippy` with no new warnings
- **Formatting**: Must pass `cargo fmt --all -- --check`
- **Tests**: Must achieve test coverage matching or exceeding current baseline
- **Build**: All crates must compile without errors
- **Security**: No unsafe code allowed (workspace forbids `unsafe_code`)

### Parallel Work Strategy

When working alongside other agents:

1. **Work in isolated areas**: Focus on specific modules/features
2. **Communicate status**: Update `PROGRESS.md` regularly with blockers/completion
3. **Merge-friendly commits**: Keep commits focused and logically separated
4. **Leave clear breadcrumbs**: Comment on WHY decisions were made, not just WHAT was done

### Testing Requirements

For each completed feature:

- Add unit tests for new functions
- Add integration tests for user-facing changes
- Ensure all existing tests still pass
- Run `npm test` or `cargo test --all` before committing

### Documentation Requirements

- Update comments for complex logic (but not obvious code)
- Keep `roadmap/PROGRESS.md` in sync with actual work
- Document any architectural decisions in the relevant implementation
- Include issue references in commit messages (e.g., `[RAD-XXX]`)

## Post-Task Checklist

After completing a task:

- [ ] Run `cargo build --all` - must succeed
- [ ] Run `cargo test --all` - all tests pass
- [ ] Run `cargo clippy --all` - no new warnings
- [ ] Run `cargo fmt --all -- --check` - formatting correct
- [ ] Run `npm run deny` and `npm run audit` if dependencies changed
- [ ] Update `roadmap/PROGRESS.md` with completion status
- [ ] Create git commit with message format: `feat/fix/refactor(scope): description [RAD-XXX]`
- [ ] Include emoji footer: `🤖 Claude Code`

## Commit Message Format

```
feat(module): brief description [RAD-XXX]

- Detailed bullet points of changes
- What was implemented and why
- Any important trade-offs or decisions

🤖 Claude Code
Co-Authored-By: Claude <noreply@anthropic.com>
```

## When Blocked

If you encounter:

- **Compilation errors**: Fix immediately, update progress
- **Test failures**: Investigate and fix before proceeding
- **Conflicting work**: Check progress file to see which agent is working on it
- **Missing dependencies**: Document in progress file and ask user for guidance
- **Unclear requirements**: Use `AskUserQuestion` tool to clarify

## Continuous Integration Expectations

The project must always maintain:

- ✅ Clean build (no errors)
- ✅ Passing tests (100% of suite)
- ✅ No clippy warnings (except whitelisted)
- ✅ Proper formatting
- ✅ No unsafe code in Rust

## Collaboration with Other Agents

### Cline (.clinerules)
- Check `.clinerules` for active work zones
- Cline may focus on: [to be determined based on .clinerules]

### Cursor (.cursor/rules)
- Check `.cursor/rules` for active work zones
- Cursor may focus on: [to be determined based on .cursor/rules]

### Synchronization
- All agents update `PROGRESS.md` in the same format
- Status: `⏳ In Progress`, `✅ Completed`, `❌ Blocked`, `🔄 Ready for Review`
- Include agent name in status: e.g., "⏳ In Progress - Claude Code"

## Current Project State

**Last Updated**: 2025-12-01

**Version**: 0.54.1

**Recent Milestones Completed**:
- Milestone 4: CLI and TUI implementation
- Desktop app with core UI features
- Comprehensive test coverage suite
- Workflow engine with full execution support

**Current Focus Areas**:
- Desktop application enhancements
- Performance optimization
- Extended test coverage
- Documentation improvements

## Resources

- Main Project: `/Users/clay/Development/RAD/new/radium`
- Progress Tracking: `roadmap/PROGRESS.md`
- Cline Rules: `.clinerules`
- Cursor Rules: `.cursor/rules`
- Build System: Nx workspace + Cargo (Rust) + Bun (Node.js)
- Test Command: `npm test` or `cargo test --all`
- Lint Command: `cargo clippy --all` + `cargo fmt --all`
