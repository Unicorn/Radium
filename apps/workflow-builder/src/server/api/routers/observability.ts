/**
 * Observability Router
 *
 * Activity Performance Analytics API
 * Provides endpoints for monitoring workflow and activity performance metrics.
 *
 * Part of the Workflow Observability Platform.
 *
 * Note: Uses type assertions for database queries because the component_metrics
 * and component_usage_daily tables require migration. Run:
 * db/migrations/001_component_metrics.sql
 * Then regenerate types: npm run db:types
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';
import type {
  ActivityPerformanceSummary,
} from '@/lib/observability/types';

// Interim types until database migration and type regeneration
// These match the schema in db/migrations/001_component_metrics.sql
interface ComponentUsageDailyRow {
  id: string;
  project_id: string;
  workflow_id: string | null;
  date: string;
  component_type: string;
  component_name: string;
  total_invocations: number;
  successful_invocations: number;
  failed_invocations: number;
  total_duration_ms: number | null;
  avg_duration_ms: number | null;
  p95_duration_ms: number | null;
}

interface ComponentMetricsRow {
  id: string;
  project_id: string;
  workflow_id: string;
  execution_id: string | null;
  component_type: string;
  component_name: string;
  component_id: string | null;
  node_id: string | null;
  duration_ms: number | null;
  status: string;
  is_retry: boolean;
  attempt_number: number;
  started_at: string;
  completed_at: string | null;
  error_type: string | null;
  error_code: string | null;
  memory_peak_mb: number | null;
  cpu_time_ms: number | null;
  io_bytes: number | null;
  metadata: Record<string, unknown> | null;
}

// Helper to verify project access - avoids type inference issues
async function verifyProjectAccess(
  supabase: any,
  projectId: string,
  userId: string
): Promise<{ id: string }> {
  const { data: project, error: projectError } = await supabase
    .from('projects')
    .select('id')
    .eq('id', projectId)
    .eq('created_by', userId)
    .single();

  if (projectError || !project) {
    throw new TRPCError({
      code: 'NOT_FOUND',
      message: 'Project not found or access denied',
    });
  }

  return project as { id: string };
}

export const observabilityRouter = createTRPCRouter({
  /**
   * Get activity performance metrics for a project
   * Returns aggregated execution data for the specified date range
   */
  getActivityPerformance: protectedProcedure
    .input(
      z.object({
        projectId: z.string().uuid(),
        startDate: z.date(),
        endDate: z.date(),
        componentType: z.string().optional(),
        componentName: z.string().optional(),
      })
    )
    .query(async ({ ctx, input }) => {
      // Verify user has access to project
      await verifyProjectAccess(ctx.supabase, input.projectId, (ctx.user as any).id);

      // Query daily usage aggregates
      // Note: Uses type assertion until migration is run
      let query = (ctx.supabase as any)
        .from('component_usage_daily')
        .select('*')
        .eq('project_id', input.projectId)
        .gte('date', input.startDate.toISOString().split('T')[0])
        .lte('date', input.endDate.toISOString().split('T')[0]);

      if (input.componentType) {
        query = query.eq('component_type', input.componentType);
      }

      if (input.componentName) {
        query = query.eq('component_name', input.componentName);
      }

      const { data, error } = await query.order('date', { ascending: false }) as {
        data: ComponentUsageDailyRow[] | null;
        error: { message: string } | null;
      };

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      // Aggregate by component
      const byComponent = new Map<string, ActivityPerformanceSummary>();

      for (const row of data || []) {
        const key = `${row.component_type}:${row.component_name}`;
        const existing = byComponent.get(key);

        if (existing) {
          existing.total_executions += row.total_invocations;
          existing.total_failures += row.failed_invocations;
          // Recalculate success rate
          existing.success_rate =
            ((existing.total_executions - existing.total_failures) /
              existing.total_executions) *
            100;
          // Weighted average for duration
          const totalDuration =
            existing.avg_duration_ms * (existing.total_executions - row.total_invocations) +
            (row.total_duration_ms || 0);
          existing.avg_duration_ms = totalDuration / existing.total_executions;
          // Update p95 with max
          if (row.p95_duration_ms && row.p95_duration_ms > (existing.p95_duration_ms || 0)) {
            existing.p95_duration_ms = row.p95_duration_ms;
          }
        } else {
          byComponent.set(key, {
            component_name: row.component_name,
            component_type: row.component_type,
            total_executions: row.total_invocations,
            success_rate:
              ((row.total_invocations - row.failed_invocations) / row.total_invocations) * 100,
            avg_duration_ms: row.avg_duration_ms || 0,
            p95_duration_ms: row.p95_duration_ms,
            total_failures: row.failed_invocations,
          });
        }
      }

      return {
        components: Array.from(byComponent.values()).sort(
          (a, b) => b.total_executions - a.total_executions
        ),
        dateRange: {
          start: input.startDate,
          end: input.endDate,
        },
      };
    }),

  /**
   * Get execution analytics summary for a project
   * Returns high-level metrics for dashboard display
   */
  getExecutionAnalytics: protectedProcedure
    .input(
      z.object({
        projectId: z.string().uuid(),
        days: z.number().min(1).max(90).default(30),
      })
    )
    .query(async ({ ctx, input }) => {
      // Verify user has access to project
      await verifyProjectAccess(ctx.supabase, input.projectId, (ctx.user as any).id);

      const startDate = new Date();
      startDate.setDate(startDate.getDate() - input.days);

      // Get aggregated data
      // Note: Uses type assertion until migration is run
      const { data, error } = await (ctx.supabase as any)
        .from('component_usage_daily')
        .select('*')
        .eq('project_id', input.projectId)
        .gte('date', startDate.toISOString().split('T')[0])
        .order('date', { ascending: true }) as {
          data: ComponentUsageDailyRow[] | null;
          error: { message: string } | null;
        };

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      // Calculate summary
      let totalExecutions = 0;
      let totalFailures = 0;
      let totalDurationMs = 0;

      const dailyData = new Map<
        string,
        { executions: number; failures: number; duration: number }
      >();

      for (const row of data || []) {
        totalExecutions += row.total_invocations;
        totalFailures += row.failed_invocations;
        totalDurationMs += row.total_duration_ms || 0;

        const dateKey = row.date;
        const existing = dailyData.get(dateKey) || {
          executions: 0,
          failures: 0,
          duration: 0,
        };
        existing.executions += row.total_invocations;
        existing.failures += row.failed_invocations;
        existing.duration += row.total_duration_ms || 0;
        dailyData.set(dateKey, existing);
      }

      const successRate =
        totalExecutions > 0 ? ((totalExecutions - totalFailures) / totalExecutions) * 100 : 100;

      const avgDurationMs = totalExecutions > 0 ? totalDurationMs / totalExecutions : 0;

      const timeline = Array.from(dailyData.entries())
        .map(([date, stats]) => ({
          date,
          executions: stats.executions,
          failures: stats.failures,
          avgDuration: stats.executions > 0 ? stats.duration / stats.executions : 0,
        }))
        .sort((a, b) => a.date.localeCompare(b.date));

      return {
        summary: {
          totalExecutions,
          successRate: Math.round(successRate * 100) / 100,
          avgDurationMs: Math.round(avgDurationMs),
          totalFailures,
        },
        timeline,
        period: {
          days: input.days,
          startDate,
          endDate: new Date(),
        },
      };
    }),

  /**
   * Get latency distribution for activities
   * Returns percentile data for latency analysis
   */
  getLatencyDistribution: protectedProcedure
    .input(
      z.object({
        projectId: z.string().uuid(),
        componentName: z.string().optional(),
        days: z.number().min(1).max(30).default(7),
      })
    )
    .query(async ({ ctx, input }) => {
      // Verify user has access to project
      await verifyProjectAccess(ctx.supabase, input.projectId, (ctx.user as any).id);

      const startDate = new Date();
      startDate.setDate(startDate.getDate() - input.days);

      // Get raw metrics for percentile calculation
      // Note: Uses type assertion until migration is run
      let query = (ctx.supabase as any)
        .from('component_metrics')
        .select('component_name, duration_ms, started_at')
        .eq('project_id', input.projectId)
        .eq('status', 'completed')
        .not('duration_ms', 'is', null)
        .gte('started_at', startDate.toISOString())
        .order('started_at', { ascending: false })
        .limit(10000); // Limit for performance

      if (input.componentName) {
        query = query.eq('component_name', input.componentName);
      }

      const { data, error } = await query as {
        data: Pick<ComponentMetricsRow, 'component_name' | 'duration_ms' | 'started_at'>[] | null;
        error: { message: string } | null;
      };

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      // Group by component and calculate percentiles
      const byComponent = new Map<string, number[]>();

      for (const row of data || []) {
        if (row.duration_ms === null) continue;
        const existing = byComponent.get(row.component_name) || [];
        existing.push(row.duration_ms);
        byComponent.set(row.component_name, existing);
      }

      const distribution = Array.from(byComponent.entries()).map(([name, durations]) => {
        durations.sort((a, b) => a - b);
        const len = durations.length;

        return {
          componentName: name,
          count: len,
          min: durations[0] || 0,
          max: durations[len - 1] || 0,
          avg: Math.round(durations.reduce((a, b) => a + b, 0) / len),
          p50: durations[Math.floor(len * 0.5)] || 0,
          p90: durations[Math.floor(len * 0.9)] || 0,
          p95: durations[Math.floor(len * 0.95)] || 0,
          p99: durations[Math.floor(len * 0.99)] || 0,
        };
      });

      return {
        distribution: distribution.sort((a, b) => b.count - a.count),
        period: {
          days: input.days,
          startDate,
          endDate: new Date(),
        },
      };
    }),

  /**
   * Get recent activity executions
   * Returns detailed recent execution data for debugging
   */
  getRecentExecutions: protectedProcedure
    .input(
      z.object({
        projectId: z.string().uuid(),
        componentName: z.string().optional(),
        status: z.enum(['completed', 'failed', 'timeout', 'cancelled']).optional(),
        limit: z.number().min(1).max(100).default(50),
      })
    )
    .query(async ({ ctx, input }) => {
      // Verify user has access to project
      await verifyProjectAccess(ctx.supabase, input.projectId, (ctx.user as any).id);

      // Note: Uses type assertion until migration is run
      let query = (ctx.supabase as any)
        .from('component_metrics')
        .select(
          `
          id,
          component_type,
          component_name,
          node_id,
          duration_ms,
          status,
          is_retry,
          attempt_number,
          error_type,
          started_at,
          completed_at,
          metadata
        `
        )
        .eq('project_id', input.projectId)
        .order('started_at', { ascending: false })
        .limit(input.limit);

      if (input.componentName) {
        query = query.eq('component_name', input.componentName);
      }

      if (input.status) {
        query = query.eq('status', input.status);
      }

      const { data, error } = await query as {
        data: ComponentMetricsRow[] | null;
        error: { message: string } | null;
      };

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return {
        executions:
          data?.map((row) => ({
            id: row.id,
            componentType: row.component_type,
            componentName: row.component_name,
            nodeId: row.node_id,
            durationMs: row.duration_ms,
            status: row.status,
            isRetry: row.is_retry,
            attemptNumber: row.attempt_number,
            errorType: row.error_type,
            startedAt: row.started_at,
            completedAt: row.completed_at,
            metadata: row.metadata,
          })) || [],
      };
    }),

  /**
   * Get component types distribution
   * Returns breakdown by component type for visualization
   */
  getComponentTypeDistribution: protectedProcedure
    .input(
      z.object({
        projectId: z.string().uuid(),
        days: z.number().min(1).max(90).default(30),
      })
    )
    .query(async ({ ctx, input }) => {
      // Verify user has access to project
      await verifyProjectAccess(ctx.supabase, input.projectId, (ctx.user as any).id);

      const startDate = new Date();
      startDate.setDate(startDate.getDate() - input.days);

      // Note: Uses type assertion until migration is run
      const { data, error } = await (ctx.supabase as any)
        .from('component_usage_daily')
        .select('component_type, total_invocations, successful_invocations, failed_invocations')
        .eq('project_id', input.projectId)
        .gte('date', startDate.toISOString().split('T')[0]) as {
          data: Pick<ComponentUsageDailyRow, 'component_type' | 'total_invocations' | 'successful_invocations' | 'failed_invocations'>[] | null;
          error: { message: string } | null;
        };

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      // Aggregate by type
      const byType = new Map<
        string,
        { total: number; successful: number; failed: number }
      >();

      for (const row of data || []) {
        const existing = byType.get(row.component_type) || {
          total: 0,
          successful: 0,
          failed: 0,
        };
        existing.total += row.total_invocations;
        existing.successful += row.successful_invocations;
        existing.failed += row.failed_invocations;
        byType.set(row.component_type, existing);
      }

      const distribution = Array.from(byType.entries())
        .map(([type, stats]) => ({
          type,
          total: stats.total,
          successful: stats.successful,
          failed: stats.failed,
          successRate:
            stats.total > 0 ? Math.round((stats.successful / stats.total) * 10000) / 100 : 100,
        }))
        .sort((a, b) => b.total - a.total);

      const grandTotal = distribution.reduce((sum, d) => sum + d.total, 0);

      return {
        distribution: distribution.map((d) => ({
          ...d,
          percentage: grandTotal > 0 ? Math.round((d.total / grandTotal) * 10000) / 100 : 0,
        })),
        total: grandTotal,
      };
    }),
});
