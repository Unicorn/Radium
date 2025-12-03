# Radium CLI and TUI

> **Status**: CLI and TUI are complete ✅  
> **See**: [01-completed.md](./01-completed.md) for implementation details

The Radium CLI and TUI are the primary interfaces for interacting with the Radium backend. They provide a rich, interactive experience for managing agents, workflows, and tasks directly from the command line.

## CLI

The CLI is a modern, user-friendly command-line interface built in Rust. It provides a comprehensive set of commands for managing the entire Radium ecosystem.

### Implemented Features ✅

- **Command Structure:** ✅ Implemented with `clap` - Commands for agents, workflows, tasks, and orchestrator
- **Rich Output:** ✅ Implemented - Colorized output with tables and progress indicators
- **CRUD Operations:** ✅ Implemented - Full CRUD for agents, workflows, and tasks

### Planned Features

- **Interactive Prompts:** Will use interactive prompts and wizards to guide users through complex tasks
- **Command-line Completion:** Will support command-line completion for popular shells (e.g., Bash, Zsh, Fish)
- **Extensible:** Will be extensible through a plugin system, allowing developers to add custom commands

### Technology Stack

- **CLI Framework:** ✅ `clap` for parsing command-line arguments
- **Rich Output:** ✅ Colorized output with tables and formatting

## TUI

The TUI is a terminal-based GUI that provides a visual and interactive way to manage Radium. It is built with `ratatui` and communicates with the backend through the gRPC API.

### Implemented Features ✅

- **Dashboard:** ✅ Implemented - Real-time dashboard with overview of agents, workflows, and tasks
- **Agent Management:** ✅ Implemented - View for managing agents with CRUD operations
- **Workflow Management:** ✅ Implemented - Views for workflow management and execution
- **Task Viewer:** ✅ Implemented - View for inspecting task results with filtering and detail views
- **Navigation System:** ✅ Implemented - Navigation system and state management

### Planned Features

- **Workflow Editor:** Visual editor for creating and modifying workflows (enhanced)
- **Enhanced UI Components:** Additional UI components matching legacy system's TUI (see [02-now-next-later.md](./02-now-next-later.md))

### Technology Stack

- **TUI Library:** ✅ `ratatui` for creating terminal-based user interfaces
- **State Management:** ✅ State management for TUI
- **Communication:** ✅ Communicates with Radium backend through gRPC API
