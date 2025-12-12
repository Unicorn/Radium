# Execution Modes

Radium supports two execution modes for plan execution: **Bounded** and **Continuous**. Choose the mode that best fits your workflow.

## Bounded Mode

Bounded mode executes a fixed number of iterations before stopping. This is the default mode and is ideal for:
- Incremental development
- Testing individual iterations
- Controlled execution with limits

### Usage

```bash
# Execute up to 5 iterations (default)
rad craft

# Execute up to 3 iterations
rad craft --iterations 3
```

### Behavior

- Executes iterations sequentially
- Stops after reaching the iteration limit
- Saves progress after each iteration
- Can be resumed later with `rad craft --resume`

### Example

```bash
# Plan has 10 iterations
# Execute first 5 iterations
rad craft

# Output:
# → Reached maximum iterations (5). Stopping execution.
# Progress saved to plan_manifest.json

# Later, resume from iteration 6
rad craft --resume
```

## Continuous Mode

Continuous mode (also known as "YOLO mode") executes all iterations until the plan is complete. This mode includes a safety limit to prevent infinite loops.

### Usage

```bash
# Execute all iterations until complete
rad craft --yolo
```

### Behavior

- Executes all iterations until plan is complete
- Includes sanity limit (1000 iterations) to prevent infinite loops
- Automatically stops when all tasks are complete
- Saves progress after each iteration

### Example

```bash
# Execute entire plan
rad craft --yolo

# Output:
# → Executing iteration 1...
# → Executing iteration 2...
# ...
# → All tasks completed. Execution finished.
```

### Safety Limit

Continuous mode includes a safety limit of 1000 iterations to prevent infinite execution:

```bash
# If plan has more than 1000 iterations
rad craft --yolo

# Output:
# → Reached sanity limit (1000). Stopping execution.
```

## Graceful Shutdown

Both modes support graceful shutdown via SIGINT (Ctrl+C):

```bash
# Start execution
rad craft

# Press Ctrl+C to abort
# Output:
# Execution aborted by user. Progress saved to plan_manifest.json
```

When aborted:
- Current progress is saved
- Execution stops immediately
- You can resume later with `rad craft --resume`

## Progress Tracking

Both modes display real-time progress:

```
[Iteration 1] Progress: 2/5 tasks | Current: I1.T2 | Elapsed: 0:05:23
  • Progress: 40%
```

Progress includes:
- Current iteration number
- Completed vs total tasks
- Currently executing task
- Elapsed time

## Choosing a Mode

### Use Bounded Mode When:
- You want to review progress incrementally
- Testing specific iterations
- Working on large plans that need staged execution
- You want explicit control over execution limits

### Use Continuous Mode When:
- You want to execute the entire plan automatically
- Plan is well-tested and ready for full execution
- You're confident the plan will complete successfully
- You want hands-off execution

## Resuming Execution

Both modes support resuming from the last checkpoint:

```bash
# Resume from last checkpoint
rad craft --resume
```

When resuming:
- Completed tasks are skipped
- Execution continues from the first incomplete task
- Progress is preserved from previous execution

## Configuration

Execution mode is determined by the `--yolo` flag:

```bash
# Bounded mode (default, 5 iterations)
rad craft

# Continuous mode
rad craft --yolo
```

## Best Practices

1. **Start with Bounded Mode**: Use bounded mode for initial testing
2. **Use Continuous for Production**: Switch to continuous mode once plan is validated
3. **Monitor Progress**: Watch progress output to catch issues early
4. **Use Graceful Shutdown**: Press Ctrl+C if you need to stop execution
5. **Resume When Needed**: Use `--resume` to continue interrupted execution

## Troubleshooting

### Execution Stops Unexpectedly

- Check if you reached the iteration limit (bounded mode)
- Verify sanity limit wasn't reached (continuous mode)
- Check for fatal errors in execution logs

### Progress Not Saving

- Ensure you have write permissions in the workspace
- Check that `.radium/plan/` directory exists
- Verify `plan_manifest.json` is being updated

### Cannot Resume

- Ensure `plan_manifest.json` exists from previous execution
- Check that the plan structure hasn't changed
- Verify you're in the correct workspace directory

## See Also

- [Autonomous Planning](./autonomous-planning.md) - Creating plans
- [Error Handling](./error-handling.md) - Handling execution errors
- [Monitoring Integration](./monitoring-integration.md) - Tracking execution

