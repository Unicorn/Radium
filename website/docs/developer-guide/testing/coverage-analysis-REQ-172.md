---
id: "coverage-analysis-REQ-172"
title: "Coverage Analysis Report for REQ-172"
sidebar_label: "Coverage Analysis Report fo..."
---

# Coverage Analysis Report for REQ-172

**Generated:** 2025-12-07  
**Crate:** radium-core  
**Tool:** cargo-llvm-cov  
**Test Suite:** Library tests only (--lib)

## Executive Summary

### Current Coverage Metrics

| Metric | Current | Target | Gap |
|--------|---------|--------|-----|
| **Line Coverage** | 76.83% | 100% | 23.17% (6,092 uncovered lines) |
| **Function Coverage** | 72.77% | 100% | 27.23% (892 uncovered functions) |
| **Region Coverage** | 79.28% | 100% | 20.72% (9,479 uncovered regions) |

**Note:** The REQ-172 document states 82.58% line coverage, but actual measurement shows 76.83%. This discrepancy may be due to:
- Different test execution scope (library vs integration tests)
- Code changes since last measurement
- Different coverage tool configuration

### Test Statistics

- **Total Tests:** 1,086 passed
- **Ignored Tests:** 2 (temporarily disabled)
- **Failed Tests:** 0

## Module-Level Coverage Analysis

### Modules with Low Coverage (&lt;70%)

These modules require immediate attention:

1. **analytics/code_changes.rs** - 0.00% coverage
   - 48 lines uncovered
   - 4 functions uncovered
   - **Priority:** Medium (analytics feature)

2. **analytics/storage.rs** - 0.00% coverage
   - 101 lines uncovered
   - 16 functions uncovered
   - **Priority:** Medium (analytics feature)

3. **context/metrics.rs** - 0.00% coverage
   - 90 lines uncovered
   - 7 functions uncovered
   - **Priority:** Low (metrics collection)

4. **engines/config.rs** - 4.92% coverage
   - 52 lines uncovered
   - 8 functions uncovered
   - **Priority:** Medium (engine configuration)

5. **extensions/analytics.rs** - 0.00% coverage
   - 150 lines uncovered
   - 13 functions uncovered
   - **Priority:** Low (extension analytics)

6. **extensions/dependency_graph.rs** - 0.00% coverage
   - 190 lines uncovered
   - 20 functions uncovered
   - **Priority:** Medium (dependency resolution)

7. **extensions/marketplace.rs** - 17.02% coverage
   - 121 lines uncovered
   - 13 functions uncovered
   - **Priority:** Low (marketplace feature)

8. **extensions/publisher.rs** - 9.22% coverage
   - 70 lines uncovered
   - 12 functions uncovered
   - **Priority:** Low (publishing feature)

9. **hooks/config.rs** - 14.14% coverage
   - 67 lines uncovered
   - 12 functions uncovered
   - **Priority:** High (hook configuration)

10. **hooks/error_hooks.rs** - 0.00% coverage
    - 94 lines uncovered
    - 14 functions uncovered
    - **Priority:** High (error handling hooks)

11. **hooks/marketplace.rs** - 0.00% coverage
    - 32 lines uncovered
    - 12 functions uncovered
    - **Priority:** Low (marketplace hooks)

12. **hooks/model.rs** - 0.00% coverage
    - 40 lines uncovered
    - 10 functions uncovered
    - **Priority:** High (model hooks)

13. **hooks/telemetry.rs** - 0.00% coverage
    - 13 lines uncovered
    - 3 functions uncovered
    - **Priority:** Medium (telemetry hooks)

14. **hooks/tool.rs** - 0.00% coverage
    - 52 lines uncovered
    - 12 functions uncovered
    - **Priority:** High (tool hooks)

15. **mcp/integration.rs** - 5.00% coverage
    - 176 lines uncovered
    - 33 functions uncovered
    - **Priority:** Medium (MCP integration)

### Modules with Medium Coverage (70-90%)

These modules need improvement to reach 90%+:

