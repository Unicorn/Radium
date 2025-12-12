/**
 * Workflow Endpoints Router
 * 
 * tRPC router for managing workflow API endpoints
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';
import { registerWorkflowEndpoints, unregisterWorkflowEndpoints } from '@/lib/kong/endpoint-registry';
import type { WorkflowEndpoint } from '@/lib/kong/endpoint-registry';

export const workflowEndpointsRouter = createTRPCRouter({
  /**
   * List endpoints for a workflow
   */
  list: protectedProcedure
    .input(z.object({
      workflowId: z.string().uuid(),
    }))
    .query(async ({ ctx, input }) => {
      const { data, error } = await ctx.supabase
        .from('workflow_endpoints')
        .select('*')
        .eq('workflow_id', input.workflowId)
        .eq('created_by', ctx.user.id)
        .order('created_at', { ascending: false });

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return data || [];
    }),

  /**
   * Get endpoint by ID
   */
  get: protectedProcedure
    .input(z.object({
      id: z.string().uuid(),
    }))
    .query(async ({ ctx, input }) => {
      const { data, error } = await ctx.supabase
        .from('workflow_endpoints')
        .select('*')
        .eq('id', input.id)
        .eq('user_id', ctx.user.id)
        .single();

      if (error || !data) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Endpoint not found',
        });
      }

      return data;
    }),

  /**
   * Register endpoints for a workflow (called during deploy)
   */
  register: protectedProcedure
    .input(z.object({
      workflowId: z.string().uuid(),
      endpoints: z.array(z.object({
        endpointPath: z.string(),
        method: z.enum(['GET', 'POST', 'PUT', 'DELETE', 'PATCH']),
        description: z.string().optional(),
        targetType: z.enum(['signal', 'query', 'start']),
        targetName: z.string(),
        authType: z.enum(['api-key', 'jwt', 'none']).optional(),
        rateLimitPerMinute: z.number().optional(),
        rateLimitPerHour: z.number().optional(),
      })),
    }))
    .mutation(async ({ ctx, input }) => {
      // Verify workflow ownership
      const { data: workflow, error: workflowError } = await ctx.supabase
        .from('workflows')
        .select('id, project_id')
        .eq('id', input.workflowId)
        .eq('created_by', ctx.user.id)
        .single();

      if (workflowError || !workflow) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Workflow not found or not authorized',
        });
      }

      // Register endpoints with Kong
      const registered = await registerWorkflowEndpoints(
        input.workflowId,
        ctx.user.id,
        workflow.project_id,
        input.endpoints as WorkflowEndpoint[],
        ctx.supabase
      );

      return {
        success: true,
        endpoints: registered,
      };
    }),

  /**
   * Unregister endpoints (delete from Kong, mark inactive in DB)
   */
  unregister: protectedProcedure
    .input(z.object({
      workflowId: z.string().uuid(),
    }))
    .mutation(async ({ ctx, input }) => {
      // Verify workflow ownership
      const { data: workflow } = await ctx.supabase
        .from('workflows')
        .select('id')
        .eq('id', input.workflowId)
        .eq('created_by', ctx.user.id)
        .single();

      if (!workflow) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Workflow not found or not authorized',
        });
      }

      // Unregister endpoints
      await unregisterWorkflowEndpoints(input.workflowId, ctx.supabase);

      return { success: true };
    }),

  /**
   * Delete endpoint (permanently remove)
   */
  delete: protectedProcedure
    .input(z.object({
      id: z.string().uuid(),
    }))
    .mutation(async ({ ctx, input }) => {
      // Get endpoint and verify ownership
      const { data: endpoint } = await ctx.supabase
        .from('workflow_endpoints')
        .select('kong_route_id')
        .eq('id', input.id)
        .eq('user_id', ctx.user.id)
        .single();

      if (!endpoint) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Endpoint not found or not authorized',
        });
      }

      // Delete from Kong if route exists
      if (endpoint.kong_route_id) {
        const { KongClient } = await import('@/lib/kong/client');
        const kong = new KongClient();
        try {
          await kong.deleteRoute(endpoint.kong_route_id);
        } catch (error) {
          console.warn('Failed to delete Kong route:', error);
        }
      }

      // Delete from database
      const { error } = await ctx.supabase
        .from('workflow_endpoints')
        .delete()
        .eq('id', input.id);

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return { success: true };
    }),
});

