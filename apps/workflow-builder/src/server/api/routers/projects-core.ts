/**
 * Projects Core Router
 * 
 * Core CRUD operations for projects
 * Split from projects.ts for better organization
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';
import type { ProjectWithRelations } from '@/lib/supabase/types';

export const projectsCoreRouter = createTRPCRouter({
  /**
   * List user's projects
   */
  list: protectedProcedure.query(async ({ ctx }) => {
    const userRecord = await ctx.getUserRecord();
    if (!userRecord) {
      throw new TRPCError({
        code: 'UNAUTHORIZED',
        message: 'User record not found',
      });
    }
    
    console.log('📋 [Projects List] Fetching projects for user:', (userRecord as any).id);
    
    const { data, error } = await ctx.supabase
      .from('projects')
      .select(`
        *,
        workflow_count:workflows(count),
        active_workers:workflow_workers(id, status, last_heartbeat)
      `)
      .eq('created_by', (userRecord as any).id)
      .eq('is_archived', false)
      .order('updated_at', { ascending: false }) as any;
    
    if (error) {
      console.error('❌ [Projects List] Error:', {
        code: error.code,
        message: error.message,
        details: error.details,
        hint: error.hint,
      });
      throw new TRPCError({
        code: 'INTERNAL_SERVER_ERROR',
        message: 'Failed to list projects',
      });
    }
    
    console.log('✅ [Projects List] Found', data?.length || 0, 'projects');
    
    // Type assertion for complex query result
    return { projects: (data || []) as ProjectWithRelations[] };
  }),
  
  /**
   * Get project by ID with full details
   */
  get: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .query(async ({ ctx, input }) => {
      const userRecord = await ctx.getUserRecord();
      if (!userRecord) {
        throw new TRPCError({
          code: 'UNAUTHORIZED',
          message: 'User record not found',
        });
      }
      
      console.log('🔍 [Projects Get] Looking for project:', {
        projectId: input.id,
        userId: (userRecord as any).id,
        authUserId: ctx.authUser?.id,
      });
      
      const { data, error } = await ctx.supabase
        .from('projects')
        .select(`
          *,
          workflows:workflows(id, name, display_name, status_id),
          workers:workflow_workers(id, worker_id, status, last_heartbeat, started_at)
        `)
        .eq('id', input.id)
        .eq('created_by', (userRecord as any).id)
        .single() as any;
      
      if (error) {
        console.error('❌ [Projects Get] Supabase error:', {
          code: error.code,
          message: error.message,
          details: error.details,
          hint: error.hint,
        });
      }
      
      if (error || !data) {
        // Check if project exists but user doesn't have access
        const { data: projectExists } = await ctx.supabase
          .from('projects')
          .select('id, name, created_by')
          .eq('id', input.id)
          .single() as any;
        
        if (projectExists) {
          console.error('❌ [Projects Get] Project exists but access denied:', {
            projectId: input.id,
            projectName: (projectExists as any).name,
            projectCreatedBy: (projectExists as any).created_by,
            currentUserId: (userRecord as any).id,
            authUserId: ctx.authUser?.id,
          });
        }
        
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Project not found',
        });
      }
      
      // Type assertion for complex query result
      const project = data as ProjectWithRelations;
      
      console.log('✅ [Projects Get] Project found:', {
        id: project.id,
        name: project.name,
      });
      
      return { project };
    }),
  
  /**
   * Create new project
   * Automatically generates task queue name: {user_id}-{project_id}
   */
  create: protectedProcedure
    .input(z.object({
      name: z.string().min(1).max(255),
      description: z.string().optional(),
    }))
    .mutation(async ({ ctx, input }) => {
      const userRecord = await ctx.getUserRecord();
      if (!userRecord) {
        throw new TRPCError({
          code: 'UNAUTHORIZED',
          message: 'User record not found',
        });
      }
      
      // Generate kebab-case from project name
      const projectKebab = input.name
        .trim()
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, '-')
        .replace(/^-+|-+$/g, '');
      
      // Generate task queue name
      // Format: {userId_prefix}-{project-kebab-name}-queue
      const userIdPrefix = (userRecord as any).id.split('-')[0];
      const taskQueueName = `${userIdPrefix}-${projectKebab}-queue`;
      
      // Create project
      const { data: project, error: projectError } = await (ctx.supabase
        .from('projects')
        .insert({
          name: input.name,
          description: input.description || null,
          created_by: (userRecord as any).id,
          task_queue_name: taskQueueName,
          is_active: true,
        } as any)
        .select()
        .single() as any);
      
      if (projectError) {
        console.error('❌ [Project Creation] Error:', {
          code: projectError.code,
          message: projectError.message,
          details: projectError.details,
          hint: projectError.hint,
        });
        
        if (projectError.code === '23505') { // Unique violation
          // Check which constraint was violated
          const errorMessage = projectError.message || '';
          if (errorMessage.includes('unique_user_project_name')) {
            throw new TRPCError({
              code: 'CONFLICT',
              message: 'You already have a project with this name. Please choose a different name.',
            });
          } else if (errorMessage.includes('task_queue_name')) {
            throw new TRPCError({
              code: 'CONFLICT',
              message: 'Task queue name conflict. Please try again or contact support.',
            });
          } else {
            throw new TRPCError({
              code: 'CONFLICT',
              message: `Project already exists: ${errorMessage}`,
            });
          }
        }
        
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: `Failed to create project: ${projectError.message}`,
        });
      }
      
      // Auto-create default task queue for the project
      const { data: taskQueue, error: queueError } = await (ctx.supabase
        .from('task_queues')
        .insert({
          name: taskQueueName,
          display_name: `${input.name} Task Queue`,
          description: `Default task queue for ${input.name} project`,
          created_by: (userRecord as any).id,
        } as any)
        .select()
        .single() as any);
      
      if (queueError || !taskQueue) {
        console.error('Error creating task queue:', queueError);
        // If task queue creation fails, delete the project and fail
        await ctx.supabase
          .from('projects')
          .delete()
          .eq('id', (project as any).id);
        
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: 'Failed to create project task queue',
        });
      }
      
      console.log('✅ [Project Created]', { projectId: (project as any).id, queueId: (taskQueue as any).id, queueName: taskQueueName });
      
      return { 
        project: project as ProjectWithRelations,
        taskQueue: taskQueue as any,
      };
    }),
  
  /**
   * Update project
   */
  update: protectedProcedure
    .input(z.object({
      id: z.string().uuid(),
      name: z.string().min(1).max(255).optional(),
      description: z.string().optional(),
      is_active: z.boolean().optional(),
    }))
    .mutation(async ({ ctx, input }) => {
      const userRecord = await ctx.getUserRecord();
      if (!userRecord) {
        throw new TRPCError({
          code: 'UNAUTHORIZED',
          message: 'User record not found',
        });
      }
      
      // Check if project exists and user owns it
      const { data: project } = await ctx.supabase
        .from('projects')
        .select('id, created_by, is_default')
        .eq('id', input.id)
        .single() as any;
      
      if (!project || (project as any).created_by !== (userRecord as any).id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'Not authorized to update this project',
        });
      }
      
      const updates: Record<string, any> = {
        updated_at: new Date().toISOString(),
      };
      
      if (input.name !== undefined) {
        updates.name = input.name;
      }
      
      if (input.description !== undefined) {
        updates.description = input.description;
      }
      
      if (input.is_active !== undefined) {
        updates.is_active = input.is_active;
      }
      
      const { data, error } = await ((ctx.supabase as any)
        .from('projects')
        .update(updates)
        .eq('id', input.id)
        .select()
        .single());
      
      if (error) {
        if (error.code === '23505') {
          throw new TRPCError({
            code: 'CONFLICT',
            message: 'A project with this name already exists',
          });
        }
        
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }
      
      return { project: data };
    }),
  
  /**
   * Archive project (replaces delete)
   * Archives all workflows in the project as well
   */
  archive: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .mutation(async ({ ctx, input }) => {
      const userRecord = await ctx.getUserRecord();
      if (!userRecord) {
        throw new TRPCError({
          code: 'UNAUTHORIZED',
          message: 'User record not found',
        });
      }
      
      // Check if project exists and user owns it
      const { data: project } = await ctx.supabase
        .from('projects')
        .select('id, created_by, is_default, is_archived')
        .eq('id', input.id)
        .single() as any;
      
      if (!project || (project as any).created_by !== (userRecord as any).id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'Not authorized to archive this project',
        });
      }
      
      // Prevent archiving default project
      if (project.is_default) {
        throw new TRPCError({
          code: 'BAD_REQUEST',
          message: 'Cannot archive default project. You can rename it, but not archive it.',
        });
      }
      
      if (project.is_archived) {
        throw new TRPCError({
          code: 'BAD_REQUEST',
          message: 'Project is already archived',
        });
      }
      
      // First check if any workers are running
      const { data: workers } = await ctx.supabase
        .from('workflow_workers')
        .select('id, status')
        .eq('project_id', input.id)
        .in('status', ['starting', 'running']);
      
      if (workers && workers.length > 0) {
        throw new TRPCError({
          code: 'PRECONDITION_FAILED',
          message: 'Cannot archive project with running workers. Stop all workers first.',
        });
      }
      
      // Archive all workflows in the project
      await ((ctx.supabase as any)
        .from('workflows')
        .update({ 
          is_archived: true,
          updated_at: new Date().toISOString()
        })
        .eq('project_id', input.id)
        .eq('is_archived', false));
      
      // Archive the project
      const { error } = await ((ctx.supabase as any)
        .from('projects')
        .update({ 
          is_archived: true,
          updated_at: new Date().toISOString()
        })
        .eq('id', input.id));
      
      if (error) {
        console.error('Error archiving project:', error);
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: 'Failed to archive project',
        });
      }
      
      return { success: true };
    }),

  /**
   * Unarchive project
   */
  unarchive: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .mutation(async ({ ctx, input }) => {
      const userRecord = await ctx.getUserRecord();
      if (!userRecord) {
        throw new TRPCError({
          code: 'UNAUTHORIZED',
          message: 'User record not found',
        });
      }
      
      // Check if project exists and user owns it
      const { data: project } = await ctx.supabase
        .from('projects')
        .select('id, created_by, is_archived')
        .eq('id', input.id)
        .single() as any;
      
      if (!project || (project as any).created_by !== (userRecord as any).id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'Not authorized to unarchive this project',
        });
      }
      
      if (!project.is_archived) {
        throw new TRPCError({
          code: 'BAD_REQUEST',
          message: 'Project is not archived',
        });
      }
      
      // Unarchive the project
      const { error } = await ((ctx.supabase as any)
        .from('projects')
        .update({ 
          is_archived: false,
          updated_at: new Date().toISOString()
        })
        .eq('id', input.id));
      
      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: 'Failed to unarchive project',
        });
      }
      
      return { success: true };
    }),
});

