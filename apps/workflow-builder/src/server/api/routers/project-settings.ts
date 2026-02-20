/**
 * Project Settings Router
 * 
 * Settings and statistics operations for projects
 * Split from projects.ts for better organization
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';

export const projectSettingsRouter = createTRPCRouter({
  /**
   * Get project statistics
   */
  stats: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .query(async ({ ctx, input }) => {
      const userRecord = await ctx.getUserRecord();
      if (!userRecord) {
        throw new TRPCError({
          code: 'UNAUTHORIZED',
          message: 'User record not found',
        });
      }
      
      // Get project with stats
      const { data: project } = await ctx.supabase
        .from('projects')
        .select('*')
        .eq('id', input.id)
        .eq('created_by', userRecord.id)
        .single();
      
      if (!project) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Project not found',
        });
      }
      
      // Get top activities
      const { data: topActivities } = await ctx.supabase
        .from('activity_statistics')
        .select('*')
        .eq('project_id', input.id)
        .order('execution_count', { ascending: false })
        .limit(10);
      
      // Get recent executions
      const { data: recentExecutions } = await ctx.supabase
        .from('workflow_executions')
        .select(`
          id,
          workflow_id,
          status,
          created_at,
          completed_at,
          workflows!inner(name, display_name, project_id)
        `)
        .eq('workflows.project_id', input.id)
        .order('created_at', { ascending: false })
        .limit(20);
      
      return {
        project: {
          id: project.id,
          name: project.name,
          total_workflow_executions: project.total_workflow_executions,
          total_activity_executions: project.total_activity_executions,
          avg_execution_duration_ms: project.avg_execution_duration_ms,
          last_execution_at: project.last_execution_at,
        },
        topActivities: topActivities || [],
        recentExecutions: recentExecutions || [],
      };
    }),
});

