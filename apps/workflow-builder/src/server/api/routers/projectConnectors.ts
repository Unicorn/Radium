/**
 * Project Connectors router - CRUD operations for cross-project connectors
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';
import {
  createProjectConnector,
  deleteProjectConnector,
  testProjectConnector,
} from '@/lib/nexus/connector-manager';

export const projectConnectorsRouter = createTRPCRouter({
  // List connectors for a source project
  list: protectedProcedure
    .input(z.object({
      sourceProjectId: z.string().uuid(),
    }))
    .query(async ({ ctx, input }) => {
      // Verify project ownership
      const { data: project } = await ctx.supabase
        .from('projects')
        .select('id')
        .eq('id', input.sourceProjectId)
        .eq('created_by', ctx.user.id)
        .single();

      if (!project) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Project not found',
        });
      }

      const { data, error } = await ctx.supabase
        .from('project_connectors')
        .select(`
          *,
          target_project:projects!project_connectors_target_project_id_fkey(id, name),
          target_service:workflows!project_connectors_target_service_id_fkey(id, name, display_name),
          target_interface:service_interfaces!project_connectors_target_interface_id_fkey(id, name, display_name)
        `)
        .eq('source_project_id', input.sourceProjectId)
        .order('created_at', { ascending: false });

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return data || [];
    }),

  // Get connector by ID
  get: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .query(async ({ ctx, input }) => {
      const { data, error } = await ctx.supabase
        .from('project_connectors')
        .select(`
          *,
          source_project:projects!project_connectors_source_project_id_fkey(id, name, created_by),
          target_project:projects!project_connectors_target_project_id_fkey(id, name),
          target_service:workflows!project_connectors_target_service_id_fkey(id, name, display_name),
          target_interface:service_interfaces!project_connectors_target_interface_id_fkey(id, name, display_name)
        `)
        .eq('id', input.id)
        .single();

      if (error || !data) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Connector not found',
        });
      }

      // Verify ownership
      const sourceProject = data.source_project as any;
      if (sourceProject.created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'Access denied',
        });
      }

      return data;
    }),

  // Create connector
  create: protectedProcedure
    .input(z.object({
      sourceProjectId: z.string().uuid(),
      targetProjectId: z.string().uuid(),
      targetServiceId: z.string().uuid(),
      targetInterfaceId: z.string().uuid(),
      name: z.string().min(1).max(255),
      displayName: z.string().min(1).max(255),
      description: z.string().optional(),
      visibility: z.enum(['private', 'public', 'organization']).optional(),
      authConfig: z.record(z.any()).optional(),
    }))
    .mutation(async ({ ctx, input }) => {
      // Verify source project ownership
      const { data: sourceProject } = await ctx.supabase
        .from('projects')
        .select('id')
        .eq('id', input.sourceProjectId)
        .eq('created_by', ctx.user.id)
        .single();

      if (!sourceProject) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Source project not found',
        });
      }

      // Verify target project ownership (users can only connect their own projects)
      const { data: targetProject } = await ctx.supabase
        .from('projects')
        .select('id')
        .eq('id', input.targetProjectId)
        .eq('created_by', ctx.user.id)
        .single();

      if (!targetProject) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Target project not found or access denied',
        });
      }

      // Create connector with Nexus integration
      try {
        const connector = await createProjectConnector(
          {
            sourceProjectId: input.sourceProjectId,
            targetProjectId: input.targetProjectId,
            targetServiceId: input.targetServiceId,
            targetInterfaceId: input.targetInterfaceId,
            name: input.name,
            displayName: input.displayName,
            description: input.description,
            visibility: input.visibility,
            authConfig: input.authConfig,
          },
          ctx.user.id,
          ctx.supabase
        );

        return connector;
      } catch (error: any) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message || 'Failed to create connector',
        });
      }
    }),

  // Delete connector
  delete: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .mutation(async ({ ctx, input }) => {
      // Verify ownership
      const { data: connector } = await ctx.supabase
        .from('project_connectors')
        .select('source_project:projects!project_connectors_source_project_id_fkey(created_by)')
        .eq('id', input.id)
        .single();

      if (!connector) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Connector not found',
        });
      }

      const sourceProject = connector.source_project as any;
      if (sourceProject.created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'Access denied',
        });
      }

      // Delete connector and clean up Nexus resources
      try {
        await deleteProjectConnector(input.id, ctx.supabase);
        return { success: true };
      } catch (error: any) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message || 'Failed to delete connector',
        });
      }
    }),

  // Test connector
  test: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .mutation(async ({ ctx, input }) => {
      // Verify ownership
      const { data: connector } = await ctx.supabase
        .from('project_connectors')
        .select('source_project:projects!project_connectors_source_project_id_fkey(created_by)')
        .eq('id', input.id)
        .single();

      if (!connector) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Connector not found',
        });
      }

      const sourceProject = connector.source_project as any;
      if (sourceProject.created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'Access denied',
        });
      }

      // Test connector
      const result = await testProjectConnector(input.id, ctx.supabase);
      return result;
    }),
});

