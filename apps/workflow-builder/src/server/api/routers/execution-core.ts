/**
 * Execution Core Router
 * 
 * Core execution operations: build, start, status, list
 * Split from execution.ts for better organization
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';
import { compileWorkflow } from '@/lib/workflow-compiler/compiler';
import { storeCompiledCode } from '@/lib/workflow-compiler/storage';
import type { TemporalWorkflow } from '@/types/advanced-patterns';
import type { SupabaseClient } from '@supabase/supabase-js';
// Note: Using direct type assertions instead of castSupabaseClient helper
// due to TypeScript type inference limitations with Supabase

// Temporal integration via standalone worker service
const TEMPORAL_ENABLED = true;
const WORKER_SERVICE_URL = process.env.WORKER_SERVICE_URL || 'http://localhost:3011';

// Shared helper functions
async function startWorkerForProject(projectId: string) {
  const response = await fetch(`${WORKER_SERVICE_URL}/workers/start`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ projectId }),
  });
  
  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error || 'Failed to start worker');
  }
  
  return response.json();
}

async function getTemporalClient() {
  // Dynamic import only for client (no worker dependencies)
  const { getTemporalClient: getClient } = await import('@/lib/temporal/connection');
  return getClient();
}

async function recordWorkflowStats(projectId: string, workflowId: string, durationMs: number, success: boolean) {
  // Dynamic import for statistics
  const { recordWorkflowExecution } = await import('@/lib/temporal/statistics');
  return recordWorkflowExecution(projectId, workflowId, durationMs, success);
}

/**
 * Monitor a Temporal workflow execution and update database when complete
 */
async function monitorExecution(
  handle: any, // WorkflowHandle type - using any to avoid import
  executionId: string,
  projectId: string,
  workflowId: string,
  startTime: Date,
  supabase: SupabaseClient
) {
  try {
    console.log(`👀 Monitoring workflow execution ${executionId}...`);
    
    // Wait for workflow to complete
    const result = await handle.result();
    const endTime = new Date();
    const durationMs = endTime.getTime() - startTime.getTime();
    
    console.log(`✅ Workflow completed in ${durationMs}ms`);
    
    // Update execution record
    await supabase
      .from('workflow_executions')
      .update({
        status: 'completed',
        completed_at: endTime.toISOString(),
        output: result,
      })
      .eq('id', executionId);
    
    // Record statistics
    await recordWorkflowStats(projectId, workflowId, durationMs, true);
    
    console.log(`📊 Execution recorded successfully`);
  } catch (error: any) {
    console.error(`❌ Workflow execution failed:`, error);
    const endTime = new Date();
    const durationMs = endTime.getTime() - startTime.getTime();
    
    // Update execution with error
    await supabase
      .from('workflow_executions')
      .update({
        status: 'failed',
        completed_at: endTime.toISOString(),
        error_message: error.message || 'Unknown error',
      })
      .eq('id', executionId);
    
    // Record failed execution statistics
    await recordWorkflowStats(projectId, workflowId, durationMs, false);
  }
}

