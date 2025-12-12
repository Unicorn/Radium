/**
 * Project Logging Router
 * 
 * tRPC router for managing project-level logging configuration
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';

export const projectLoggingRouter = createTRPCRouter({
  // Get project logging configuration
  get: protectedProcedure
    .input(z.object({
      projectId: z.string().uuid(),
    }))
    .query(async ({ ctx, input }) => {
      // Verify ownership
      const { data: project, error: projectError } = await ctx.supabase
        .from('projects')
        .select('id, created_by')
        .eq('id', input.projectId)
        .single();

      if (projectError || !project) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Project not found',
        });
      }

      if (project.created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'You do not have access to this project',
        });
      }

      const { data, error } = await ctx.supabase
        .from('project_logging_config')
        .select(`
          *,
          connector:connectors(id, name, display_name, connector_type)
        `)
        .eq('project_id', input.projectId)
        .single();

      if (error) {
        if (error.code === 'PGRST116') {
          return null; // No logging config exists yet
        }
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return data;
    }),

  // Create or update project logging configuration
  upsert: protectedProcedure
    .input(z.object({
      projectId: z.string().uuid(),
      connectorId: z.string().uuid(),
      loggingComponentId: z.string().uuid(),
      enabledEndpoints: z.array(z.string().uuid()).default([]),
    }))
    .mutation(async ({ ctx, input }) => {
      // Verify ownership
      const { data: project, error: projectError } = await ctx.supabase
        .from('projects')
        .select('id, created_by')
        .eq('id', input.projectId)
        .single();

      if (projectError || !project) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Project not found',
        });
      }

      if (project.created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'You do not have access to this project',
        });
      }

      // Verify connector exists and belongs to project
      const { data: connector, error: connectorError } = await ctx.supabase
        .from('connectors')
        .select('id, project_id')
        .eq('id', input.connectorId)
        .single();

      if (connectorError || !connector) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Connector not found',
        });
      }

      if (connector.project_id !== input.projectId) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'Connector does not belong to this project',
        });
      }

      // Upsert logging config
      const { data, error } = await ctx.supabase
        .from('project_logging_config')
        .upsert({
          project_id: input.projectId,
          connector_id: input.connectorId,
          logging_component_id: input.loggingComponentId,
          enabled_endpoints: input.enabledEndpoints,
        }, {
          onConflict: 'project_id',
        })
        .select()
        .single();

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return data;
    }),

  // Update enabled endpoints
  updateEndpoints: protectedProcedure
    .input(z.object({
      projectId: z.string().uuid(),
      enabledEndpoints: z.array(z.string().uuid()),
    }))
    .mutation(async ({ ctx, input }) => {
      // Verify ownership
      const { data: project, error: projectError } = await ctx.supabase
        .from('projects')
        .select('id, created_by')
        .eq('id', input.projectId)
        .single();

      if (projectError || !project) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Project not found',
        });
      }

      if (project.created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'You do not have access to this project',
        });
      }

      const { data, error } = await ctx.supabase
        .from('project_logging_config')
        .update({
          enabled_endpoints: input.enabledEndpoints,
        })
        .eq('project_id', input.projectId)
        .select()
        .single();

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return data;
    }),

  // Toggle endpoint logging (for bidirectional sync)
  toggleEndpoint: protectedProcedure
    .input(z.object({
      projectId: z.string().uuid(),
      endpointId: z.string().uuid(),
      enabled: z.boolean(),
    }))
    .mutation(async ({ ctx, input }) => {
      // Verify ownership
      const { data: project, error: projectError } = await ctx.supabase
        .from('projects')
        .select('id, created_by')
        .eq('id', input.projectId)
        .single();

      if (projectError || !project) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Project not found',
        });
      }

      if (project.created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'You do not have access to this project',
        });
      }

      // Get current config
      const { data: currentConfig, error: getError } = await ctx.supabase
        .from('project_logging_config')
        .select('enabled_endpoints')
        .eq('project_id', input.projectId)
        .single();

      if (getError && getError.code !== 'PGRST116') {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: getError.message,
        });
      }

      const currentEndpoints = (currentConfig?.enabled_endpoints as string[]) || [];
      let newEndpoints: string[];

      if (input.enabled) {
        // Add endpoint if not already present
        newEndpoints = currentEndpoints.includes(input.endpointId)
          ? currentEndpoints
          : [...currentEndpoints, input.endpointId];
      } else {
        // Remove endpoint
        newEndpoints = currentEndpoints.filter(id => id !== input.endpointId);
      }

      // Update config
      const { data, error } = await ctx.supabase
        .from('project_logging_config')
        .upsert({
          project_id: input.projectId,
          enabled_endpoints: newEndpoints,
        }, {
          onConflict: 'project_id',
        })
        .select()
        .single();

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return data;
    }),
});

