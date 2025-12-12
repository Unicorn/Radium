/**
 * Task Queues router - Manage Temporal task queues
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure, publicProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';

export const taskQueuesRouter = createTRPCRouter({
  // List task queues
  list: publicProcedure
    .query(async ({ ctx }) => {
      const { data, error } = await ctx.supabase
        .from('task_queues')
        .select(`
          *,
          channel_display_name,
          created_by_user:users!task_queues_created_by_fkey(id, display_name)
        `)
        .order('is_system_queue', { ascending: false })
        .order('name');
      
      if (error) {
        throw new TRPCError({ 
          code: 'INTERNAL_SERVER_ERROR', 
          message: error.message 
        });
      }
      
      return data || [];
    }),

  // Get single task queue
  get: publicProcedure
    .input(z.object({ id: z.string().uuid() }))
    .query(async ({ ctx, input }) => {
      const { data, error } = await ctx.supabase
        .from('task_queues')
        .select(`
          *,
          channel_display_name,
          created_by_user:users!task_queues_created_by_fkey(id, display_name, email)
        `)
        .eq('id', input.id)
        .single();
      
      if (error) {
        throw new TRPCError({ 
          code: 'NOT_FOUND', 
          message: 'Task queue not found' 
        });
      }
      
      return data;
    }),

  // Create task queue
  create: protectedProcedure
    .input(z.object({
      name: z.string().min(1).max(255).regex(/^[a-z0-9-]+$/, 'Must be lowercase alphanumeric with hyphens'),
      description: z.string().optional(),
      maxConcurrentWorkflows: z.number().int().min(1).max(1000).default(100),
      maxConcurrentActivities: z.number().int().min(1).max(10000).default(1000),
    }))
    .mutation(async ({ ctx, input }) => {
      const { supabase, user } = ctx;
      
      // Check for duplicate name
      const { data: existing } = await supabase
        .from('task_queues')
        .select('id')
        .eq('name', input.name)
        .single();
      
      if (existing) {
        throw new TRPCError({ 
          code: 'BAD_REQUEST', 
          message: 'Task queue with this name already exists' 
        });
      }
      
      // Create
      const { data, error } = await supabase
        .from('task_queues')
        .insert({
          name: input.name,
          description: input.description,
          max_concurrent_workflows: input.maxConcurrentWorkflows,
          max_concurrent_activities: input.maxConcurrentActivities,
          created_by: user.id,
          is_system_queue: false,
        })
        .select()
        .single();
      
      if (error) {
        throw new TRPCError({ 
          code: 'INTERNAL_SERVER_ERROR', 
          message: error.message 
        });
      }
      
      return data;
    }),

  // Update task queue
  update: protectedProcedure
    .input(z.object({
      id: z.string().uuid(),
      description: z.string().optional(),
      maxConcurrentWorkflows: z.number().int().min(1).max(1000).optional(),
      maxConcurrentActivities: z.number().int().min(1).max(10000).optional(),
    }))
    .mutation(async ({ ctx, input }) => {
      const { id, ...updates } = input;
      
      // Check ownership and not system queue
      const { data: queue } = await ctx.supabase
        .from('task_queues')
        .select('created_by, is_system_queue')
        .eq('id', id)
        .single();
      
      if (!queue) {
        throw new TRPCError({ 
          code: 'NOT_FOUND', 
          message: 'Task queue not found' 
        });
      }
      
      if (queue.is_system_queue) {
        throw new TRPCError({ 
          code: 'FORBIDDEN', 
          message: 'Cannot update system queues' 
        });
      }
      
      if (queue.created_by !== ctx.user.id) {
        throw new TRPCError({ 
          code: 'FORBIDDEN', 
          message: 'Not authorized to update this task queue' 
        });
      }
      
      // Build update
      const updateData: any = {
        updated_at: new Date().toISOString(),
      };
      
      if (updates.description !== undefined) updateData.description = updates.description;
      if (updates.maxConcurrentWorkflows !== undefined) updateData.max_concurrent_workflows = updates.maxConcurrentWorkflows;
      if (updates.maxConcurrentActivities !== undefined) updateData.max_concurrent_activities = updates.maxConcurrentActivities;
      
      const { data, error } = await ctx.supabase
        .from('task_queues')
        .update(updateData)
        .eq('id', id)
        .select()
        .single();
      
      if (error) {
        throw new TRPCError({ 
          code: 'INTERNAL_SERVER_ERROR', 
          message: error.message 
        });
      }
      
      return data;
    }),

  // Delete task queue (prevent deleting default queues)
  delete: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .mutation(async ({ ctx, input }) => {
      // Check ownership and not system/default queue
      const { data: queue } = await ctx.supabase
        .from('task_queues')
        .select('created_by, is_system_queue, is_default')
        .eq('id', input.id)
        .single();
      
      if (!queue) {
        throw new TRPCError({ 
          code: 'NOT_FOUND', 
          message: 'Task queue not found' 
        });
      }
      
      if (queue.is_system_queue) {
        throw new TRPCError({ 
          code: 'FORBIDDEN', 
          message: 'Cannot delete system queues' 
        });
      }
      
      if (queue.is_default) {
        throw new TRPCError({ 
          code: 'FORBIDDEN', 
          message: 'Cannot delete default queue. This is your default queue and cannot be removed.' 
        });
      }
      
      if (queue.created_by !== ctx.user.id) {
        throw new TRPCError({ 
          code: 'FORBIDDEN', 
          message: 'Not authorized to delete this task queue' 
        });
      }
      
      // Check if used by workflows
      const { count } = await ctx.supabase
        .from('workflows')
        .select('*', { count: 'exact', head: true })
        .eq('task_queue_id', input.id);
      
      if (count && count > 0) {
        throw new TRPCError({ 
          code: 'BAD_REQUEST', 
          message: `Task queue is used by ${count} workflow(s). Cannot delete.` 
        });
      }
      
      const { error } = await ctx.supabase
        .from('task_queues')
        .delete()
        .eq('id', input.id);
      
      if (error) {
        throw new TRPCError({ 
          code: 'INTERNAL_SERVER_ERROR', 
          message: error.message 
        });
      }
      
      return { success: true };
    }),
});

