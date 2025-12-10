// Tool execution module with parallel execution strategies
//
// This module provides configurable tool execution strategies including
// concurrent, sequential, and batched concurrent execution.

use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{timeout, Duration};

use super::{
    tool::{Tool, ToolArguments, ToolCall, ToolResult},
    FunctionExecutionStrategy, ToolExecutionConfig, ToolErrorHandling,
};
use crate::error::{OrchestrationError, Result};
use tokio::time::sleep;
use tracing::{debug, warn};

/// Execute tool calls according to the configured execution strategy.
///
/// This function coordinates the execution of multiple tool calls using
/// the strategy specified in the configuration. It handles timeouts,
/// parallel execution limits, error handling, and collects all results.
///
/// # Arguments
/// * `calls` - The tool calls to execute
/// * `tools` - Available tools (used to find the handler for each call)
/// * `config` - Execution configuration including strategy and limits
///
/// # Returns
/// Vector of results, one per tool call (in order of calls)
pub async fn execute_tool_calls(
    calls: &[ToolCall],
    tools: &[Tool],
    config: &ToolExecutionConfig,
) -> Vec<Result<ToolResult>> {
    if calls.is_empty() {
        return vec![];
    }

    // Execute according to strategy
    let mut execution_results = match config.execution_strategy {
        FunctionExecutionStrategy::Concurrent => {
            execute_concurrent(calls, tools, config).await
        }
        FunctionExecutionStrategy::Sequential => {
            execute_sequential(calls, tools, config).await
        }
        FunctionExecutionStrategy::ConcurrentBatched { batch_size } => {
            execute_batched(calls, tools, batch_size, config).await
        }
    };

    // Apply error handling strategy
    for (i, result) in execution_results.iter_mut().enumerate() {
        if result.is_err() {
            let call = &calls[i];
            let tool = tools.iter().find(|t| t.name == call.name);

            if let Some(tool_ref) = tool {
                // Take ownership of the error using std::mem::replace
                let error = std::mem::replace(result, Ok(ToolResult::success("temporary".to_string())));
                let error = error.unwrap_err();

                match handle_tool_error(call, error, &config.error_handling, tool_ref).await {
                    Ok(handled_result) => {
                        *result = Ok(handled_result);
                    }
                    Err(e) => {
                        // Error handling strategy wants to propagate error
                        *result = Err(e);
                    }
                }
            }
        }
    }

    execution_results
}

/// Handle tool execution error according to the configured strategy.
async fn handle_tool_error(
    call: &ToolCall,
    error: OrchestrationError,
    strategy: &ToolErrorHandling,
    tool: &Tool,
) -> Result<ToolResult> {
    match strategy {
        ToolErrorHandling::ReturnToModel => {
            Ok(ToolResult::error_for_model(format!("Error: {}", error)))
        }
        ToolErrorHandling::RetryWithBackoff { max_retries, initial_delay } => {
            retry_with_backoff(call, tool, *max_retries, *initial_delay).await
        }
        ToolErrorHandling::FailFast => {
            Err(error)
        }
        ToolErrorHandling::SkipAndContinue => {
            Ok(ToolResult::error_for_model("Skipped due to error".to_string()))
        }
    }
}

/// Retry tool execution with exponential backoff.
async fn retry_with_backoff(
    call: &ToolCall,
    tool: &Tool,
    max_retries: usize,
    initial_delay: Duration,
) -> Result<ToolResult> {
    let args = ToolArguments::new(call.arguments.clone());
    
    for attempt in 0..=max_retries {
        match tool.execute(&args).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt < max_retries {
                    let delay = initial_delay * 2_u32.pow(attempt as u32);
                    debug!(
                        "Tool '{}' failed (attempt {}/{}), retrying after {:?}",
                        call.name,
                        attempt + 1,
                        max_retries,
                        delay
                    );
                    sleep(delay).await;
                } else {
                    // Final attempt failed
                    return Err(OrchestrationError::ToolExecutionFailed(format!(
                        "Tool '{}' failed after {} retries: {}",
                        call.name, max_retries, e
                    )));
                }
            }
        }
    }
    
    Err(OrchestrationError::Other("Retry logic error".to_string()))
}

