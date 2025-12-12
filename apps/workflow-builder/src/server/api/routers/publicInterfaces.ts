/**
 * Public Interfaces router - CRUD operations for public interfaces (Kong routes)
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';
import {
  registerPublicInterface,
  updatePublicInterface,
  unregisterPublicInterface,
  type PublicInterfaceConfig,
} from '@/lib/kong/service-interface-registry';

export const publicInterfacesRouter = createTRPCRouter({
  // List public interfaces for a service interface or project
  list: protectedProcedure
    .input(z.object({
      serviceInterfaceId: z.string().uuid().optional(),
      projectId: z.string().uuid().optional(),
    }))
    .query(async ({ ctx, input }) => {
      if (input.projectId) {
        // List all public interfaces for a project
        const { data: workflows } = await ctx.supabase
          .from('workflows')
          .select('id')
          .eq('project_id', input.projectId)
          .eq('created_by', ctx.user.id);

        if (!workflows || workflows.length === 0) {
          return { interfaces: [] };
        }

        const workflowIds = workflows.map(w => w.id);
        const { data: serviceInterfaces } = await ctx.supabase
          .from('service_interfaces')
          .select('id')
          .in('workflow_id', workflowIds);

        if (!serviceInterfaces || serviceInterfaces.length === 0) {
          return { interfaces: [] };
        }

        const serviceInterfaceIds = serviceInterfaces.map(si => si.id);
        const { data, error } = await ctx.supabase
          .from('public_interfaces')
          .select(`
            *,
            service_interface:service_interfaces!inner(
              *,
              workflow:workflows!inner(
                id,
                name,
                display_name
              )
            )
          `)
          .in('service_interface_id', serviceInterfaceIds)
          .order('created_at', { ascending: false });

        if (error) {
          throw new TRPCError({
            code: 'INTERNAL_SERVER_ERROR',
            message: error.message,
          });
        }

        return { interfaces: data || [] };
      }

      // Original behavior: list for a specific service interface
      if (!input.serviceInterfaceId) {
        throw new TRPCError({
          code: 'BAD_REQUEST',
          message: 'Either serviceInterfaceId or projectId must be provided',
        });
      }
      // Verify service interface ownership
      const { data: serviceInterface } = await ctx.supabase
        .from('service_interfaces')
        .select('workflow:workflows!inner(created_by)')
        .eq('id', input.serviceInterfaceId)
        .single();

      if (!serviceInterface) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Service interface not found',
        });
      }

      const workflow = serviceInterface.workflow as any;
      if (workflow.created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'Access denied',
        });
      }

      const { data, error } = await ctx.supabase
        .from('public_interfaces')
        .select(`
          *,
          service_interface:service_interfaces!inner(
            *,
            workflow:workflows!inner(
              id,
              name,
              display_name
            )
          )
        `)
        .eq('service_interface_id', input.serviceInterfaceId)
        .order('created_at', { ascending: false });

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return { interfaces: data || [] };
    }),

  // Get public interface by ID
  get: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .query(async ({ ctx, input }) => {
      const { data, error } = await ctx.supabase
        .from('public_interfaces')
        .select(`
          *,
          service_interface:service_interfaces!inner(
            workflow:workflows!inner(created_by)
          )
        `)
        .eq('id', input.id)
        .single();

      if (error || !data) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Public interface not found',
        });
      }

      // Verify ownership
      const si = data.service_interface as any;
      const workflow = si.workflow as any;
      if (workflow.created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'Access denied',
        });
      }

      return data;
    }),

  // Create public interface (register with Kong)
  create: protectedProcedure
    .input(z.object({
      serviceInterfaceId: z.string().uuid(),
      httpMethod: z.enum(['GET', 'POST', 'PUT', 'DELETE', 'PATCH']),
      httpPath: z.string().min(1),
      authType: z.enum(['api_key', 'oauth2', 'jwt', 'none']).optional(),
      authConfig: z.record(z.any()).optional(),
      rateLimitPerMinute: z.number().int().positive().optional(),
      rateLimitPerHour: z.number().int().positive().optional(),
    }))
    .mutation(async ({ ctx, input }) => {
      // Verify service interface ownership
      const { data: serviceInterface } = await ctx.supabase
        .from('service_interfaces')
        .select('*')
        .eq('id', input.serviceInterfaceId)
        .single();

      if (!serviceInterface) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Service interface not found',
        });
      }

      // Verify workflow ownership
      const { data: workflow } = await ctx.supabase
        .from('workflows')
        .select('id')
        .eq('id', serviceInterface.workflow_id)
        .eq('created_by', ctx.user.id)
        .single();

      if (!workflow) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'Access denied',
        });
      }

      // Verify service interface is public
      if (!serviceInterface.is_public) {
        throw new TRPCError({
          code: 'BAD_REQUEST',
          message: 'Service interface must be marked as public first',
        });
      }

      // Register with Kong
      const config: PublicInterfaceConfig = {
        httpMethod: input.httpMethod,
        httpPath: input.httpPath,
        authType: input.authType || 'api_key',
        authConfig: input.authConfig,
        rateLimitPerMinute: input.rateLimitPerMinute,
        rateLimitPerHour: input.rateLimitPerHour,
      };

      try {
        const registered = await registerPublicInterface(
          input.serviceInterfaceId,
          config,
          ctx.supabase
        );

        return registered;
      } catch (error: any) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message || 'Failed to register public interface',
        });
      }
    }),

  // Update public interface
  update: protectedProcedure
    .input(z.object({
      id: z.string().uuid(),
      rateLimitPerMinute: z.number().int().positive().optional(),
      rateLimitPerHour: z.number().int().positive().optional(),
      authType: z.enum(['api_key', 'oauth2', 'jwt', 'none']).optional(),
      authConfig: z.record(z.any()).optional(),
    }))
    .mutation(async ({ ctx, input }) => {
      const { id, ...updates } = input;

      // Verify ownership
      const { data: existing } = await ctx.supabase
        .from('public_interfaces')
        .select(`
          service_interface:service_interfaces!inner(
            workflow:workflows!inner(created_by)
          )
        `)
        .eq('id', id)
        .single();

      if (!existing) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Public interface not found',
        });
      }

      const si = existing.service_interface as any;
      const workflow = si.workflow as any;
      if (workflow.created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'Access denied',
        });
      }

      // Update
      const config: Partial<PublicInterfaceConfig> = {};
      if (updates.rateLimitPerMinute !== undefined) config.rateLimitPerMinute = updates.rateLimitPerMinute;
      if (updates.rateLimitPerHour !== undefined) config.rateLimitPerHour = updates.rateLimitPerHour;
      if (updates.authType !== undefined) config.authType = updates.authType;
      if (updates.authConfig !== undefined) config.authConfig = updates.authConfig;

      try {
        const updated = await updatePublicInterface(id, config, ctx.supabase);
        return updated;
      } catch (error: any) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message || 'Failed to update public interface',
        });
      }
    }),

  // Delete public interface (unregister from Kong)
  delete: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .mutation(async ({ ctx, input }) => {
      // Verify ownership
      const { data: existing } = await ctx.supabase
        .from('public_interfaces')
        .select(`
          service_interface:service_interfaces!inner(
            workflow:workflows!inner(created_by)
          )
        `)
        .eq('id', input.id)
        .single();

      if (!existing) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Public interface not found',
        });
      }

      const si = existing.service_interface as any;
      const workflow = si.workflow as any;
      if (workflow.created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'Access denied',
        });
      }

      // Unregister from Kong
      try {
        await unregisterPublicInterface(input.id, ctx.supabase);
        return { success: true };
      } catch (error: any) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message || 'Failed to delete public interface',
        });
      }
    }),
});

