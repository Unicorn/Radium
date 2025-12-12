/**
 * Observability Types
 *
 * TypeScript types for the component performance metrics system.
 * These types mirror the database schema in db/migrations/001_component_metrics.sql
 */

export type MetricStatus = 'completed' | 'failed' | 'timeout' | 'cancelled';

export type ComponentType =
  | 'activity'
  | 'agent'
  | 'transform'
  | 'http'
  | 'database'
  | 'notification'
  | 'state'
  | 'custom';

/**
 * Raw component metric record
 * Represents a single activity execution
 */
export interface ComponentMetric {
  id: string;
  project_id: string;
  workflow_id: string;
  workflow_execution_id: string | null;
  component_type: string;
  component_name: string;
  component_id: string | null;
  node_id: string | null;
  invocation_count: number;
  duration_ms: number | null;
  queue_time_ms: number | null;
  status: MetricStatus;
  is_retry: boolean;
  attempt_number: number;
  memory_peak_mb: number | null;
  cpu_time_ms: number | null;
  io_bytes: number | null;
  error_type: string | null;
  error_code: string | null;
  started_at: string;
  completed_at: string | null;
  recorded_at: string;
  metadata: Record<string, unknown>;
}

/**
 * Input for recording a new metric
 */
export interface ComponentMetricInput {
  projectId: string;
  workflowId: string;
  executionId?: string;
  componentType: string;
  componentName: string;
  componentId?: string;
  nodeId?: string;
  durationMs: number;
  queueTimeMs?: number;
  status: MetricStatus;
  isRetry: boolean;
  attemptNumber: number;
  startedAt: Date;
  completedAt: Date;
  errorType?: string;
  errorCode?: string;
  memoryPeakMb?: number;
  cpuTimeMs?: number;
  ioBytes?: number;
  metadata?: Record<string, unknown>;
}

/**
 * Daily aggregated usage data
 */
export interface ComponentUsageDaily {
  id: string;
  date: string;
  project_id: string;
  component_type: string;
  component_name: string;
  total_invocations: number;
  successful_invocations: number;
  failed_invocations: number;
  retried_invocations: number;
  total_duration_ms: number;
  avg_duration_ms: number | null;
  p50_duration_ms: number | null;
  p95_duration_ms: number | null;
  p99_duration_ms: number | null;
  max_duration_ms: number | null;
  total_memory_mb: number;
  total_cpu_time_ms: number;
  updated_at: string;
}

/**
 * Activity performance summary (from get_activity_performance function)
 */
export interface ActivityPerformanceSummary {
  component_name: string;
  component_type: string;
  total_executions: number;
  success_rate: number;
  avg_duration_ms: number;
  p95_duration_ms: number | null;
  total_failures: number;
}

/**
 * Query parameters for performance analytics
 */
export interface PerformanceQueryParams {
  projectId: string;
  startDate: Date;
  endDate: Date;
  componentType?: string;
  componentName?: string;
}

/**
 * Aggregated metrics for dashboard display
 */
export interface PerformanceDashboardData {
  summary: {
    totalExecutions: number;
    successRate: number;
    avgDurationMs: number;
    totalFailures: number;
  };
  byComponent: ActivityPerformanceSummary[];
  timeline: {
    date: string;
    executions: number;
    failures: number;
    avgDuration: number;
  }[];
}

/**
 * Configuration for metrics collection
 */
export interface MetricsCollectorConfig {
  /** Enable/disable metrics collection */
  enabled: boolean;
  /** Sampling rate (0-1, default 1.0 = 100%) */
  sampleRate: number;
  /** Capture input/output payload sizes */
  capturePayloads: boolean;
  /** Batch size before flush */
  batchSize: number;
  /** Flush interval in milliseconds */
  flushIntervalMs: number;
}

/**
 * Default configuration
 */
export const DEFAULT_METRICS_CONFIG: MetricsCollectorConfig = {
  enabled: true,
  sampleRate: 1.0,
  capturePayloads: false,
  batchSize: 100,
  flushIntervalMs: 5000,
};

// ============================================================================
// Workflow Execution Metrics
// ============================================================================

export type WorkflowStatus = 'running' | 'completed' | 'failed' | 'cancelled' | 'timeout';

export type TriggerType = 'manual' | 'schedule' | 'webhook' | 'api' | 'signal';

/**
 * Input for recording workflow execution metrics
 */
