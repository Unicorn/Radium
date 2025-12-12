/**
 * Execution Results Router
 * 
 * Results, outputs, and sync operations for workflow executions
 * Split from execution.ts for better organization
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';

export const executionResultsRouter = createTRPCRouter({
  /**
   * Get detailed execution information including component-level details
   */
  getExecutionDetails: protectedProcedure
    .input(z.object({ executionId: z.string() }))
    .query(async ({ ctx, input }) => {
      // Check database (cache check)
      const { data: execution, error: execError } = await ctx.supabase
        .from('workflow_executions')
        .select('*')
        .eq('id', input.executionId)
        .eq('created_by', ctx.user.id)
        .single();

      if (execError || !execution) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Execution not found',
        });
      }

      // Check if needs syncing
      const { checkSyncStatus, requestSync } = await import('@/lib/temporal/sync-service');
      const { needsSync } = await checkSyncStatus(input.executionId);

      // If not synced or stale, trigger immediate sync
      if (needsSync && execution.temporal_workflow_id && execution.temporal_run_id) {
        await requestSync(input.executionId, true);
      }

      // Fetch component executions
      const { data: componentExecutions } = await ctx.supabase
        .from('component_executions')
        .select('*')
        .eq('workflow_execution_id', input.executionId)
        .order('started_at', { ascending: true });

      return {
        id: execution.id,
        workflowId: execution.workflow_id,
        status: execution.status,
        startedAt: new Date(execution.started_at),
        completedAt: execution.completed_at ? new Date(execution.completed_at) : null,
        durationMs: execution.duration_ms,
        input: execution.input,
        output: execution.output,
        error: execution.error_message,
        temporalWorkflowId: execution.temporal_workflow_id,
        temporalRunId: execution.temporal_run_id,
        historySyncStatus: execution.history_sync_status,
        historySyncedAt: execution.history_synced_at ? new Date(execution.history_synced_at) : null,
        componentExecutions: componentExecutions || [],
      };
    }),

  /**
   * Manually trigger sync from Temporal (cache-aside pattern)
   */
  syncExecutionFromTemporal: protectedProcedure
    .input(z.object({
      executionId: z.string(),
      immediate: z.boolean().default(false),
    }))
    .mutation(async ({ ctx, input }) => {
      // Verify execution belongs to user
      const { data: execution } = await ctx.supabase
        .from('workflow_executions')
        .select('id')
        .eq('id', input.executionId)
        .eq('created_by', ctx.user.id)
        .single();

      if (!execution) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Execution not found',
        });
      }

      // Request sync
      const { requestSync, waitForSync } = await import('@/lib/temporal/sync-service');
      const result = await requestSync(input.executionId, input.immediate);

      if (input.immediate && result.synced) {
        // Wait for sync to complete
        const waitResult = await waitForSync(input.executionId);
        return {
          success: waitResult.success,
          synced: waitResult.synced,
          componentExecutionsCount: result.componentExecutionsCount,
          error: waitResult.error,
        };
      }

      return result;
    }),
});

