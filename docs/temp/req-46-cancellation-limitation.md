# Cancellation Support Limitation

## Current Implementation

A basic cancellation mechanism has been added to the TUI:
- Users can press `Ctrl+C` during orchestration to request cancellation
- A cancellation flag is set and a message is displayed
- However, the current orchestration operation will continue until completion or timeout

## Technical Limitation

The orchestration engine uses blocking async calls (`await`) which means:
- Once orchestration starts, it cannot be immediately cancelled
- The operation will complete or timeout (120s protection exists)
- The cancellation flag provides user feedback but doesn't stop execution

## Future Enhancement

For proper cancellation support, the following changes would be needed:
1. Add cancellation token support to `OrchestrationEngine`
2. Use `tokio::select!` or similar to check cancellation during execution
3. Propagate cancellation through tool execution loops
4. Handle graceful shutdown of in-progress tool calls

## Current Protection

- **Timeout Protection**: 120 second timeout prevents indefinite hanging
- **Max Iterations**: 5 iteration limit prevents infinite loops
- **User Feedback**: Cancellation request is acknowledged to user

## Recommendation

For REQ-46 completion:
- Basic cancellation mechanism is implemented (user can request cancellation)
- Full cancellation support is deferred to future enhancement
- Timeout protection provides adequate protection for most use cases

