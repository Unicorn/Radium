---
id: "error-handling"
title: "Error Handling and Retry Logic"
sidebar_label: "Error Handling and Retry Logic"
---

# Error Handling and Retry Logic

Radium's plan execution includes intelligent error handling with automatic retries for recoverable errors and immediate failure for fatal errors.

## Error Categories

Errors are automatically categorized as either **Recoverable** or **Fatal**:

### Recoverable Errors

These errors are automatically retried with exponential backoff:

- **Rate Limits** (429): API rate limit exceeded
- **Network Errors**: Connection timeouts, network failures
- **Server Errors** (5xx): Temporary server issues
- **File Locks**: Temporary file locking issues
- **Timeouts**: Request timeouts

### Fatal Errors

These errors fail immediately without retries:

- **Authentication Errors** (401, 403): Invalid credentials, unauthorized access
- **Configuration Errors**: Missing config files, invalid settings
- **Dependency Errors**: Missing dependencies, unmet requirements
- **Not Found Errors** (404): Resources that don't exist

## Retry Logic

### Exponential Backoff

Recoverable errors are retried with exponential backoff delays:

```
Attempt 1: Immediate
Attempt 2: base_delay_ms * 2^0 = 100ms
Attempt 3: base_delay_ms * 2^1 = 200ms
Attempt 4: base_delay_ms * 2^2 = 400ms
...
```

Default base delay is 100ms, configurable per execution.

### Retry Limits

By default, tasks are retried up to 3 times (initial attempt + 2 retries):

```bash
# Task execution with retry
rad craft

# If task fails with recoverable error:
# → Attempt 1: Failed (rate limit)
# → Waiting 100ms...
# → Attempt 2: Failed (rate limit)
# → Waiting 200ms...
# → Attempt 3: Success
```

## Error Handling Examples

### Recoverable Error: Rate Limit

```bash
# Task execution hits rate limit
rad craft

# Output:
# → Executing I1.T1...
# → Error: 429 rate limit exceeded
# → Retrying in 100ms...
# → Retry 1: Success
```

### Fatal Error: Authentication

```bash
# Task execution fails with auth error
rad craft

# Output:
# → Executing I1.T1...
# → Error: 401 unauthorized
# → Fatal error: Authentication failed
# → Execution stopped
```

### Retry Exhausted

```bash
# All retries exhausted
rad craft

# Output:
# → Executing I1.T1...
# → Error: 429 rate limit exceeded
# → Retrying in 100ms...
# → Retry 1: Failed (rate limit)
# → Retrying in 200ms...
# → Retry 2: Failed (rate limit)
# → Error: Task execution failed after retries
```

## Error Messages

Error messages are designed to be actionable:

### Rate Limit Error

```
Error: 429 rate limit exceeded
Suggestion: Wait a few moments and retry, or upgrade your API plan
```

### Authentication Error

```
Error: 401 unauthorized
Suggestion: Run 'rad auth login' to authenticate
```

### Network Error

```
Error: network connection failed
Suggestion: Check your internet connection and retry
```

## Checkpoint Recovery

After errors, execution state is saved to allow recovery:

```bash
# Execution fails partway through
rad craft

# Output:
# → Executing I1.T3...
# → Error: network connection failed
# → Progress saved to plan_manifest.json

# Resume from checkpoint
rad craft --resume

# Output:
# → Resuming from checkpoint...
# → Skipping completed tasks: I1.T1, I1.T2
# → Executing I1.T3...
```

## Error Handling Configuration

Error handling is configured automatically, but you can influence behavior:

### Model Selection

Some models have better error handling:

```bash
# Use model with better retry support
rad craft --engine claude --model claude-3-sonnet
```

### Execution Mode

Bounded mode allows easier error recovery:

```bash
# Execute in bounded mode for easier error handling
rad craft  # Stops after 5 iterations, easier to debug
```

## Best Practices

1. **Monitor Errors**: Watch for error patterns in execution logs
2. **Use Retries**: Let automatic retries handle transient errors
3. **Fix Fatal Errors**: Address fatal errors immediately (auth, config)
4. **Resume After Errors**: Use `--resume` to continue after fixing errors
5. **Check Logs**: Review execution logs for detailed error information

## Troubleshooting

### Too Many Retries

- Check if errors are actually recoverable
- Verify network connectivity
- Check API rate limits and quotas

### Immediate Failures

- Verify authentication credentials
- Check configuration files
- Ensure all dependencies are met

### Retry Not Working

- Verify error is categorized as recoverable
- Check retry limits haven't been exceeded
- Review exponential backoff delays

## Error Categories Reference

### Recoverable Patterns

- `429` - Rate limit
- `timeout` - Request timeout
- `network` - Network issues
- `connection` - Connection failures
- `5` - Server errors (500, 502, 503, etc.)
- `server error` - Generic server errors
- `file lock` - File locking issues
- `temporary` - Temporary errors

### Fatal Patterns

- `401` - Unauthorized
- `403` - Forbidden
- `unauthorized` - Auth failures
- `forbidden` - Access denied
- `missing` - Missing resources
- `invalid` - Invalid input
- `not found` - Resource not found
- `dependency not met` - Unmet dependencies

## See Also

- [Execution Modes](./execution-modes.md) - Execution configuration
- [Monitoring Integration](./monitoring-integration.md) - Error tracking
- [Best Practices](./best-practices.md) - Error prevention

