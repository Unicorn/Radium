# Radium Milestones and Timeline

> ⚠️ **ORIGINAL TIMELINE**: This document represents the original milestone plan.  
> **See**: [01-completed.md](./01-completed.md) for completed work and [02-now-next-later.md](./02-now-next-later.md) for current priorities  
> **Status**: Reference document - milestones 1-5 are complete

---

This document outlines the original milestones and timeline for the development of Radium.

## Phase 1: Backend Development (Months 1-3)

- **Milestone 1: Core Backend Infrastructure (Month 1)** ✅ **COMPLETED**
  - ✅ Set up the Rust project and CI/CD pipeline.
  - ✅ Implement the basic gRPC API server with gRPC-Web support.
  - ✅ Design and implement the core data structures for agents, workflows, and tasks.
  - ✅ Implement the initial data storage layer with SQLite.
  - ✅ Implement gRPC CRUD operations for workflows and tasks.

- **Milestone 2: Agent Orchestrator and Model Integration (Month 2)** ✅ **COMPLETED**
  - ✅ Implement the Model Abstraction Layer.
  - ✅ Integrate initial set of models (e.g., Gemini, OpenAI).
  - ✅ Implement the agent orchestrator, including agent lifecycle management and scheduling, leveraging the Model Abstraction Layer.
  - ✅ Implement a basic plugin system for custom agents.
  - ✅ Create a few example agents to test the orchestrator.

- **Milestone 3: Workflow Engine (Month 3)** ✅ **COMPLETED**
  - ✅ Design and implement the workflow engine, including support for basic control flow.
  - ✅ Implement the workflow execution logic.
  - ✅ Create a few example workflows to test the engine.

## Phase 2: CLI and TUI Development (Month 4)

- **Milestone 4: CLI and TUI (Month 4)** ✅ **COMPLETED**
  - ✅ Implement the basic CLI with `clap`.
  - ✅ Implement commands for managing agents and workflows.
  - ✅ Implement the basic TUI with `ratatui`, including the dashboard and agent management views.

## Phase 3: Monorepo and GUI Foundation (Month 5)

- **Milestone 5: Monorepo Setup (Month 5)** ✅ **COMPLETED**
  - ✅ Set up the Nx monorepo.
  - ✅ Create the initial package structure (`apps`, `packages`).
  - ✅ Set up the shared UI component library, state management, and API client packages.
  - ✅ Desktop app with full UI (Tauri v2)

## Phase 4: Web and Desktop Application Development (Months 6-9)

- **Milestone 6: GUI Shells (Month 6)**
  - Set up the Next.js web application and the Tauri desktop application shells.
  - Implement basic navigation and layout in both applications using the shared UI library.

- **Milestone 7: Core GUI Features (Months 7-8)**
  - Implement the agent management, workflow editor, and task viewer features in the shared UI library.
  - Integrate these features into both the web and desktop applications.

- **Milestone 8: Application Polish (Month 9)**
  - Polish the user interface and user experience for both applications.
  - Add application-specific features and configurations.

## Phase 5: Alpha Release and Community Building (Month 10)

- **Milestone 9: Alpha Release (Month 10)**
  - Package and release the alpha version of Radium, including the backend, CLI, TUI, web app, and desktop app.
  - Write documentation and tutorials.
  - Start building a community around Radium.

## Future Phases

- **Beta Release:** Incorporate feedback from the alpha release and add new features.
- **1.0 Release:** The first stable release of Radium.
- **SaaS and Enterprise Features:** After the 1.0 release, development will focus on building out the commercial features of Radium.
  - **Multi-tenancy and Authentication:** Implement a robust multi-tenancy and authentication system to support the SaaS offering.
  - **Billing Integration:** Integrate with a billing provider (e.g., Stripe) to handle subscriptions and usage-based billing.
  - **Team Collaboration:** Implement features for team collaboration, such as shared workflows and agent pools.
- **Ongoing Development:** Continue to improve Radium and add new features based on community feedback.
