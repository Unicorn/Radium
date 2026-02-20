# ADR-002: Component Abstraction Layer for External Services

## Status

Accepted

## Date

2024 (Phase 2-3 of Rust migration)

## Context

Workflows interact with external services:
- Kong API Gateway (logging, caching, CORS, rate limiting)
- Supabase (database, auth, storage)
- Temporal (workflows, activities, signals)
- External APIs (HTTP requests)
- AI services (agent activities)

Each service has its own API and configuration requirements. We needed to decide how to represent these in the workflow builder and code generation.

## Decision

Create an abstraction layer where:
1. Each external service integration is a "component type"
2. Components define their inputs, outputs, and behaviors
3. Code generation produces configuration objects, not direct API calls
4. Actual API integration happens at deployment time

## Rationale

### Separation of Concerns
Workflow definitions describe WHAT should happen, not HOW it integrates with external services. This separation allows:
- Changing integration implementations without modifying workflows
- Testing workflows without external service dependencies
- Deploying to different environments with different configurations

### Configuration Over Code
For Kong plugins (logging, caching, CORS), we generate configuration objects rather than API calls because:
- Kong plugins are configured declaratively
- Configuration can be version-controlled
- Deployment tools can diff and apply changes
- No runtime coupling between workflows and Kong

### Component Consistency
All components follow the same pattern:
```typescript
const result_component = { type: 'component-type', ...config };
```

This consistency:
- Simplifies code generation
- Makes workflows easier to understand
- Enables consistent tooling

### Future Flexibility
The abstraction layer allows:
- Swapping Kong for another API gateway
- Supporting multiple database backends
- Adding new external services easily

## Implementation

### Kong Components
```rust
enum KongComponentType {
    Logging(KongLoggingConfig),
    Cache(KongCacheConfig),
    Cors(KongCorsConfig),
    RateLimit(KongRateLimitConfig),
}
```

Each generates a configuration object:
```typescript
const result_logging = {
  type: 'kong-logging-config',
  connector: 'production-kong'
};
```

### Activity Components
Activities that call external services generate actual function calls:
```typescript
const result = await httpRequest(input);
```

The activity implementation handles the external service integration.

### Deployment Integration
Deployment tools read the generated configuration objects and:
1. Create/update Kong plugins
2. Configure service connections
3. Set up database schemas
4. Register Temporal workflows

## Alternatives Considered

### Direct API Calls in Workflows
**Rejected because:**
- Couples workflows to specific service versions
- Makes testing harder
- Prevents environment-specific configuration

### Generic "External Service" Component
**Rejected because:**
- Too abstract, loses type safety
- Harder to validate configurations
- Less discoverable in UI

### Code Generation for Each Integration
**Rejected because:**
- Generated code becomes complex
- Harder to maintain across service versions
- Deployment becomes build step

## Consequences

### Positive
- Clean separation between workflow logic and infrastructure
- Easy to add new external service integrations
- Configuration is declarative and version-controlled
- Testing doesn't require external services

### Negative
- Two-step process (generate + deploy)
- Configuration objects aren't directly executable
- Requires deployment tooling to interpret configs

## Related Decisions

- ADR-001: Rust for validation (enables strong typing of configs)
- ADR-003: Temporal patterns (how we abstract Temporal specifics)
