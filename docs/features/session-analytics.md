# Session Analytics

Session Analytics provides comprehensive tracking and reporting for Radium agent sessions, enabling users to monitor costs, analyze performance, and optimize their workflows.

## Overview

Session Analytics automatically tracks every agent session, collecting detailed metrics about:
- Token usage and costs per model
- Tool execution and success rates
- Performance metrics (wall time, agent active time, API time)
- Code changes (lines added/removed)
- Cache effectiveness

All session data is stored locally in `.radium/_internals/sessions/` as JSON files, providing a complete history of your agent interactions.

## Key Features

### Session Tracking

Every agent session is automatically assigned a unique session ID and tracked from start to finish. Session data includes:
- Start and end timestamps
- Total duration (wall time)
- Agent active time (time agents were actually running)
- Breakdown of API time vs tool execution time

### Cost Transparency

Track token usage and estimated costs per model:
- Input and output tokens per model
- Cached token usage (showing cache savings)
- Estimated cost calculations
- Aggregated costs across all sessions

### Performance Metrics

Understand where time is spent:
- Wall time (total session duration)
- Agent active time (time agents were running)
- API time (time spent in API calls)
- Tool time (time spent executing tools)
- Success rate for tool calls

### Code Change Tracking

Automatically tracks code changes via git diff:
- Lines added
- Lines removed
- Files changed

Requires a git repository in the workspace.

### Cache Optimization Metrics

Monitor cache effectiveness:
- Cache hit rate
- Total cached tokens
- Cache creation vs read tokens
- Cost savings from cache usage

## Storage Location

Session reports are stored in:
```
.radium/_internals/sessions/<session-id>.json
```

Each session is saved as a JSON file containing complete metrics. By default, reports are stored in pretty-printed format for readability. You can enable compact JSON format using the `RADIUM_COMPACT_SESSION_JSON` environment variable:

```bash
export RADIUM_COMPACT_SESSION_JSON=true
```

## CLI Commands

### View Current Session

Show statistics for the current or most recent session:

```bash
rad stats session
rad stats session --session-id <session-id>
rad stats session --json  # Output as JSON
```

**Example Output:**
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

Model Usage                  Reqs   Input Tokens  Output Tokens
───────────────────────────────────────────────────────────────
gemini-2.5-flash-lite          28         60,389          2,422
gemini-3-pro-preview          168     31,056,954         44,268
```

### Model Usage Breakdown

View detailed model usage statistics:

```bash
rad stats model                    # Aggregated across all sessions
rad stats model --session-id <id>  # For specific session
rad stats model --json             # JSON output
```

**Example Output:**
```
Aggregated Model Usage (All Sessions)

Model                          Requests  Input Tokens  Output Tokens  Cached Tokens  Cost
────────────────────────────────────────────────────────────────────────────────────────────
gemini-3-pro-preview                168     31,056,954         44,268         12,000  $0.1250
gemini-2.5-flash-lite                28         60,389          2,422          5,000  $0.0025
────────────────────────────────────────────────────────────────────────────────────────────
TOTAL                               196     31,117,343         46,690         17,000  $0.1275
```

### Engine Usage Breakdown

View engine-specific performance metrics:

```bash
rad stats engine --session-id <id>
rad stats engine --json
```

### Session History

View historical session summaries:

```bash
rad stats history              # Last 10 sessions (default)
rad stats history --limit 20   # Last 20 sessions (max 100)
rad stats history --json       # JSON output
```

**Example Output:**
```
Recent Session Summaries

Session ID                    Duration            Tool Calls      Success Rate    Cost
─────────────────────────────────────────────────────────────────────────────────────
3c6ddcd3-85b6-48f1-88e1...   4h 9m               231             92.6%          $0.1250
a1b2c3d4-e5f6-7890-abcd...   2h 15m              145             95.2%          $0.0850
```

### Compare Sessions

Compare two sessions to identify improvements or regressions:

```bash
rad stats compare <session-id-1> <session-id-2>
rad stats compare <session-id-1> <session-id-2> --json
```

**Example Output:**
```
Session Comparison
═══════════════════

Session A: 3c6ddcd3-85b6-48f1-88e1-f428ca458337
Session B: a1b2c3d4-e5f6-7890-abcd-ef1234567890

Token Usage
───────────
  Session A: 50000 input, 2000 output (total: 52000)
  Session B: 45000 input, 1800 output (total: 46800)
  Delta: -5200 (-10.0%)

Cost
────
  Session A: $0.1250
  Session B: $0.1100
  Delta: -0.0150 (-12.0%)

Performance
───────────
  Wall Time: -1h 54m (-46.2%)
  Agent Active: -1h 20m (-44.4%)

Tool Calls
──────────
  Session A: 231
  Session B: 145
  Delta: -86 (-37.2%)
  Success Rate: 92.6% → 95.2% (+2.6%)

Code Changes
────────────
  Session A: +505 -208
  Session B: +320 -150
  Delta: -185 / -58
```

### Export Analytics

Export session data to JSON:

```bash
rad stats export                    # Export all sessions to stdout
rad stats export --output data.json # Export to file
rad stats export --session-id <id> # Export specific session
```

## Troubleshooting

### No Sessions Found

If you see "No session history found" or "No sessions found":
1. Ensure you're in a Radium workspace (run `rad init` if needed)
2. Verify that agent sessions have been executed
3. Check that `.radium/_internals/sessions/` directory exists

### Corrupted Session Files

If session files are corrupted:
1. Check the logs for warnings about corrupted files
2. Corrupted files are automatically skipped during listing
3. You can manually delete corrupted `.json` files from the sessions directory
4. The system will continue to function with remaining valid sessions

### Missing Code Change Tracking

If code changes show as 0:
1. Ensure your workspace is a git repository
2. Run `git init` if needed
3. Code changes are calculated using `git diff`, so uncommitted changes are tracked

### Performance Issues with Large History

If `rad stats history` is slow:
1. Use the `--limit` flag to limit results (default: 10, max: 100)
2. The system uses pagination to efficiently load only requested sessions
3. For very large histories (1000+ sessions), consider using `rad stats export` and filtering externally

## Related Documentation

- [Optimizing Costs](optimizing-costs.md) - Strategies for reducing session costs
- [Monitoring & Telemetry](../development/agent-instructions.md) - Underlying telemetry system