/// Execute tool calls concurrently with max_parallel limit enforcement.
async fn execute_concurrent(
    calls: &[ToolCall],
    tools: &[Tool],
    config: &ToolExecutionConfig,
) -> Vec<Result<ToolResult>> {
    let max_parallel = config.max_parallel.unwrap_or(calls.len());
    let semaphore = Arc::new(Semaphore::new(max_parallel));
    
    // Create a shared vector to store results in order
    use std::sync::Mutex;
    let mut init_results = Vec::with_capacity(calls.len());
    for _ in 0..calls.len() {
        init_results.push(None::<Result<ToolResult>>);
    }
    let results = Arc::new(Mutex::new(init_results));
    
    let mut tasks = Vec::new();
    
    for (index, call) in calls.iter().enumerate() {
        let call = call.clone();
        let tools = tools.to_vec();
        let semaphore = Arc::clone(&semaphore);
        let results = Arc::clone(&results);
        let timeout_duration = config.timeout_per_call;
        let index = index; // Move index into closure
        
        let task = tokio::spawn(async move {
            // Acquire permit for parallel execution limit
            let _permit = match semaphore.acquire().await {
                Ok(p) => p,
                Err(e) => {
                    let mut res = results.lock().unwrap();
                    res[index] = Some(Err(OrchestrationError::Other(format!(
                        "Failed to acquire semaphore: {}",
                        e
                    ))));
                    return;
                }
            };
            
            // Find tool handler
            let tool = match tools.iter().find(|t| t.name == call.name) {
                Some(t) => t,
                None => {
                    let mut res = results.lock().unwrap();
                    res[index] = Some(Err(OrchestrationError::Other(format!(
                        "Tool '{}' not found",
                        call.name
                    ))));
                    return;
                }
            };
            
            // Execute with timeout
            let args = ToolArguments::new(call.arguments);
            let result = match timeout(timeout_duration, tool.execute(&args)).await {
                Ok(Ok(result)) => Ok(result),
                Ok(Err(e)) => Err(OrchestrationError::ToolExecutionFailed(format!(
                    "Tool '{}' execution failed: {}",
                    call.name, e
                ))),
                Err(_) => Err(OrchestrationError::Other(format!(
                    "Tool '{}' execution timed out after {:?}",
                    call.name, timeout_duration
                ))),
            };
            
            let mut res = results.lock().unwrap();
            res[index] = Some(result);
        });
        
        tasks.push(task);
    }
    
    // Wait for all tasks to complete
    for task in tasks {
        if let Err(e) = task.await {
            warn!("Task join error: {}", e);
        }
    }
    
    // Extract results in order
    let results = Arc::try_unwrap(results).unwrap().into_inner().unwrap();
    results
        .into_iter()
        .map(|opt| {
            opt.unwrap_or_else(|| {
                Err(OrchestrationError::Other(
                    "Tool call did not complete".to_string(),
                ))
            })
        })
        .collect()
}

/// Execute tool calls sequentially (one after another).
async fn execute_sequential(
    calls: &[ToolCall],
    tools: &[Tool],
    config: &ToolExecutionConfig,
) -> Vec<Result<ToolResult>> {
    let mut results = Vec::new();
    
    for call in calls {
        // Find tool handler
        let tool = tools.iter().find(|t| t.name == call.name);

        let result = if let Some(tool) = tool {
            // Execute with timeout
            let args = ToolArguments::new(call.arguments.clone());
            match timeout(config.timeout_per_call, tool.execute(&args)).await {
                Ok(Ok(result)) => Ok(result),
                Ok(Err(e)) => Err(OrchestrationError::ToolExecutionFailed(format!(
                    "Tool '{}' execution failed: {}",
                    call.name, e
                ))),
                Err(_) => Err(OrchestrationError::Other(format!(
                    "Tool '{}' execution timed out after {:?}",
                    call.name, config.timeout_per_call
                ))),
            }
        } else {
            Err(OrchestrationError::Other(format!("Tool '{}' not found", call.name)))
        };

        results.push(result);
    }
    
    results
}

