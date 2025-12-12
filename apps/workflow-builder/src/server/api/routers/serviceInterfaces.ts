/**
 * Service Interfaces router - CRUD operations for service interfaces
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';
import { syncPublicInterfacesForServiceInterface } from '@/lib/kong/service-interface-registry';
import { createClient } from '@/lib/supabase/server';
import { createServiceInterfaceFromComponent } from '@/lib/interfaces/interface-component-manager';

export const serviceInterfacesRouter = createTRPCRouter({
  // List service interfaces for a workflow
  list: protectedProcedure
    .input(z.object({
      workflowId: z.string().uuid(),
    }))
    .query(async ({ ctx, input }) => {
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
          message: 'Workflow not found',
        });
      }

      const { data, error } = await ctx.supabase
        .from('service_interfaces')
        .select('*')
        .eq('workflow_id', input.workflowId)
        .order('created_at', { ascending: false });

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return data || [];
    }),

  // Get service interface by ID
  get: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .query(async ({ ctx, input }) => {
      const { data, error } = await ctx.supabase
        .from('service_interfaces')
        .select(`
          *,
          workflow:workflows!inner(id, created_by)
        `)
        .eq('id', input.id)
        .single();

      if (error || !data) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Service interface not found',
        });
      }

      // Verify ownership
      const workflow = data.workflow as any;
      if (workflow.created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'Access denied',
        });
      }

      return data;
    }),

  // Create service interface
  create: protectedProcedure
    .input(z.object({
      workflowId: z.string().uuid(),
      name: z.string().min(1).max(255),
      displayName: z.string().min(1).max(255),
      description: z.string().optional(),
      interfaceType: z.enum(['signal', 'query', 'update']),
      callableName: z.string().min(1),
      inputSchema: z.record(z.any()).optional(),
      outputSchema: z.record(z.any()).optional(),
      isPublic: z.boolean().default(false),
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
          message: 'Workflow not found',
        });
      }

      const { data, error } = await ctx.supabase
        .from('service_interfaces')
        .insert({
          workflow_id: input.workflowId,
          name: input.name,
          display_name: input.displayName,
          description: input.description || null,
          interface_type: input.interfaceType,
          callable_name: input.callableName,
          input_schema: input.inputSchema || null,
          output_schema: input.outputSchema || null,
          is_public: input.isPublic,
        })
        .select()
        .single();

      if (error) {
        if (error.code === '23505') {
          throw new TRPCError({
            code: 'CONFLICT',
            message: 'Service interface with this name already exists',
          });
        }
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      // If marked as public, sync with Kong
      if (input.isPublic) {
        try {
          const supabase = await createClient();
          await syncPublicInterfacesForServiceInterface(data.id, supabase);
        } catch (syncError) {
          console.error('Failed to sync public interface:', syncError);
          // Don't fail the request, just log the error
        }
      }

      return data;
    }),

  // Update service interface
  update: protectedProcedure
    .input(z.object({
      id: z.string().uuid(),
      name: z.string().min(1).max(255).optional(),
      displayName: z.string().min(1).max(255).optional(),
      description: z.string().nullable().optional(),
      interfaceType: z.enum(['signal', 'query', 'update']).optional(),
      callableName: z.string().min(1).optional(),
      inputSchema: z.record(z.any()).nullable().optional(),
      outputSchema: z.record(z.any()).nullable().optional(),
      isPublic: z.boolean().optional(),
    }))
    .mutation(async ({ ctx, input }) => {
      const { id, ...updates } = input;

      // Verify ownership
      const { data: existing } = await ctx.supabase
        .from('service_interfaces')
        .select('workflow:workflows!inner(created_by)')
        .eq('id', id)
        .single();

      if (!existing) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Service interface not found',
        });
      }

      const workflow = existing.workflow as any;
      if (workflow.created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'Access denied',
        });
      }

      // Build update object
      const updateData: Record<string, any> = {};
      if (updates.name !== undefined) updateData.name = updates.name;
      if (updates.displayName !== undefined) updateData.display_name = updates.displayName;
      if (updates.description !== undefined) updateData.description = updates.description;
      if (updates.interfaceType !== undefined) updateData.interface_type = updates.interfaceType;
      if (updates.callableName !== undefined) updateData.callable_name = updates.callableName;
      if (updates.inputSchema !== undefined) updateData.input_schema = updates.inputSchema;
      if (updates.outputSchema !== undefined) updateData.output_schema = updates.outputSchema;
      if (updates.isPublic !== undefined) updateData.is_public = updates.isPublic;

      const { data, error } = await ctx.supabase
        .from('service_interfaces')
        .update(updateData)
        .eq('id', id)
        .select()
        .single();

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      // Sync with Kong if public status changed
      if (updates.isPublic !== undefined) {
        try {
          const supabase = await createClient();
          await syncPublicInterfacesForServiceInterface(data.id, supabase);
        } catch (syncError) {
          console.error('Failed to sync public interface:', syncError);
        }
      }

      return data;
    }),

  // Delete service interface
  delete: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .mutation(async ({ ctx, input }) => {
      // Verify ownership
      const { data: existing } = await ctx.supabase
        .from('service_interfaces')
        .select('workflow:workflows!inner(created_by)')
        .eq('id', input.id)
        .single();

      if (!existing) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Service interface not found',
        });
      }

      const workflow = existing.workflow as any;
      if (workflow.created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'Access denied',
        });
      }

      // Delete (cascade will handle public_interfaces)
      const { error } = await ctx.supabase
        .from('service_interfaces')
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

  // Create service interface from interface component
  createFromComponent: protectedProcedure
    .input(z.object({
      componentId: z.string().uuid(),
      workflowId: z.string().uuid(),
      endpointPath: z.string().min(1),
      httpMethod: z.enum(['GET', 'POST', 'PATCH', 'PUT', 'DELETE']),
      inputSchema: z.record(z.any()).optional(),
      outputSchema: z.record(z.any()).optional(),
      isPublic: z.boolean().default(false),
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
          message: 'Workflow not found',
        });
      }

      // Verify component ownership
      const { data: component } = await ctx.supabase
        .from('components')
        .select('*')
        .eq('id', input.componentId)
        .eq('created_by', ctx.user.id)
        .single();

      if (!component) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Component not found',
        });
      }

      // Create service interface from component
      const serviceInterface = await createServiceInterfaceFromComponent(
        input.componentId,
        input.workflowId,
        {
          endpointPath: input.endpointPath,
          httpMethod: input.httpMethod,
          inputSchema: input.inputSchema,
          outputSchema: input.outputSchema,
          isPublic: input.isPublic,
        },
        ctx.supabase
      );

      // If marked as public, sync with Kong
      if (input.isPublic) {
        try {
          const supabase = await createClient();
          await syncPublicInterfacesForServiceInterface(serviceInterface.id, supabase);
        } catch (syncError) {
          console.error('Failed to sync public interface:', syncError);
          // Don't fail the request, just log the error
        }
      }

      return serviceInterface;
    }),
});

