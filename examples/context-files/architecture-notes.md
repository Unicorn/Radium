# Architecture Notes

High-level architecture information for the project.

## System Architecture

The system uses a modular monorepo structure:

### Core Components

- **radium-core**: Core backend with orchestration engine
- **radium-cli**: Command-line interface
- **radium-tui**: Terminal user interface
- **radium-desktop**: Desktop application (Tauri)

### Design Principles

- Separation of concerns between layers
- Dependency injection for testability
- Async/await for concurrent operations
- Strong type safety throughout

## Data Flow

1. User interacts via CLI/TUI/Desktop
2. Requests routed to orchestration engine
3. Engine coordinates agent execution
4. Results returned to user interface

## Key Abstractions

- **Agent**: Encapsulates AI model interaction
- **Workspace**: Project structure and metadata
- **Plan**: Task breakdown and execution
- **Context**: Information injected into prompts

## Extension Points

- Custom agents via configuration
- Execution hooks for behavior customization
- MCP servers for external tool integration
- Extension packages for bundled functionality

## Performance Considerations

- Use connection pooling for external services
- Cache agent configurations and prompts
- Batch operations when possible
- Profile and optimize hot paths

