/**
 * Execution Router
 * 
 * Main router that re-exports all execution sub-routers for backward compatibility.
 * Split into sub-routers for better organization:
 * - execution-core.ts: Core operations (build, getStatus, list)
 * - execution-monitoring.ts: Monitoring and statistics
 * - execution-results.ts: Results and sync operations
 */

import { createTRPCRouter } from '../trpc';
import { executionCoreRouter } from './execution-core';
import { executionMonitoringRouter } from './execution-monitoring';
import { executionResultsRouter } from './execution-results';

// Re-export all procedures from sub-routers for backward compatibility
// Using manual assignment to preserve TypeScript types
export const executionRouter = createTRPCRouter({
  // Core operations
  build: executionCoreRouter._def.procedures.build,
  getStatus: executionCoreRouter._def.procedures.getStatus,
  list: executionCoreRouter._def.procedures.list,
  
  // Monitoring operations
  getExecutionHistory: executionMonitoringRouter._def.procedures.getExecutionHistory,
  getWorkflowStatistics: executionMonitoringRouter._def.procedures.getWorkflowStatistics,
  getProjectStatistics: executionMonitoringRouter._def.procedures.getProjectStatistics,
  listUserExecutions: executionMonitoringRouter._def.procedures.listUserExecutions,
  getGlobalStats: executionMonitoringRouter._def.procedures.getGlobalStats,
  
  // Results operations
  getExecutionDetails: executionResultsRouter._def.procedures.getExecutionDetails,
  syncExecutionFromTemporal: executionResultsRouter._def.procedures.syncExecutionFromTemporal,
});
