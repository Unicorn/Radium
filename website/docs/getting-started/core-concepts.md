---
id: "core-concepts"
title: "Core Concepts"
sidebar_label: "Core Concepts"
---

# Core Concepts

Understanding Radium's fundamental concepts will help you build powerful, composable AI systems. This guide introduces the key ideas that make Radium unique.

## Vision: Composable Intelligence Infrastructure

Radium is evolving toward a **composable intelligence infrastructure** where AI systems are built from reusable, validated components that can automatically assemble themselves. This vision, inspired by the OpenKor architecture, enables:

- **Component Reusability**: Build once, use everywhere
- **Automatic Assembly**: Systems compose themselves from available components
- **Continuous Improvement**: Self-healing and self-improving ecosystems
- **Global Collaboration**: Shared component marketplace

Learn more in our [Roadmap](../roadmap/index.md).

## Agents

**Agents** are specialized AI assistants configured for specific tasks. Each agent has:

- **Identity**: Name, description, and role
- **Capabilities**: Defined by prompts and tools
- **Configuration**: Model selection, parameters, and behavior
- **Context**: Access to workspace files, memory, and other agents

### Agent Types

- **Specialist Agents**: Focused on specific domains (code review, architecture, testing)
- **General Agents**: Broad capabilities for diverse tasks
- **Orchestrator Agents**: Coordinate multiple agents for complex workflows

### Agent Lifecycle

1. **Creation**: Define agent configuration and prompts
2. **Discovery**: Radium automatically finds agents in workspace
3. **Execution**: Agents run tasks using their configured models
4. **Memory**: Results stored for future reference
5. **Learning**: Agents improve from feedback and usage patterns

**Learn more**: [Agent Configuration](../user-guide/agent-configuration.md)

## Orchestration

**Orchestration** is Radium's intelligent task routing system that automatically:

- **Analyzes** your natural language requests
- **Selects** the best agent(s) for the task
- **Coordinates** multi-agent workflows
- **Synthesizes** results from multiple agents

Instead of manually choosing agents, you describe what you need, and the orchestrator handles the rest.

### Orchestration Benefits

- **Natural Interaction**: Type requests naturally without command syntax
- **Intelligent Routing**: Automatically finds the right specialist
- **Multi-Agent Coordination**: Handles complex workflows automatically
- **Model Agnostic**: Works with any AI provider

**Learn more**: [Orchestration Guide](../user-guide/orchestration.md)

## Components & Extensions

**Components** are reusable building blocks that can be shared and composed:

- **Prompts**: Agent prompt templates
- **MCP Servers**: Model Context Protocol integrations
- **Commands**: Custom CLI commands
- **Hooks**: Native or WASM modules for custom behavior

**Extensions** package components for distribution and sharing.

### Component Foundry Pattern

The Component Foundry Pattern (from OpenKor) provides:

- **Standardized Interfaces**: Consistent component patterns
- **Validation Framework**: Automated quality checks
- **Composition Rules**: Clear guidelines for combining components
- **Version Management**: Semantic versioning and compatibility

**Learn more**: [Extension System](../extensions/README.md) | [Roadmap: Component Foundry](../roadmap/vision.md#1-component-foundry-pattern-cfp)

## Policies & Security

**Policies** provide fine-grained control over agent behavior:

- **Tool Execution Control**: Allow, deny, or require approval for specific tools
- **Context-Aware Rules**: Different policies for different contexts
- **Approval Modes**: Yolo, AutoEdit, or Ask modes
- **Session Constitutions**: Temporary rules for specific sessions

### Policy Engine

The policy engine ensures:

- **Safety**: Prevent unwanted operations
- **Compliance**: Enforce organizational rules
- **Flexibility**: Different policies for different scenarios
- **Transparency**: Clear policy application and logging

**Learn more**: [Policy Engine](../features/policy-engine.md)

## Memory & Context

**Memory** enables agents to maintain continuity across sessions:

- **Plan-Scoped Memory**: Storage per requirement/plan
- **Agent Output Storage**: Automatic persistence of agent results
- **Context Retrieval**: Access previous outputs for context

**Context Sources** provide information to agents:

- **File Sources**: Project files and documentation
- **HTTP Sources**: External APIs and documentation
- **Jira Integration**: Issue tracking integration
- **BrainGrid Integration**: Requirement management

**Learn more**: [Memory & Context](../user-guide/memory-and-context.md)

## Planning & Execution

**Planning** converts high-level goals into structured, executable workflows:

- **Goal Decomposition**: Break down complex goals into tasks
- **Dependency Analysis**: Build dependency graphs (DAGs)
- **Validation**: Multi-stage validation with retry logic
- **Workflow Generation**: Create executable workflow templates

**Execution** runs plans with:

- **Automatic Agent Selection**: Choose agents for each task
- **Dependency Resolution**: Execute tasks in correct order
- **Error Handling**: Graceful failure and recovery
- **Progress Tracking**: Monitor execution status

**Learn more**: [Autonomous Planning](../features/planning/autonomous-planning.md)

## Persona System

The **Persona System** provides intelligent model selection:

- **Cost Optimization**: Automatically choose cost-effective models
- **Performance Profiles**: Balance speed, cost, and quality
- **Fallback Chains**: Automatic fallback to alternative models
- **Model Selection**: Primary, fallback, and premium model tiers

**Learn more**: [Persona System](../user-guide/persona-system.md)

## Learning System

The **Learning System** tracks and applies knowledge:

- **Mistake Tracking**: Learn from errors
- **Preference Learning**: Remember user preferences
- **Success Patterns**: Identify what works
- **ACE Skillbook**: Reusable strategies from past work

**Learn more**: [Learning System](../user-guide/learning-system.md)

## Vibe Check (Metacognitive Oversight)

**Vibe Check** provides Chain-Pattern Interrupt (CPI) functionality:

- **Reasoning Lock-In Prevention**: Detect when agents get stuck
- **Risk Assessment**: Identify potential issues early
- **Pattern Detection**: Recognize problematic patterns
- **Phase-Aware Feedback**: Adapt to planning, implementation, or review phases

Research shows CPI systems improve success rates by +27% and reduce harmful actions by -41%.

**Learn more**: [Vibe Check](../user-guide/vibe-check.md)

## Future: Component Ecosystem

Radium is evolving toward a **global component ecosystem**:

### Component Foundry
- Systematic component creation and validation
- Quality assurance frameworks
- Reusable component patterns

### Global Component Graph
- Discover components across the ecosystem
- Automatic composition from available components
- Component relationship tracking

### Autonomous Assembly
- Systems that compose themselves
- Goal-driven component selection
- Dynamic reconfiguration

**Learn more**: [Roadmap: Vision & Innovation](../roadmap/vision.md)

## Key Architectural Patterns

### Durable Autonomous Continuous Remediation (DACR)
Self-healing systems that maintain component quality over time without manual intervention.

### Durable Recursive Component Generation (DRCG)
Components that generate other components recursively, creating self-extending systems.

### Autonomous Component-Centric Assembly (ACCA)
Systems that automatically assemble themselves from available components based on goals and constraints.

**Learn more**: [Roadmap: Vision & Innovation](../roadmap/vision.md#key-innovations)

## Next Steps

- **[Quick Start](./quick-start.md)** - Create your first agent
- **[User Guide](../user-guide/overview.md)** - Explore all features
- **[Developer Guide](../developer-guide/overview.md)** - Extend Radium
- **[Roadmap](../roadmap/index.md)** - See the future vision

---

**Understanding these concepts** will help you build more powerful and composable AI systems with Radium.