export const executionCoreRouter = createTRPCRouter({
  /**
   * Build and execute a workflow
   * This creates a "build workflow" Temporal workflow that:
   * 1. Compiles the workflow definition
   * 2. Validates the generated code
   * 3. Creates/updates the worker configuration
   * 4. Starts the workflow execution
   */
  build: protectedProcedure
    .input(z.object({
      workflowId: z.string(),
      input: z.any().optional(),
    }))
    .mutation(async ({ ctx, input }) => {
      // Fetch workflow with all related data
      const { data: workflowData, error: workflowError } = await ((ctx.supabase as any)
        .from('workflows')
        .select(`
          *,
          workflow_nodes(*),
          workflow_edges(*)
        `)
        .eq('id', input.workflowId)
        .eq('created_by', (ctx.user as any).id)
        .single());

      if (workflowError || !workflowData) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Workflow not found or not authorized',
        });
      }

      // Fetch work queues, signals, queries
      const [
        { data: workQueues },
        { data: signals },
        { data: queries }
      ] = await Promise.all([
        ctx.supabase.from('workflow_work_queues').select('*').eq('workflow_id', input.workflowId),
        ctx.supabase.from('workflow_signals').select('*').eq('workflow_id', input.workflowId),
        ctx.supabase.from('workflow_queries').select('*').eq('workflow_id', input.workflowId),
      ]);

      // Convert to TemporalWorkflow format
      const wfData = workflowData as any;
      if (!wfData) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Workflow not found',
        });
      }
      const workflow: TemporalWorkflow = {
        id: wfData.id,
        name: wfData.display_name || wfData.kebab_name, // deprecated, kept for backward compatibility
        kebab_name: wfData.kebab_name,
        display_name: wfData.display_name,
        description: wfData.description,
        stages: wfData.workflow_nodes?.map((node: any) => ({
          id: node.node_id,
          type: node.node_type_id,
          position: { x: node.position_x, y: node.position_y },
          metadata: node.metadata,
        })) || [],
        transitions: wfData.workflow_edges?.map((edge: any) => ({
          from: edge.source_node_id,
          to: edge.target_node_id,
          condition: edge.label,
        })) || [],
        workQueues: (workQueues as any)?.map((wq: any) => ({
          id: wq.id,
          workflowId: wq.workflow_id,
          name: wq.queue_name,
          description: wq.description,
          signalName: wq.signal_name,
          queryName: wq.query_name,
          maxSize: wq.max_size,
          priority: wq.priority as 'fifo' | 'lifo' | 'priority',
          deduplicate: wq.deduplicate,
          workItemSchema: wq.work_item_schema,
          createdAt: new Date(wq.created_at),
          updatedAt: new Date(wq.updated_at),
        })) || [],
        signals: (signals as any)?.map((s: any) => ({
          id: s.id,
          workflowId: s.workflow_id,
          name: s.signal_name,
          description: s.description,
          parametersSchema: s.parameters_schema,
          autoGenerated: s.auto_generated,
          createdAt: new Date(s.created_at),
        })) || [],
        queries: (queries as any)?.map((q: any) => ({
          id: q.id,
          workflowId: q.workflow_id,
          name: q.query_name,
          description: q.description,
          returnTypeSchema: q.return_type_schema,
          autoGenerated: q.auto_generated,
          createdAt: new Date(q.created_at),
        })) || [],
      };

      // Create execution record
      const { data: execution, error: execError } = await ((ctx.supabase as any)
        .from('workflow_executions')
        .insert({
          workflow_id: input.workflowId,
          status: 'building',
          started_at: new Date().toISOString(),
          input: input.input,
          created_by: (ctx.user as any).id,
        })
        .select()
        .single());

      if (execError || !execution) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: 'Failed to create execution record',
        });
      }
      const execData = (execution as any);
      if (!execData || !execData.id) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: 'Failed to create execution record',
        });
      }

      // Declare codeId outside try block so it's accessible in catch
      let codeId: string | null = null;

      try {
        // Get user record for storage
        const userRecord = await ctx.getUserRecord();
        if (!userRecord) {
          throw new TRPCError({
            code: 'UNAUTHORIZED',
            message: 'User record not found',
          });
        }
        
        // Step 1: Compile workflow definition
        const workflowName = (workflowData as any)?.name || 'unknown';
        console.log(`🔨 Compiling workflow ${workflowName}...`);
        const compiled = compileWorkflow(workflow, {
          includeComments: true,
          strictMode: true,
        });

        // Step 2: Store compiled code in database
        console.log(`💾 Storing compiled code...`);
        codeId = await storeCompiledCode(
          input.workflowId,
          (workflowData as any)?.version || '1.0.0',
          compiled,
          (userRecord as any).id
        );
        
        console.log(`✅ Compiled code stored with ID: ${codeId}`);

        // Step 3: Get project for this workflow
        const projectId = (workflowData as any)?.project_id;
        if (!projectId) {
          throw new Error('Workflow has no associated project');
        }
        
        const { data: project } = await ((ctx.supabase as any)
          .from('projects')
          .select('*')
          .eq('id', projectId)
          .single());
        
        if (!project) {
          throw new Error('Project not found');
        }
        
        console.log(`📋 Project: ${(project as any).name} (task queue: ${(project as any).task_queue_name})`);

        if (!TEMPORAL_ENABLED) {
          // Temporal is disabled - just mark as completed with compiled code
          await ((ctx.supabase as any)
            .from('workflow_executions')
            .update({
              status: 'completed',
              completed_at: new Date().toISOString(),
              output: { 
                message: 'Workflow compiled successfully. Temporal integration temporarily disabled.',
                compiledCodeId: codeId 
              },
            })
            .eq('id', execData.id));

          return {
            success: true,
            executionId: execData.id,
            message: `Workflow compiled successfully! Temporal integration is currently disabled. You can view the compiled code in the database.`,
          };
        }

        // Step 4: Ensure worker is running for this project (via worker service)
        console.log(`🔧 Ensuring worker is running for project...`);
        await startWorkerForProject((project as any).id);
        
        // Step 5: Get Temporal client
        console.log(`📦 Connecting to Temporal...`);
        const client = await getTemporalClient();
        
        // Step 6: Start workflow execution on Temporal
        console.log(`🚀 Starting workflow execution on Temporal...`);
        
        const wfKebabName = wfData.kebab_name || wfData.name;
        const workflowId = `${wfKebabName}-${Date.now()}`;
        const handle = await client.workflow.start(wfKebabName, {
          taskQueue: (project as any).task_queue_name,
          workflowId,
          args: [input.input || {}],
        });
        
        console.log(`✅ Workflow started: ${workflowId}`);
        console.log(`   Run ID: ${handle.firstExecutionRunId}`);
        
        // Update execution with Temporal workflow ID
        await ((ctx.supabase as any)
          .from('workflow_executions')
          .update({
            status: 'running',
            temporal_workflow_id: workflowId,
            temporal_run_id: handle.firstExecutionRunId,
          })
          .eq('id', execData.id));
        
        // Monitor execution in background (don't await)
        monitorExecution(
          handle,
          execData.id,
          (project as any).id,
          wfData.id,
          new Date(),
          ctx.supabase as any
        ).catch(err => console.error('Error monitoring execution:', err));
        
        return {
          success: true,
          executionId: execData.id,
          workflowId,
          runId: handle.firstExecutionRunId,
          message: 'Workflow execution started successfully!',
        };
      } catch (error: any) {
        console.error(`❌ Build/execution failed:`, error);

        const message = error?.message || 'Unknown error';

        // If the failure is due to infrastructure not being reachable (e.g. the
        // standalone worker service or Temporal is not running in this
        // environment), degrade gracefully: treat this as a successful
        // "compile-only" run instead of surfacing a hard 500 to the UI.
        const infraUnavailable =
          typeof message === 'string' &&
          (
            message.includes('fetch failed') ||
            message.includes('ECONNREFUSED') ||
            message.includes('ENOTFOUND')
          );

        if (infraUnavailable) {
          console.warn(
            '⚠️ Build/execution failed due to unavailable worker/Temporal infrastructure. ' +
              'Marking execution as completed with compile-only result.',
          );

          // Mark execution as completed with a descriptive output payload.
          await ((ctx.supabase as any)
            .from('workflow_executions')
            .update({
              status: 'completed',
              completed_at: new Date().toISOString(),
              output: {
                message:
                  'Workflow compiled successfully, but Temporal/worker infrastructure is not available in this environment. ' +
                  'Execution was not started, but the compiled code is ready.',
                compiledCodeId: codeId,
              },
            })
            .eq('id', execData.id));

          return {
            success: true,
            executionId: (execution as any).id,
            message:
              'Workflow compiled successfully, but Temporal/worker infrastructure is not available in this environment.',
          };
        }

        // Update execution with failure
        await ((ctx.supabase as any)
          .from('workflow_executions')
          .update({
            status: 'failed',
            completed_at: new Date().toISOString(),
            error_message: error.message,
          })
          .eq('id', execData.id));

        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: `Build/execution failed: ${error.message}`,
          cause: error,
        });
      }
    }),

  /**
   * Get execution status
   */
  getStatus: protectedProcedure
    .input(z.object({
      executionId: z.string(),
    }))
    .query(async ({ ctx, input }) => {
      const { data, error } = await ((ctx.supabase as any)
        .from('workflow_executions')
        .select('*')
        .eq('id', input.executionId)
        .eq('created_by', (ctx.user as any).id)
        .single());

      if (error || !data) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Execution not found',
        });
      }

      const execResult = data as any;
      return {
        id: execResult.id,
        status: execResult.status,
        startedAt: new Date(execResult.started_at),
        completedAt: execResult.completed_at ? new Date(execResult.completed_at) : null,
        result: execResult.output,  // Use 'output' column
        error: execResult.error_message,  // Use 'error_message' column
        input: execResult.input,
      };
    }),

  /**
   * List executions for a workflow
   */
  list: protectedProcedure
    .input(z.object({
      workflowId: z.string(),
      page: z.number().default(1),
      pageSize: z.number().default(10),
    }))
    .query(async ({ ctx, input }) => {
      const from = (input.page - 1) * input.pageSize;
      const to = from + input.pageSize - 1;

      const { data, error, count } = await ((ctx.supabase as any)
        .from('workflow_executions')
        .select('*', { count: 'exact' })
        .eq('workflow_id', input.workflowId)
        .eq('created_by', (ctx.user as any).id)
        .order('started_at', { ascending: false })
        .range(from, to));

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return {
        executions: (data as any)?.map((e: any) => ({
          id: e.id,
          status: e.status,
          startedAt: new Date(e.started_at),
          completedAt: e.completed_at ? new Date(e.completed_at) : null,
          result: e.output,  // Use 'output' column
          error: e.error_message,  // Use 'error_message' column
        })) || [],
        total: count || 0,
        page: input.page,
        pageSize: input.pageSize,
      };
    }),
});

