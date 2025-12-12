---
id: "t1-core-architecture"
title: "T1: Core Architecture Specification"
sidebar_label: "T1: Core Architecture"
---

# T1: Core Architecture Specification

**Source**: `T1_ Core Architecture Specification.pdf`
**Status**: 🚧 Extraction in Progress
**Roadmap**: [Technical Architecture Roadmap](../../roadmap/technical-architecture.md#core-architecture-specification-t1)

## Overview

The Core Architecture Specification defines the foundation systems and patterns for the composable intelligence infrastructure. This document provides detailed technical specifications for building the core platform.

## Architecture Principles

### Modularity
- Systems designed as composable modules
- Clear separation of concerns
- Independent module lifecycle
- Standardized interfaces

### Extensibility
- Plugin architecture for custom components
- Hook system for behavior injection
- Extension system for component distribution
- API-first design

### Performance
- Low-latency orchestration (<100ms overhead)
- Concurrent execution support
- Efficient resource utilization
- Scalable architecture

### Reliability
- Fault-tolerant design
- Automatic recovery mechanisms
- Health monitoring
- Graceful degradation

## Core Components

### 1. Agent Orchestration Engine

#### Responsibilities
- Multi-agent task coordination
- Intelligent agent selection
- Workflow execution
- Result synthesis

#### Architecture
```
┌─────────────────────────────────────┐
│     Orchestration Engine            │
├─────────────────────────────────────┤
│  - Task Analyzer                    │
│  - Agent Selector                   │
│  - Workflow Executor                │
│  - Result Synthesizer               │
└─────────────────────────────────────┘
```

#### Key Interfaces

**Task Analysis**
```rust
pub trait TaskAnalyzer {
    fn analyze(&self, task: &Task) -> Result<TaskAnalysis>;
}

pub struct TaskAnalysis {
    pub complexity: Complexity,
    pub required_capabilities: Vec<Capability>,
    pub estimated_duration: Duration,
    pub dependencies: Vec<TaskId>,
}
```

**Agent Selection**
```rust
pub trait AgentSelector {
    fn select_agents(&self, analysis: &TaskAnalysis) -> Result<Vec<AgentId>>;
    fn rank_agents(&self, agents: &[AgentId], context: &Context) -> Vec<RankedAgent>;
}
```

**Workflow Execution**
```rust
pub trait WorkflowExecutor {
    fn execute(&self, workflow: &Workflow) -> Result<WorkflowResult>;
    fn execute_parallel(&self, tasks: &[Task]) -> Result<Vec<TaskResult>>;
}
```

### 2. Policy Engine

#### Responsibilities
- Tool execution control
- Context-aware policy application
- Approval workflow management
- Security enforcement

#### Policy Rule Structure
```toml
[[rules]]
name = "rule-name"
priority = "admin"  # admin | user | default
action = "allow"   # allow | deny | ask_user
tool_pattern = "read_*"
arg_pattern = "*.md"
context = { workspace = "specific-workspace" }
```

#### Policy Evaluation
```rust
pub struct PolicyEngine {
    rules: Vec<PolicyRule>,
    approval_mode: ApprovalMode,
}

pub enum ApprovalMode {
    Yolo,      // Automatic approval
    AutoEdit,  // Auto-approve with edit capability
    Ask,       // Require user approval
}

impl PolicyEngine {
    pub fn evaluate(&self, request: &ToolRequest) -> PolicyDecision {
        // Rule matching and evaluation logic
    }
}
```

### 3. Extension System

#### Component Types
- **Prompts**: Agent prompt templates
- **MCP Servers**: Model Context Protocol integrations
- **Commands**: Custom CLI commands
- **Hooks**: Native/WASM behavior modules

#### Extension Manifest
```json
{
  "name": "extension-name",
  "version": "1.0.0",
  "description": "Extension description",
  "author": "Author Name",
  "components": {
    "prompts": ["prompts/**/*.md"],
    "mcp_servers": ["mcp/*.json"],
    "commands": ["commands/*.toml"],
    "hooks": ["hooks/*.toml"]
  },
  "dependencies": ["other-extension@^1.0.0"],
  "metadata": {
    "tags": ["category"],
    "compatibility": {
      "radium": ">=1.0.0"
    }
  }
}
```

#### Extension Lifecycle
1. **Discovery**: Scan extension directories
2. **Validation**: Verify manifest and components
3. **Installation**: Copy to extension directory
4. **Registration**: Register with core system
5. **Activation**: Make available for use

### 4. Model Abstraction Layer

#### Engine Interface
```rust
pub trait Engine {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
    async fn stream_chat(&self, request: ChatRequest) -> Result<Stream<ChatChunk>>;
    fn supports_reasoning(&self) -> bool;
    fn get_models(&self) -> Vec<ModelInfo>;
}

pub struct ChatRequest {
    pub messages: Vec<Message>,
    pub model: String,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub tools: Option<Vec<Tool>>,
}

pub struct ChatResponse {
    pub content: String,
    pub usage: TokenUsage,
    pub finish_reason: FinishReason,
}
```

#### Supported Providers
- **Gemini**: Google AI models
- **Claude**: Anthropic models
- **OpenAI**: GPT models
- **Self-Hosted**: Ollama, vLLM, LocalAI

### 5. Component Architecture (Foundation)

#### Component Interface
```rust
pub trait Component {
    fn id(&self) -> &ComponentId;
    fn version(&self) -> &Version;
    fn metadata(&self) -> &ComponentMetadata;
    fn validate(&self) -> Result<ValidationResult>;
    fn execute(&self, input: ComponentInput) -> Result<ComponentOutput>;
}

pub struct ComponentMetadata {
    pub name: String,
    pub description: String,
    pub author: String,
    pub tags: Vec<String>,
    pub dependencies: Vec<ComponentDependency>,
    pub interfaces: Vec<Interface>,
}
```

#### Component Lifecycle
1. **Creation**: Define component interface and implementation
2. **Validation**: Verify component meets standards
3. **Registration**: Register in component registry
4. **Discovery**: Make discoverable via component graph
5. **Composition**: Use in composed systems
6. **Evolution**: Update and version management

## Service Architecture

### Service Discovery
```rust
pub trait ServiceRegistry {
    fn register(&self, service: ServiceDescriptor) -> Result<ServiceId>;
    fn discover(&self, criteria: ServiceCriteria) -> Vec<ServiceDescriptor>;
    fn health_check(&self, service_id: &ServiceId) -> Result<HealthStatus>;
}
```

### Inter-Service Communication
- **gRPC**: Primary communication protocol
- **Message Queue**: Async message passing
- **Event Bus**: Event-driven communication
- **REST API**: HTTP-based APIs

### State Management
```rust
pub trait StateStore {
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>>;
    fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<()>;
    fn delete(&self, key: &str) -> Result<()>;
}
```

### Event System
```rust
pub trait EventBus {
    fn publish(&self, event: Event) -> Result<()>;
    fn subscribe(&self, filter: EventFilter, handler: EventHandler) -> SubscriptionId;
    fn unsubscribe(&self, subscription_id: SubscriptionId) -> Result<()>;
}

pub struct Event {
    pub event_type: String,
    pub source: String,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}
```

## Data Models

### Agent Configuration
```rust
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub prompt_path: PathBuf,
    pub engine: EngineType,
    pub model: String,
    pub persona: Option<PersonaConfig>,
    pub tools: Vec<ToolConfig>,
}
```

### Workspace Structure
```
.radium/
├── config.toml           # Workspace configuration
├── policy.toml           # Policy rules
├── agents/               # Agent configurations
│   └── category/
│       └── agent.toml
├── extensions/           # Installed extensions
│   └── extension-name/
├── plan/                 # Plan execution data
│   └── REQ-XXX/
│       ├── plan.json
│       └── memory/
└── _internals/          # Internal state
    ├── artifacts/
    ├── memory/
    └── logs/
```

## Performance Requirements

### Latency Targets
- **Orchestration overhead**: <100ms
- **Agent selection**: <50ms
- **Component lookup**: <10ms
- **Policy evaluation**: <5ms

### Throughput Targets
- **Concurrent agents**: 100+
- **Requests per second**: 1000+
- **Component operations**: 10,000+/sec

### Resource Limits
- **Memory per agent**: <100MB
- **CPU per agent**: <1 core
- **Network bandwidth**: Adaptive

## Security Architecture

### Authentication
- API key management
- OAuth integration
- Token-based authentication

### Authorization
- Role-based access control (RBAC)
- Policy-based authorization
- Context-aware permissions

### Data Protection
- Encryption at rest
- Encryption in transit
- Secure credential storage

## Error Handling

### Error Types
```rust
pub enum CoreError {
    AgentNotFound(AgentId),
    PolicyDenied(ToolRequest),
    ComponentValidationFailed(ValidationError),
    ServiceUnavailable(ServiceId),
    Timeout(Duration),
    NetworkError(NetworkError),
}
```

### Recovery Strategies
- Automatic retry with exponential backoff
- Fallback to alternative agents
- Graceful degradation
- Circuit breaker pattern

## Implementation Status

### ✅ Completed
- Multi-agent orchestration engine
- Policy engine and security framework
- Extension system
- Model abstraction layer

### 🚧 In Progress
- Component interface definitions
- Component lifecycle management
- Service discovery and registration

### 📋 Planned
- Component validation framework
- Component composition engine
- Inter-service communication protocols
- State management and persistence
- Event system and messaging

## Related Documentation

- **[Technical Architecture Roadmap](../../roadmap/technical-architecture.md#core-architecture-specification-t1)**
- **[Agent System Architecture](../../developer-guide/agent-system-architecture.md)**
- **[Extension System Architecture](../../extensions/architecture.md)**
- **[Architecture Overview](../../developer-guide/architecture/overview.md)**

---

**Note**: This specification is extracted from the OpenKor T1 document. Some details may need manual review and enhancement from the source PDF.

