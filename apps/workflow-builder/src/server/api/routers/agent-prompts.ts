/**
 * Agent Prompts router - CRUD operations for AI agent prompts
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure, publicProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';

export const agentPromptsRouter = createTRPCRouter({
  // List agent prompts
  list: protectedProcedure
    .input(z.object({
      capability: z.string().optional(),
      tags: z.array(z.string()).optional(),
      includeDeprecated: z.boolean().default(false),
      page: z.number().default(1),
      pageSize: z.number().default(50),
    }))
    .query(async ({ ctx, input }) => {
      const { supabase } = ctx;
      
      let query = supabase
        .from('agent_prompts')
        .select(`
          *,
          created_by_user:users!agent_prompts_created_by_fkey(id, display_name),
          visibility:component_visibility(id, name)
        `, { count: 'exact' });
      
      if (input.capability) {
        query = query.contains('capabilities', [input.capability]);
      }
      
      if (input.tags && input.tags.length > 0) {
        query = query.overlaps('tags', input.tags);
      }
      
      if (!input.includeDeprecated) {
        query = query.eq('deprecated', false);
      }
      
      const from = (input.page - 1) * input.pageSize;
      const to = from + input.pageSize - 1;
      
      const { data, error, count } = await query
        .range(from, to)
        .order('created_at', { ascending: false });
      
      if (error) {
        throw new TRPCError({ 
          code: 'INTERNAL_SERVER_ERROR', 
          message: error.message 
        });
      }
      
      return {
        prompts: data || [],
        total: count || 0,
        page: input.page,
        pageSize: input.pageSize,
        totalPages: Math.ceil((count || 0) / input.pageSize),
      };
    }),

  // Get single agent prompt
  get: publicProcedure
    .input(z.object({ id: z.string().uuid() }))
    .query(async ({ ctx, input }) => {
      const { data, error } = await ctx.supabase
        .from('agent_prompts')
        .select(`
          *,
          created_by_user:users!agent_prompts_created_by_fkey(id, display_name, email),
          visibility:component_visibility(id, name, description)
        `)
        .eq('id', input.id)
        .single();
      
      if (error) {
        throw new TRPCError({ 
          code: 'NOT_FOUND', 
          message: 'Agent prompt not found' 
        });
      }
      
      return data;
    }),

  // Create agent prompt
  create: protectedProcedure
    .input(z.object({
      name: z.string().min(1).max(255),
      displayName: z.string().min(1).max(255),
      description: z.string().optional(),
      version: z.string().regex(/^\d+\.\d+\.\d+$/, 'Must be valid semver'),
      promptContent: z.string().min(10),
      promptVariables: z.any().optional(),
      visibility: z.enum(['public', 'private', 'organization']),
      capabilities: z.array(z.string()).optional(),
      tags: z.array(z.string()).optional(),
      recommendedModels: z.array(z.object({
        provider: z.string(),
        model: z.string(),
        reason: z.string(),
      })).optional(),
    }))
    .mutation(async ({ ctx, input }) => {
      const { supabase, user } = ctx;
      
      // Get visibility ID
      const { data: visibility } = await supabase
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
      
      // Check for duplicate
      const { data: existing } = await supabase
        .from('agent_prompts')
        .select('id')
        .eq('name', input.name)
        .eq('version', input.version)
        .eq('created_by', user.id)
        .single();
      
      if (existing) {
        throw new TRPCError({ 
          code: 'BAD_REQUEST', 
          message: 'Agent prompt with this name and version already exists' 
        });
      }
      
      // Create
      const { data, error } = await supabase
        .from('agent_prompts')
        .insert({
          name: input.name,
          display_name: input.displayName,
          description: input.description,
          version: input.version,
          prompt_content: input.promptContent,
          prompt_variables: input.promptVariables || null,
          created_by: user.id,
          visibility_id: visibility.id,
          capabilities: input.capabilities || null,
          tags: input.tags || null,
          recommended_models: input.recommendedModels || null,
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

  // Update agent prompt
  update: protectedProcedure
    .input(z.object({
      id: z.string().uuid(),
      displayName: z.string().min(1).max(255).optional(),
      description: z.string().optional(),
      promptContent: z.string().min(10).optional(),
      capabilities: z.array(z.string()).optional(),
      tags: z.array(z.string()).optional(),
      deprecated: z.boolean().optional(),
      deprecatedMessage: z.string().optional(),
    }))
    .mutation(async ({ ctx, input }) => {
      const { id, ...updates } = input;
      
      // Check ownership
      const { data: prompt } = await ctx.supabase
        .from('agent_prompts')
        .select('created_by')
        .eq('id', id)
        .single();
      
      if (!prompt || prompt.created_by !== ctx.user.id) {
        throw new TRPCError({ 
          code: 'FORBIDDEN', 
          message: 'Not authorized to update this agent prompt' 
        });
      }
      
      // Build update
      const updateData: any = {
        updated_at: new Date().toISOString(),
      };
      
      if (updates.displayName !== undefined) updateData.display_name = updates.displayName;
      if (updates.description !== undefined) updateData.description = updates.description;
      if (updates.promptContent !== undefined) updateData.prompt_content = updates.promptContent;
      if (updates.capabilities !== undefined) updateData.capabilities = updates.capabilities;
      if (updates.tags !== undefined) updateData.tags = updates.tags;
      if (updates.deprecated !== undefined) updateData.deprecated = updates.deprecated;
      if (updates.deprecatedMessage !== undefined) updateData.deprecated_message = updates.deprecatedMessage;
      
      const { data, error } = await ctx.supabase
        .from('agent_prompts')
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

  // Delete agent prompt
  delete: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .mutation(async ({ ctx, input }) => {
      // Check ownership
      const { data: prompt } = await ctx.supabase
        .from('agent_prompts')
        .select('created_by')
        .eq('id', input.id)
        .single();
      
      if (!prompt || prompt.created_by !== ctx.user.id) {
        throw new TRPCError({ 
          code: 'FORBIDDEN', 
          message: 'Not authorized to delete this agent prompt' 
        });
      }
      
      // Check if used by components
      const { count } = await ctx.supabase
        .from('components')
        .select('*', { count: 'exact', head: true })
        .eq('agent_prompt_id', input.id);
      
      if (count && count > 0) {
        throw new TRPCError({ 
          code: 'BAD_REQUEST', 
          message: `Agent prompt is used by ${count} component(s). Deprecate it instead.` 
        });
      }
      
      const { error } = await ctx.supabase
        .from('agent_prompts')
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

