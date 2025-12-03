# Radium Backend Architecture

> **Status**: Core backend infrastructure is complete ✅  
> **See**: [01-completed.md](./01-completed.md) for implementation details

The Radium backend is a high-performance, concurrent system built in Rust. It is responsible for orchestrating agents, executing workflows, and managing the core data of the application.

## Core Components

### Implemented ✅

- **Agent Orchestrator:** ✅ Implemented - Manages the lifecycle of agents, schedules their execution, and facilitates communication between them. Includes registry, lifecycle management, execution queue, and execution engine.
- **Model Abstraction Layer:** ✅ Implemented - Provides a unified interface for interacting with different AI models using a provider-based approach. Currently supports Gemini and OpenAI models.
- **Workflow Engine:** ✅ Implemented - Executes workflows with support for control flow, branching, and step execution.
- **Data Storage:** ✅ Implemented - SQLite-based storage layer using repository pattern. Stores agent configurations, workflow definitions, and task results.
- **API Server:** ✅ Implemented - gRPC-based API server with gRPC-Web support. Exposes functionality to CLI, TUI, and desktop application.
  - **Implemented RPCs:**
    - `Ping` - Health check endpoint
    - `RegisterAgent` - Register a new agent (basic implementation)
    - `CreateWorkflow` - Create a new workflow
    - `GetWorkflow` - Retrieve a workflow by ID
    - `ListWorkflows` - List all workflows
    - `UpdateWorkflow` - Update an existing workflow
    - `DeleteWorkflow` - Delete a workflow by ID
    - `CreateTask` - Create a new task
    - `GetTask` - Retrieve a task by ID
    - `ListTasks` - List all tasks
    - `UpdateTask` - Update an existing task
    - `DeleteTask` - Delete a task by ID
- **Plugin System:** ✅ Implemented - Basic plugin system for custom agents. Supports static agent loading with validation.

### Planned (Future)

- **Additional Model Providers:** Support for more AI models (Claude, Codex, etc.) and local models
- **Alternative Database Backends:** PostgreSQL and key-value store support
- **Enhanced Plugin System:** Dynamic library loading for plugins

## SaaS and Enterprise Architecture

To support the commercial SaaS offering and self-hosted enterprise deployments, the backend will be designed with the following principles in mind:

- **Multi-tenancy:** The system will be designed to support multiple tenants (users or organizations) with strong data isolation between them. This will likely involve a combination of database schemas and application-level logic to ensure that each tenant can only access their own data.
- **Authentication and Authorization:** A robust authentication and authorization system will be implemented to manage user accounts, roles, and permissions. This will support both self-hosted and SaaS deployments, with integrations for popular identity providers (e.g., OAuth, SAML).
- **Feature Flagging:** A feature flagging system will be used to enable or disable features based on a tenant's subscription plan (e.g., free vs. pro). This will allow for a flexible and easily configurable way to manage different pricing tiers.
- **Usage-based Billing:** A usage tracking and metering system will be implemented to monitor resource consumption (e.g., agent execution time, number of workflows). This data will be used to feed into a billing provider (e.g., Stripe) to handle subscriptions and invoicing.

## Technology Stack

- **Programming Language:** Rust ✅
- **Concurrency:** Tokio for asynchronous I/O ✅
- **API:** gRPC with Tonic, gRPC-Web support ✅
- **Database:** Rusqlite with repository pattern ✅
  - **Repository Pattern:** ✅ Implemented - Data access abstracted through repository traits (`AgentRepository`, `WorkflowRepository`, `TaskRepository`) with SQLite implementations.
  - **Schema:** ✅ Implemented - Tables for `agents`, `workflows`, `workflow_steps`, and `tasks` with proper indexes and foreign key constraints.
- **Serialization:** Serde ✅
- **Configuration:** ✅ Implemented - Layered configuration system reading from files (TOML), environment variables, and command-line arguments.

## Design Principles

- **Performance:** The backend will be designed for high performance and low latency, capable of handling a large number of concurrent agents and workflows.
- **Reliability:** Rust's ownership model and type system will be leveraged to ensure the reliability and safety of the backend.
- **Extensibility:** The plugin system and modular architecture will make it easy to extend the functionality of the backend without modifying the core code.
- **Scalability:** The architecture will be designed to scale both vertically and horizontally, allowing it to handle increasing workloads.
