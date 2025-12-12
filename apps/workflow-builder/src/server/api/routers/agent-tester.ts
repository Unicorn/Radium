/**
 * Agent Tester router - Control agent tester workflows
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';
import { getTemporalClient } from '@/lib/temporal/connection';
import { randomUUID } from 'crypto';

export const agentTesterRouter = createTRPCRouter({
  /**
   * Start an agent test workflow
   */
  startTest: protectedProcedure
    .input(z.object({
      agentPromptId: z.string().uuid(),
      initialMessage: z.string().optional(),
    }))
    .mutation(async ({ ctx, input }) => {
      // Check if user already has an active test for this agent
      const { data: activeTest } = await ctx.supabase
        .from('agent_test_sessions')
        .select('id, temporal_workflow_id')
        .eq('agent_prompt_id', input.agentPromptId)
        .eq('user_id', ctx.user.id)
        .eq('status', 'active')
        .single();
      
      if (activeTest) {
        throw new TRPCError({
          code: 'BAD_REQUEST',
          message: 'You already have an active test session for this agent',
        });
      }
      
      // Get system project and task queue
      const { data: systemUser } = await ctx.supabase
        .from('users')
        .select('id')
        .eq('email', 'system@example.com')
        .single();
      
      if (!systemUser) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: 'System user not found. Run seed migration 20251118000003.',
        });
      }
      
      const { data: systemProject } = await ctx.supabase
        .from('projects')
        .select('task_queue_name')
        .eq('name', 'System Workflows')
        .eq('created_by', systemUser.id)
        .single();
      
      if (!systemProject) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: 'System project not found. Run seed migration 20251118000003.',
        });
      }
      
      // Get Temporal client
      const client = await getTemporalClient();
      
      // Generate session ID
      const sessionId = randomUUID();
      
      // Start workflow
      const workflowId = `agent-test-${input.agentPromptId}-${Date.now()}`;
      const handle = await client.workflow.start('agentTesterWorkflow', {
        taskQueue: systemProject.task_queue_name,
        workflowId,
        args: [{
          agentPromptId: input.agentPromptId,
          userId: ctx.user.id,
          sessionId,
          initialMessage: input.initialMessage,
        }],
      });
      
      // Create test session record
      const { error } = await ctx.supabase
        .from('agent_test_sessions')
        .insert({
          id: sessionId,
          agent_prompt_id: input.agentPromptId,
          user_id: ctx.user.id,
          temporal_workflow_id: workflowId,
          temporal_run_id: handle.firstExecutionRunId,
          status: 'active',
          conversation_history: input.initialMessage ? [{
            role: 'user',
            content: input.initialMessage,
            timestamp: new Date().toISOString(),
          }] : [],
          message_count: input.initialMessage ? 1 : 0,
        });
      
      if (error) {
        // Try to cancel workflow if database insert fails
        try {
          await handle.terminate();
        } catch (e) {
          // Ignore termination errors
        }
        
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: `Failed to create test session: ${error.message}`,
        });
      }
      
      return {
        sessionId,
        workflowId,
        runId: handle.firstExecutionRunId,
      };
    }),

  /**
   * Send a message to the agent test workflow
   */
  sendMessage: protectedProcedure
    .input(z.object({
      workflowId: z.string(),
      message: z.string().min(1),
    }))
    .mutation(async ({ ctx, input }) => {
      // Verify user owns this workflow
      const { data: session } = await ctx.supabase
        .from('agent_test_sessions')
        .select('user_id, status')
        .eq('temporal_workflow_id', input.workflowId)
        .eq('user_id', ctx.user.id)
        .single();
      
      if (!session) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Test session not found',
        });
      }
      
      if (session.status !== 'active') {
        throw new TRPCError({
          code: 'BAD_REQUEST',
          message: 'Test session is not active',
        });
      }
      
      // Get Temporal client and send signal
      const client = await getTemporalClient();
      const handle = client.workflow.getHandle(input.workflowId);
      
      await handle.signal('sendMessage', input.message);
      
      return { success: true };
    }),

  /**
   * Get current conversation state
   */
  getConversation: protectedProcedure
    .input(z.object({
      workflowId: z.string(),
    }))
    .query(async ({ ctx, input }) => {
      // Verify user owns this workflow
      const { data: session } = await ctx.supabase
        .from('agent_test_sessions')
        .select('user_id, status, conversation_history')
        .eq('temporal_workflow_id', input.workflowId)
        .eq('user_id', ctx.user.id)
        .single();
      
      if (!session) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Test session not found',
        });
      }
      
      // Try to get state from workflow query
      try {
        const client = await getTemporalClient();
        const handle = client.workflow.getHandle(input.workflowId);
        const state = await handle.query('getConversation');
        
        return {
          messages: state.messages,
          isActive: state.isActive,
        };
      } catch (error) {
        // Fallback to database state
        return {
          messages: (session.conversation_history as any[]) || [],
          isActive: session.status === 'active',
        };
      }
    }),

  /**
   * End the test session
   */
  endTest: protectedProcedure
    .input(z.object({
      workflowId: z.string(),
    }))
    .mutation(async ({ ctx, input }) => {
      // Verify user owns this workflow
      const { data: session } = await ctx.supabase
        .from('agent_test_sessions')
        .select('user_id, status')
        .eq('temporal_workflow_id', input.workflowId)
        .eq('user_id', ctx.user.id)
        .single();
      
      if (!session) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Test session not found',
        });
      }
      
      // Send end signal
      try {
        const client = await getTemporalClient();
        const handle = client.workflow.getHandle(input.workflowId);
        await handle.signal('endTest');
      } catch (error) {
        // If workflow doesn't exist, just update database
        console.warn('Failed to send end signal to workflow:', error);
      }
      
      // Update database
      await ctx.supabase
        .from('agent_test_sessions')
        .update({
          status: 'cancelled',
          completed_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        })
        .eq('temporal_workflow_id', input.workflowId);
      
      return { success: true };
    }),

  /**
   * Get active test for an agent
   */
  getActiveTest: protectedProcedure
    .input(z.object({
      agentPromptId: z.string().uuid(),
    }))
    .query(async ({ ctx, input }) => {
      const { data: session } = await ctx.supabase
        .from('agent_test_sessions')
        .select('id, temporal_workflow_id, status, message_count')
        .eq('agent_prompt_id', input.agentPromptId)
        .eq('user_id', ctx.user.id)
        .eq('status', 'active')
        .single();
      
      return session || null;
    }),
});

