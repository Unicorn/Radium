# Prioritized Testing Backlog for REQ-172

**Generated:** 2025-12-07  
**Based on:** Coverage analysis from TASK-1  
**Target:** 100% line coverage, 100% function coverage, ≥95% branch coverage

## Overview

This backlog categorizes all uncovered code by module, criticality, and testing phase. Items are prioritized based on:
1. **Module Criticality** - Core domain logic > utilities
2. **Current Coverage Level** - Lowest coverage first
3. **Risk Assessment** - Error handlers, concurrent operations, critical paths

## Phase 2: Critical Path Coverage (Target: 90%+)

### Priority 1: Hooks Module (Current: ~50% average)

**Criticality:** HIGH  
**Current Coverage:** 50.34% average across hooks module  
**Target Coverage:** 90%+  
**Estimated Tests Needed:** ~150 tests

#### hooks/error_hooks.rs - 0.00% coverage
- **Priority:** CRITICAL
- **Uncovered:** 94 lines, 14 functions
- **Risk:** HIGH - Error handling hooks are critical for error recovery
- **Test Focus:**
  - Error hook registration and execution
  - Error transformation hooks
  - Error recovery hooks
  - Error logging hooks
  - Hook failure scenarios
- **Estimated Tests:** 25

#### hooks/tool.rs - 0.00% coverage
- **Priority:** CRITICAL
- **Uncovered:** 52 lines, 12 functions
- **Risk:** HIGH - Tool hooks intercept tool execution
- **Test Focus:**
  - Tool hook registration (before, after, selection)
  - Tool execution interception
  - Tool selection hooks
  - Hook priority ordering
  - Hook failure handling
- **Estimated Tests:** 20

#### hooks/model.rs - 0.00% coverage
- **Priority:** CRITICAL
- **Uncovered:** 40 lines, 10 functions
- **Risk:** HIGH - Model hooks intercept model calls
- **Test Focus:**
  - Model hook registration (before, after)
  - Model call interception
  - Request/response transformation
  - Hook execution sequence
- **Estimated Tests:** 15

#### hooks/config.rs - 14.14% coverage
- **Priority:** HIGH
- **Uncovered:** 67 lines, 12 functions
- **Risk:** HIGH - Hook configuration loading
- **Test Focus:**
  - Configuration file parsing
  - Hook factory registration
  - Configuration validation
  - Invalid configuration handling
- **Estimated Tests:** 20

#### hooks/loader.rs - 55.04% coverage
- **Priority:** HIGH
- **Uncovered:** 107 lines, 12 functions
- **Risk:** MEDIUM - Hook loading from files
- **Test Focus:**
  - Hook discovery from directories
  - Dynamic hook loading
  - Hook factory resolution
  - Loading error scenarios
- **Estimated Tests:** 15

#### hooks/registry.rs - 70.34% coverage
- **Priority:** MEDIUM
- **Uncovered:** 70 lines, 11 functions
- **Risk:** MEDIUM - Hook registry operations
- **Test Focus:**
  - Concurrent hook registration
  - Hook priority ordering
  - Hook filtering by type
  - Hook unregistration
- **Estimated Tests:** 10

#### hooks/composition.rs - 63.59% coverage
- **Priority:** MEDIUM
- **Uncovered:** 67 lines, 20 functions
- **Risk:** MEDIUM - Hook composition patterns
- **Test Focus:**
  - Composite hook execution
  - Conditional hook execution
  - Hook chain execution
- **Estimated Tests:** 15

#### hooks/types.rs - 34.09% coverage
- **Priority:** MEDIUM
- **Uncovered:** 29 lines, 5 functions
- **Risk:** LOW - Hook type definitions
- **Test Focus:**
  - Hook type conversions
  - Hook priority handling
  - Hook context creation
- **Estimated Tests:** 5

#### hooks/telemetry.rs - 0.00% coverage
- **Priority:** LOW
- **Uncovered:** 13 lines, 3 functions
- **Risk:** LOW - Telemetry hooks
- **Test Focus:**
  - Telemetry hook registration
  - Telemetry data collection
- **Estimated Tests:** 5

#### hooks/marketplace.rs - 0.00% coverage
- **Priority:** LOW
- **Uncovered:** 32 lines, 12 functions
- **Risk:** LOW - Marketplace hooks (feature-gated)
- **Test Focus:**
  - Marketplace hook discovery
  - Hook installation from marketplace
- **Estimated Tests:** 5

### Priority 2: Error Handlers (Current: 90.48% in error.rs)

**Criticality:** HIGH  
**Current Coverage:** 90.48% in main error.rs, varies by module  
**Target Coverage:** 90%+  
**Estimated Tests Needed:** ~50 tests

