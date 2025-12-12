/**
 * Connectors router - Manage connectors for external services
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';

const connectorTypeSchema = z.enum(['email', 'slack', 'database', 'api', 'oauth']);

export const connectorsRouter = createTRPCRouter({
  // List connectors for a project
  list: protectedProcedure
    .input(z.object({
      projectId: z.string().uuid(),
      connectorType: connectorTypeSchema.optional(),
    }))
    .query(async ({ ctx, input }) => {
      // Verify user owns the project
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

      let query = ctx.supabase
        .from('connectors')
        .select(`
          id,
          project_id,
          connector_type,
          name,
          display_name,
          description,
          config_schema,
          is_active,
          created_by,
          created_at,
          updated_at
        `)
        .eq('project_id', input.projectId);

      if (input.connectorType) {
        query = query.eq('connector_type', input.connectorType);
      }

      const { data, error } = await query
        .order('connector_type')
        .order('name');

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      // Note: We don't return config_data or credentials_encrypted for security
      return data || [];
    }),

  // Get connector by ID
  get: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .query(async ({ ctx, input }) => {
      const { data, error } = await ctx.supabase
        .from('connectors')
        .select(`
          *,
          project:projects(id, name, created_by)
        `)
        .eq('id', input.id)
        .single();

      if (error || !data) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Connector not found',
        });
      }

      // Verify ownership
      if (data.project?.created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'You do not have access to this connector',
        });
      }

      // TODO: Decrypt credentials_encrypted here
      // For now, we'll return it as-is (should be decrypted in production)

      return data;
    }),

  // Create new connector (with encryption)
  create: protectedProcedure
    .input(z.object({
      projectId: z.string().uuid(),
      connectorType: connectorTypeSchema,
      name: z.string().min(1).max(255),
      displayName: z.string().min(1).max(255),
      description: z.string().optional(),
      configSchema: z.record(z.any()),
      configData: z.record(z.any()),
      credentials: z.record(z.any()).optional(), // Will be encrypted
      oauthConfig: z.record(z.any()).optional(),
      isActive: z.boolean().default(true),
    }))
    .mutation(async ({ ctx, input }) => {
      // Verify user owns the project
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

      // TODO: Encrypt credentials here
      // For now, we'll store them as-is (should be encrypted in production)
      const credentialsEncrypted = input.credentials 
        ? Buffer.from(JSON.stringify(input.credentials))
        : null;

      const { data, error } = await ctx.supabase
        .from('connectors')
        .insert({
          project_id: input.projectId,
          connector_type: input.connectorType,
          name: input.name,
          display_name: input.displayName,
          description: input.description,
          config_schema: input.configSchema,
          config_data: input.configData,
          credentials_encrypted: credentialsEncrypted,
          oauth_config: input.oauthConfig,
          is_active: input.isActive,
          created_by: ctx.user.id,
        })
        .select()
        .single();

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      // Don't return encrypted credentials
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const { credentials_encrypted, ...safeData } = data;
      return safeData;
    }),

  // Update connector
  update: protectedProcedure
    .input(z.object({
      id: z.string().uuid(),
      displayName: z.string().min(1).max(255).optional(),
      description: z.string().optional(),
      configData: z.record(z.any()).optional(),
      credentials: z.record(z.any()).optional(),
      oauthConfig: z.record(z.any()).optional(),
      isActive: z.boolean().optional(),
    }))
    .mutation(async ({ ctx, input }) => {
      // Get and verify ownership
      const { data: connector, error: getError } = await ctx.supabase
        .from('connectors')
        .select(`
          id,
          project:projects(id, created_by)
        `)
        .eq('id', input.id)
        .single();

      if (getError || !connector) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Connector not found',
        });
      }

      if (connector.project?.created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'You do not have access to this connector',
        });
      }

      const updateData: Record<string, any> = {};
      if (input.displayName !== undefined) updateData.display_name = input.displayName;
      if (input.description !== undefined) updateData.description = input.description;
      if (input.configData !== undefined) updateData.config_data = input.configData;
      if (input.oauthConfig !== undefined) updateData.oauth_config = input.oauthConfig;
      if (input.isActive !== undefined) updateData.is_active = input.isActive;

      // Encrypt credentials if provided
      if (input.credentials !== undefined) {
        updateData.credentials_encrypted = Buffer.from(JSON.stringify(input.credentials));
      }

      const { data, error } = await ctx.supabase
        .from('connectors')
        .update(updateData)
        .eq('id', input.id)
        .select()
        .single();

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      // Don't return encrypted credentials
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const { credentials_encrypted, ...safeData } = data;
      return safeData;
    }),

  // Delete connector
  delete: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .mutation(async ({ ctx, input }) => {
      // Get and verify ownership
      const { data: connector, error: getError } = await ctx.supabase
        .from('connectors')
        .select(`
          id,
          project:projects(id, created_by)
        `)
        .eq('id', input.id)
        .single();

      if (getError || !connector) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Connector not found',
        });
      }

      if (connector.project?.created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'You do not have access to this connector',
        });
      }

      const { error } = await ctx.supabase
        .from('connectors')
        .delete()
        .eq('id', input.id);

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return { success: true };
    }),

  // Test connector connection
  test: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .mutation(async ({ ctx, input }) => {
      // Get connector with decrypted credentials
      const { data: connector, error: getError } = await ctx.supabase
        .from('connectors')
        .select(`
          *,
          project:projects(id, created_by)
        `)
        .eq('id', input.id)
        .single();

      if (getError || !connector) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Connector not found',
        });
      }

      if (connector.project?.created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'You do not have access to this connector',
        });
      }

      // TODO: Implement actual connection testing based on connector_type
      // For now, return a placeholder response
      return {
        success: true,
        message: 'Connection test not yet implemented',
        connectorType: connector.connector_type,
      };
    }),

  // Get connectors by type
  getByType: protectedProcedure
    .input(z.object({
      projectId: z.string().uuid(),
      connectorType: connectorTypeSchema,
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

      const { data, error } = await ctx.supabase
        .from('connectors')
        .select(`
          id,
          name,
          display_name,
          description,
          is_active
        `)
        .eq('project_id', input.projectId)
        .eq('connector_type', input.connectorType)
        .eq('is_active', true)
        .order('name');

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return data || [];
    }),

  // Get connectors by classification
  getByClassification: protectedProcedure
    .input(z.object({
      projectId: z.string().uuid(),
      classification: z.enum(['redis', 'http-log', 'syslog', 'file-log', 'tcp-log', 'udp-log']),
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

      // Query connectors with the specified classification
      const { data, error } = await ctx.supabase
        .from('connectors')
        .select(`
          id,
          name,
          display_name,
          description,
          connector_type,
          is_active,
          classifications
        `)
        .eq('project_id', input.projectId)
        .eq('is_active', true)
        .contains('classifications', [input.classification])
        .order('name');

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return data || [];
    }),

  // Add classification to connector
  addClassification: protectedProcedure
    .input(z.object({
      connectorId: z.string().uuid(),
      classification: z.enum(['redis', 'http-log', 'syslog', 'file-log', 'tcp-log', 'udp-log']),
    }))
    .mutation(async ({ ctx, input }) => {
      // Get and verify ownership
      const { data: connector, error: getError } = await ctx.supabase
        .from('connectors')
        .select(`
          id,
          project:projects(id, created_by)
        `)
        .eq('id', input.connectorId)
        .single();

      if (getError || !connector) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Connector not found',
        });
      }

      if (connector.project?.created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'You do not have access to this connector',
        });
      }

      // Insert classification (trigger will update classifications JSONB column)
      const { error } = await ctx.supabase
        .from('connector_classifications')
        .insert({
          connector_id: input.connectorId,
          classification: input.classification,
        });

      if (error) {
        // If it's a unique constraint violation, that's OK (already exists)
        if (error.code !== '23505') {
          throw new TRPCError({
            code: 'INTERNAL_SERVER_ERROR',
            message: error.message,
          });
        }
      }

      return { success: true };
    }),

  // Remove classification from connector
  removeClassification: protectedProcedure
    .input(z.object({
      connectorId: z.string().uuid(),
      classification: z.enum(['redis', 'http-log', 'syslog', 'file-log', 'tcp-log', 'udp-log']),
    }))
    .mutation(async ({ ctx, input }) => {
      // Get and verify ownership
      const { data: connector, error: getError } = await ctx.supabase
        .from('connectors')
        .select(`
          id,
          project:projects(id, created_by)
        `)
        .eq('id', input.connectorId)
        .single();

      if (getError || !connector) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Connector not found',
        });
      }

      if (connector.project?.created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'FORBIDDEN',
          message: 'You do not have access to this connector',
        });
      }

      // Delete classification (trigger will update classifications JSONB column)
      const { error } = await ctx.supabase
        .from('connector_classifications')
        .delete()
        .eq('connector_id', input.connectorId)
        .eq('classification', input.classification);

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return { success: true };
    }),
});

