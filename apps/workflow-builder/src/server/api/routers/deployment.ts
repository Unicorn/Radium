/**
 * Deployment tRPC Router
 * API endpoints for deploying compiled workflows to workers
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';
import { DeploymentService } from '@/lib/deployment/deployment-service';

/**
 * Deployment Router
 */
export const deploymentRouter = createTRPCRouter({
  /**
   * Deploy compiled workflow
   */
  deploy: protectedProcedure
    .input(z.object({
      workflowId: z.string(),
      forceRedeploy: z.boolean().default(false),
    }))
    .mutation(async ({ ctx, input }) => {
      // 1. Get workflow from database
      const { data: workflow, error } = await ctx.supabase
        .from('workflows')
        .select('*')
        .eq('id', input.workflowId)
        .eq('created_by', ctx.user.id)
        .single();

      if (error || !workflow) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Workflow not found or not authorized',
        });
      }

      // 2. Check if workflow has been compiled
      if (!workflow.compiled_typescript) {
        throw new TRPCError({
          code: 'BAD_REQUEST',
          message: 'Workflow has not been compiled yet. Compile it first before deploying.',
        });
      }

      // 3. Parse compiled code (stored as JSON with all files)
      let code;
      try {
        // If compiled_typescript is a string, it's the workflow code only
        // We need to get the full compilation result
        if (typeof workflow.compiled_typescript === 'string') {
          throw new TRPCError({
            code: 'BAD_REQUEST',
            message: 'Workflow compilation is incomplete. Please recompile the workflow.',
          });
        }

        code = workflow.compiled_typescript as {
          workflowCode: string;
          activitiesCode: string;
          workerCode: string;
        };

        // Validate code structure
        if (!code.workflowCode || !code.activitiesCode || !code.workerCode) {
          throw new TRPCError({
            code: 'BAD_REQUEST',
            message: 'Compiled code is missing required components. Please recompile the workflow.',
          });
        }
      } catch (parseError) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: `Failed to parse compiled code: ${parseError instanceof Error ? parseError.message : 'Unknown error'}`,
        });
      }

      // 4. Validate code before deployment
      const deploymentService = new DeploymentService();
      const validation = deploymentService.validateWorkflowCode(code);

      if (!validation.valid) {
        throw new TRPCError({
          code: 'BAD_REQUEST',
          message: `Invalid workflow code: ${validation.errors.join(', ')}`,
        });
      }

      // 5. Check if already deployed (unless force redeploy)
      const deploymentInfo = await deploymentService.getDeploymentInfo(input.workflowId);
      if (deploymentInfo.exists && !input.forceRedeploy) {
        throw new TRPCError({
          code: 'CONFLICT',
          message: 'Workflow is already deployed. Use forceRedeploy=true to redeploy.',
        });
      }

      // 6. Deploy to worker
      const result = await deploymentService.deployWorkflow(input.workflowId, code);

      // 7. Update deployment status in database
      if (result.success) {
        await ctx.supabase
          .from('workflows')
          .update({
            deployment_status: 'DEPLOYED',
            deployed_at: result.deployedAt?.toISOString(),
            updated_at: new Date().toISOString(),
          })
          .eq('id', input.workflowId);
      }

      return {
        success: result.success,
        workflowId: result.workflowId,
        deployedAt: result.deployedAt,
        files: result.files,
        error: result.error,
      };
    }),

  /**
   * Undeploy workflow
   */
  undeploy: protectedProcedure
    .input(z.object({
      workflowId: z.string(),
    }))
    .mutation(async ({ ctx, input }) => {
      // 1. Verify workflow ownership
      const { data: workflow, error } = await ctx.supabase
        .from('workflows')
        .select('id, name')
        .eq('id', input.workflowId)
        .eq('created_by', ctx.user.id)
        .single();

      if (error || !workflow) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Workflow not found or not authorized',
        });
      }

      // 2. Undeploy from worker
      const deploymentService = new DeploymentService();

      try {
        await deploymentService.undeployWorkflow(input.workflowId);
      } catch (undeployError) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: `Failed to undeploy workflow: ${undeployError instanceof Error ? undeployError.message : 'Unknown error'}`,
        });
      }

      // 3. Update database
      await ctx.supabase
        .from('workflows')
        .update({
          deployment_status: 'UNDEPLOYED',
          updated_at: new Date().toISOString(),
        })
        .eq('id', input.workflowId);

      return {
        success: true,
        workflowId: input.workflowId,
        message: 'Workflow undeployed successfully',
      };
    }),

  /**
   * Get deployment status
   */
  status: protectedProcedure
    .input(z.object({
      workflowId: z.string(),
    }))
    .query(async ({ ctx, input }) => {
      // 1. Get workflow from database
      const { data: workflow, error } = await ctx.supabase
        .from('workflows')
        .select('id, name, deployment_status, deployed_at, compiled_typescript, updated_at')
        .eq('id', input.workflowId)
        .eq('created_by', ctx.user.id)
        .single();

      if (error || !workflow) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Workflow not found or not authorized',
        });
      }

      // 2. Get deployment info from filesystem
      const deploymentService = new DeploymentService();
      const deploymentInfo = await deploymentService.getDeploymentInfo(input.workflowId);

      return {
        workflowId: workflow.id,
        workflowName: workflow.name,
        deploymentStatus: workflow.deployment_status || 'UNDEPLOYED',
        deployedAt: workflow.deployed_at,
        compiledAt: workflow.updated_at,
        isCompiled: !!workflow.compiled_typescript,
        isDeployed: deploymentInfo.exists,
        deploymentPath: deploymentInfo.path,
        deployedFiles: deploymentInfo.files,
      };
    }),

  /**
   * List all deployed workflows
   */
  list: protectedProcedure
    .query(async ({ ctx }) => {
      // 1. Get deployed workflows from filesystem
      const deploymentService = new DeploymentService();
      const deployedWorkflowIds = await deploymentService.listDeployed();

      if (deployedWorkflowIds.length === 0) {
        return {
          workflows: [],
          count: 0,
        };
      }

      // 2. Get workflow details from database
      const { data: workflows, error } = await ctx.supabase
        .from('workflows')
        .select('id, name, deployment_status, deployed_at')
        .eq('created_by', ctx.user.id)
        .in('id', deployedWorkflowIds);

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: 'Failed to fetch deployed workflows',
        });
      }

      return {
        workflows: workflows || [],
        count: (workflows || []).length,
      };
    }),

  /**
   * Redeploy workflow (undeploy and deploy again)
   */
  redeploy: protectedProcedure
    .input(z.object({
      workflowId: z.string(),
    }))
    .mutation(async ({ ctx, input }) => {
      // 1. Verify workflow ownership
      const { data: workflow, error } = await ctx.supabase
        .from('workflows')
        .select('*')
        .eq('id', input.workflowId)
        .eq('created_by', ctx.user.id)
        .single();

      if (error || !workflow) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Workflow not found or not authorized',
        });
      }

      // 2. Check if compiled
      if (!workflow.compiled_typescript) {
        throw new TRPCError({
          code: 'BAD_REQUEST',
          message: 'Workflow has not been compiled yet',
        });
      }

      const deploymentService = new DeploymentService();

      // 3. Undeploy if currently deployed
      const deploymentInfo = await deploymentService.getDeploymentInfo(input.workflowId);
      if (deploymentInfo.exists) {
        try {
          await deploymentService.undeployWorkflow(input.workflowId);
        } catch (undeployError) {
          console.warn('Failed to undeploy during redeploy:', undeployError);
          // Continue with deployment even if undeploy fails
        }
      }

      // 4. Parse compiled code
      let code;
      try {
        code = workflow.compiled_typescript as {
          workflowCode: string;
          activitiesCode: string;
          workerCode: string;
        };
      } catch (parseError) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: 'Failed to parse compiled code',
        });
      }

      // 5. Deploy
      const result = await deploymentService.deployWorkflow(input.workflowId, code);

      // 6. Update database
      if (result.success) {
        await ctx.supabase
          .from('workflows')
          .update({
            deployment_status: 'DEPLOYED',
            deployed_at: result.deployedAt?.toISOString(),
            updated_at: new Date().toISOString(),
          })
          .eq('id', input.workflowId);
      }

      return {
        success: result.success,
        workflowId: result.workflowId,
        deployedAt: result.deployedAt,
        files: result.files,
        error: result.error,
      };
    }),

  /**
   * Get deployment logs (for debugging)
   */
  logs: protectedProcedure
    .input(z.object({
      workflowId: z.string(),
      lines: z.number().default(100),
    }))
    .query(async ({ ctx, input }) => {
      // Verify workflow ownership
      const { data: workflow, error } = await ctx.supabase
        .from('workflows')
        .select('id, name')
        .eq('id', input.workflowId)
        .eq('created_by', ctx.user.id)
        .single();

      if (error || !workflow) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Workflow not found or not authorized',
        });
      }

      // TODO: Implement log retrieval from worker service
      // For now, return placeholder
      return {
        workflowId: input.workflowId,
        logs: [],
        message: 'Log retrieval not yet implemented',
      };
    }),
});