#### Module-specific error types
- **auth/error.rs** - 100% coverage ✅
- **mcp/error.rs** - 100% coverage ✅
- **checkpoint/error.rs** - Need to verify coverage
- **storage/error.rs** - Need to verify coverage
- **monitoring/error.rs** - Need to verify coverage
- **hooks/error.rs** - Need to verify coverage
- **context/error.rs** - Need to verify coverage

**Test Focus:**
- Error type construction for each variant
- Error conversion between types (StorageError → RadiumError)
- Error display formatting
- Error source chain preservation
- Error context preservation

**Estimated Tests:** 50

### Priority 3: Agents Module (Current: ~75% average)

**Criticality:** HIGH  
**Current Coverage:** 75.23% average  
**Target Coverage:** 90%+  
**Estimated Tests Needed:** ~80 tests

#### agents/validation.rs - 47.90% coverage
- **Priority:** HIGH
- **Uncovered:** 62 lines, 9 functions
- **Risk:** HIGH - Agent validation is critical
- **Test Focus:**
  - ID format validation edge cases
  - Prompt path validation
  - Engine validation
  - Configuration validation rules
  - Invalid configuration error messages
- **Estimated Tests:** 15

#### agents/telemetry.rs - 45.95% coverage
- **Priority:** MEDIUM
- **Uncovered:** 40 lines, 5 functions
- **Risk:** MEDIUM - Telemetry collection
- **Test Focus:**
  - Telemetry data collection
  - Usage recording
  - Telemetry aggregation
- **Estimated Tests:** 10

#### agents/model_selector.rs - 62.76% coverage
- **Priority:** MEDIUM
- **Uncovered:** 54 lines, 3 functions
- **Risk:** MEDIUM - Model selection logic
- **Test Focus:**
  - Model selection algorithms
  - Fallback chain execution
  - Speed profile matching
- **Estimated Tests:** 10

#### agents/analytics.rs - 61.06% coverage
- **Priority:** MEDIUM
- **Uncovered:** 88 lines, 11 functions
- **Risk:** LOW - Analytics collection
- **Test Focus:**
  - Usage recording
  - Popular agents calculation
  - Analytics aggregation
- **Estimated Tests:** 10

#### agents/discovery.rs - 73.12% coverage
- **Priority:** MEDIUM
- **Uncovered:** 107 lines, 5 functions
- **Risk:** MEDIUM - Agent discovery
- **Test Focus:**
  - Discovery from multiple paths
  - Malformed TOML handling
  - Category extraction
  - Sub-agent filtering
- **Estimated Tests:** 15

#### agents/registry.rs - 77.58% coverage
- **Priority:** MEDIUM
- **Uncovered:** 141 lines, 32 functions
- **Risk:** MEDIUM - Agent registry operations
- **Test Focus:**
  - Concurrent registration
  - Filtering and sorting edge cases
  - Search functionality
  - Registry state management
- **Estimated Tests:** 20

### Priority 4: Monitoring Module (Current: ~70% average)

**Criticality:** MEDIUM  
**Current Coverage:** 69.15% average  
**Target Coverage:** 90%+  
**Estimated Tests Needed:** ~60 tests

#### monitoring/service.rs - 65.05% coverage
- **Priority:** HIGH
- **Uncovered:** 151 lines, 17 functions
- **Risk:** MEDIUM - Monitoring service lifecycle
- **Test Focus:**
  - Service initialization
  - Agent registration and tracking
  - Agent completion handling
  - Service cleanup
  - Database operations
- **Estimated Tests:** 25

#### monitoring/telemetry.rs - 73.43% coverage
- **Priority:** MEDIUM
- **Uncovered:** 131 lines, 9 functions
- **Risk:** MEDIUM - Telemetry processing
- **Test Focus:**
  - Telemetry parsing
  - Token counting
  - Cost estimation
  - Telemetry aggregation
- **Estimated Tests:** 15

#### monitoring/logs.rs - 98.44% coverage ✅
- **Priority:** LOW
- **Uncovered:** 2 lines
- **Test Focus:** Edge cases only
- **Estimated Tests:** 2

#### monitoring/schema.rs - 94.05% coverage ✅
- **Priority:** LOW
- **Uncovered:** 11 lines
- **Test Focus:** Edge cases only
- **Estimated Tests:** 3

## Phase 3: Error Scenario Coverage (Target: 95%+)

### Database Error Scenarios
- Connection failures during operations
- Transaction rollback on errors
- Database lock timeouts
- Constraint violations
- **Estimated Tests:** 30

### File System Error Scenarios
- Permission denied errors
- Missing file/directory errors
- Corrupted configuration files
- Disk full scenarios
- **Estimated Tests:** 25

