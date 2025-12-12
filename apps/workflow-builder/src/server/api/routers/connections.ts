/**
 * Project Connections API Router
 * 
 * Manages project-level database and service connections
 * (PostgreSQL, Redis, etc.)
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';
import { encryptString, decryptString } from '@/lib/security/encryption';

export const connectionsRouter = createTRPCRouter({
  /**
   * List connections for a project
   */
  list: protectedProcedure
    .input(z.object({
      projectId: z.string(),
    }))
    .query(async ({ ctx, input }) => {
      // Verify project belongs to user
      const { data: project } = await ctx.supabase
        .from('projects')
        .select('id')
        .eq('id', input.projectId)
        .eq('created_by', ctx.user.id)
        .single();

      if (!project) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Project not found',
        });
      }

      const { data, error } = await ctx.supabase
        .from('project_connections')
        .select('*')
        .eq('project_id', input.projectId)
        .order('created_at', { ascending: false });

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      // Decrypt connection URLs
      const connections = await Promise.all(
        (data || []).map(async (conn) => {
          let connectionUrl = conn.connection_url;
          
          // If encrypted credentials exist, decrypt them
          if (conn.encrypted_credentials) {
            try {
              const encryptedBase64 = Buffer.from(conn.encrypted_credentials).toString('base64');
              connectionUrl = await decryptString(encryptedBase64, conn.encryption_key_id || undefined);
            } catch (error) {
              console.error('Failed to decrypt connection credentials:', error);
              // Fall back to plain connection_url if decryption fails
            }
          }
          
          return {
            id: conn.id,
            projectId: conn.project_id,
            connectionType: conn.connection_type,
            name: conn.name,
            connectionUrl,
            config: conn.config,
            createdAt: new Date(conn.created_at),
            updatedAt: new Date(conn.updated_at),
          };
        })
      );

      return { connections };
    }),

  /**
   * Create a new connection
   */
  create: protectedProcedure
    .input(z.object({
      projectId: z.string(),
      connectionType: z.enum(['postgresql', 'redis']),
      name: z.string().min(1),
      connectionUrl: z.string().min(1),
      config: z.record(z.any()).optional(),
    }))
    .mutation(async ({ ctx, input }) => {
      // Verify project belongs to user
      const { data: project } = await ctx.supabase
        .from('projects')
        .select('id')
        .eq('id', input.projectId)
        .eq('created_by', ctx.user.id)
        .single();

      if (!project) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Project not found',
        });
      }

      // Check if connection name already exists
      const { data: existing } = await ctx.supabase
        .from('project_connections')
        .select('id')
        .eq('project_id', input.projectId)
        .eq('name', input.name)
        .single();

      if (existing) {
        throw new TRPCError({
          code: 'CONFLICT',
          message: 'Connection with this name already exists',
        });
      }

      // Encrypt connection URL
      const encryptedCredentials = await encryptString(input.connectionUrl);
      const encryptedBuffer = Buffer.from(encryptedCredentials, 'base64');

      const { data, error } = await ctx.supabase
        .from('project_connections')
        .insert({
          project_id: input.projectId,
          connection_type: input.connectionType,
          name: input.name,
          connection_url: null, // Don't store plain text
          encrypted_credentials: encryptedBuffer,
          encryption_key_id: 'default', // For now, use 'default'. Later support key rotation
          config: input.config || {},
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

      return {
        id: data.id,
        projectId: data.project_id,
        connectionType: data.connection_type,
        name: data.name,
        connectionUrl: data.connection_url,
        config: data.config,
        createdAt: new Date(data.created_at),
        updatedAt: new Date(data.updated_at),
      };
    }),

  /**
   * Update a connection
   */
  update: protectedProcedure
    .input(z.object({
      connectionId: z.string(),
      name: z.string().min(1).optional(),
      connectionUrl: z.string().min(1).optional(),
      config: z.record(z.any()).optional(),
    }))
    .mutation(async ({ ctx, input }) => {
      const { connectionId, ...updates } = input;

      // Verify connection belongs to user's project
      const { data: connection } = await ctx.supabase
        .from('project_connections')
        .select('project_id, projects!inner(created_by)')
        .eq('id', connectionId)
        .single();

      if (!connection || (connection.projects as any).created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Connection not found',
        });
      }

      // If updating name, check for conflicts
      if (updates.name) {
        const { data: existing } = await ctx.supabase
          .from('project_connections')
          .select('id')
          .eq('project_id', (connection as any).project_id)
          .eq('name', updates.name)
          .neq('id', connectionId)
          .single();

        if (existing) {
          throw new TRPCError({
            code: 'CONFLICT',
            message: 'Connection with this name already exists',
          });
        }
      }

      const updateData: any = {};
      if (updates.name) updateData.name = updates.name;
      if (updates.connectionUrl) {
        // Encrypt new connection URL
        const encryptedCredentials = await encryptString(updates.connectionUrl);
        const encryptedBuffer = Buffer.from(encryptedCredentials, 'base64');
        updateData.connection_url = null; // Don't store plain text
        updateData.encrypted_credentials = encryptedBuffer;
        updateData.encryption_key_id = 'default';
      }
      if (updates.config !== undefined) updateData.config = updates.config;

      const { data, error } = await ctx.supabase
        .from('project_connections')
        .update(updateData)
        .eq('id', connectionId)
        .select()
        .single();

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return {
        id: data.id,
        projectId: data.project_id,
        connectionType: data.connection_type,
        name: data.name,
        connectionUrl: data.connection_url,
        config: data.config,
        createdAt: new Date(data.created_at),
        updatedAt: new Date(data.updated_at),
      };
    }),

  /**
   * Delete a connection
   */
  delete: protectedProcedure
    .input(z.object({
      connectionId: z.string(),
    }))
    .mutation(async ({ ctx, input }) => {
      // Verify connection belongs to user's project
      const { data: connection } = await ctx.supabase
        .from('project_connections')
        .select('project_id, projects!inner(created_by)')
        .eq('id', input.connectionId)
        .single();

      if (!connection || (connection.projects as any).created_by !== ctx.user.id) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Connection not found',
        });
      }

      const { error } = await ctx.supabase
        .from('project_connections')
        .delete()
        .eq('id', input.connectionId);

      if (error) {
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message,
        });
      }

      return { success: true };
    }),

  /**
   * Test connection connectivity
   */
  test: protectedProcedure
    .input(z.object({
      connectionId: z.string(),
    }))
    .mutation(async ({ ctx, input }) => {
      // Verify connection belongs to user's project
      const { data: connection } = await ctx.supabase
        .from('project_connections')
        .select('*')
        .eq('id', input.connectionId)
        .single();

      if (!connection) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Connection not found',
        });
      }

      // Verify project ownership
      const { data: project } = await ctx.supabase
        .from('projects')
        .select('id')
        .eq('id', connection.project_id)
        .eq('created_by', ctx.user.id)
        .single();

      if (!project) {
        throw new TRPCError({
          code: 'NOT_FOUND',
          message: 'Project not found',
        });
      }

      // Decrypt connection URL for testing
      let connectionUrl = connection.connection_url;
      if (connection.encrypted_credentials) {
        try {
          const encryptedBase64 = Buffer.from(connection.encrypted_credentials).toString('base64');
          connectionUrl = await decryptString(encryptedBase64, connection.encryption_key_id || undefined);
        } catch (error) {
          return {
            success: false,
            message: 'Failed to decrypt connection credentials',
            error: error instanceof Error ? error.message : 'Decryption failed',
          };
        }
      }

      if (!connectionUrl) {
        return {
          success: false,
          message: 'Connection URL not found',
          error: 'Connection URL is missing',
        };
      }

      // Test connection based on type
      try {
        if (connection.connection_type === 'postgresql') {
          // Test PostgreSQL connection
          const { testPostgresConnection } = await import('@/lib/connections/postgresql');
          const result = await testPostgresConnection(connectionUrl);
          return {
            success: result.success,
            message: result.message,
            error: result.error,
          };
        } else if (connection.connection_type === 'redis') {
          // Test Redis connection
          const { testRedisConnection } = await import('@/lib/connections/redis');
          const result = await testRedisConnection(connectionUrl);
          return {
            success: result.success,
            message: result.message,
            error: result.error,
          };
        } else {
          return {
            success: false,
            message: 'Unknown connection type',
            error: `Unsupported connection type: ${connection.connection_type}`,
          };
        }
      } catch (error: any) {
        return {
          success: false,
          message: 'Connection test failed',
          error: error.message || 'Unknown error',
        };
      }
    }),
});

