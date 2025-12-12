/**
 * Workflows router - CRUD and deployment operations
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';

export const workflowsRouter = createTRPCRouter({
  // List workflows
  list: protectedProcedure
    .input(z.object({
      status: z.string().optional(),
      page: z.number().default(1),
      pageSize: z.number().default(20),
    }))
    .query(async ({ ctx, input }) => {
      // Get public visibility ID
      const { data: publicVisibility } = await ctx.supabase
        .from('component_visibility')
        .select('id')
        .eq('name', 'public')
        .single();
      
      // Get system user ID to exclude system workflows (like agent tester)
      const { data: systemUser } = await ctx.supabase
        .from('users')
        .select('id')
        .eq('email', 'system@example.com')
        .single();
      
      let query = ctx.supabase
        .from('workflows')
        .select(`
          *,
          
          status:workflow_statuses(id, name, color),
          task_queue:task_queues(id, name),
          created_by_user:users!workflows_created_by_fkey(id, display_name)
        `, { count: 'exact' });
      
      // Only show user's own workflows (not public, not system workflows)
      query = query.eq('created_by', ctx.user.id);
      
      // Exclude system workflows (agent tester, etc.)
      if (systemUser) {
        query = query.neq('created_by', systemUser.id);
      }
      
      // Exclude archived workflows
      query = query.eq('is_archived', false);
      
      if (input.status) {
        const { data: statusData } = await ctx.supabase
          .from('workflow_statuses')
          .select('id')
          .eq('name', input.status)
          .single();
        
        if (statusData) {
          query = query.eq('status_id', statusData.id);
        }
      }
      
      const from = (input.page - 1) * input.pageSize;
      const to = from + input.pageSize - 1;
      
      const { data, error, count } = await query
        .range(from, to)
        .order('updated_at', { ascending: false });
      
      if (error) {
        throw new TRPCError({ 
          code: 'INTERNAL_SERVER_ERROR', 
          message: error.message 
        });
      }
      
      return {
        workflows: data || [],
        total: count || 0,
        page: input.page,
        pageSize: input.pageSize,
        totalPages: Math.ceil((count || 0) / input.pageSize),
      };
    }),

  // Get workflow by ID
  get: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .query(async ({ ctx, input }) => {
      const { data, error } = await ctx.supabase
        .from('workflows')
        .select(`
          *,
          
          status:workflow_statuses(id, name, color, description),
          task_queue:task_queues(id, name,  description),
          created_by_user:users!workflows_created_by_fkey(id, display_name, email),
          visibility:component_visibility(id, name),
          project:projects(id, name, description, task_queue_name),
          nodes:workflow_nodes(*),
          edges:workflow_edges(*)
        `)
        .eq('id', input.id)
        .single();
      
      if (error) {
        throw new TRPCError({ 
          code: 'NOT_FOUND', 
          message: 'Workflow not found' 
        });
      }
      
      // Check access
      const { data: publicVisibility } = await ctx.supabase
        .from('component_visibility')
        .select('id')
        .eq('name', 'public')
        .single();
      
      if (data.created_by !== ctx.user.id && data.visibility_id !== publicVisibility?.id) {
        throw new TRPCError({ 
          code: 'FORBIDDEN', 
          message: 'Not authorized to view this workflow' 
        });
      }
      
      return { workflow: data };
    }),

  // Create workflow
  create: protectedProcedure
    .input(z.object({
      kebabName: z.string()
        .min(1, 'Kebab name is required')
        .max(255)
        .regex(/^[a-z0-9]+(-[a-z0-9]+)*$/, 'Kebab name must contain only lowercase letters, numbers, and hyphens'),
      displayName: z.string().min(1).max(255),
      description: z.string().optional(),
      visibility: z.enum(['public', 'private', 'organization']),
      projectId: z.string().uuid(),
      taskQueueId: z.string().uuid().optional(), // If provided, skip lookup
      definition: z.any().optional(), // React Flow JSON
    }))
    .mutation(async ({ ctx, input }) => {
      // Get status ID for 'draft'
      const { data: status } = await ctx.supabase
        .from('workflow_statuses')
        .select('id')
        .eq('name', 'draft')
        .single();
      
      if (!status) {
        throw new TRPCError({ 
          code: 'INTERNAL_SERVER_ERROR', 
          message: 'Draft status not found' 
        });
      }
      
      // Get visibility ID
      const { data: visibility } = await ctx.supabase
        .from('component_visibility')
        .select('id')
        .eq('name', input.visibility)
        .single();
      
      if (!visibility) {
        throw new TRPCError({ 
          code: 'BAD_REQUEST', 
          message: 'Invalid visibility' 
        });
      }
      
      // Check for duplicate kebab_name (unique constraint on created_by + kebab_name)
      const { data: existing } = await ctx.supabase
        .from('workflows')
        .select('id')
        .eq('kebab_name', input.kebabName)
        .eq('created_by', ctx.user.id)
        .single();
      
      if (existing) {
        throw new TRPCError({ 
          code: 'BAD_REQUEST', 
          message: 'A workflow with this identifier already exists. Please choose a different identifier.' 
        });
      }
      
      // Verify project ownership
      const { data: project } = await ctx.supabase
        .from('projects')
        .select('id, task_queue_name')
        .eq('id', input.projectId)
        .eq('created_by', ctx.user.id)
        .single();
      
      if (!project) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Project not found',
        });
      }
      
      // Use provided taskQueueId or look it up from project
      let taskQueueId: string;
      
      if (input.taskQueueId) {
        // Task queue ID provided directly (e.g., from project creation)
        taskQueueId = input.taskQueueId;
        console.log('✅ [Workflow Create] Using provided task queue:', taskQueueId);
      } else {
        // Look up task queue from project
        let { data: taskQueue } = await ctx.supabase
          .from('task_queues')
          .select('id')
          .eq('name', project.task_queue_name)
          .single();
        
        // If task queue doesn't exist, create it
        if (!taskQueue) {
          console.log('⚠️  [Workflow Create] Task queue not found, creating:', project.task_queue_name);
          const { data: newTaskQueue, error: createError } = await ctx.supabase
            .from('task_queues')
            .insert({
              name: project.task_queue_name,
              description: `Task queue for project ${project.id}`,
              created_by: ctx.user.id,
              is_system_queue: false,
              max_concurrent_workflows: 100,
              max_concurrent_activities: 1000,
            })
            .select('id')
            .single();
          
          if (createError || !newTaskQueue) {
            console.error('❌ [Workflow Create] Failed to create task queue:', createError);
            throw new TRPCError({
              code: 'INTERNAL_SERVER_ERROR',
              message: `Failed to create task queue: ${createError?.message || 'Unknown error'}`,
            });
          }
          
          taskQueue = newTaskQueue;
          console.log('✅ [Workflow Create] Created task queue:', taskQueue.id);
        }
        
        taskQueueId = taskQueue.id;
        console.log('✅ [Workflow Create] Using task queue:', taskQueueId);
      }
      
      // Create workflow
      const { data: workflow, error } = await ctx.supabase
        .from('workflows')
        .insert({
          name: input.kebabName, // 'name' column is required - use kebab_name as the name
          kebab_name: input.kebabName,
          display_name: input.displayName,
          description: input.description,
          created_by: ctx.user.id,
          visibility_id: visibility.id,
          status_id: status.id,
          task_queue_id: taskQueueId,
          project_id: project.id,
          definition: input.definition || { nodes: [], edges: [] },
        })
        .select()
        .single();
      
      if (error) {
        throw new TRPCError({ 
          code: 'INTERNAL_SERVER_ERROR', 
          message: error.message 
        });
      }
      
      return workflow;
    }),

  // Update workflow
  update: protectedProcedure
    .input(z.object({
      id: z.string().uuid(),
      displayName: z.string().min(1).max(255).optional(),
      description: z.string().optional(),
      definition: z.any().optional(),
      taskQueueId: z.string().uuid().optional(),
    }))
    .mutation(async ({ ctx, input }) => {
      const { id, ...updates } = input;
      
      // Check ownership
      const { data: workflow } = await ctx.supabase
        .from('workflows')
        .select('created_by')
        .eq('id', id)
        .single();
      
      if (!workflow || workflow.created_by !== ctx.user.id) {
        throw new TRPCError({ 
          code: 'FORBIDDEN', 
          message: 'Not authorized to update this workflow' 
        });
      }
      
      const updateData: any = {
        updated_at: new Date().toISOString(),
      };
      
      if (updates.displayName !== undefined) updateData.display_name = updates.displayName;
      if (updates.description !== undefined) updateData.description = updates.description;
      if (updates.definition !== undefined) updateData.definition = updates.definition;
      if (updates.taskQueueId !== undefined) updateData.task_queue_id = updates.taskQueueId;
      
      const { data, error } = await ctx.supabase
        .from('workflows')
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

  // Deploy workflow (change status to active)
  deploy: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .mutation(async ({ ctx, input }) => {
      // Get workflow
      const { data: workflow } = await ctx.supabase
        .from('workflows')
        .select('*')
        .eq('id', input.id)
        .single();
      
      if (!workflow || workflow.created_by !== ctx.user.id) {
        throw new TRPCError({ 
          code: 'FORBIDDEN', 
          message: 'Not authorized to deploy this workflow' 
        });
      }
      
      // Validate workflow has nodes
      if (!workflow.definition || 
          typeof workflow.definition !== 'object' ||
          !('nodes' in workflow.definition) ||
          !Array.isArray((workflow.definition as any).nodes) ||
          (workflow.definition as any).nodes.length === 0) {
        throw new TRPCError({ 
          code: 'BAD_REQUEST', 
          message: 'Cannot deploy empty workflow' 
        });
      }
      
      // Get active status
      const { data: activeStatus } = await ctx.supabase
        .from('workflow_statuses')
        .select('id')
        .eq('name', 'active')
        .single();
      
      if (!activeStatus) {
        throw new TRPCError({ 
          code: 'INTERNAL_SERVER_ERROR', 
          message: 'Active status not found' 
        });
      }
      
      // TODO: Generate TypeScript code (Phase 4)
      // TODO: Register with Temporal (Phase 4)
      
      // Extract API endpoint nodes from workflow definition
      const definition = workflow.definition as any;
      const endpointNodes = (definition?.nodes || []).filter(
        (node: any) => node.type === 'api-endpoint' && node.data?.config
      );

      // Register endpoints with Kong if any exist
      let registeredEndpoints: any[] = [];
      if (endpointNodes.length > 0) {
        try {
          const { registerWorkflowEndpoints } = await import('@/lib/kong/endpoint-registry');
          
          // Get project ID
          const { data: workflowWithProject } = await ctx.supabase
            .from('workflows')
            .select('project_id')
            .eq('id', input.id)
            .single();

          if (workflowWithProject?.project_id) {
            const endpoints = endpointNodes.map((node: any) => ({
              endpointPath: node.data.config.endpointPath || '/',
              method: node.data.config.method || 'POST',
              description: node.data.config.description,
              targetType: node.data.config.targetType || 'start',
              targetName: node.data.config.targetName || 'start',
              authType: node.data.config.authType || 'api-key',
              rateLimitPerMinute: node.data.config.rateLimitPerMinute || 60,
              rateLimitPerHour: node.data.config.rateLimitPerHour || 1000,
            }));

            registeredEndpoints = await registerWorkflowEndpoints(
              input.id,
              ctx.user.id,
              workflowWithProject.project_id,
              endpoints,
              ctx.supabase
            );

            console.log(`✅ Registered ${registeredEndpoints.length} API endpoints`);
          }
        } catch (error) {
          console.error('⚠️  Failed to register API endpoints:', error);
          // Don't fail deployment if endpoint registration fails
        }
      }
      
      // Update status
      const { data, error } = await ctx.supabase
        .from('workflows')
        .update({
          status_id: activeStatus.id,
          temporal_workflow_id: `${workflow.kebab_name}-${Date.now()}`,
          temporal_workflow_type: workflow.kebab_name,
          deployed_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        })
        .eq('id', input.id)
        .select()
        .single();
      
      if (error) {
        throw new TRPCError({ 
          code: 'INTERNAL_SERVER_ERROR', 
          message: error.message 
        });
      }
      
      return {
        ...data,
        endpoints: registeredEndpoints,
      };
    }),

  // Pause workflow
  pause: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .mutation(async ({ ctx, input }) => {
      const { data: pausedStatus } = await ctx.supabase
        .from('workflow_statuses')
        .select('id')
        .eq('name', 'paused')
        .single();
      
      if (!pausedStatus) {
        throw new TRPCError({ 
          code: 'INTERNAL_SERVER_ERROR', 
          message: 'Paused status not found' 
        });
      }
      
      const { data, error } = await ctx.supabase
        .from('workflows')
        .update({
          status_id: pausedStatus.id,
          updated_at: new Date().toISOString(),
        })
        .eq('id', input.id)
        .eq('created_by', ctx.user.id)
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

  // Archive workflow (replaces delete)
  archive: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .mutation(async ({ ctx, input }) => {
      // Check ownership
      const { data: workflow } = await ctx.supabase
        .from('workflows')
        .select('created_by, is_archived')
        .eq('id', input.id)
        .single();
      
      if (!workflow || workflow.created_by !== ctx.user.id) {
        throw new TRPCError({ 
          code: 'FORBIDDEN', 
          message: 'Not authorized to archive this workflow' 
        });
      }
      
      if (workflow.is_archived) {
        throw new TRPCError({ 
          code: 'BAD_REQUEST', 
          message: 'Workflow is already archived' 
        });
      }
      
      const { error } = await ctx.supabase
        .from('workflows')
        .update({ 
          is_archived: true,
          updated_at: new Date().toISOString()
        })
        .eq('id', input.id);
      
      if (error) {
        throw new TRPCError({ 
          code: 'INTERNAL_SERVER_ERROR', 
          message: error.message 
        });
      }
      
      return { success: true };
    }),

  // Unarchive workflow
  unarchive: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .mutation(async ({ ctx, input }) => {
      // Check ownership
      const { data: workflow } = await ctx.supabase
        .from('workflows')
        .select('created_by, is_archived')
        .eq('id', input.id)
        .single();
      
      if (!workflow || workflow.created_by !== ctx.user.id) {
        throw new TRPCError({ 
          code: 'FORBIDDEN', 
          message: 'Not authorized to unarchive this workflow' 
        });
      }
      
      if (!workflow.is_archived) {
        throw new TRPCError({ 
          code: 'BAD_REQUEST', 
          message: 'Workflow is not archived' 
        });
      }
      
      const { error } = await ctx.supabase
        .from('workflows')
        .update({ 
          is_archived: false,
          updated_at: new Date().toISOString()
        })
        .eq('id', input.id);
      
      if (error) {
        throw new TRPCError({ 
          code: 'INTERNAL_SERVER_ERROR', 
          message: error.message 
        });
      }
      
      return { success: true };
    }),

  // Get workflow statuses (for dropdowns)
  getStatuses: protectedProcedure
    .query(async ({ ctx }) => {
      const { data, error } = await ctx.supabase
        .from('workflow_statuses')
        .select('*')
        .order('name');
      
      if (error) {
        throw new TRPCError({ 
          code: 'INTERNAL_SERVER_ERROR', 
          message: error.message 
        });
      }
      
      return data;
    }),
});

