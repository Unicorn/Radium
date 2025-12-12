/**
 * Project Workers Router
 * 
 * Worker management operations for projects
 * Split from projects.ts for better organization
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';

export const projectWorkersRouter = createTRPCRouter({
  /**
   * Get worker health for a project
   */
  workerHealth: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .query(async ({ ctx, input }) => {
      const userRecord = await ctx.getUserRecord();
      if (!userRecord) {
        throw new TRPCError({
          code: 'UNAUTHORIZED',
          message: 'User record not found',
        });
      }
      
      // Verify user owns this project
      const { data: project } = await ctx.supabase
        .from('projects')
        .select('id, name, task_queue_name')
        .eq('id', input.id)
        .eq('created_by', userRecord.id)
        .single();
      
      if (!project) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Project not found',
        });
      }
      
      // Get most recent worker for this project
      const { data: workers } = await ctx.supabase
        .from('workflow_workers')
        .select('*')
        .eq('project_id', input.id)
        .order('started_at', { ascending: false })
        .limit(1);
      
      const worker = workers?.[0];
      
      if (!worker) {
        return {
          status: 'stopped' as const,
          isHealthy: false,
          workerId: null,
          lastHeartbeat: null,
          taskQueueName: project.task_queue_name,
        };
      }
      
      // Check if worker is healthy (heartbeat within last 60 seconds)
      const lastHeartbeat = worker.last_heartbeat ? new Date(worker.last_heartbeat) : null;
      const isHealthy = lastHeartbeat 
        ? (Date.now() - lastHeartbeat.getTime()) < 60000 
        : false;
      
      return {
        status: worker.status,
        isHealthy,
        workerId: worker.worker_id,
        lastHeartbeat: worker.last_heartbeat,
        taskQueueName: worker.task_queue_name,
        startedAt: worker.started_at,
        host: worker.host,
      };
    }),
  
  /**
   * Start worker for a project
   */
  startWorker: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .mutation(async ({ ctx, input }) => {
      const userRecord = await ctx.getUserRecord();
      if (!userRecord) {
        throw new TRPCError({
          code: 'UNAUTHORIZED',
          message: 'User record not found',
        });
      }
      
      // Verify user owns this project
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
      
      // Check if worker is already running
      const { data: existingWorkers } = await ctx.supabase
        .from('workflow_workers')
        .select('*')
        .eq('project_id', input.id)
        .in('status', ['running', 'starting'])
        .order('started_at', { ascending: false })
        .limit(1);
      
      if (existingWorkers && existingWorkers.length > 0) {
        const worker = existingWorkers[0];
        if (!worker) {
          throw new TRPCError({
            code: 'INTERNAL_SERVER_ERROR',
            message: 'Worker data is invalid',
          });
        }
        const lastHeartbeat = worker.last_heartbeat ? new Date(worker.last_heartbeat) : null;
        const isHealthy = lastHeartbeat 
          ? (Date.now() - lastHeartbeat.getTime()) < 60000 
          : false;
        
        if (isHealthy) {
          return {
            success: true,
            message: 'Worker is already running',
            workerId: worker.worker_id,
          };
        }
      }
      
      // Call worker service via HTTP
      const WORKER_SERVICE_URL = process.env.WORKER_SERVICE_URL || 'http://localhost:3011';
      
      try {
        const response = await fetch(`${WORKER_SERVICE_URL}/workers/start`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ projectId: input.id }),
        });
        
        const data = await response.json();
        
        if (!response.ok) {
          throw new Error(data.error || 'Failed to start worker');
        }
        
        console.log('✅ [Start Worker] Worker started for project:', input.id);
        
        return {
          success: true,
          message: 'Worker started successfully',
        };
      } catch (error) {
        console.error('❌ [Start Worker] Error:', error);
        
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error instanceof Error ? error.message : 'Failed to start worker',
        });
      }
    }),
  
  /**
   * Stop worker for a project
   */
  stopWorker: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .mutation(async ({ ctx, input }) => {
      const userRecord = await ctx.getUserRecord();
      if (!userRecord) {
        throw new TRPCError({
          code: 'UNAUTHORIZED',
          message: 'User record not found',
        });
      }
      
      // Verify user owns this project
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
      
      // Call worker service via HTTP
      const WORKER_SERVICE_URL = process.env.WORKER_SERVICE_URL || 'http://localhost:3011';
      
      try {
        const response = await fetch(`${WORKER_SERVICE_URL}/workers/stop`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ projectId: input.id }),
        });
        
        const data = await response.json();
        
        if (!response.ok) {
          throw new Error(data.error || 'Failed to stop worker');
        }
        
        console.log('✅ [Stop Worker] Worker stopped for project:', input.id);
        
        return {
          success: true,
          message: 'Worker stopped successfully',
        };
      } catch (error) {
        console.error('❌ [Stop Worker] Error:', error);
        
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error instanceof Error ? error.message : 'Failed to stop worker',
        });
      }
    }),
});

