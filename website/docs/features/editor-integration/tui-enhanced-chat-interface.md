---
id: "tui-enhanced-chat-interface"
title: "Enhanced TUI Chat Interface"
sidebar_label: "Enhanced TUI Chat Interface"
---

# Enhanced TUI Chat Interface

## Overview

The enhanced TUI chat interface provides better visibility into orchestrator operations by adding dedicated panels for task tracking and orchestrator reasoning. This addresses the limitation where users couldn't see task assignments, progress, or orchestrator decision-making during chat sessions.

## Features

### Three-Panel Layout

The enhanced interface displays three panels when the orchestrator is running:

```
┌─────────────────────────────────────────────────────────────┐
│ Chat/Output Area (60%)                                      │
│                                                             │
│ [User input and agent responses]                            │
│                                                             │
├──────────────────────────┬──────────────────────────────────┤
│ Task List (20%)          │ Orchestrator Thinking (20%)      │
│ ● Running: code-agent    │ [Orchestrator] Analyzing deps... │
│ ○ Queued: review-agent   │ [Orchestrator] Selected agent... │
│ ✓ Completed: plan-agent │ [Orchestrator] Executing step... │
└──────────────────────────┴──────────────────────────────────┘
```

### Task List Panel

The task list panel displays:
- **Status Icons**: Visual indicators for task status
  - ● (Running) - Yellow
  - ○ (Queued) - Gray
  - ✓ (Completed) - Green
  - ✗ (Error) - Red
  - ⏸ (Paused) - Blue
  - ⊗ (Cancelled) - Dim
- **Task Name**: The name/description of the task
- **Agent ID**: The agent assigned to execute the task
- **Progress Summary**: Shows completed/total tasks and failed count

### Orchestrator Thinking Panel

The orchestrator thinking panel displays:
- **Real-time Logs**: Orchestrator decision-making process
- **Syntax Highlighting**:
  - `[Orchestrator]` prefix in primary color
  - Keywords like "Analyzing", "Selected", "Executing" in info color
  - Errors in error color
  - Success messages in success color
- **Auto-scroll**: Automatically scrolls to latest output
- **Scroll Position Indicator**: Shows current line position

## Keyboard Shortcuts

### Panel Navigation

- **Tab**: Cycle focus between Chat, Task List, and Orchestrator panels
- **Ctrl+T**: Toggle task list panel visibility
- **Ctrl+O**: Toggle orchestrator thinking panel visibility

### Scrolling

When a panel is focused, you can scroll within it:

- **↑/↓ Arrow Keys**: Scroll up/down by 1 line
- **Page Up/Page Down**: Scroll up/down by 10 lines
- **Home**: Jump to top of panel
- **End**: Jump to bottom of panel

### Focus Indicators

The focused panel displays a highlighted border (using `border_active` theme color) to indicate which panel accepts keyboard input.

## Responsive Layout

The interface adapts to different terminal sizes:

- **Wide terminals (≥100 cols)**: Three-panel horizontal layout (60/20/20)
- **Narrow terminals (60-99 cols)**: Vertical stack with chat on top, task/orchestrator split on bottom
- **Very narrow terminals (<60 cols)**: Chat only with toggle indicators in title

## Troubleshooting

### Connection Errors

If you see "Connection lost" messages in the orchestrator thinking panel:
- The gRPC server may not be running
- Check that the Radium server is accessible
- The system will automatically retry after 2 seconds

### Missing Panels

If panels don't appear:
- Ensure orchestration is running (`orchestration_running` must be true)
- Check panel visibility with Ctrl+T and Ctrl+O
- On very narrow terminals, panels are hidden by default

### No Tasks Displayed

If the task list shows "No active workflow":
- No tasks are currently queued or running
- Tasks are only displayed when a workflow is active
- Start a workflow or requirement execution to see tasks

## Technical Details

### State Management

- **TaskListState**: Tracks task list with agent assignments and progress
- **OrchestratorThinkingPanel**: Uses OutputBuffer for log storage (1000 line capacity)
- **Panel Focus**: Managed via `PanelFocus` enum (Chat, TaskList, Orchestrator)

### Data Integration

- **Task List**: Polled from gRPC `ListTasks` RPC every 500ms
- **Orchestrator Logs**: Polled from monitoring service every 500ms
- **Error Handling**: Connection errors display in orchestrator panel with retry logic

### Component Architecture

- **TaskListPanel**: Renders task table with status icons and colors
- **OrchestratorThinkingPanel**: Renders scrollable log view with syntax highlighting
- **orchestrator_view**: Manages layout and responsive behavior

## Developer Guide

### Adding Custom Panels

To add a new panel to the orchestrator view:

1. Create a new component in `apps/tui/src/components/`
2. Add panel visibility state to `App` struct
3. Update `render_orchestrator_view` to include new panel
4. Add keyboard shortcuts for panel toggle
5. Update `PanelFocus` enum if needed

### Extending Task List

To add more task information:

1. Extend `TaskListItem` struct in `task_list_state.rs`
2. Update `TaskListPanel::render` to display new fields
3. Update gRPC polling to populate new fields

### Customizing Syntax Highlighting

To modify orchestrator log highlighting:

1. Edit `OrchestratorThinkingPanel::apply_syntax_highlighting`
2. Add new patterns and color mappings
3. Use theme colors for consistency

