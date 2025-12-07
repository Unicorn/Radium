# Retry Hook Example

This is an example hook implementation that demonstrates automatic error recovery with retry logic and exponential backoff in Radium.

## Overview

The retry hook automatically retries failed operations for retryable errors (network errors, timeouts, etc.) with exponential backoff. It demonstrates:

- Error type detection
- Retry count tracking
- Exponential backoff calculation
- Automatic recovery

## Features

- Detects retryable errors (NetworkError, TimeoutError, etc.)
- Implements exponential backoff
- Tracks retry counts per error
- Configurable max retries and delays
- Automatic retry on recoverable errors

## Usage

### Building

```bash
cd examples/hooks/retry-hook
cargo build --release
```

### Integration

To use this hook in your Radium workspace:

1. Register the hook programmatically
2. Configure it in `.radium/hooks.toml`
3. The hook will automatically retry recoverable errors

### Example Configuration

```toml
[[hooks]]
name = "retry-hook"
type = "error_recovery"
priority = 150
enabled = true
```

## Implementation Details

The hook implements the `ErrorHook` trait and focuses on the `error_recovery` method. It uses exponential backoff to avoid overwhelming the system with retries.

## Retryable Errors

The hook automatically retries:
- NetworkError
- TimeoutError
- RateLimitError
- TemporaryError
- Errors containing "timeout", "network", or "temporary" in the message

## Configuration

Default configuration:
- Max retries: 3
- Initial delay: 100ms
- Max delay: 5000ms
- Backoff multiplier: 2.0

## Customization

You can customize the hook by:

- Adjusting max retries
- Modifying backoff parameters
- Adding custom retryable error types
- Integrating with external retry systems

