# TODO Tracking Document

Generated: 2026-01-12

This document tracks all TODO and FIXME comments in the codebase (75 total).

## Summary by Category

- **Braingrid Integration** (7 items) - Features waiting for Braingrid CLI updates
- **Budget/Analytics** (7 items) - Disabled features due to circular dependencies
- **MCP Proxy** (3 items) - Missing prompts aggregation and prompt retrieval
- **Workflow** (6 items) - Parallel execution, git integration, test results
- **Configuration** (3 items) - Config loading, checkpoint policies
- **Server/Orchestration** (5 items) - Agent execution integration, event emission
- **Type Mismatches** (2 items) - Tool type incompatibilities between crates
- **Learning System** (1 item) - Re-enable learning system integration
- **Other** (11 items) - Miscellaneous improvements

---

## PRIORITY 1: CRITICAL (Blocking Features)

### Circular Dependency - Budget Checking
**Status:** Partially resolved (collaboration module re-enabled)
- `crates/radium-orchestrator/src/executor.rs:574` - Budget checking disabled
- `crates/radium-orchestrator/src/executor.rs:611` - Budget cost recording disabled

**Action:** Budget tracking needs trait-based integration to avoid circular deps

### Type Mismatches - Tool Definitions
- `crates/radium-orchestrator/src/orchestration/mod.rs:12` - Tool type mismatch with radium_abstraction
- `crates/radium-orchestrator/src/orchestration/mod.rs:40` - Related type mismatch

**Action:** Unify Tool type definitions or create adapter layer

---

## PRIORITY 2: HIGH (Important Features)

### Workflow Integration
1. **Parallel Execution** (`workflow/parallel.rs:64`)
   - Requires refactoring repositories to Arc<dyn Repository + Send + Sync>
   - Currently uses sequential execution with parallel facade

2. **Git Integration** (`workflow/parallel_executor.rs:308`, `report_generator.rs:143`)
   - Extract git commits from execution results
   - Requires git workspace integration

3. **Test Results** (`workflow/parallel_executor.rs:311`, `report_generator.rs:146`)
   - Aggregate test results from task executions
   - Requires test framework integration

### Server/Orchestration
1. **Agent Execution** (`server/radium_service.rs:2021`)
   - Trigger agent execution via OrchestrationService
   - Currently not fully integrated

2. **Event Emission** (`server/radium_service.rs:2067`)
   - Integrate with agent execution for ToolCallEvent, ToolResultEvent
   - Real-time event streaming to clients

3. **Metadata Extraction** (`server/radium_service.rs:907`)
   - Extract metadata from ExecutionResult when available

---

## PRIORITY 3: MEDIUM (Enhancement Features)

### MCP Proxy Server
1. **Prompts Aggregation** (`mcp/proxy/server.rs:372`)
   - Implement prompts/list aggregation (similar to tools/list)

2. **Prompt Retrieval** (`mcp/proxy/server.rs:376`)
   - Implement prompts/get for retrieving specific prompts

3. **Tool Catalog Rebuild** (`mcp/proxy/types.rs:323`)
   - Add rebuild_catalog method to ToolCatalog trait

4. **Priority Sorting** (`mcp/proxy/router.rs:53`)
   - Sort by actual priority from config instead of hardcoded

### Configuration System
1. **Config Loading** (`config/mod.rs:270`)
   - Implement config file and env var loading
   - Currently uses defaults only

2. **Checkpoint Policy** (`checkpoint/snapshot.rs:376`, `snapshot.rs:877`)
   - Load CheckpointPolicy from config
   - Implement cleanup policy evaluation

3. **Secret Filter** (`context/manager.rs:190`)
   - Initialize SecretFilter when needed for context management

### Analytics/Budget
1. **Budget Recording** (`monitoring/budget.rs:306`, `442`, `456`, `554`)
   - Re-enable analytics module integration
   - Currently disabled due to module visibility

---

## PRIORITY 4: LOW (Nice to Have)

### Braingrid Integration (7 items)
**Waiting on upstream:** Braingrid CLI needs to add playbook support

1. `playbooks/braingrid_storage.rs:41` - Load playbook by URI
2. `playbooks/braingrid_storage.rs:59` - List playbooks
3. `playbooks/braingrid_storage.rs:73` - Search playbooks
4. `playbooks/braingrid_storage.rs:87` - Save playbook
5. `playbooks/braingrid_storage.rs:103` - Delete playbook
6. `context/braingrid_client.rs:464` - Add notes support
7. `agents/registry.rs:529` - Extract tags from agent metadata

