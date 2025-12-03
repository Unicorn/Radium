# Radium Web and Desktop Applications

> **Status**: Desktop app and monorepo are complete ✅  
> **See**: [01-completed.md](./01-completed.md) for implementation details

The Radium GUI applications provide user-friendly interfaces for Radium. To ensure a consistent user experience and maximize code reuse, both the web and desktop applications are built from a shared set of UI components and business logic.

## Shared UI Strategy

### Implemented ✅

- **Monorepo:** ✅ Implemented - Nx monorepo managing shared packages and applications
- **UI Component Library:** ✅ Implemented - React component library with AgentTable, WorkflowEditor, TaskViewer, Dashboard, and common components
- **State Management:** ✅ Implemented - Zustand stores for agents, workflows, tasks, and orchestrator
- **API Client:** ✅ Implemented - gRPC-Web client with service wrappers
- **Shared Types:** ✅ Implemented - TypeScript type definitions matching Rust proto definitions

## Web Application

**Status**: Planned - Not yet implemented

The web application will provide a feature-rich interface for Radium, accessible from any modern browser.

### Planned Technology Stack

- **Framework:** Next.js or a similar React-based framework
- **Styling:** CSS-in-JS or a utility-first CSS framework like Tailwind CSS
- **Deployment:** Designed for easy deployment to platforms like Vercel or Netlify

## Desktop Application

**Status**: ✅ Complete

The desktop application is a cross-platform native application that provides a seamless user experience.

### Implemented Features ✅

- **Full UI:** ✅ Agent management, workflow management, task viewer, orchestrator UI
- **Navigation:** ✅ Hash-based navigation system with sidebar
- **Dashboard:** ✅ Dashboard with summary cards and connection status
- **gRPC Integration:** ✅ Full gRPC client integration via Tauri commands
- **Tests:** ✅ Comprehensive integration tests (33 tests)

### Technology Stack

- **Framework:** ✅ Tauri v2 wrapping web-based UI in native shell
- **Frontend:** ✅ React-based UI using shared component library
- **Backend Communication:** ✅ Communicates with Radium backend through gRPC API via Tauri commands

## Design Principles

- **Consistency:** The web and desktop applications will have a consistent look, feel, and functionality.
- **Code Reuse:** The monorepo and shared packages will maximize code reuse, reducing development time and ensuring consistency.
- **Performance:** Both applications will be designed for performance, providing a responsive and enjoyable user experience.