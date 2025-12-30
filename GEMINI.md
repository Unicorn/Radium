# Radium Project Context

## Project Overview
Radium is a high-performance, Rust-based platform for creating, managing, and deploying autonomous AI agents. It features a concurrent agent orchestration engine, a flexible plugin system, and multiple user interfaces (CLI, TUI, Desktop, Web).

## Architecture
The project is a **hybrid monorepo** using Rust for the core logic and TypeScript/Node.js for frontend applications and scripting, managed by **Nx** and **Bun**.

### Core Backend (Rust)
Located in `crates/`, these components form the backbone of the system:
*   **`radium-core`**: The main server, gRPC endpoints, and orchestration logic.
*   **`radium-orchestrator`**: Intelligent routing and task management for agents.
*   **`radium-models`**: Shared data structures and types.
*   **`radium-abstraction`**: Traits and interfaces for extensibility.
*   **`radium-training`**: Modules for the learning and skillbook systems.

### Applications
Located in `apps/`:
*   **`cli`**: Rust-based Command Line Interface (`radium-cli`).
*   **`tui`**: Rust-based Terminal User Interface (`radium-tui`).
*   **`desktop`**: Tauri + React desktop application.
*   **`mobile`**: Mobile application (placeholder/WIP).

### Web & Documentation
*   **`website`**: Docusaurus-based documentation and landing page.
*   **`packages/`**: Shared TypeScript libraries (UI components, API clients).

## Development Workflow

### Prerequisites
*   **Rust**: Latest stable version.
*   **Runtime**: Bun (preferred) or Node.js.
*   **Tooling**: `cargo`, `nx` (globally or via `bun run`).

### Common Commands
Use `bun run` or `npm run` to execute project scripts defined in `package.json`.

**Running Applications:**
*   **Desktop:** `bun run dev:desktop`
*   **CLI:** `bun run dev:cli` (or `cargo run --bin radium-cli`)
*   **TUI:** `bun run dev:tui` (or `cargo run --bin radium-tui`)
*   **Server:** `bun run dev:server` (or `cargo run --bin radium-core`)
*   **Website:** `bun run dev:website`

**Building:**
*   **Rust Core:** `bun run build:rust` (Release mode)
*   **Desktop:** `bun run build:desktop`
*   **All:** `bun run build` (via Nx)

**Testing:**
*   **Rust Unit Tests:** `bun run test:rust` (or `cargo test --all`)
*   **Frontend/Packages:** `bun run test:packages`
*   **Full Suite:** `bun run test`

### Code Quality & Formatting
*   **Rust Formatting:** `bun run fmt` (runs `cargo fmt --all`).
*   **Rust Linting:** `bun run lint:fix` (runs `cargo clippy --fix`). The project has strict linting rules configured in `Cargo.toml`.
*   **Type Checking:** `bun run type-check`.

## Configuration & Agents
*   **Agent Definitions:** Defined in `agents/` using TOML files (e.g., `agents/core/code-agent.toml`).
*   **Prompts:** Associated Markdown prompts are stored in `prompts/`.
*   **Policies:** Security and execution policies are managed via the Policy Engine (`.radium/policy.toml`).
*   **Context Files:** The project itself supports `GEMINI.md` files for providing persistent context to agents.

## Directory Structure
*   `apps/` - Client applications (CLI, TUI, Desktop).
*   `crates/` - Rust workspace members (Core logic).
*   `agents/` - TOML configurations for built-in agents.
*   `prompts/` - Markdown prompt templates for agents.
*   `packages/` - Shared TypeScript packages.
*   `website/` - Documentation source.
*   `scripts/` - Shell scripts for maintenance and testing.


<!-- nx configuration start-->
<!-- Leave the start & end comments to automatically receive updates. -->

# General Guidelines for working with Nx

- When running tasks (for example build, lint, test, e2e, etc.), always prefer running the task through `nx` (i.e. `nx run`, `nx run-many`, `nx affected`) instead of using the underlying tooling directly
- You have access to the Nx MCP server and its tools, use them to help the user
- When answering questions about the repository, use the `nx_workspace` tool first to gain an understanding of the workspace architecture where applicable.
- When working in individual projects, use the `nx_project_details` mcp tool to analyze and understand the specific project structure and dependencies
- For questions around nx configuration, best practices or if you're unsure, use the `nx_docs` tool to get relevant, up-to-date docs. Always use this instead of assuming things about nx configuration
- If the user needs help with an Nx configuration or project graph error, use the `nx_workspace` tool to get any errors

<!-- nx configuration end-->