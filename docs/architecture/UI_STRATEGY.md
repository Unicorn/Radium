# UI Strategy & Implementation Plan

**Last Updated**: 2025-12-02
**Status**: Planning
**Goal**: Unified cross-platform UI (Desktop, Mobile, Web) using `@unicorn-love` ecosystem and Tamagui.

## Executive Summary

We are transitioning the Radium frontend architecture to a centralized, component-driven system. This involves adopting the `@unicorn-love/*` package ecosystem for domain-specific and primitive UI components, leveraging **Tamagui** for universal styling, and establishing an **Expo** repository to support native mobile and web targets alongside our existing Tauri desktop app.

## Core Dependencies

We will integrate the following `@unicorn-love` packages to drive our UI:

*   **Foundation**:
    *   `@unicorn-love/core`: Core utilities and theme tokens.
    *   `@unicorn-love/primitives`: Low-level unstyled components.
    *   `@unicorn-love/layout`: Layout primitives (Stack, Grid, etc.).
*   **Features**:
    *   `@unicorn-love/data-display`: Tables, Lists, Cards.
    *   `@unicorn-love/forms`: Form inputs, validation, and layout.
    *   `@unicorn-love/tasks`: Task management specific UI.
*   **Domain Specific**:
    *   `@unicorn-love/insurance`: Insurance domain components.
    *   `@unicorn-love/compliance`: Compliance reporting and visualization.
    *   `@unicorn-love/forsured`: (Specific domain/feature set).

## Architecture

```mermaid
graph TD
    subgraph Apps
        Desktop[apps/desktop (Tauri)]
        Mobile[apps/mobile (Expo)]
        Web[apps/web (Next.js/Expo)]
    end

    subgraph UI_Layer
        Tamagui[Tamagui Config]
        Unicorn[(@unicorn-love/*)]
    end

    subgraph Logic_Layer
        State[packages/state]
        API[packages/api-client]
    end

    Desktop --> Tamagui
    Desktop --> Unicorn
    Desktop --> State
    
    Mobile --> Tamagui
    Mobile --> Unicorn
    Mobile --> State
    
    Web --> Tamagui
    Web --> Unicorn
    Web --> State
```

## Implementation Phases

### Phase 1: Foundation & Mobile Setup
**Goal**: Initialize the mobile environment and install core dependencies.

1.  **Initialize Expo Project**:
    *   Create `apps/mobile` using `create-expo-app`.
    *   Configure TypeScript and Monorepo support.
2.  **Install Dependencies**:
    *   Add `@unicorn-love/*` packages to `apps/mobile` and `apps/desktop`.
    *   Add `tamagui` and `@tamagui/*` dependencies.
3.  **Shared Configuration**:
    *   Create `packages/ui-config` (or use `packages/ui`) to house the shared `tamagui.config.ts` and theme definitions.
    *   Ensure both Tauri and Expo apps consume this configuration.

### Phase 2: Tauri Refactor (Desktop)
**Goal**: Update the existing Tauri frontend to align with the new stack.

1.  **Frontend Migration**:
    *   Ensure `apps/desktop/ui` is set up for React (if not already).
    *   Configure Vite for Tamagui (using `@tamagui/vite-plugin`).
2.  **Component Replacement**:
    *   Replace existing hardcoded UI with `@unicorn-love` components.
    *   Implement "Workflow Builder" and "Dashboard" using `@unicorn-love/layout` and `@unicorn-love/data-display`.

### Phase 3: Feature Integration
**Goal**: Connect the UI to Radium's core logic.

1.  **Tasks Integration**: Use `@unicorn-love/tasks` to build the Task Monitoring view.
2.  **Forms**: Use `@unicorn-love/forms` for Agent configuration and Settings.
3.  **Domain Modules**: Integrate `@unicorn-love/insurance` and others where relevant (or map generic features to these modules if they are placeholders for our specific business logic).

## Contribution Workflow

Since `@unicorn-love` packages are our own:
*   **Missing Features**: If a component is missing in the npm package, we will document it and plan a contribution to the package source.
*   **Local Development**: We may temporarily patch or `npm link` these packages if rapid iteration is needed, but the goal is to consume stable versions.

## Roadmap Tasks

### Setup
- [ ] **UI-001**: Initialize `apps/mobile` (Expo).
- [ ] **UI-002**: Create shared Tamagui config in `packages/ui`.
- [ ] **UI-003**: Install `@unicorn-love` packages in workspace.

### Desktop (Tauri)
- [ ] **UI-004**: Configure Tamagui in `apps/desktop`.
- [ ] **UI-005**: Refactor Dashboard to use `@unicorn-love/layout` & `data-display`.
- [ ] **UI-006**: Implement Task View using `@unicorn-love/tasks`.

### Mobile (Expo)
- [ ] **UI-007**: Implement basic App Shell (Navigation).
- [ ] **UI-008**: Port Dashboard view to Mobile.

## Technical Constraints

*   **Tauri**: Must ensure `ipc` calls are abstracted so components can be shared with Web/Mobile where possible (or mocked).
*   **Tamagui**: Ensure `compiler` is set up correctly for performance on Web/Electron/Tauri.