/// Execute tool calls in batches with concurrent execution within each batch.
async fn execute_batched(
    calls: &[ToolCall],
    tools: &[Tool],
    batch_size: usize,
    config: &ToolExecutionConfig,
) -> Vec<Result<ToolResult>> {
    let mut all_results = Vec::new();
    
    // Process calls in batches
    for batch in calls.chunks(batch_size) {
        // Execute batch concurrently (with max_parallel limit)
        let batch_config = ToolExecutionConfig {
            execution_strategy: FunctionExecutionStrategy::Concurrent,
            max_parallel: config.max_parallel,
            ..config.clone()
        };
        
        let batch_results = execute_concurrent(batch, tools, &batch_config).await;
        all_results.extend(batch_results);
    }
    
    all_results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestration::tool::{ToolParameters, ToolHandler, ToolArguments};
    use async_trait::async_trait;
    use std::sync::Arc;
    use std::time::Instant;

    struct DelayedToolHandler {
        delay_ms: u64,
    }

    #[async_trait]
    impl ToolHandler for DelayedToolHandler {
        async fn execute(&self, _args: &ToolArguments) -> Result<ToolResult> {
            tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;
            Ok(ToolResult::success("ok"))
        }
    }

    #[tokio::test]
    async fn test_execute_sequential() {
        let tools = vec![
            Tool::new(
                "tool1",
                "tool1",
                "Tool 1",
                ToolParameters::new(),
                Arc::new(DelayedToolHandler { delay_ms: 50 }),
            ),
            Tool::new(
                "tool2",
                "tool2",
                "Tool 2",
                ToolParameters::new(),
                Arc::new(DelayedToolHandler { delay_ms: 50 }),
            ),
        ];

        let calls = vec![
            ToolCall {
                id: "call1".to_string(),
                name: "tool1".to_string(),
                arguments: serde_json::json!({}),
            },
            ToolCall {
                id: "call2".to_string(),
                name: "tool2".to_string(),
                arguments: serde_json::json!({}),
            },
        ];

        let config = ToolExecutionConfig {
            execution_strategy: FunctionExecutionStrategy::Sequential,
            timeout_per_call: Duration::from_secs(5),
            max_parallel: None,
            ..Default::default()
        };

        let start = Instant::now();
        let results = execute_tool_calls(&calls, &tools, &config).await;
        let elapsed = start.elapsed();

        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        
        // Sequential should take at least 100ms (50ms * 2)
        assert!(elapsed.as_millis() >= 100);
    }

    #[tokio::test]
    async fn test_execute_concurrent() {
        let tools = vec![
            Tool::new(
                "tool1",
                "tool1",
                "Tool 1",
                ToolParameters::new(),
                Arc::new(DelayedToolHandler { delay_ms: 50 }),
            ),
            Tool::new(
                "tool2",
                "tool2",
                "Tool 2",
                ToolParameters::new(),
                Arc::new(DelayedToolHandler { delay_ms: 50 }),
            ),
        ];

        let calls = vec![
            ToolCall {
                id: "call1".to_string(),
                name: "tool1".to_string(),
                arguments: serde_json::json!({}),
            },
            ToolCall {
                id: "call2".to_string(),
                name: "tool2".to_string(),
                arguments: serde_json::json!({}),
            },
        ];

        let config = ToolExecutionConfig {
            execution_strategy: FunctionExecutionStrategy::Concurrent,
            timeout_per_call: Duration::from_secs(5),
            max_parallel: None,
            ..Default::default()
        };

        let start = Instant::now();
        let results = execute_tool_calls(&calls, &tools, &config).await;
        let elapsed = start.elapsed();

        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        
        // Concurrent should take approximately 50ms (both run in parallel)
        assert!(elapsed.as_millis() < 100);
    }
}