export interface WorkflowExecutionMetricInput {
  projectId: string;
  workflowId: string;
  executionId?: string;
  workflowName: string;
  workflowVersion?: string;
  taskQueueName?: string;
  temporalWorkflowId?: string;
  temporalRunId?: string;
  triggerType: TriggerType;
  triggerSource?: string;
  inputSizeBytes?: number;
  outputSizeBytes?: number;
  durationMs?: number;
  queueTimeMs?: number;
  activityCount?: number;
  retryCount?: number;
  status: WorkflowStatus;
  errorType?: string;
  errorMessage?: string;
  totalMemoryMb?: number;
  totalCpuTimeMs?: number;
  startedAt: Date;
  completedAt?: Date;
  metadata?: Record<string, unknown>;
}

/**
 * Workflow usage daily aggregation
 */
export interface WorkflowUsageDaily {
  id: string;
  date: string;
  project_id: string;
  workflow_id: string;
  workflow_name: string;
  total_executions: number;
  successful_executions: number;
  failed_executions: number;
  cancelled_executions: number;
  timeout_executions: number;
  total_duration_ms: number;
  avg_duration_ms: number | null;
  max_duration_ms: number | null;
  total_activities_executed: number;
  total_input_bytes: number;
  total_output_bytes: number;
  manual_triggers: number;
  schedule_triggers: number;
  webhook_triggers: number;
  api_triggers: number;
  updated_at: string;
}

// ============================================================================
// Resource Events (Interfaces, Variables, Agents, Connectors)
// ============================================================================

export type ResourceType = 'interface' | 'variable' | 'agent' | 'connector';

export type ResourceSubtype =
  | 'service_interface'
  | 'public_interface'
  | 'state_variable'
  | 'workflow_variable'
  | 'project_connector'
  | 'database_connector'
  | 'ai_agent'
  | 'custom';

export type ResourceOperation = 'invoke' | 'read' | 'write' | 'call' | 'connect';

export type ResourceDirection = 'inbound' | 'outbound';

export type ResourceEventStatus = 'success' | 'failure' | 'timeout';

/**
 * Input for recording resource events
 */
export interface ResourceEventInput {
  projectId: string;
  workflowId?: string;
  executionId?: string;
  componentMetricId?: string;
  resourceType: ResourceType;
  resourceSubtype?: ResourceSubtype;
  resourceId?: string;
  resourceName: string;
  operation: ResourceOperation;
  direction?: ResourceDirection;
  durationMs?: number;
  latencyMs?: number;
  requestSizeBytes?: number;
  responseSizeBytes?: number;
  status: ResourceEventStatus;
  errorType?: string;
  errorCode?: string;
  // Agent-specific fields
  modelName?: string;
  promptTokens?: number;
  completionTokens?: number;
  totalTokens?: number;
  // Connector-specific fields
  targetProjectId?: string;
  targetService?: string;
  startedAt: Date;
  completedAt?: Date;
  metadata?: Record<string, unknown>;
}

/**
 * Resource usage daily aggregation
 */
export interface ResourceUsageDaily {
  id: string;
  date: string;
  project_id: string;
  resource_type: string;
  resource_name: string;
  total_invocations: number;
  successful_invocations: number;
  failed_invocations: number;
  total_duration_ms: number;
  avg_duration_ms: number | null;
  avg_latency_ms: number | null;
  total_request_bytes: number;
  total_response_bytes: number;
  total_prompt_tokens: number;
  total_completion_tokens: number;
  total_tokens: number;
  updated_at: string;
}

/**
 * Resource event record from database
 */
export interface ResourceEvent {
  id: string;
  project_id: string;
  workflow_id: string | null;
  workflow_execution_id: string | null;
  component_metric_id: string | null;
  resource_type: string;
  resource_subtype: string | null;
  resource_id: string | null;
  resource_name: string;
  operation: string;
  direction: string | null;
  duration_ms: number | null;
  latency_ms: number | null;
  request_size_bytes: number | null;
  response_size_bytes: number | null;
  status: string;
  error_type: string | null;
  error_code: string | null;
  model_name: string | null;
  prompt_tokens: number | null;
  completion_tokens: number | null;
  total_tokens: number | null;
  target_project_id: string | null;
  target_service: string | null;
  started_at: string;
  completed_at: string | null;
  recorded_at: string;
  metadata: Record<string, unknown> | null;
}