1. **agents/analytics.rs** - 61.06% line coverage
2. **agents/model_selector.rs** - 62.76% line coverage
3. **agents/telemetry.rs** - 45.95% line coverage
4. **agents/validation.rs** - 47.90% line coverage
5. **commands/custom.rs** - 60.53% line coverage
6. **config/cli_config.rs** - 62.72% line coverage
7. **context/sources/braingrid.rs** - 32.14% line coverage
8. **context/sources/jira.rs** - 30.30% line coverage
9. **context/validator.rs** - 38.98% line coverage
10. **engines/providers/claude.rs** - 54.65% line coverage
11. **engines/providers/gemini.rs** - 46.15% line coverage
12. **engines/providers/openai.rs** - 46.67% line coverage
13. **engines/registry.rs** - 56.35% line coverage
14. **extensions/installer.rs** - 59.12% line coverage
15. **extensions/validator.rs** - 57.74% line coverage
16. **hooks/composition.rs** - 63.59% line coverage
17. **hooks/loader.rs** - 55.04% line coverage
18. **hooks/registry.rs** - 70.34% line coverage
19. **hooks/types.rs** - 34.09% line coverage
20. **learning/store.rs** - 59.32% line coverage
21. **mcp/auth.rs** - 52.02% line coverage
22. **mcp/client.rs** - 35.99% line coverage
23. **mcp/content.rs** - 58.90% line coverage
24. **mcp/prompts.rs** - 60.36% line coverage
25. **monitoring/service.rs** - 65.05% line coverage
26. **monitoring/telemetry.rs** - 73.43% line coverage
27. **planning/executor.rs** - 54.42% line coverage
28. **policy/templates.rs** - 69.94% line coverage

### Modules with High Coverage (90%+)

These modules are in good shape but may need edge case coverage:

1. **agents/config.rs** - 87.09% line coverage
2. **agents/discovery.rs** - 73.12% line coverage (needs improvement)
3. **agents/linter.rs** - 87.63% line coverage
4. **agents/metadata.rs** - 80.54% line coverage
5. **agents/persona.rs** - 86.17% line coverage
6. **agents/registry.rs** - 77.58% line coverage (needs improvement)
7. **auth/credentials.rs** - 97.47% line coverage
8. **auth/error.rs** - 100% line coverage
9. **auth/providers.rs** - 100% line coverage
10. **checkpoint/snapshot.rs** - 86.07% line coverage
11. **config/mod.rs** - 100% line coverage
12. **context/files.rs** - 93.62% line coverage
13. **context/history.rs** - 93.55% line coverage
14. **context/injection.rs** - 97.35% line coverage
15. **context/manager.rs** - 93.13% line coverage
16. **context/sources/local.rs** - 89.36% line coverage
17. **context/sources/registry.rs** - 95.24% line coverage
18. **context/templates.rs** - 96.67% line coverage
19. **engines/detection.rs** - 95.52% line coverage
20. **engines/engine_trait.rs** - 100% line coverage
21. **engines/metrics.rs** - 80.58% line coverage
22. **engines/providers/mock.rs** - 95.77% line coverage
23. **error.rs** - 90.48% line coverage
24. **extensions/discovery.rs** - 96.88% line coverage
25. **extensions/manifest.rs** - 97.17% line coverage
26. **extensions/signing.rs** - 77.64% line coverage
27. **extensions/structure.rs** - 68.86% line coverage
28. **extensions/versioning.rs** - 74.74% line coverage
29. **hooks/profiler.rs** - 91.83% line coverage
30. **learning/updates.rs** - 72.73% line coverage
31. **mcp/config.rs** - 86.62% line coverage
32. **mcp/error.rs** - 100% line coverage
33. **mcp/messages.rs** - 97.51% line coverage
34. **mcp/mod.rs** - 98.92% line coverage
35. **mcp/tools.rs** - 80.08% line coverage
36. **mcp/transport/http.rs** - 78.43% line coverage
37. **mcp/transport/sse.rs** - 75.81% line coverage
38. **mcp/transport/stdio.rs** - 81.30% line coverage
39. **memory/adapter.rs** - 100% line coverage
40. **memory/store.rs** - 99.11% line coverage
41. **models/agent.rs** - 99.45% line coverage
42. **models/plan.rs** - 87.43% line coverage
43. **models/proto_convert.rs** - 99.09% line coverage
44. **models/selector.rs** - 68.64% line coverage
45. **models/task.rs** - 95.95% line coverage
46. **models/workflow.rs** - 96.92% line coverage
47. **monitoring/logs.rs** - 98.44% line coverage
48. **monitoring/schema.rs** - 94.05% line coverage
49. **planning/dag.rs** - 89.91% line coverage
50. **planning/generator.rs** - 98.22% line coverage
51. **planning/markdown.rs** - 100% line coverage
52. **planning/parser.rs** - 98.19% line coverage
53. **policy/conflict_resolution.rs** - 74.48% line coverage
54. **policy/constitution.rs** - 90.60% line coverage
55. **policy/rules.rs** - 78.59% line coverage
56. **policy/types.rs** - 100% line coverage
57. **prompts/processing.rs** - 96.93% line coverage
58. **prompts/templates.rs** - 99.20% line coverage
59. **sandbox/config.rs** - 99.50% line coverage
60. **sandbox/docker.rs** - 85.63% line coverage
61. **sandbox/sandbox.rs** - 98.27% line coverage
62. **sandbox/seatbelt.rs** - 79.07% line coverage
63. **storage/database.rs** - 95.77% line coverage
64. **storage/repositories.rs** - 93.42% line coverage
65. **workspace/mod.rs** - 94.89% line coverage
66. **workspace/plan_discovery.rs** - 96.48% line coverage
67. **workspace/requirement_id.rs** - 100% line coverage
68. **workspace/structure.rs** - 97.57% line coverage

