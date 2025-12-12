/**
 * Kong Cache Router
 * 
 * tRPC router for managing Kong caching component configuration and cache keys
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';
import { randomUUID } from 'crypto';

export const kongCacheRouter = createTRPCRouter({
  // Get cache key configuration for a component
  get: protectedProcedure
    .input(z.object({
      componentId: z.string().uuid(),
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

      // Get cache key configuration
      const { data: cacheKey, error: cacheKeyError } = await ctx.supabase
        .from('kong_cache_keys')
        .select('*')
        .eq('component_id', input.componentId)
        .eq('project_id', input.projectId)
        .single();

      if (cacheKeyError && cacheKeyError.code !== 'PGRST116') {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: 'Failed to fetch cache key configuration',
        });
      }

      return cacheKey;
    }),

  // Create or update cache key configuration
  upsert: protectedProcedure
    .input(z.object({
      componentId: z.string().uuid(),
      projectId: z.string().uuid(),
      connectorId: z.string().uuid(),
      cacheKey: z.string().uuid(), // UUID format
      ttlSeconds: z.number().int().positive().default(3600),
      cacheKeyStrategy: z.enum(['path', 'query', 'header', 'custom']).default('path'),
      contentTypes: z.array(z.string()).default(['application/json']),
      responseCodes: z.array(z.number().int()).default([200, 201, 202]),
      config: z.record(z.any()).optional(),
      isSaved: z.boolean().default(false),
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

      // Verify connector exists and belongs to project
      const { data: connector, error: connectorError } = await ctx.supabase
        .from('connectors')
        .select('id, project_id')
        .eq('id', input.connectorId)
        .single();

      if (connectorError || !connector) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Connector not found',
        });
      }

      if (connector.project_id !== input.projectId) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'Connector does not belong to this project',
        });
      }

      // Check if cache key already exists for this component
      const { data: existing } = await ctx.supabase
        .from('kong_cache_keys')
        .select('id, is_saved, cache_key')
        .eq('component_id', input.componentId)
        .single();

      // If component is saved, cache key is immutable
      if (existing?.is_saved && existing.cache_key !== input.cacheKey) {
        throw new TRPCError({
          code: 'BAD_REQUEST',
          message: 'Cache key is immutable after component is saved. Remove and recreate the component to change the key.',
        });
      }

      // Upsert cache key configuration
      const { data: cacheKey, error: upsertError } = await ctx.supabase
        .from('kong_cache_keys')
        .upsert({
          id: existing?.id,
          component_id: input.componentId,
          project_id: input.projectId,
          connector_id: input.connectorId,
          cache_key: input.cacheKey,
          ttl_seconds: input.ttlSeconds,
          cache_key_strategy: input.cacheKeyStrategy,
          content_types: input.contentTypes,
          response_codes: input.responseCodes,
          config: input.config || {},
          is_saved: input.isSaved,
          marked_for_deletion: false,
          updated_at: new Date().toISOString(),
        }, {
          onConflict: 'component_id',
        })
        .select()
        .single();

      if (upsertError) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: 'Failed to save cache key configuration',
        });
      }

      return cacheKey;
    }),

  // Generate a new cache key (UUID)
  generateKey: protectedProcedure
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

      // Generate a new UUID for the cache key
      return { cacheKey: randomUUID() };
    }),

  // Mark cache key for deletion (when component is removed)
  markForDeletion: protectedProcedure
    .input(z.object({
      componentId: z.string().uuid(),
      projectId: z.string().uuid(),
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

      // Mark cache key for deletion
      const { data: cacheKey, error: updateError } = await ctx.supabase
        .from('kong_cache_keys')
        .update({
          marked_for_deletion: true,
          updated_at: new Date().toISOString(),
        })
        .eq('component_id', input.componentId)
        .eq('project_id', input.projectId)
        .select()
        .single();

      if (updateError) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: 'Failed to mark cache key for deletion',
        });
      }

      return cacheKey;
    }),

  // Get cache keys marked for deletion (for cleanup on deploy)
  getMarkedForDeletion: protectedProcedure
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

      // Get cache keys marked for deletion
      const { data: cacheKeys, error: fetchError } = await ctx.supabase
        .from('kong_cache_keys')
        .select('*')
        .eq('project_id', input.projectId)
        .eq('marked_for_deletion', true);

      if (fetchError) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: 'Failed to fetch cache keys marked for deletion',
        });
      }

      return cacheKeys || [];
    }),

  // Delete cache keys (cleanup after deploy)
  delete: protectedProcedure
    .input(z.object({
      componentIds: z.array(z.string().uuid()),
      projectId: z.string().uuid(),
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

      // Delete cache keys
      const { error: deleteError } = await ctx.supabase
        .from('kong_cache_keys')
        .delete()
        .in('component_id', input.componentIds)
        .eq('project_id', input.projectId)
        .eq('marked_for_deletion', true);

      if (deleteError) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: 'Failed to delete cache keys',
        });
      }

      return { success: true, deletedCount: input.componentIds.length };
    }),
});

