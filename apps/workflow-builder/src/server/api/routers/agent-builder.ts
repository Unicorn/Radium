/**
 * Agent Builder router - AI-assisted agent prompt creation
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';
import {
  createSession,
  getSession,
  sendMessage as sendMessageToAI,
  regeneratePrompt,
  updateMetadata,
  deleteSession,
} from '@/lib/agent-builder/conversation-service';

export const agentBuilderRouter = createTRPCRouter({
  /**
   * Start a new builder session
   */
  startSession: protectedProcedure
    .mutation(async ({ ctx }) => {
      const session = createSession(ctx.user.id);
      
      // Optionally save to database for analytics
      try {
        await ctx.supabase
          .from('agent_builder_sessions')
          .insert({
            user_id: ctx.user.id,
            conversation_messages: session.messages,
            status: 'active',
            message_count: session.messages.length,
          });
      } catch (error) {
        // Non-critical, log but don't fail
        console.warn('Failed to save builder session to database:', error);
      }
      
      return {
        sessionId: session.sessionId,
        messages: session.messages,
      };
    }),

  /**
   * Send a message in the conversation
   */
  sendMessage: protectedProcedure
    .input(z.object({
      sessionId: z.string(),
      message: z.string().min(1),
    }))
    .mutation(async ({ ctx, input }) => {
      const session = getSession(input.sessionId);
      
      if (!session) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Session not found or expired',
        });
      }
      
      if (session.userId !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'Not authorized to access this session',
        });
      }
      
      try {
        const result = await sendMessageToAI(input.sessionId, input.message);
        
        // Update database session
        try {
          await ctx.supabase
            .from('agent_builder_sessions')
            .update({
              conversation_messages: result.session.messages,
              message_count: result.session.messages.length,
              updated_at: new Date().toISOString(),
            })
            .eq('user_id', ctx.user.id)
            .eq('id', input.sessionId); // Note: sessionId format may differ
        } catch (error) {
          // Non-critical
          console.warn('Failed to update builder session in database:', error);
        }
        
        return {
          messages: result.session.messages,
          response: result.response,
          generatedPrompt: result.session.generatedPrompt,
        };
      } catch (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error instanceof Error ? error.message : 'Failed to send message',
        });
      }
    }),

  /**
   * Regenerate the prompt with current context
   */
  regeneratePrompt: protectedProcedure
    .input(z.object({
      sessionId: z.string(),
    }))
    .mutation(async ({ ctx, input }) => {
      const session = getSession(input.sessionId);
      
      if (!session) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Session not found or expired',
        });
      }
      
      if (session.userId !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'Not authorized to access this session',
        });
      }
      
      try {
        const prompt = await regeneratePrompt(input.sessionId);
        const updatedSession = getSession(input.sessionId);
        
        return {
          prompt,
          messages: updatedSession?.messages || session.messages,
        };
      } catch (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error instanceof Error ? error.message : 'Failed to regenerate prompt',
        });
      }
    }),

  /**
   * Save the generated prompt to agent_prompts table
   */
  savePrompt: protectedProcedure
    .input(z.object({
      sessionId: z.string(),
      name: z.string().min(1).max(255),
      displayName: z.string().min(1).max(255),
      description: z.string().optional(),
      version: z.string().regex(/^\d+\.\d+\.\d+$/, 'Must be valid semver').default('1.0.0'),
      visibility: z.enum(['public', 'private', 'organization']).default('private'),
      capabilities: z.array(z.string()).optional(),
      tags: z.array(z.string()).optional(),
    }))
    .mutation(async ({ ctx, input }) => {
      const session = getSession(input.sessionId);
      
      if (!session) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Session not found or expired',
        });
      }
      
      if (session.userId !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'Not authorized to access this session',
        });
      }
      
      if (!session.generatedPrompt) {
        throw new TRPCError({
          code: 'BAD_REQUEST',
          message: 'No prompt generated yet. Continue the conversation or regenerate.',
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
          message: 'Invalid visibility',
        });
      }
      
      // Check for duplicate
      const { data: existing } = await ctx.supabase
        .from('agent_prompts')
        .select('id')
        .eq('name', input.name)
        .eq('version', input.version)
        .eq('created_by', ctx.user.id)
        .single();
      
      if (existing) {
        throw new TRPCError({
          code: 'BAD_REQUEST',
          message: 'Agent prompt with this name and version already exists',
        });
      }
      
      // Create agent prompt
      const { data, error } = await ctx.supabase
        .from('agent_prompts')
        .insert({
          name: input.name,
          display_name: input.displayName,
          description: input.description,
          version: input.version,
          prompt_content: session.generatedPrompt,
          created_by: ctx.user.id,
          visibility_id: visibility.id,
          capabilities: input.capabilities || null,
          tags: input.tags || null,
        })
        .select()
        .single();
      
      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }
      
      // Update builder session
      try {
        await ctx.supabase
          .from('agent_builder_sessions')
          .update({
            resulting_prompt_id: data.id,
            status: 'completed',
            completed_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          })
          .eq('user_id', ctx.user.id);
      } catch (error) {
        // Non-critical
        console.warn('Failed to update builder session:', error);
      }
      
      // Clean up session
      deleteSession(input.sessionId);
      
      return data;
    }),

  /**
   * Cancel and clean up a session
   */
  cancelSession: protectedProcedure
    .input(z.object({
      sessionId: z.string(),
    }))
    .mutation(async ({ ctx, input }) => {
      const session = getSession(input.sessionId);
      
      if (session && session.userId !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'Not authorized to access this session',
        });
      }
      
      // Update database session
      try {
        await ctx.supabase
          .from('agent_builder_sessions')
          .update({
            status: 'cancelled',
            completed_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          })
          .eq('user_id', ctx.user.id);
      } catch (error) {
        // Non-critical
        console.warn('Failed to update builder session:', error);
      }
      
      // Delete from memory
      deleteSession(input.sessionId);
      
      return { success: true };
    }),

  /**
   * Get session state
   */
  getSession: protectedProcedure
    .input(z.object({
      sessionId: z.string(),
    }))
    .query(async ({ ctx, input }) => {
      const session = getSession(input.sessionId);
      
      if (!session) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Session not found or expired',
        });
      }
      
      if (session.userId !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'Not authorized to access this session',
        });
      }
      
      return {
        sessionId: session.sessionId,
        messages: session.messages,
        generatedPrompt: session.generatedPrompt,
        metadata: session.metadata,
      };
    }),
});

