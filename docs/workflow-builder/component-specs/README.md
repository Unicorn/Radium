# Component Behavior Specifications

This directory contains YAML specifications for each workflow component type.
These specs define the expected inputs, outputs, and behaviors for Phase 6 migration.

## Specification Structure

Each component spec follows this structure:

```yaml
name: component-name
category: control-flow | activity | integration | state
description: What the component does

inputs:
  - name: input_name
    type: data_type
    required: true/false
    description: What this input is for
    default: optional_default_value

outputs:
  - name: output_name
    type: data_type
    description: What this output contains

behaviors:
  - name: behavior_name
    condition: "when this behavior applies"
    output: "what happens"

temporal_mapping:
  type: workflow | activity | signal | query
  # Temporal-specific configuration

example_usage: |
  Code example showing how to use this component
```

## Component Categories

### Control Flow Components
- `trigger.yaml` - Workflow entry points
- `end.yaml` - Workflow termination
- `condition.yaml` - Conditional branching
- `phase.yaml` - Execution grouping

### Activity Components
- `activity.yaml` - Generic activities
- `agent.yaml` - AI agent activities
- `retry.yaml` - Retry wrappers

### Workflow Components
- `child-workflow.yaml` - Child workflow invocation
- `signal.yaml` - Signal handlers
- `state-variable.yaml` - State management

### Integration Components
- `kong-logging.yaml` - Kong logging plugin
- `kong-cache.yaml` - Kong caching plugin
- `kong-cors.yaml` - Kong CORS plugin
- `graphql-gateway.yaml` - GraphQL endpoints
- `mcp-server.yaml` - MCP protocol server
- `api-endpoint.yaml` - REST API endpoints

## Usage

These specs are used by:
1. **Phase 6**: To validate Rust schema implementations match expected behavior
2. **Phase 9**: To train the Component Builder Agent
3. **Development**: As reference for understanding component capabilities
