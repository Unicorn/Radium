/**
 * API Keys router - CRUD operations for API keys
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';
import { generateApiKey, extractApiKeyPrefix } from '@/lib/security/api-key-generator';
import { hashApiKey, getApiKeyDisplayPrefix } from '@/lib/security/api-key-hasher';

export const apiKeysRouter = createTRPCRouter({
  // List API keys for user
  list: protectedProcedure
    .input(z.object({
      projectId: z.string().uuid().optional(),
      publicInterfaceId: z.string().uuid().optional(),
    }))
    .query(async ({ ctx, input }) => {
      let query = ctx.supabase
        .from('api_keys')
        .select(`
          *,
          project:projects(id, name),
          public_interface:public_interfaces(id, http_path, http_method)
        `)
        .eq('user_id', ctx.user.id)
        .eq('is_active', true)
        .order('created_at', { ascending: false });

      if (input.projectId) {
        query = query.eq('project_id', input.projectId);
      }

      if (input.publicInterfaceId) {
        query = query.eq('public_interface_id', input.publicInterfaceId);
      }

      const { data, error } = await query;

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      // Don't return the full hash, just the prefix for display
      return (data || []).map((key) => ({
        ...key,
        key_hash: undefined, // Don't expose hash
        key_prefix: key.key_prefix || getApiKeyDisplayPrefix(key.key_prefix),
      }));
    }),

  // Get API key by ID
  get: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .query(async ({ ctx, input }) => {
      const { data, error } = await ctx.supabase
        .from('api_keys')
        .select(`
          *,
          project:projects(id, name),
          public_interface:public_interfaces(id, http_path, http_method)
        `)
        .eq('id', input.id)
        .eq('user_id', ctx.user.id)
        .single();

      if (error) {
        throw new TRPCError({
          code: error.code === 'PGRST116' ? 'NOT_FOUND' : 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      if (!data) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'API key not found',
        });
      }

      return {
        ...data,
        key_hash: undefined, // Don't expose hash
        key_prefix: data.key_prefix || getApiKeyDisplayPrefix(data.key_prefix),
      };
    }),

  // Create new API key
  create: protectedProcedure
    .input(z.object({
      name: z.string().min(1).max(255),
      projectId: z.string().uuid().optional(),
      publicInterfaceId: z.string().uuid().optional(),
      expiresAt: z.string().datetime().optional(),
    }))
    .mutation(async ({ ctx, input }) => {
      // Verify project ownership if projectId provided
      if (input.projectId) {
        const { data: project, error: projectError } = await ctx.supabase
          .from('projects')
          .select('id')
          .eq('id', input.projectId)
          .eq('created_by', ctx.user.id)
          .single();

        if (projectError || !project) {
          throw new TRPCError({
            code: 'NOT_FOUND',
            message: 'Project not found or access denied',
          });
        }
      }

      // Verify public interface ownership if publicInterfaceId provided
      if (input.publicInterfaceId) {
        const { data: publicInterface, error: interfaceError } = await ctx.supabase
          .from('public_interfaces')
          .select(`
            id,
            service_interface:service_interfaces!inner(
              workflow:workflows!inner(
                created_by
              )
            )
          `)
          .eq('id', input.publicInterfaceId)
          .single();

        if (interfaceError || !publicInterface) {
          throw new TRPCError({
            code: 'NOT_FOUND',
            message: 'Public interface not found',
          });
        }

        // Check if user owns the workflow
        const workflow = (publicInterface as any).service_interface?.workflow;
        if (!workflow || workflow.created_by !== ctx.user.id) {
          throw new TRPCError({
            code: 'FORBIDDEN',
            message: 'Access denied to this public interface',
          });
        }
      }

      // Generate new API key
      const apiKey = generateApiKey();
      const keyHash = hashApiKey(apiKey);
      const keyPrefix = extractApiKeyPrefix(apiKey);

      // Insert into database
      const { data, error } = await ctx.supabase
        .from('api_keys')
        .insert({
          user_id: ctx.user.id,
          project_id: input.projectId || null,
          public_interface_id: input.publicInterfaceId || null,
          name: input.name,
          key_hash: keyHash,
          key_prefix: keyPrefix,
          expires_at: input.expiresAt || null,
          is_active: true,
        })
        .select()
        .single();

      if (error) {
        // Check for unique constraint violation
        if (error.code === '23505') {
          throw new TRPCError({
            code: 'CONFLICT',
            message: 'An API key with this name already exists',
          });
        }

        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      // Return the API key (only time it's shown in plain text)
      return {
        ...data,
        apiKey, // Include plain key only on creation
        key_hash: undefined, // Don't expose hash
      };
    }),

  // Update API key (name, expiration, active status)
  update: protectedProcedure
    .input(z.object({
      id: z.string().uuid(),
      name: z.string().min(1).max(255).optional(),
      isActive: z.boolean().optional(),
      expiresAt: z.string().datetime().nullable().optional(),
    }))
    .mutation(async ({ ctx, input }) => {
      const { id, ...updates } = input;

      // Verify ownership
      const { data: existing, error: fetchError } = await ctx.supabase
        .from('api_keys')
        .select('id')
        .eq('id', id)
        .eq('user_id', ctx.user.id)
        .single();

      if (fetchError || !existing) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'API key not found',
        });
      }

      // Build update object
      const updateData: Record<string, any> = {};
      if (updates.name !== undefined) updateData.name = updates.name;
      if (updates.isActive !== undefined) updateData.is_active = updates.isActive;
      if (updates.expiresAt !== undefined) updateData.expires_at = updates.expiresAt;

      const { data, error } = await ctx.supabase
        .from('api_keys')
        .update(updateData)
        .eq('id', id)
        .select()
        .single();

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return {
        ...data,
        key_hash: undefined, // Don't expose hash
      };
    }),

  // Delete (revoke) API key
  delete: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .mutation(async ({ ctx, input }) => {
      // Verify ownership
      const { data: existing, error: fetchError } = await ctx.supabase
        .from('api_keys')
        .select('id')
        .eq('id', input.id)
        .eq('user_id', ctx.user.id)
        .single();

      if (fetchError || !existing) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'API key not found',
        });
      }

      // Soft delete by setting is_active = false
      const { error } = await ctx.supabase
        .from('api_keys')
        .update({ is_active: false })
        .eq('id', input.id);

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return { success: true };
    }),

  // Regenerate API key (creates new key, revokes old one)
  regenerate: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .mutation(async ({ ctx, input }) => {
      // Get existing key to copy metadata
      const { data: existing, error: fetchError } = await ctx.supabase
        .from('api_keys')
        .select('*')
        .eq('id', input.id)
        .eq('user_id', ctx.user.id)
        .single();

      if (fetchError || !existing) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'API key not found',
        });
      }

      // Revoke old key
      await ctx.supabase
        .from('api_keys')
        .update({ is_active: false })
        .eq('id', input.id);

      // Generate new API key with same metadata
      const apiKey = generateApiKey();
      const keyHash = hashApiKey(apiKey);
      const keyPrefix = extractApiKeyPrefix(apiKey);

      const { data, error } = await ctx.supabase
        .from('api_keys')
        .insert({
          user_id: ctx.user.id,
          project_id: existing.project_id,
          public_interface_id: existing.public_interface_id,
          name: `${existing.name} (regenerated)`,
          key_hash: keyHash,
          key_prefix: keyPrefix,
          expires_at: existing.expires_at,
          is_active: true,
        })
        .select()
        .single();

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      // Return the new API key (only time it's shown in plain text)
      return {
        ...data,
        apiKey, // Include plain key only on regeneration
        key_hash: undefined, // Don't expose hash
      };
    }),
});

