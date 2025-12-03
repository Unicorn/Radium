# @radium/ui

React component library for building Radium applications.

## Overview

This package provides reusable React components for agent management, workflow editing, task viewing, and dashboard displays.

## Components

### Common Components

- `Button` - Styled button with variants (primary, secondary, danger) and sizes
- `Input` - Form input with label and error display
- `Modal` - Modal dialog component

### Feature Components

- `AgentTable` - Table display for agents with actions
- `WorkflowEditor` - Editor for creating/editing workflows
- `TaskViewer` - Detailed view for tasks
- `Dashboard` - Dashboard with summary cards

## Usage

```typescript
import { AgentTable, Dashboard, Button } from '@radium/ui';
import { useAgentStore } from '@radium/state';

function AgentsPage() {
  const { agents, loading, fetchAgents, deleteAgent } = useAgentStore();
  
  return (
    <div>
      <AgentTable
        agents={agents}
        loading={loading}
        onDelete={deleteAgent}
      />
    </div>
  );
}
```

## Styling

Components use Tailwind CSS classes. Ensure Tailwind is configured in your application.

## Component Props

See TypeScript definitions for detailed prop interfaces:
- `AgentTableProps`
- `WorkflowEditorProps`
- `TaskViewerProps`
- `DashboardProps`
- `ButtonProps`
- `InputProps`
- `ModalProps`

