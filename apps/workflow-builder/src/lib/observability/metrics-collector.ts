/**
 * Metrics Collector Service
 *
 * Handles recording of component execution metrics to the database.
 * Designed for high-throughput with batching and error resilience.
 *
 * Features:
 * - Batched writes to reduce database load
 * - Non-blocking operation (failures don't impact execution)
 * - Configurable via environment variables
 * - Graceful shutdown with pending metric flush
 */

import { createClient, SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import {
  ComponentMetricInput,
  WorkflowExecutionMetricInput,
  ResourceEventInput,
  MetricsCollectorConfig,
  DEFAULT_METRICS_CONFIG,
} from './types';

// Singleton collector instance
let collectorInstance: MetricsCollector | null = null;

/**
 * Get the metrics collector instance
 * Creates one if it doesn't exist
 */
export function getMetricsCollector(): MetricsCollector {
  if (!collectorInstance) {
    collectorInstance = new MetricsCollector(getConfigFromEnv());
  }
  return collectorInstance;
}

/**
 * Load configuration from environment variables
 */
function getConfigFromEnv(): MetricsCollectorConfig {
  return {
    enabled: process.env.METRICS_ENABLED !== 'false',
    sampleRate: parseFloat(process.env.METRICS_SAMPLE_RATE || '1.0'),
    capturePayloads: process.env.METRICS_CAPTURE_PAYLOADS === 'true',
    batchSize: parseInt(process.env.METRICS_BATCH_SIZE || '100', 10),
    flushIntervalMs: parseInt(process.env.METRICS_FLUSH_INTERVAL_MS || '5000', 10),
  };
}

/**
 * Create Supabase client with service role for server-side operations
 */
function getStorageClient(): SupabaseClient<Database> {
  const url = process.env.NEXT_PUBLIC_SUPABASE_URL;
  const key = process.env.SUPABASE_SERVICE_ROLE_KEY;

  if (!url || !key) {
    throw new Error('Supabase URL and service role key are required for metrics collection');
  }

  return createClient<Database>(url, key);
}

/**
 * MetricsCollector class
 * Handles batched metric recording with automatic flushing
 */
export class MetricsCollector {
  private config: MetricsCollectorConfig;
  private buffer: ComponentMetricInput[] = [];
  private workflowBuffer: WorkflowExecutionMetricInput[] = [];
  private resourceBuffer: ResourceEventInput[] = [];
  private flushTimeout: NodeJS.Timeout | null = null;
  private isShuttingDown = false;
  private supabase: SupabaseClient<Database> | null = null;

  constructor(config: Partial<MetricsCollectorConfig> = {}) {
    this.config = { ...DEFAULT_METRICS_CONFIG, ...config };
  }

  /**
   * Get Supabase client (lazy initialization)
   */
  private getClient(): SupabaseClient<Database> {
    if (!this.supabase) {
      this.supabase = getStorageClient();
    }
    return this.supabase;
  }

  /**
   * Record a component execution metric
   *
   * Metrics are buffered and flushed in batches for efficiency.
   * This is a non-blocking operation - failures are logged but don't
   * impact activity execution.
   */
  async recordMetric(input: ComponentMetricInput): Promise<void> {
    // Check if collection is enabled
    if (!this.config.enabled) {
      return;
    }

    // Apply sampling
    if (Math.random() > this.config.sampleRate) {
      return;
    }

    // Don't accept new metrics during shutdown
    if (this.isShuttingDown) {
      console.warn('[Metrics] Collector is shutting down, metric dropped');
      return;
    }

    // Add to buffer
    this.buffer.push(input);

    // Flush if buffer is full
    if (this.buffer.length >= this.config.batchSize) {
      await this.flush();
    } else {
      this.scheduleFlush();
    }
  }

  /**
   * Record a workflow execution metric (for service/billing analytics)
   *
   * Call this when a workflow starts and when it completes.
   */
  async recordWorkflowExecution(input: WorkflowExecutionMetricInput): Promise<void> {
    if (!this.config.enabled) return;
    if (Math.random() > this.config.sampleRate) return;
    if (this.isShuttingDown) {
      console.warn('[Metrics] Collector is shutting down, workflow metric dropped');
      return;
    }

    this.workflowBuffer.push(input);

    if (this.workflowBuffer.length >= this.config.batchSize) {
      await this.flush();
    } else {
      this.scheduleFlush();
    }
  }

  /**
   * Record a resource event (interface, variable, agent, connector usage)
   *
   * These events track usage of supporting resources within workflow execution.
   */
  async recordResourceEvent(input: ResourceEventInput): Promise<void> {
    if (!this.config.enabled) return;
    if (Math.random() > this.config.sampleRate) return;
    if (this.isShuttingDown) {
      console.warn('[Metrics] Collector is shutting down, resource event dropped');
      return;
    }

    this.resourceBuffer.push(input);

    if (this.resourceBuffer.length >= this.config.batchSize) {
      await this.flush();
    } else {
      this.scheduleFlush();
    }
  }

  /**
   * Schedule a flush if not already scheduled
   */
  private scheduleFlush(): void {
    if (!this.flushTimeout) {
      this.flushTimeout = setTimeout(() => {
        this.flush().catch((err) =>
          console.error('[Metrics] Scheduled flush error:', err)
        );
      }, this.config.flushIntervalMs) as unknown as NodeJS.Timeout;
    }
  }

  /**
   * Flush all buffered metrics to database
   */
  async flush(): Promise<void> {
    // Clear timeout
    if (this.flushTimeout) {
      clearTimeout(this.flushTimeout);
      this.flushTimeout = null;
    }

    const hasComponentMetrics = this.buffer.length > 0;
    const hasWorkflowMetrics = this.workflowBuffer.length > 0;
    const hasResourceEvents = this.resourceBuffer.length > 0;

    if (!hasComponentMetrics && !hasWorkflowMetrics && !hasResourceEvents) {
      return;
    }

    const startTime = Date.now();
    const supabase = this.getClient();

    // Flush component metrics
    if (hasComponentMetrics) {
      await this.flushComponentMetrics(supabase);
    }

    // Flush workflow execution metrics
    if (hasWorkflowMetrics) {
      await this.flushWorkflowMetrics(supabase);
    }

    // Flush resource events
    if (hasResourceEvents) {
      await this.flushResourceEvents(supabase);
    }

    const flushDuration = Date.now() - startTime;
    const totalFlushed =
      (hasComponentMetrics ? this.buffer.length : 0) +
      (hasWorkflowMetrics ? this.workflowBuffer.length : 0) +
      (hasResourceEvents ? this.resourceBuffer.length : 0);

    if (totalFlushed > 0) {
      console.log(`[Metrics] Flush completed in ${flushDuration}ms`);
    }
  }

  /**
   * Flush component metrics buffer
   */
  private async flushComponentMetrics(supabase: SupabaseClient<Database>): Promise<void> {
    if (this.buffer.length === 0) return;

    const metrics = this.buffer.splice(0, this.buffer.length);

    try {
      // Transform to database format
      const metricsToInsert = metrics.map((m) => ({
        project_id: m.projectId,
        workflow_id: m.workflowId,
        workflow_execution_id: m.executionId || null,
        component_type: m.componentType,
        component_name: m.componentName,
        component_id: m.componentId || null,
        node_id: m.nodeId || null,
        duration_ms: m.durationMs,
        queue_time_ms: m.queueTimeMs || null,
        status: m.status,
        is_retry: m.isRetry,
        attempt_number: m.attemptNumber,
        started_at: m.startedAt.toISOString(),
        completed_at: m.completedAt.toISOString(),
        error_type: m.errorType || null,
        error_code: m.errorCode || null,
        memory_peak_mb: m.memoryPeakMb || null,
        cpu_time_ms: m.cpuTimeMs || null,
        io_bytes: m.ioBytes || null,
        metadata: m.metadata || {},
      }));

      // Batch insert
      const { error: insertError } = await (supabase as any)
        .from('component_metrics')
        .insert(metricsToInsert);

      if (insertError) {
        console.error('[Metrics] Component metrics insert error:', insertError);
        return;
      }

      console.log(`[Metrics] Flushed ${metrics.length} component metrics`);

      // Fire-and-forget aggregation updates via RPC
      for (const metric of metrics) {
        (supabase as any)
          .rpc('record_component_metric', {
            p_project_id: metric.projectId,
            p_workflow_id: metric.workflowId,
            p_execution_id: metric.executionId || null,
            p_component_type: metric.componentType,
            p_component_name: metric.componentName,
            p_component_id: metric.componentId || null,
            p_node_id: metric.nodeId || null,
            p_duration_ms: metric.durationMs,
            p_status: metric.status,
            p_is_retry: metric.isRetry,
            p_attempt_number: metric.attemptNumber,
            p_started_at: metric.startedAt.toISOString(),
            p_completed_at: metric.completedAt.toISOString(),
            p_error_type: metric.errorType || null,
            p_metadata: metric.metadata || {},
          })
          .catch((err: unknown) => {
            console.warn('[Metrics] Component aggregation error:', err);
          });
      }
    } catch (error) {
      console.error('[Metrics] Component metrics flush failed:', error);
    }
  }

  /**
   * Flush workflow execution metrics buffer
   */
  private async flushWorkflowMetrics(supabase: SupabaseClient<Database>): Promise<void> {
    if (this.workflowBuffer.length === 0) return;

    const metrics = this.workflowBuffer.splice(0, this.workflowBuffer.length);

    try {
      // Transform to database format
      const metricsToInsert = metrics.map((m) => ({
        project_id: m.projectId,
        workflow_id: m.workflowId,
        workflow_execution_id: m.executionId || null,
        workflow_name: m.workflowName,
        workflow_version: m.workflowVersion || null,
        task_queue_name: m.taskQueueName || null,
        temporal_workflow_id: m.temporalWorkflowId || null,
        temporal_run_id: m.temporalRunId || null,
        trigger_type: m.triggerType,
        trigger_source: m.triggerSource || null,
        input_size_bytes: m.inputSizeBytes || null,
        output_size_bytes: m.outputSizeBytes || null,
        duration_ms: m.durationMs || null,
        queue_time_ms: m.queueTimeMs || null,
        activity_count: m.activityCount || 0,
        retry_count: m.retryCount || 0,
        status: m.status,
        error_type: m.errorType || null,
        error_message: m.errorMessage || null,
        total_memory_mb: m.totalMemoryMb || null,
        total_cpu_time_ms: m.totalCpuTimeMs || null,
        started_at: m.startedAt.toISOString(),
        completed_at: m.completedAt?.toISOString() || null,
        metadata: m.metadata || {},
      }));

      // Batch insert
      const { error: insertError } = await (supabase as any)
        .from('workflow_execution_metrics')
        .insert(metricsToInsert);

      if (insertError) {
        console.error('[Metrics] Workflow metrics insert error:', insertError);
        return;
      }

      console.log(`[Metrics] Flushed ${metrics.length} workflow execution metrics`);

      // Fire-and-forget aggregation updates via RPC
      for (const metric of metrics) {
        (supabase as any)
          .rpc('record_workflow_execution_metric', {
            p_project_id: metric.projectId,
            p_workflow_id: metric.workflowId,
            p_execution_id: metric.executionId || null,
            p_workflow_name: metric.workflowName,
            p_workflow_version: metric.workflowVersion || null,
            p_task_queue_name: metric.taskQueueName || null,
            p_temporal_workflow_id: metric.temporalWorkflowId || null,
            p_temporal_run_id: metric.temporalRunId || null,
            p_trigger_type: metric.triggerType,
            p_trigger_source: metric.triggerSource || null,
            p_input_size_bytes: metric.inputSizeBytes || null,
            p_output_size_bytes: metric.outputSizeBytes || null,
            p_duration_ms: metric.durationMs || null,
            p_activity_count: metric.activityCount || 0,
            p_status: metric.status,
            p_error_type: metric.errorType || null,
            p_started_at: metric.startedAt.toISOString(),
            p_completed_at: metric.completedAt?.toISOString() || null,
            p_metadata: metric.metadata || {},
          })
          .catch((err: unknown) => {
            console.warn('[Metrics] Workflow aggregation error:', err);
          });
      }
    } catch (error) {
      console.error('[Metrics] Workflow metrics flush failed:', error);
    }
  }

  /**
   * Flush resource events buffer
   */
  private async flushResourceEvents(supabase: SupabaseClient<Database>): Promise<void> {
    if (this.resourceBuffer.length === 0) return;

    const events = this.resourceBuffer.splice(0, this.resourceBuffer.length);

    try {
      // Transform to database format
      const eventsToInsert = events.map((e) => ({
        project_id: e.projectId,
        workflow_id: e.workflowId || null,
        workflow_execution_id: e.executionId || null,
        component_metric_id: e.componentMetricId || null,
        resource_type: e.resourceType,
        resource_subtype: e.resourceSubtype || null,
        resource_id: e.resourceId || null,
        resource_name: e.resourceName,
        operation: e.operation,
        direction: e.direction || null,
        duration_ms: e.durationMs || null,
        latency_ms: e.latencyMs || null,
        request_size_bytes: e.requestSizeBytes || null,
        response_size_bytes: e.responseSizeBytes || null,
        status: e.status,
        error_type: e.errorType || null,
        error_code: e.errorCode || null,
        model_name: e.modelName || null,
        prompt_tokens: e.promptTokens || null,
        completion_tokens: e.completionTokens || null,
        total_tokens: e.totalTokens || null,
        target_project_id: e.targetProjectId || null,
        target_service: e.targetService || null,
        started_at: e.startedAt.toISOString(),
        completed_at: e.completedAt?.toISOString() || null,
        metadata: e.metadata || {},
      }));

      // Batch insert
      const { error: insertError } = await (supabase as any)
        .from('resource_events')
        .insert(eventsToInsert);

      if (insertError) {
        console.error('[Metrics] Resource events insert error:', insertError);
        return;
      }

      console.log(`[Metrics] Flushed ${events.length} resource events`);

      // Fire-and-forget aggregation updates via RPC
      for (const event of events) {
        (supabase as any)
          .rpc('record_resource_event', {
            p_project_id: event.projectId,
            p_workflow_id: event.workflowId || null,
            p_execution_id: event.executionId || null,
            p_component_metric_id: event.componentMetricId || null,
            p_resource_type: event.resourceType,
            p_resource_subtype: event.resourceSubtype || null,
            p_resource_id: event.resourceId || null,
            p_resource_name: event.resourceName,
            p_operation: event.operation,
            p_direction: event.direction || null,
            p_duration_ms: event.durationMs || null,
            p_latency_ms: event.latencyMs || null,
            p_request_size_bytes: event.requestSizeBytes || null,
            p_response_size_bytes: event.responseSizeBytes || null,
            p_status: event.status,
            p_error_type: event.errorType || null,
            p_model_name: event.modelName || null,
            p_prompt_tokens: event.promptTokens || null,
            p_completion_tokens: event.completionTokens || null,
            p_total_tokens: event.totalTokens || null,
            p_target_project_id: event.targetProjectId || null,
            p_target_service: event.targetService || null,
            p_started_at: event.startedAt.toISOString(),
            p_completed_at: event.completedAt?.toISOString() || null,
            p_metadata: event.metadata || {},
          })
          .catch((err: unknown) => {
            console.warn('[Metrics] Resource event aggregation error:', err);
          });
      }
    } catch (error) {
      console.error('[Metrics] Resource events flush failed:', error);
    }
  }

  /**
   * Gracefully shutdown the collector
   * Ensures all pending metrics are flushed before returning
   */
  async shutdown(): Promise<void> {
    console.log('[Metrics] Shutting down collector...');
    this.isShuttingDown = true;

    // Clear scheduled flush
    if (this.flushTimeout) {
      clearTimeout(this.flushTimeout);
      this.flushTimeout = null;
    }

    // Flush remaining metrics
    const totalPending =
      this.buffer.length + this.workflowBuffer.length + this.resourceBuffer.length;
    if (totalPending > 0) {
      console.log(`[Metrics] Flushing ${totalPending} pending metrics...`);
      await this.flush();
    }

    console.log('[Metrics] Collector shutdown complete');
  }

  /**
   * Get current buffer sizes (for monitoring)
   */
  getBufferSize(): number {
    return this.buffer.length;
  }

  /**
   * Get all buffer sizes (for monitoring)
   */
  getAllBufferSizes(): { component: number; workflow: number; resource: number } {
    return {
      component: this.buffer.length,
      workflow: this.workflowBuffer.length,
      resource: this.resourceBuffer.length,
    };
  }

  /**
   * Get current configuration
   */
  getConfig(): MetricsCollectorConfig {
    return { ...this.config };
  }

  /**
   * Update configuration at runtime
   */
  updateConfig(updates: Partial<MetricsCollectorConfig>): void {
    this.config = { ...this.config, ...updates };
  }
}

/**
 * Convenience function to record a component metric
 * Uses the singleton collector instance
 */
export async function recordComponentMetric(
  input: ComponentMetricInput
): Promise<void> {
  return getMetricsCollector().recordMetric(input);
}

/**
 * Convenience function to record a workflow execution metric
 * Call on workflow start (status='running') and completion (status='completed'/'failed'/etc.)
 */
export async function recordWorkflowExecutionMetric(
  input: WorkflowExecutionMetricInput
): Promise<void> {
  return getMetricsCollector().recordWorkflowExecution(input);
}

/**
 * Convenience function to record a resource event
 * For tracking interface, variable, agent, and connector usage
 */
export async function recordResourceEvent(
  input: ResourceEventInput
): Promise<void> {
  return getMetricsCollector().recordResourceEvent(input);
}

/**
 * Flush all pending metrics
 * Call this on process shutdown
 */
export async function flushPendingMetrics(): Promise<void> {
  if (collectorInstance) {
    await collectorInstance.flush();
  }
}

/**
 * Shutdown the metrics collector
 * Call this on process exit
 */
export async function shutdownMetricsCollector(): Promise<void> {
  if (collectorInstance) {
    await collectorInstance.shutdown();
    collectorInstance = null;
  }
}
