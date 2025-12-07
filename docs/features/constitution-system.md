# Constitution System

The Constitution System provides session-scoped rules for temporary constraints during workflow execution. Unlike static policy rules, constitution rules are applied per-session and automatically cleaned up after inactivity.

## Overview

Constitution rules allow you to add temporary, session-specific constraints that complement the static policy rules. These rules are useful for:

- Per-session security constraints
- Temporary restrictions for specific tasks
- Dynamic policy adjustments without modifying policy.toml
- Session-specific behavioral guidelines

## Key Features

- **Session-scoped**: Rules apply only to a specific session ID
- **Automatic cleanup**: Rules are removed after 1 hour of inactivity (TTL)
- **Rule limits**: Maximum 50 rules per session
- **In-memory storage**: Fast access, no file I/O overhead
- **FIFO eviction**: Oldest rules removed when limit reached

## How It Works

1. **Session Creation**: A session ID is generated or provided for workflow execution
2. **Rule Addition**: Rules are added via `ConstitutionManager::update_constitution()`
3. **Rule Retrieval**: Rules are fetched via `get_constitution()` with automatic timestamp update
4. **TTL Enforcement**: Background task cleans stale sessions every hour
5. **FIFO Eviction**: Oldest rules removed when MAX_RULES_PER_SESSION (50) is reached

## Usage

### Adding Rules

```rust
use radium_core::policy::ConstitutionManager;

let manager = ConstitutionManager::new();
manager.update_constitution("session-123", "no external network calls".to_string());
manager.update_constitution("session-123", "prefer unit tests over integration tests".to_string());
```

### Getting Rules

```rust
let rules = manager.get_constitution("session-123");
// Returns: Vec<String> with all rules for the session
```

### Resetting Rules

```rust
// Clear all rules for a session
manager.reset_constitution("session-123", vec![]);

// Or replace with new rules
manager.reset_constitution("session-123", vec![
    "new rule 1".to_string(),
    "new rule 2".to_string(),
]);
```

## CLI Commands

### Update Constitution

Add or update a rule for a session:

```bash
rad constitution update <session-id> "<rule text>"
```

Example:
```bash
rad constitution update session-123 "no external API calls"
rad constitution update session-123 "use TypeScript strict mode"
```

### Reset Constitution

Clear all rules for a session:

```bash
rad constitution reset <session-id>
```

Example:
```bash
rad constitution reset session-123
```

### Get Constitution

View all rules for a session:

```bash
rad constitution get <session-id>
```

Example:
```bash
rad constitution get session-123
```

Output:
```
Constitution Rules for Session: session-123
==========================================
1. no external API calls
2. use TypeScript strict mode
3. prefer unit tests over integration tests

Total rules: 3
```

### List Sessions

List all active sessions (note: currently shows info about session management):

```bash
rad constitution list
```

## Integration with Workflow Execution

Constitution rules are automatically integrated with workflow execution through the `WorkflowExecutor`. When a workflow is executed with a session ID, constitution rules are retrieved and applied alongside static policy rules.

### Example Workflow Integration

```rust
use radium_core::policy::ConstitutionManager;
use radium_core::workflow::WorkflowExecutor;

let constitution_manager = Arc::new(ConstitutionManager::new());
let executor = WorkflowExecutor::new(orchestrator, agent_executor, monitoring);

// Add session-specific rules
constitution_manager.update_constitution("workflow-session-1", 
    "no database modifications".to_string());

// Rules are automatically applied during workflow execution
```

## TTL and Cleanup

### Time-to-Live (TTL)

- **Default TTL**: 1 hour (60 minutes)
- **Automatic cleanup**: Background task runs every hour
- **Timestamp update**: Accessing rules via `get_constitution()` updates the timestamp

### Stale Session Detection

Sessions are considered stale if:
- Last access was more than 1 hour ago
- Session has no active rules

Stale sessions are automatically removed during cleanup.

## Rule Limits

### Maximum Rules Per Session

- **Limit**: 50 rules per session
- **Enforcement**: When limit is reached, oldest rules are removed (FIFO)
- **Recommendation**: Keep rules concise and focused

### Best Practices

1. **Keep rules focused**: Write specific, actionable rules
2. **Use session IDs consistently**: Use the same session ID throughout workflow execution
3. **Monitor rule count**: Check rule count with `rad constitution get`
4. **Reset when needed**: Use `reset` to clear rules instead of adding many new ones

## Architecture

### Components

- **ConstitutionManager**: Main manager for session rules
- **ConstitutionEntry**: Internal structure storing rules and timestamp
- **Cleanup Task**: Background tokio task for TTL enforcement

### Thread Safety

- **RwLock**: Thread-safe access to session data
- **Arc**: Shared ownership across threads
- **Lock-free reads**: Read operations use read lock (concurrent reads allowed)

## Comparison with Policy Rules

| Feature | Policy Rules | Constitution Rules |
|---------|-------------|-------------------|
| **Scope** | Workspace-wide | Session-specific |
| **Persistence** | File-based (.radium/policy.toml) | In-memory only |
| **Lifetime** | Permanent until removed | 1 hour TTL |
| **Priority** | Admin/User/Default | All equal |
| **Max Rules** | Unlimited | 50 per session |
| **Use Case** | Static security policies | Temporary constraints |

## Examples

### Use Case: Restricted Development Session

```bash
# Start workflow with restrictive rules
rad constitution update dev-session-1 "no network calls"
rad constitution update dev-session-1 "no file system writes outside workspace"
rad constitution update dev-session-1 "require approval for database operations"

# Execute workflow (rules automatically applied)
rad craft REQ-123

# View rules
rad constitution get dev-session-1

# Clean up when done
rad constitution reset dev-session-1
```

### Use Case: Testing Session

```bash
# Add testing-specific constraints
rad constitution update test-session "prefer mocking over real API calls"
rad constitution update test-session "use test database only"
rad constitution update test-session "cleanup after each test"

# Run tests with constraints
rad run test-suite

# Rules automatically cleaned up after 1 hour
```

## Troubleshooting

### Rules Not Applied

- Verify session ID matches between rule addition and workflow execution
- Check that rules haven't expired (1 hour TTL)
- Ensure ConstitutionManager is properly initialized in WorkflowExecutor

### Rules Disappear

- Check if TTL has expired (1 hour of inactivity)
- Verify rule count hasn't exceeded 50 (oldest rules removed)
- Check cleanup task is running (should run every hour)

### Session Not Found

- Verify session ID is correct
- Check if session has expired (no access for 1+ hour)
- Ensure rules were added to the correct session ID

## See Also

- [Policy Engine](./policy-engine.md) - Static policy rules
- [Workflow Behaviors](./workflow-behaviors.md) - Dynamic execution control
- [CLI Reference](../cli/commands/constitution.md) - Command documentation

