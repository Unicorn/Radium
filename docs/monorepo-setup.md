# Radium Monorepo Setup Guide

This guide explains the Nx monorepo structure and how to work with shared packages.

## Overview

Radium uses Nx to manage a monorepo containing:
- Rust crates (backend, CLI, TUI)
- JavaScript/TypeScript packages (shared libraries)
- Applications (desktop app, future web app)

## Package Structure

```
radium/
├── packages/
│   ├── shared-types/     # TypeScript type definitions
│   ├── api-client/       # gRPC-Web client
│   ├── state/            # Zustand stores
│   └── ui/               # React components
├── apps/
│   ├── desktop/          # Tauri desktop app
│   └── (future: web/)    # Next.js web app
└── (Rust crates...)
```

## Dependency Graph

```
shared-types (no dependencies)
    ↓
api-client (depends on shared-types)
    ↓
state (depends on api-client, shared-types)
    ↓
ui (depends on state, shared-types)
    ↓
apps (depend on ui, state, api-client, shared-types)
```

## Working with Packages

### Building Packages

```bash
# Build all shared packages
npm run build:packages

# Build a specific package
nx build shared-types
nx build api-client
nx build state
nx build ui
```

### Type Checking

```bash
# Type check all TypeScript packages
npm run type-check

# Type check a specific package
nx type-check shared-types
```

### Adding Dependencies

When adding a dependency to a package:

1. Add it to the package's `package.json`
2. If it's a workspace package, use `"*"` as the version
3. Run `npm install` in the workspace root

### Creating a New Package

1. Create directory: `packages/my-package/`
2. Add `package.json` with name `@radium/my-package`
3. Add `tsconfig.json` extending `../../tsconfig.base.json`
4. Add `project.json` for Nx configuration
5. Update `tsconfig.base.json` paths if needed
6. Add to consuming packages' dependencies

### Using Packages in Apps

```typescript
// In apps/desktop or apps/web
import { Agent } from '@radium/shared-types';
import { createRadiumClient } from '@radium/api-client';
import { useAgentStore } from '@radium/state';
import { AgentTable } from '@radium/ui';
```

## Nx Commands

```bash
# Run a target on all projects
nx run-many -t build

# Run on projects with specific tags
nx run-many -t build --projects=tag:scope:shared

# Graph dependencies
nx graph

# Affected projects (based on git changes)
nx affected:build
```

## TypeScript Configuration

All packages extend `tsconfig.base.json` which defines:
- Path mappings for workspace packages
- Compiler options
- Shared configuration

Path mappings allow importing packages by name:
```typescript
import { Agent } from '@radium/shared-types';
```

## Development Workflow

1. Make changes to a package
2. Type check: `nx type-check <package-name>`
3. Build if needed: `nx build <package-name>`
4. Changes are automatically available to consuming packages (no build needed in dev)
5. For production builds, ensure dependencies are built first

## Troubleshooting

### Type errors in consuming packages
- Ensure the package is built: `nx build <package-name>`
- Check TypeScript paths in `tsconfig.base.json`
- Verify package exports in package's `index.ts`

### Build failures
- Check dependency order (shared-types → api-client → state → ui → apps)
- Ensure all dependencies are installed: `npm install`
- Clear Nx cache: `nx reset`

### Import errors
- Verify path mappings in `tsconfig.base.json`
- Check package name matches import path
- Ensure package has proper exports in `index.ts`

