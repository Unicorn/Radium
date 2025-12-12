import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';
import {
  validateWorkQueueConfig,
  generateDefaultSignalName,
  generateDefaultQueryName,
  calculateQueueStatus,
} from '@/utils/work-queue-utils';

/**
 * Work Queues Router
 * 
 * Manages work queues on coordinator workflows
 */
export const workQueuesRouter = createTRPCRouter({
  // List work queues for a workflow
  list: protectedProcedure
    .input(
      z.object({
        workflowId: z.string().uuid(),
      })
    )
    .query(async ({ ctx, input }) => {
      const { data, error } = await ctx.supabase
        .from('workflow_work_queues')
        .select('*')
        .eq('workflow_id', input.workflowId)
        .order('created_at', { ascending: true });

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return {
        workQueues: data || [],
      };
    }),

  // Get single work queue
  get: protectedProcedure
    .input(
      z.object({
        id: z.string().uuid(),
      })
    )
    .query(async ({ ctx, input }) => {
      const { data, error } = await ctx.supabase
        .from('workflow_work_queues')
        .select('*')
        .eq('id', input.id)
        .single();

      if (error) {
        if (error.code === 'PGRST116') {
          throw new TRPCError({
            code: 'NOT_FOUND',
            message: 'Work queue not found',
          });
        }
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return {
        workQueue: data,
      };
    }),

  // Create work queue
  create: protectedProcedure
    .input(
      z.object({
        workflowId: z.string().uuid(),
        queueName: z.string().min(1).max(100),
        description: z.string().optional(),
        signalName: z.string().min(1).max(100).optional(),
        queryName: z.string().min(1).max(100).optional(),
        maxSize: z.number().int().min(1).max(10000).nullable().optional(),
        priority: z.enum(['fifo', 'lifo', 'priority']).default('fifo'),
        deduplicate: z.boolean().default(false),
        workItemSchema: z.record(z.any()).optional(),
      })
    )
    .mutation(async ({ ctx, input }) => {
      // Generate default signal/query names if not provided
      const signalName = input.signalName || generateDefaultSignalName(input.queueName);
      const queryName = input.queryName || generateDefaultQueryName(input.queueName);

      // Validate configuration
      const validation = validateWorkQueueConfig({
        queue_name: input.queueName,
        signal_name: signalName,
        query_name: queryName,
        max_size: input.maxSize || null,
        priority: input.priority,
      });

      if (!validation.valid) {
        throw new TRPCError({
          code: 'BAD_REQUEST',
          message: validation.errors.map(e => e.message).join(', '),
        });
      }

      // Check if queue name already exists for this workflow
      const { data: existing } = await ctx.supabase
        .from('workflow_work_queues')
        .select('id')
        .eq('workflow_id', input.workflowId)
        .eq('queue_name', input.queueName)
        .single();

      if (existing) {
        throw new TRPCError({
          code: 'CONFLICT',
          message: `Work queue '${input.queueName}' already exists for this workflow`,
        });
      }

      // Create work queue
      const { data, error } = await ctx.supabase
        .from('workflow_work_queues')
        .insert({
          workflow_id: input.workflowId,
          queue_name: input.queueName,
          description: input.description,
          signal_name: signalName,
          query_name: queryName,
          max_size: input.maxSize || null,
          priority: input.priority,
          deduplicate: input.deduplicate,
          work_item_schema: input.workItemSchema || null,
          created_by: ctx.user.id,
        })
        .select()
        .single();

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      // Note: Trigger function will auto-create signal and query handlers

      return {
        workQueue: data,
      };
    }),

  // Update work queue
  update: protectedProcedure
    .input(
      z.object({
        id: z.string().uuid(),
        queueName: z.string().min(1).max(100).optional(),
        description: z.string().optional(),
        maxSize: z.number().int().min(1).max(10000).nullable().optional(),
        priority: z.enum(['fifo', 'lifo', 'priority']).optional(),
        deduplicate: z.boolean().optional(),
        workItemSchema: z.record(z.any()).optional(),
      })
    )
    .mutation(async ({ ctx, input }) => {
      const { id, ...updates } = input;

      // If updating queue name, validate uniqueness
      if (updates.queueName) {
        const { data: workQueue } = await ctx.supabase
          .from('workflow_work_queues')
          .select('workflow_id')
          .eq('id', id)
          .single();

        if (workQueue) {
          const { data: existing } = await ctx.supabase
            .from('workflow_work_queues')
            .select('id')
            .eq('workflow_id', workQueue.workflow_id)
            .eq('queue_name', updates.queueName)
            .neq('id', id)
            .single();

          if (existing) {
            throw new TRPCError({
              code: 'CONFLICT',
              message: `Work queue '${updates.queueName}' already exists for this workflow`,
            });
          }
        }
      }

      const { data, error } = await ctx.supabase
        .from('workflow_work_queues')
        .update({
          queue_name: updates.queueName,
          description: updates.description,
          max_size: updates.maxSize,
          priority: updates.priority,
          deduplicate: updates.deduplicate,
          work_item_schema: updates.workItemSchema,
          updated_at: new Date().toISOString(),
        })
        .eq('id', id)
        .select()
        .single();

      if (error) {
        if (error.code === 'PGRST116') {
          throw new TRPCError({
            code: 'NOT_FOUND',
            message: 'Work queue not found',
          });
        }
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return {
        workQueue: data,
      };
    }),

  // Delete work queue
  delete: protectedProcedure
    .input(
      z.object({
        id: z.string().uuid(),
      })
    )
    .mutation(async ({ ctx, input }) => {
      // Check if any nodes reference this queue
      const { data: referencingNodes } = await ctx.supabase
        .from('workflow_nodes')
        .select('id, label')
        .or(`work_queue_target.eq.${input.id},block_until_queue.eq.${input.id}`)
        .limit(5);

      if (referencingNodes && referencingNodes.length > 0) {
        const nodeLabels = referencingNodes.map(n => n.label).join(', ');
        throw new TRPCError({
          code: 'CONFLICT',
          message: `Cannot delete work queue because it is referenced by nodes: ${nodeLabels}`,
        });
      }

      const { error } = await ctx.supabase
        .from('workflow_work_queues')
        .delete()
        .eq('id', input.id);

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return {
        success: true,
      };
    }),

  // Get work queue with its signal and query handlers
  getWithHandlers: protectedProcedure
    .input(
      z.object({
        id: z.string().uuid(),
      })
    )
    .query(async ({ ctx, input }) => {
      // Get work queue
      const { data: workQueue, error: queueError } = await ctx.supabase
        .from('workflow_work_queues')
        .select('*')
        .eq('id', input.id)
        .single();

      if (queueError) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Work queue not found',
        });
      }

      // Get associated signal handler
      const { data: signal } = await ctx.supabase
        .from('workflow_signals')
        .select('*')
        .eq('work_queue_id', input.id)
        .single();

      // Get associated query handler
      const { data: query } = await ctx.supabase
        .from('workflow_queries')
        .select('*')
        .eq('work_queue_id', input.id)
        .single();

      return {
        workQueue,
        signal: signal || null,
        query: query || null,
      };
    }),

  // Validate work queue configuration
  validate: protectedProcedure
    .input(
      z.object({
        queueName: z.string(),
        signalName: z.string(),
        queryName: z.string(),
        maxSize: z.number().nullable().optional(),
        priority: z.enum(['fifo', 'lifo', 'priority']).optional(),
      })
    )
    .mutation(async ({ input }) => {
      const validation = validateWorkQueueConfig({
        queue_name: input.queueName,
        signal_name: input.signalName,
        query_name: input.queryName,
        max_size: input.maxSize || null,
        priority: input.priority,
      });

      return validation;
    }),
});