### Network Error Scenarios
- gRPC connection failures
- Model API timeouts
- Network interruptions during operations
- Invalid response handling
- **Estimated Tests:** 20

### Concurrent Access Scenarios
- Race conditions in agent registration
- Concurrent workflow execution conflicts
- Database concurrent access handling
- **Estimated Tests:** 15

## Phase 4: Edge Cases & Boundaries (Target: 100%)

### Boundary Conditions
- Empty strings, empty vectors, empty maps
- Zero values, null/None values
- Maximum string lengths, maximum array sizes
- Minimum and maximum numeric values
- **Estimated Tests:** 40

### State Machine Transitions
- Agent states: all transitions
- Workflow states: all transitions
- Task states: all transitions
- Invalid state transition attempts
- **Estimated Tests:** 30

### Conditional Branches
- All if/else branches
- All match arms in enums
- All pattern matching branches
- Short-circuit evaluation paths
- **Estimated Tests:** 50

### Enum Variants
- All AgentState variants
- All WorkflowState variants
- All TaskState variants
- All error type variants
- **Estimated Tests:** 20

## Low Priority Modules (Can be deferred)

### Analytics Module
- **analytics/code_changes.rs** - 0.00% (48 lines)
- **analytics/storage.rs** - 0.00% (101 lines)
- **Priority:** LOW - Analytics feature
- **Estimated Tests:** 20

### Extensions Module (Non-Critical)
- **extensions/analytics.rs** - 0.00% (150 lines)
- **extensions/dependency_graph.rs** - 0.00% (190 lines)
- **extensions/marketplace.rs** - 17.02% (121 lines)
- **extensions/publisher.rs** - 9.22% (70 lines)
- **Priority:** LOW - Extension features
- **Estimated Tests:** 50

### Context Module (Non-Critical)
- **context/metrics.rs** - 0.00% (90 lines)
- **context/sources/braingrid.rs** - 32.14% (57 lines)
- **context/sources/jira.rs** - 30.30% (69 lines)
- **context/validator.rs** - 38.98% (72 lines)
- **Priority:** LOW - Context sources
- **Estimated Tests:** 40

### MCP Module (Partial)
- **mcp/integration.rs** - 5.00% (176 lines)
- **mcp/client.rs** - 35.99% (201 lines)
- **mcp/auth.rs** - 52.02% (178 lines)
- **Priority:** MEDIUM - MCP integration
- **Estimated Tests:** 60

### Engines Module (Partial)
- **engines/config.rs** - 4.92% (52 lines)
- **engines/providers/claude.rs** - 54.65% (39 lines)
- **engines/providers/gemini.rs** - 46.15% (56 lines)
- **engines/providers/openai.rs** - 46.67% (56 lines)
- **engines/registry.rs** - 56.35% (220 lines)
- **Priority:** MEDIUM - Engine providers
- **Estimated Tests:** 50

## Testing Patterns to Reuse

### Test Utilities
- `crates/radium-core/tests/common/mod.rs` - Common test helpers
- Database test fixtures
- Mock agent configurations
- Mock hook implementations

### Test Patterns
- **Unit Tests:** In `#[cfg(test)]` blocks within source modules
- **Integration Tests:** In `crates/radium-core/tests/` directory
- **Error Tests:** Use `#[should_panic]` or `Result` assertions
- **Concurrent Tests:** Use `tokio::test` with `Arc` and `Mutex`

## Summary

### Total Estimated Tests Needed
- **Phase 2 (90%+):** ~340 tests
- **Phase 3 (95%+):** ~90 tests
- **Phase 4 (100%):** ~140 tests
- **Total:** ~570 tests

### Priority Order
1. **Hooks Module** - CRITICAL (150 tests)
2. **Error Handlers** - HIGH (50 tests)
3. **Agents Module** - HIGH (80 tests)
4. **Monitoring Module** - MEDIUM (60 tests)
5. **Error Scenarios** - MEDIUM (90 tests)
6. **Edge Cases** - LOW (140 tests)

### Risk Assessment
- **HIGH RISK:** Hooks (error_hooks, tool, model), Error handlers
- **MEDIUM RISK:** Agents validation, Monitoring service, MCP integration
- **LOW RISK:** Analytics, Extensions marketplace, Context sources

## Next Steps

1. Begin with Priority 1: Hooks Module (error_hooks.rs, tool.rs, model.rs)
2. Move to Priority 2: Error Handlers
3. Continue with Priority 3: Agents Module
4. Complete Priority 4: Monitoring Module
5. Implement error scenarios across all modules
6. Finish with edge cases and boundaries

