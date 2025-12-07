# Future Enhancements

This document tracks feature ideas and enhancements for future implementation.

## In Development (NEXT Phase)

The following features are planned for implementation in the NEXT phase of development. See [02-now-next-later.md](../project/02-now-next-later.md) for detailed roadmap information.

### MCP Integration

**Status**: Planned for Step 1  
**Priority**: High

Integration with the Model Context Protocol to enable external tool discovery and execution from MCP servers. Supports multiple transports (stdio, SSE, HTTP), OAuth authentication, and rich content responses.

**Reference**: [gemini-cli-enhancements.md](./gemini-cli-enhancements.md#mcp-model-context-protocol-integration)

---

### Policy Engine

**Status**: Planned for Step 3  
**Priority**: High

Fine-grained control over tool execution through rule-based policies. Supports allow/deny/ask_user decisions, priority-based rule matching, and approval modes.

**Reference**: [gemini-cli-enhancements.md](./gemini-cli-enhancements.md#policy-engine-for-tool-execution)

---

### Context Files (GEMINI.md)

**Status**: Planned for Step 1  
**Priority**: High

Hierarchical context file system for providing persistent instructions to agents. Supports global, project, and subdirectory context files with import syntax.

**Reference**: [gemini-cli-enhancements.md](./gemini-cli-enhancements.md#context-files-geminimd)

---

### Custom Commands (TOML)

**Status**: Planned for Step 5  
**Priority**: High

TOML-based system for defining reusable agent commands with shell and file injection syntax.

**Reference**: [gemini-cli-enhancements.md](./gemini-cli-enhancements.md#custom-commands-toml-based)

---

### Checkpointing System

**Status**: Planned for Step 6  
**Priority**: High

Automatic Git snapshots and conversation history preservation for safe experimentation with code changes.

**Reference**: [gemini-cli-enhancements.md](./gemini-cli-enhancements.md#checkpointing-system)

---

### Sandboxing

**Status**: Planned for Step 6.5  
**Priority**: High

Isolated execution environments for safe agent operations. Supports Docker/Podman and macOS Seatbelt sandboxing.

**Reference**: [gemini-cli-enhancements.md](./gemini-cli-enhancements.md#sandboxing)

---

## Later / Nice-to-Have Features

### Session Reports & Analytics

**Inspiration**: Gemini CLI session summary feature

**Description**: Add comprehensive session reporting at the end of each CLI session or on-demand, optimizing for token reuse and providing transparency into system performance.

**Example Output**:
```
Agent powering down. Goodbye!

Interaction Summary
Session ID:                 3c6ddcd3-85b6-48f1-88e1-f428ca458337
Tool Calls:                 231 ( ✓ 214 x 17 )
Success Rate:               92.6%
User Agreement:             100.0% (230 reviewed)
Code Changes:               +505 -208

Performance
Wall Time:                  4h 9m 54s
Agent Active:               2h 53m 17s
  » API Time:               1h 9m 42s (40.2%)
  » Tool Time:              1h 43m 35s (59.8%)

Model Usage                  Reqs   Input Tokens  Output Tokens
───────────────────────────────────────────────────────────────
gemini-2.5-flash-lite          28         60,389          2,422
gemini-3-pro-preview          168     31,056,954         44,268
gemini-2.5-flash               83     10,707,161         48,817

Savings Highlight: 37,841,980 (90.5%) of input tokens were served from
cache, reducing costs.

» Tip: For a full token breakdown, run `rad stats model`.
```

**Key Features**:
- **Session Tracking**: Track session ID, duration, and active time
- **Tool Metrics**: Success rate, failure count, user approval metrics
- **Code Impact**: Lines added/removed during session
- **Performance Breakdown**: Wall time vs agent active time, API vs tool execution
- **Model Usage**: Per-model token consumption tracking
- **Cache Optimization**: Highlight token reuse from caching
- **Cost Transparency**: Show estimated costs and savings
- **Export Options**: JSON output for programmatic analysis

**Implementation Considerations**:
- Session persistence across CLI invocations
- Background telemetry collection (opt-in)
- Privacy-focused (no code content, just metrics)
- Integration with existing workspace structure
- Storage in `.radium/_internals/sessions/`
- Commands:
  - `rad stats session` - Current session stats
  - `rad stats model` - Detailed model usage breakdown
  - `rad stats history` - Historical session summaries
  - `rad stats export` - Export analytics to JSON

**Benefits**:
- Transparency into model costs
- Performance optimization insights
- Token reuse optimization
- User productivity metrics
- Debugging session issues

**Status**: ✅ Complete (2025-01-XX)
**Priority**: Low (Future Enhancement) → ✅ Implemented

---

## Other Future Ideas

### Enhanced Agent Discovery
- Agent marketplace/registry
- Remote agent repositories
- Agent versioning and updates
- Community-contributed agents

### Workflow Templates
- Template marketplace
- Custom template creation wizard
- Template variables and parameterization
- Template inheritance and composition

### Interactive Mode
- REPL for agent interaction
- Live agent output streaming
- Interactive plan refinement
- Real-time collaboration features

### Advanced Orchestration
- Distributed agent execution
- Agent-to-agent communication
- Dynamic agent spawning
- Resource-aware scheduling

### Integration & Plugins
- IDE plugins (VSCode, JetBrains)
- Git hooks integration
- CI/CD pipeline integration
- Webhook support for external triggers

### Observability
- Real-time agent monitoring dashboard
- Execution trace visualization
- Performance profiling tools
- Alert system for failures

### Extension System

**Status**: Planned for Step 10  
**Priority**: Medium

Installable extensions that package prompts, MCP servers, and custom commands. Enables community-contributed extensions and easy sharing of agent configurations.

**Reference**: [gemini-cli-enhancements.md](./gemini-cli-enhancements.md#extension-system)

---

### Hooks System

**Status**: Planned for Step 10  
**Priority**: Medium

Intercept and customize behavior at various points in the execution flow. Supports before/after model calls, tool selection, error handling, and telemetry hooks.

**Reference**: [gemini-cli-enhancements.md](./gemini-cli-enhancements.md#hooks-system)

---

### Security & Compliance
- Permission systems
- Audit logging
- Secrets management
- Compliance reporting

---

## Contributing

Have an idea for a feature? Add it to this document or open an issue on the repository.