### Learning System
1. **Re-enable Learning** (`autonomous/orchestrator.rs:689`)
   - Re-enable once method visibility issues are resolved
   - Learning system currently disconnected

### Syntax Highlighting
1. **tmTheme Loading** (`syntax/tmtheme_loader.rs:13`, `26`)
   - Implement proper tmTheme loading using plist crate
   - Currently using basic theme loading

### Hook System
1. **Marketplace Integration** (`hooks/marketplace.rs:96`)
   - Implement actual HTTP client when marketplace backend is available
   - Currently returns mock data

### Policy Engine
1. **Remember Decisions** (`apps/cli/src/policy_engine.rs:336`, `340`)
   - Remember user policy decisions for future runs

2. **Policy Learn Command** (`apps/cli/src/commands/policy.rs:3`, `9`)
   - Implement policy_learn module for learning from approvals

### Plugin System
1. **Dynamic Loading** (`plugin.rs:74`)
   - Implement actual dynamic loading using libloading
   - Currently stub implementation

### Other
1. **Batch Executor** (`radium-orchestrator/src/lib.rs:6`, `34`)
   - Fix compilation errors in batch_executor module

2. **Checkpoint Command** (`apps/cli/src/commands/checkpoint.rs:87`)
   - Implement policy_command for checkpoint management

3. **Daemon Integration** (`apps/cli/src/main.rs:750`)
   - Integrate daemon execution when session management is connected

4. **Tool Unblocking** (`server/radium_service.rs:2050`)
   - Unblock waiting tool execution on approval/denial

5. **Error Router** (`error_router.rs:368`, `418`)
   - Full implementation notes for production deployment

6. **Cost Tracking** (`routing/cost_tracker.rs:203`)
   - Implement full per-model cost tracking when needed

7. **Test Helpers** (`extensions/structure.rs:416`)
   - Use test helper that can set env vars safely

8. **Workflow Tasks** (`workflow/report_generator.rs:134`)
   - Get actual task titles from requirements instead of "Task {id}"

---

## Action Plan

### Immediate (Next Sprint)
1. ✅ Fix circular dependencies (collaboration module re-enabled)
2. Resolve tool type mismatches between crates
3. Re-enable budget checking with trait-based approach

### Short Term (1-2 Months)
1. Implement MCP proxy prompts aggregation
2. Add config file loading system
3. Complete workflow git integration
4. Implement event emission for agent execution

### Medium Term (3-6 Months)
1. Implement parallel workflow execution
2. Complete analytics module integration
3. Add marketplace HTTP client for hooks
4. Implement dynamic plugin loading

### Long Term (6+ Months)
1. Wait for Braingrid playbook support (7 items)
2. Implement policy learning system
3. Complete test results aggregation
4. Enhanced tmTheme loading

---

## How to Create GitHub Issues

Use this template for creating issues:

```markdown
**Title:** [Category] Brief description

**Type:** Enhancement / Bug / Feature

**Priority:** Critical / High / Medium / Low

**Location:** `file:line`

**Description:**
<!-- Copy the TODO comment context -->

**Current Behavior:**
<!-- What happens now -->

**Expected Behavior:**
<!-- What should happen -->

**Dependencies:**
<!-- Any blockers or prerequisites -->

**Acceptance Criteria:**
- [ ] Item 1
- [ ] Item 2

**Related Issues:**
<!-- Link to related TODOs if applicable -->
```

---

## Statistics

- **Total TODOs:** 75
- **Critical:** 4 (5%)
- **High:** 9 (12%)
- **Medium:** 16 (21%)
- **Low:** 46 (62%)

- **radium-core:** 49 items (65%)
- **radium-orchestrator:** 12 items (16%)
- **CLI/TUI:** 14 items (19%)

---

## Maintenance

This document should be regenerated periodically:

```bash
# Regenerate TODO list
grep -rn "TODO\|FIXME" crates/ apps/ | sed 's|/Users/clay/Development/RAD-Radium/||' > todos.txt

# Count TODOs
grep -r "TODO\|FIXME" crates/ apps/ | wc -l
```

Last updated: 2026-01-12
