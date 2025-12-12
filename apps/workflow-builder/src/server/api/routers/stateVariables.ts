/**
 * State Variables Router
 * 
 * tRPC router for managing state variables (project-level and workflow-level)
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';
import {
  getProjectStateVariables,
  createProjectStateVariable,
  updateProjectStateVariable,
  deleteProjectStateVariable,
  getProjectStateVariable,
} from '@/lib/state/project-state-manager';

const stateVariableTypeSchema = z.enum(['string', 'number', 'boolean', 'object', 'array']);
const storageTypeSchema = z.enum(['workflow', 'database', 'redis', 'external']);

export const stateVariablesRouter = createTRPCRouter({
  // List project-level state variables
  listProject: protectedProcedure
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

      return getProjectStateVariables(input.projectId, ctx.supabase);
    }),

  // Get project-level state variable by ID
  getProject: protectedProcedure
    .input(z.object({
      variableId: z.string().uuid(),
    }))
    .query(async ({ ctx, input }) => {
      const { data: variable, error } = await ctx.supabase
        .from('project_state_variables')
        .select(`
          *,
          project:projects(id, created_by)
        `)
        .eq('id', input.variableId)
        .single();

      if (error || !variable) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'State variable not found',
        });
      }

      // Verify ownership
      if ((variable.project as any)?.created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'You do not have access to this state variable',
        });
      }

      const { project, ...variableData } = variable;
      return variableData;
    }),

  // Create project-level state variable
  createProject: protectedProcedure
    .input(z.object({
      projectId: z.string().uuid(),
      name: z.string().min(1),
      type: stateVariableTypeSchema,
      storageType: storageTypeSchema,
      storageConfig: z.record(z.any()).optional(),
      schema: z.record(z.any()).optional(),
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

      return createProjectStateVariable(
        input.projectId,
        {
          name: input.name,
          type: input.type,
          storage_type: input.storageType,
          storage_config: input.storageConfig,
          schema: input.schema,
        },
        ctx.supabase
      );
    }),

  // Update project-level state variable
  updateProject: protectedProcedure
    .input(z.object({
      variableId: z.string().uuid(),
      name: z.string().min(1).optional(),
      type: stateVariableTypeSchema.optional(),
      storageType: storageTypeSchema.optional(),
      storageConfig: z.record(z.any()).optional(),
      schema: z.record(z.any()).optional(),
    }))
    .mutation(async ({ ctx, input }) => {
      const { variableId, ...updates } = input;

      // Get and verify ownership
      const { data: variable, error: getError } = await ctx.supabase
        .from('project_state_variables')
        .select(`
          id,
          project:projects(id, created_by)
        `)
        .eq('id', variableId)
        .single();

      if (getError || !variable) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'State variable not found',
        });
      }

      if ((variable.project as any)?.created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'You do not have access to this state variable',
        });
      }

      const updateData: any = {};
      if (updates.name !== undefined) updateData.name = updates.name;
      if (updates.type !== undefined) updateData.type = updates.type;
      if (updates.storageType !== undefined) updateData.storage_type = updates.storageType;
      if (updates.storageConfig !== undefined) updateData.storage_config = updates.storageConfig;
      if (updates.schema !== undefined) updateData.schema = updates.schema;

      return updateProjectStateVariable(variableId, updateData, ctx.supabase);
    }),

  // Delete project-level state variable
  deleteProject: protectedProcedure
    .input(z.object({
      variableId: z.string().uuid(),
    }))
    .mutation(async ({ ctx, input }) => {
      // Get and verify ownership
      const { data: variable, error: getError } = await ctx.supabase
        .from('project_state_variables')
        .select(`
          id,
          project:projects(id, created_by)
        `)
        .eq('id', input.variableId)
        .single();

      if (getError || !variable) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'State variable not found',
        });
      }

      if ((variable.project as any)?.created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'You do not have access to this state variable',
        });
      }

      await deleteProjectStateVariable(input.variableId, ctx.supabase);
      return { success: true };
    }),

  // List workflow-level state variables
  listWorkflow: protectedProcedure
    .input(z.object({
      workflowId: z.string().uuid(),
    }))
    .query(async ({ ctx, input }) => {
      // Verify ownership
      const { data: workflow, error: workflowError } = await ctx.supabase
        .from('workflows')
        .select(`
          id,
          project:projects(id, created_by)
        `)
        .eq('id', input.workflowId)
        .single();

      if (workflowError || !workflow) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Workflow not found',
        });
      }

      if ((workflow.project as any)?.created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'You do not have access to this workflow',
        });
      }

      const { data, error } = await ctx.supabase
        .from('workflow_state_variables')
        .select('*')
        .eq('workflow_id', input.workflowId)
        .order('name');

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return data || [];
    }),
});

