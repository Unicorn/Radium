/**
 * State Monitoring Router
 * 
 * Provides API endpoints for state variable metrics and alerts
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { getStateMetrics, checkStateAlerts, getProjectStateAlerts, recordStateAccess } from '@/lib/monitoring/state-metrics';

export const stateMonitoringRouter = createTRPCRouter({
  /**
   * Get metrics for a state variable
   */
  getMetrics: protectedProcedure
    .input(
      z.object({
        variableId: z.string().uuid(),
        scope: z.enum(['project', 'workflow']),
      })
    )
    .query(async ({ ctx, input }) => {
      const metrics = await getStateMetrics(ctx.supabase, input.variableId, input.scope);
      return metrics;
    }),

  /**
   * Get alerts for a state variable
   */
  getAlerts: protectedProcedure
    .input(
      z.object({
        variableId: z.string().uuid(),
        scope: z.enum(['project', 'workflow']),
      })
    )
    .query(async ({ ctx, input }) => {
      const alerts = await checkStateAlerts(ctx.supabase, input.variableId, input.scope);
      return alerts;
    }),

  /**
   * Get all alerts for a project
   */
  getProjectAlerts: protectedProcedure
    .input(
      z.object({
        projectId: z.string().uuid(),
      })
    )
    .query(async ({ ctx, input }) => {
      // Verify project ownership
      const { data: project } = await ctx.supabase
        .from('projects')
        .select('id')
        .eq('id', input.projectId)
        .eq('created_by', ctx.user.id)
        .single();

      if (!project) {
        throw new Error('Project not found');
      }

      const alerts = await getProjectStateAlerts(ctx.supabase, input.projectId);
      return alerts;
    }),

  /**
   * Record state variable access (called from activities)
   */
  recordAccess: protectedProcedure
    .input(
      z.object({
        variableId: z.string().uuid(),
        scope: z.enum(['project', 'workflow']),
        sizeBytes: z.number().int().min(0),
      })
    )
    .mutation(async ({ ctx, input }) => {
      await recordStateAccess(
        ctx.supabase,
        input.variableId,
        input.scope,
        input.sizeBytes
      );
      return { success: true };
    }),
});

