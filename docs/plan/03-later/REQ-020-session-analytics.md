---
req_id: REQ-020
title: Session Analytics
phase: LATER
status: Completed
priority: Low
estimated_effort: TBD
dependencies: [REQ-007]
related_docs:
  - docs/features/future-enhancements.md#session-reports--analytics
  - docs/project/PROGRESS.md#session-reports--analytics-system
---

# Session Analytics

## Problem Statement

Users need visibility into session performance, costs, and optimization opportunities. Without session analytics, users cannot:
- Track token usage and costs per session
- Analyze performance metrics
- Identify optimization opportunities
- Understand cache effectiveness
- Monitor tool success rates

Modern AI tools (like gemini-cli) provide comprehensive session reporting. Radium needs an equivalent system that provides detailed analytics and insights.

## Solution Overview

Implement a comprehensive session analytics system that provides:
- Session tracking with unique IDs
- Tool metrics (success rate, calls, user approval)
- Code change tracking via git diff
- Performance breakdown (wall time, agent active time, API/tool time)
- Model usage per-model (tokens, requests, costs)
- Cache optimization metrics
- Cost transparency
- CLI commands for viewing analytics

The session analytics system enables users to track costs, optimize performance, and gain insights into agent execution patterns.

## Functional Requirements

### FR-1: Session Tracking

**Description**: Track sessions with unique IDs and metadata.

**Acceptance Criteria**:
- [x] Session ID generation
- [x] Session start/end tracking
- [x] Session persistence in `.radium/_internals/sessions/`
- [x] Session metadata storage
- [x] Session retrieval by ID

**Implementation**: 
- `crates/radium-core/src/analytics/session.rs`
- `crates/radium-core/src/analytics/storage.rs`

### FR-2: Tool Metrics

**Description**: Track tool execution metrics.

**Acceptance Criteria**:
- [x] Tool call counting
- [x] Success rate calculation
- [x] Failure tracking
- [x] User approval metrics
- [x] Tool-specific metrics

**Implementation**: `crates/radium-core/src/analytics/report.rs`

### FR-3: Code Change Tracking

**Description**: Track code changes via git diff.

**Acceptance Criteria**:
- [x] Git diff calculation
- [x] Lines added/removed tracking
- [x] File change tracking
- [x] Change summary generation

**Implementation**: `crates/radium-core/src/analytics/code_changes.rs`

### FR-4: Performance Metrics

**Description**: Track performance breakdown.

**Acceptance Criteria**:
- [x] Wall time tracking
- [x] Agent active time tracking
- [x] API time vs tool time breakdown
- [x] Performance percentage calculations

**Implementation**: `crates/radium-core/src/analytics/report.rs`

### FR-5: Model Usage Tracking

**Description**: Track model usage per-model.

**Acceptance Criteria**:
- [x] Token counting per model
- [x] Request counting per model
- [x] Cost calculation per model
- [x] Model usage aggregation

**Implementation**: `crates/radium-core/src/analytics/report.rs`

### FR-6: Cache Optimization Metrics

**Description**: Track cache effectiveness.

**Acceptance Criteria**:
- [x] Cache hit/miss tracking
- [x] Token reuse calculation
- [x] Cache savings highlighting
- [x] Cache optimization insights

**Implementation**: `crates/radium-core/src/analytics/report.rs`

### FR-7: CLI Commands

**Description**: CLI commands for viewing analytics.

**Acceptance Criteria**:
- [x] `rad stats session` - Current session stats
- [x] `rad stats model` - Detailed model usage breakdown
- [x] `rad stats history` - Historical session summaries
- [x] `rad stats export` - Export analytics to JSON

**Implementation**: `apps/cli/src/commands/stats.rs`

## Technical Requirements

### TR-1: Session Data Structure

**Description**: Data structure for session tracking.

**Data Models**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub agent_active_time: Duration,
    pub api_time: Duration,
    pub tool_time: Duration,
    pub tool_calls: u32,
    pub tool_successes: u32,
    pub tool_failures: u32,
    pub code_changes: CodeChanges,
    pub model_usage: Vec<ModelUsage>,
    pub cache_stats: CacheStats,
}
```

### TR-2: Analytics API

**Description**: APIs for session analytics.

**APIs**:
```rust
pub struct AnalyticsService {
    storage: AnalyticsStorage,
}

impl AnalyticsService {
    pub fn start_session(&self) -> Result<String>;
    pub fn end_session(&self, session_id: &str) -> Result<SessionReport>;
    pub fn get_session_report(&self, session_id: &str) -> Result<SessionReport>;
    pub fn get_model_stats(&self, model: &str) -> Result<ModelStats>;
    pub fn get_history(&self) -> Result<Vec<SessionSummary>>;
}
```

## User Experience

### UX-1: Session Summary

**Description**: Users see session summary at end of session.

**Example**:
```
Interaction Summary
Session ID:                 3c6ddcd3-85b6-48f1-88e1-f428ca458337
Tool Calls:                 231 ( ✓ 214 x 17 )
Success Rate:               92.6%
Code Changes:               +505 -208

Performance
Wall Time:                  4h 9m 54s
Agent Active:               2h 53m 17s
  » API Time:               1h 9m 42s (40.2%)
  » Tool Time:              1h 43m 35s (59.8%)
```

### UX-2: Model Usage

**Description**: Users view detailed model usage.

**Example**:
```bash
$ rad stats model
Model Usage                  Reqs   Input Tokens  Output Tokens
───────────────────────────────────────────────────────────────
gemini-2.5-flash-lite          28         60,389          2,422
gemini-3-pro-preview          168     31,056,954         44,268
```

## Data Requirements

### DR-1: Session Storage

**Description**: JSON files storing session data.

**Location**: `.radium/_internals/sessions/<session-id>.json`

**Format**: JSON with session metadata and metrics

## Dependencies

- **REQ-007**: Monitoring & Telemetry - Required for telemetry tracking

## Success Criteria

1. [x] Sessions are tracked with unique IDs
2. [x] Tool metrics are calculated correctly
3. [x] Code changes are tracked via git diff
4. [x] Performance metrics are accurate
5. [x] Model usage is tracked per-model
6. [x] Cache optimization metrics are calculated
7. [x] CLI commands provide comprehensive analytics
8. [x] All analytics operations have comprehensive test coverage

**Completion Metrics**:
- **Status**: ✅ Complete
- **Implementation**: Comprehensive session analytics system
- **Files**: 
  - `crates/radium-core/src/analytics/` (session, report, storage, code_changes)
  - `apps/cli/src/commands/stats.rs`

## Out of Scope

- Advanced analytics visualization (future enhancement)
- Predictive analytics (future enhancement)
- Real-time analytics dashboard (future enhancement)

## References

- [Future Enhancements](../features/future-enhancements.md#session-reports--analytics)
- [Progress Documentation](../project/PROGRESS.md#session-reports--analytics-system)
- [Analytics Implementation](../../crates/radium-core/src/analytics/)
- [Stats Command Implementation](../../apps/cli/src/commands/stats.rs)

