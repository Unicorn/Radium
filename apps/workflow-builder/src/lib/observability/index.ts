/**
 * Observability Module
 *
 * Workflow Observability Platform - Performance Analytics
 *
 * This module provides:
 * - Component execution metrics collection
 * - Workflow execution metrics (service-level analytics)
 * - Resource event tracking (interfaces, variables, agents, connectors)
 * - Batched, non-blocking metric recording
 * - Performance analytics data types
 * - Helper functions for tracking resource usage in activities
 */

export {
  MetricsCollector,
  getMetricsCollector,
  recordComponentMetric,
  recordWorkflowExecutionMetric,
  recordResourceEvent,
  flushPendingMetrics,
  shutdownMetricsCollector,
} from './metrics-collector';

export {
  trackInterfaceCall,
  trackVariableOperation,
  trackAgentCall,
  trackConnectorOperation,
  recordResourceEventAsync,
} from './resource-tracking';

export type { ResourceTrackingContext } from './resource-tracking';

export type {
  // Component metrics
  ComponentMetric,
  ComponentMetricInput,
  ComponentUsageDaily,
  ActivityPerformanceSummary,
  PerformanceQueryParams,
  PerformanceDashboardData,
  MetricsCollectorConfig,
  MetricStatus,
  ComponentType,
  // Workflow execution metrics
  WorkflowExecutionMetricInput,
  WorkflowUsageDaily,
  WorkflowStatus,
  TriggerType,
  // Resource events
  ResourceEventInput,
  ResourceUsageDaily,
  ResourceEvent,
  ResourceType,
  ResourceSubtype,
  ResourceOperation,
  ResourceDirection,
  ResourceEventStatus,
} from './types';

export { DEFAULT_METRICS_CONFIG } from './types';
