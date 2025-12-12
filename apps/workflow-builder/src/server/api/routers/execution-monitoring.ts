/**
 * Execution Monitoring Router
 * 
 * Monitoring and statistics operations for workflow executions
 * Split from execution.ts for better organization
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';

export const executionMonitoringRouter = createTRPCRouter({
  /**
   * Get execution history for a workflow with filtering
   */
  getExecutionHistory: protectedProcedure
    .input(z.object({
      workflowId: z.string(),
      page: z.number().default(1),
      pageSize: z.number().default(20),
      status: z.enum(['all', 'completed', 'failed', 'running']).optional(),
    }))
    .query(async ({ ctx, input }) => {
      const from = (input.page - 1) * input.pageSize;
      const to = from + input.pageSize - 1;

      let query = ctx.supabase
        .from('workflow_executions')
        .select('*', { count: 'exact' })
        .eq('workflow_id', input.workflowId)
        .eq('created_by', ctx.user.id);

      if (input.status && input.status !== 'all') {
        query = query.eq('status', input.status);
      }

      const { data, error, count } = await query
        .order('started_at', { ascending: false })
        .range(from, to);

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return {
        executions: data?.map(e => ({
          id: e.id,
          status: e.status,
          startedAt: new Date(e.started_at),
          completedAt: e.completed_at ? new Date(e.completed_at) : null,
          durationMs: e.duration_ms,
          result: e.output,
          error: e.error_message,
          historySyncStatus: e.history_sync_status,
        })) || [],
        total: count || 0,
        page: input.page,
        pageSize: input.pageSize,
      };
    }),

  /**
   * Get aggregated statistics for a workflow
   */
  getWorkflowStatistics: protectedProcedure
    .input(z.object({ workflowId: z.string() }))
    .query(async ({ ctx, input }) => {
      // Verify workflow belongs to user
      const { data: workflow } = await ctx.supabase
        .from('workflows')
        .select('id')
        .eq('id', input.workflowId)
        .eq('created_by', ctx.user.id)
        .single();

      if (!workflow) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Workflow not found',
        });
      }

      // Get statistics
      const { data: stats } = await ctx.supabase
        .from('workflow_statistics')
        .select('*')
        .eq('workflow_id', input.workflowId)
        .single();

      return stats || {
        total_runs: 0,
        successful_runs: 0,
        failed_runs: 0,
        avg_duration_ms: null,
        min_duration_ms: null,
        max_duration_ms: null,
        most_used_component_id: null,
        most_used_component_count: 0,
        total_errors: 0,
        last_error_at: null,
        last_run_at: null,
      };
    }),

  /**
   * Get aggregated statistics for a project
   */
  getProjectStatistics: protectedProcedure
    .input(z.object({ projectId: z.string() }))
    .query(async ({ ctx, input }) => {
      // Verify project belongs to user
      const { data: project } = await ctx.supabase
        .from('projects')
        .select('id')
        .eq('id', input.projectId)
        .eq('created_by', ctx.user.id)
        .single();

      if (!project) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Project not found',
        });
      }

      // Get statistics
      const { data: stats } = await ctx.supabase
        .from('project_statistics')
        .select('*')
        .eq('project_id', input.projectId)
        .single();

      return stats || {
        most_used_workflow_id: null,
        most_used_workflow_count: 0,
        most_used_component_id: null,
        most_used_component_count: 0,
        most_used_task_queue_id: null,
        most_used_task_queue_count: 0,
        total_executions: 0,
        longest_run_duration_ms: null,
        longest_run_workflow_id: null,
        total_failures: 0,
        last_failure_at: null,
        last_execution_at: null,
      };
    }),

  /**
   * List all executions for the current user across all workflows
   */
  listUserExecutions: protectedProcedure
    .input(z.object({
      page: z.number().default(1),
      pageSize: z.number().default(20),
      status: z.enum(['running', 'completed', 'failed', 'cancelled', 'timed_out', 'building']).optional(),
    }))
    .query(async ({ ctx, input }) => {
      const from = (input.page - 1) * input.pageSize;
      const to = from + input.pageSize - 1;

      let query = ctx.supabase
        .from('workflow_executions')
        .select(`
          *,
          workflow:workflows!workflow_executions_workflow_id_fkey (
            id,
            name,
            display_name,
            description
          )
        `, { count: 'exact' })
        .eq('created_by', ctx.user.id);

      if (input.status) {
        query = query.eq('status', input.status);
      }

      const { data, error, count } = await query
        .order('started_at', { ascending: false })
        .range(from, to);

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return {
        executions: data?.map(e => ({
          id: e.id,
          workflowId: e.workflow_id,
          status: e.status,
          startedAt: new Date(e.started_at).toISOString(),
          completedAt: e.completed_at ? new Date(e.completed_at).toISOString() : null,
          durationMs: e.duration_ms,
          errorMessage: e.error_message,
          workflow: e.workflow as any,
        })) || [],
        total: count || 0,
        page: input.page,
        pageSize: input.pageSize,
      };
    }),

  /**
   * Get global execution statistics for the current user
   */
  getGlobalStats: protectedProcedure
    .query(async ({ ctx }) => {
      const { data: executions, error } = await ctx.supabase
        .from('workflow_executions')
        .select('status, duration_ms')
        .eq('created_by', ctx.user.id);

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      if (!executions || executions.length === 0) {
        return {
          total: 0,
          running: 0,
          completed: 0,
          failed: 0,
          cancelled: 0,
          timedOut: 0,
          avgDuration: 0,
          successRate: 0,
        };
      }

      const total = executions.length;
      const running = executions.filter(e => e.status === 'running' || e.status === 'building').length;
      const completed = executions.filter(e => e.status === 'completed').length;
      const failed = executions.filter(e => e.status === 'failed').length;
      const cancelled = executions.filter(e => e.status === 'cancelled').length;
      const timedOut = executions.filter(e => e.status === 'timed_out').length;

      const durations = executions
        .filter(e => e.duration_ms !== null && e.duration_ms !== undefined)
        .map(e => e.duration_ms!);

      const avgDuration = durations.length > 0
        ? Math.round(durations.reduce((sum, d) => sum + d, 0) / durations.length)
        : 0;

      const successRate = total > 0 ? Math.round((completed / total) * 10000) / 100 : 0;

      return {
        total,
        running,
        completed,
        failed,
        cancelled,
        timedOut,
        avgDuration,
        successRate,
      };
    }),
});