## Critical Uncovered Paths

### Error Handling

1. **hooks/error_hooks.rs** - 0% coverage
   - All error hook implementations are untested
   - Critical for error recovery and logging

2. **hooks/tool.rs** - 0% coverage
   - Tool hook implementations untested
   - Critical for tool execution interception

3. **hooks/model.rs** - 0% coverage
   - Model hook implementations untested
   - Critical for model call interception

### Configuration

1. **hooks/config.rs** - 14.14% coverage
   - Hook configuration loading and parsing mostly untested
   - Critical for hook system initialization

2. **engines/config.rs** - 4.92% coverage
   - Engine configuration mostly untested
   - Important for engine setup

### Integration Points

1. **mcp/integration.rs** - 5.00% coverage
   - MCP integration logic mostly untested
   - Important for MCP server integration

2. **mcp/client.rs** - 35.99% coverage
   - MCP client implementation partially tested
   - Important for MCP communication

## Uncovered Function Analysis

### Top Uncovered Functions by Module

**Hooks Module:**
- `hooks/error_hooks.rs`: 14 functions (100% uncovered)
- `hooks/tool.rs`: 12 functions (100% uncovered)
- `hooks/model.rs`: 10 functions (100% uncovered)
- `hooks/config.rs`: 12 functions (80% uncovered)
- `hooks/loader.rs`: 12 functions (33% uncovered)

**MCP Module:**
- `mcp/integration.rs`: 33 functions (94% uncovered)
- `mcp/client.rs`: 20 functions (43% uncovered)
- `mcp/auth.rs`: 25 functions (57% uncovered)

**Analytics Module:**
- `analytics/storage.rs`: 16 functions (100% uncovered)
- `analytics/code_changes.rs`: 4 functions (100% uncovered)

**Extensions Module:**
- `extensions/dependency_graph.rs`: 20 functions (100% uncovered)
- `extensions/analytics.rs`: 13 functions (100% uncovered)
- `extensions/publisher.rs`: 12 functions (86% uncovered)

## State Transitions

Based on the codebase structure, the following state machines need comprehensive testing:

1. **Agent States** - Covered in `agents/registry.rs` (77.58% coverage)
2. **Workflow States** - Covered in `models/workflow.rs` (96.92% coverage)
3. **Task States** - Covered in `models/task.rs` (95.95% coverage)
4. **Hook Execution States** - Partially covered in `hooks/registry.rs` (70.34% coverage)

## Recommendations

### Phase 1: Critical Path Coverage (Target: 90%+)

1. **Hooks Module** (Current: ~50% average)
   - Priority: HIGH
   - Focus: error_hooks.rs, tool.rs, model.rs, config.rs
   - Estimated effort: High

2. **Error Handlers** (Current: 90.48% in error.rs)
   - Priority: HIGH
   - Focus: Module-specific error types
   - Estimated effort: Medium

3. **Agents Module** (Current: ~75% average)
   - Priority: HIGH
   - Focus: validation.rs, telemetry.rs, model_selector.rs
   - Estimated effort: Medium

4. **Monitoring Module** (Current: ~70% average)
   - Priority: MEDIUM
   - Focus: service.rs, telemetry.rs
   - Estimated effort: Medium

### Phase 2: Error Scenario Coverage (Target: 95%+)

1. Database error scenarios
2. File system error scenarios
3. Network error scenarios (MCP, HTTP)
4. Concurrent access scenarios

### Phase 3: Edge Cases & Boundaries (Target: 100%)

1. Empty inputs and null values
2. Maximum size limits
3. All state transitions
4. All conditional branches
5. All enum variants

## Coverage Report Locations

- **HTML Report:** `coverage-report/html/index.html`
- **JSON Data:** `coverage-report/coverage.json`
- **This Analysis:** `docs/testing/coverage-analysis-REQ-172.md`

## Next Steps

1. Review this analysis with the team
2. Prioritize modules based on criticality and current coverage
3. Create detailed testing backlog (Task 2)
4. Begin implementing tests for highest priority modules

