/**
 * Activities router - CRUD operations and discovery for activities
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure, publicProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';
import { createActivityRegistry } from '@/lib/activities/activity-registry';

/**
 * Sync activities to components - Creates component entries for all activities
 * that don't have corresponding components yet
 */
async function syncActivitiesToComponents(supabase: any, userId: string) {
  // Get all active, non-deprecated activities
  const { data: activities, error: activitiesError } = await supabase
    .from('activities')
    .select('*')
    .eq('is_active', true)
    .eq('deprecated', false);

  if (activitiesError) {
    throw new TRPCError({
      code: 'INTERNAL_SERVER_ERROR',
      message: `Failed to fetch activities: ${activitiesError.message}`,
    });
  }

  if (!activities || activities.length === 0) {
    return { synced: 0, skipped: 0, errors: [] };
  }

  // Get component type ID for 'activity'
  const { data: activityType, error: typeError } = await supabase
    .from('component_types')
    .select('id')
    .eq('name', 'activity')
    .single();

  if (typeError || !activityType) {
    throw new TRPCError({
      code: 'INTERNAL_SERVER_ERROR',
      message: 'Activity component type not found',
    });
  }

  // Get public visibility ID
  const { data: publicVisibility, error: visibilityError } = await supabase
    .from('component_visibility')
    .select('id')
    .eq('name', 'public')
    .single();

  if (visibilityError || !publicVisibility) {
    throw new TRPCError({
      code: 'INTERNAL_SERVER_ERROR',
      message: 'Public visibility not found',
    });
  }

  const results = { synced: 0, skipped: 0, errors: [] as string[] };

  for (const activity of activities) {
    try {
      // Check if component already exists
      const { data: existing } = await supabase
        .from('components')
        .select('id')
        .eq('name', activity.name)
        .single();

      if (existing) {
        results.skipped++;
        continue;
      }

      // Create component from activity
      const { error: insertError } = await supabase
        .from('components')
        .insert({
          name: activity.name,
          display_name: activity.name.replace(/([A-Z])/g, ' $1').trim() || activity.name,
          description: activity.description || '',
          component_type_id: activityType.id,
          version: '1.0.0', // Default version
          created_by: userId,
          visibility_id: publicVisibility.id,
          tags: activity.tags || [],
          capabilities: activity.category ? [activity.category.toLowerCase()] : [],
          config_schema: activity.input_schema || {},
          input_schema: activity.input_schema || {},
          output_schema: activity.output_schema || {},
          implementation_path: activity.module_path || null,
          npm_package: activity.package_name || null,
        });

      if (insertError) {
        results.errors.push(`${activity.name}: ${insertError.message}`);
      } else {
        results.synced++;
      }
    } catch (err: any) {
      results.errors.push(`${activity.name}: ${err.message}`);
    }
  }

  return results;
}

export const activitiesRouter = createTRPCRouter({
  /**
   * List all activities with optional filters
   */
  list: protectedProcedure
    .input(z.object({
      category: z.string().optional(),
      tags: z.array(z.string()).optional(),
      search: z.string().optional(),
      includeDeprecated: z.boolean().optional().default(false),
      isActive: z.boolean().optional(),
    }).optional())
    .query(async ({ ctx, input }) => {
      const registry = createActivityRegistry(ctx.supabase);

      try {
        const activities = await registry.listActivities(input);
        return activities;
      } catch (error) {
        console.error('Error listing activities:', error);
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: 'Failed to list activities',
        });
      }
    }),

  /**
   * Get activity details by name
   */
  get: publicProcedure
    .input(z.object({ name: z.string() }))
    .query(async ({ ctx, input }) => {
      const registry = createActivityRegistry(ctx.supabase);

      try {
        const activity = await registry.getActivity(input.name);

        if (!activity) {
          throw new TRPCError({
            code: 'NOT_FOUND',
            message: `Activity '${input.name}' not found`,
          });
        }

        return activity;
      } catch (error) {
        if (error instanceof TRPCError) throw error;

        console.error('Error getting activity:', error);
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: 'Failed to get activity',
        });
      }
    }),

  /**
   * Get activity by ID
   */
  getById: publicProcedure
    .input(z.object({ id: z.string().uuid() }))
    .query(async ({ ctx, input }) => {
      const registry = createActivityRegistry(ctx.supabase);

      try {
        const activity = await registry.getActivityById(input.id);

        if (!activity) {
          throw new TRPCError({
            code: 'NOT_FOUND',
            message: `Activity with ID '${input.id}' not found`,
          });
        }

        return activity;
      } catch (error) {
        if (error instanceof TRPCError) throw error;

        console.error('Error getting activity by ID:', error);
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: 'Failed to get activity',
        });
      }
    }),

  /**
   * Get all activity categories
   */
  categories: publicProcedure
    .query(async ({ ctx }) => {
      const registry = createActivityRegistry(ctx.supabase);

      try {
        const categories = await registry.getCategoryDetails();
        return categories;
      } catch (error) {
        console.error('Error getting categories:', error);
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: 'Failed to get categories',
        });
      }
    }),

  /**
   * Register a new activity
   */
  register: protectedProcedure
    .input(z.object({
      name: z.string().regex(/^[a-zA-Z][a-zA-Z0-9_]*$/, 'Invalid activity name format'),
      description: z.string().optional(),
      inputSchema: z.any(),
      outputSchema: z.any().optional(),
      packageName: z.string(),
      modulePath: z.string(),
      functionName: z.string().regex(/^[a-zA-Z][a-zA-Z0-9_]*$/, 'Invalid function name format'),
      category: z.string().optional(),
      tags: z.array(z.string()).optional(),
      examples: z.any().optional(),
    }))
    .mutation(async ({ ctx, input }) => {
      const registry = createActivityRegistry(ctx.supabase);

      try {
        const activity = await registry.registerActivity({
          ...input,
          createdBy: ctx.user.id,
        });

        return activity;
      } catch (error: any) {
        console.error('Error registering activity:', error);

        // Handle unique constraint violation
        if (error.code === '23505') {
          throw new TRPCError({
            code: 'CONFLICT',
            message: `Activity '${input.name}' already exists`,
          });
        }

        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: 'Failed to register activity',
        });
      }
    }),

  /**
   * Discover activities from a package
   */
  discover: protectedProcedure
    .input(z.object({
      packagePath: z.string(),
    }))
    .mutation(async ({ ctx, input }) => {
      const registry = createActivityRegistry(ctx.supabase);

      try {
        const activities = await registry.discoverActivities(
          input.packagePath,
          ctx.user.id
        );

        return {
          discovered: activities.length,
          activities,
        };
      } catch (error) {
        console.error('Error discovering activities:', error);
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: 'Failed to discover activities',
        });
      }
    }),

  /**
   * Sync activities to components - Creates component entries for activities
   * that don't have corresponding components yet
   */
  syncToComponents: protectedProcedure
    .mutation(async ({ ctx }) => {
      try {
        const results = await syncActivitiesToComponents(ctx.supabase, ctx.user.id);
        return results;
      } catch (error: any) {
        console.error('Error syncing activities to components:', error);
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: error.message || 'Failed to sync activities to components',
        });
      }
    }),

  /**
   * Track activity usage
   */
  trackUsage: protectedProcedure
    .input(z.object({
      name: z.string(),
    }))
    .mutation(async ({ ctx, input }) => {
      const registry = createActivityRegistry(ctx.supabase);

      try {
        await registry.trackUsage(input.name);
        return { success: true };
      } catch (error) {
        console.error('Error tracking activity usage:', error);
        // Don't throw - usage tracking is non-critical
        return { success: false };
      }
    }),

  /**
   * Deprecate an activity
   */
  deprecate: protectedProcedure
    .input(z.object({
      id: z.string().uuid(),
      message: z.string().optional(),
      migrateToActivityId: z.string().uuid().optional(),
    }))
    .mutation(async ({ ctx, input }) => {
      const registry = createActivityRegistry(ctx.supabase);

      try {
        // Verify ownership or admin role
        const activity = await registry.getActivityById(input.id);

        if (!activity) {
          throw new TRPCError({
            code: 'NOT_FOUND',
            message: 'Activity not found',
          });
        }

        if (activity.created_by !== ctx.user.id) {
          // Check if user is admin
          const { data: userRole } = await ctx.supabase
            .from('user_roles')
            .select('name')
            .eq('id', ctx.user.role_id)
            .single();

          if (!userRole || userRole.name !== 'admin') {
            throw new TRPCError({
              code: 'FORBIDDEN',
              message: 'Not authorized to deprecate this activity',
            });
          }
        }

        const deprecated = await registry.deprecateActivity(
          input.id,
          input.message,
          input.migrateToActivityId
        );

        return deprecated;
      } catch (error) {
        if (error instanceof TRPCError) throw error;

        console.error('Error deprecating activity:', error);
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: 'Failed to deprecate activity',
        });
      }
    }),

  /**
   * Deactivate an activity (soft delete)
   */
  deactivate: protectedProcedure
    .input(z.object({
      id: z.string().uuid(),
    }))
    .mutation(async ({ ctx, input }) => {
      const registry = createActivityRegistry(ctx.supabase);

      try {
        // Verify ownership or admin role
        const activity = await registry.getActivityById(input.id);

        if (!activity) {
          throw new TRPCError({
            code: 'NOT_FOUND',
            message: 'Activity not found',
          });
        }

        if (activity.created_by !== ctx.user.id) {
          // Check if user is admin
          const { data: userRole } = await ctx.supabase
            .from('user_roles')
            .select('name')
            .eq('id', ctx.user.role_id)
            .single();

          if (!userRole || userRole.name !== 'admin') {
            throw new TRPCError({
              code: 'FORBIDDEN',
              message: 'Not authorized to deactivate this activity',
            });
          }
        }

        const deactivated = await registry.deactivateActivity(input.id);
        return deactivated;
      } catch (error) {
        if (error instanceof TRPCError) throw error;

        console.error('Error deactivating activity:', error);
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: 'Failed to deactivate activity',
        });
      }
    }),

  /**
   * Get activity usage statistics
   */
  usageStats: protectedProcedure
    .query(async ({ ctx }) => {
      const registry = createActivityRegistry(ctx.supabase);

      try {
        const stats = await registry.getUsageStats();
        return stats;
      } catch (error) {
        console.error('Error getting usage stats:', error);
        throw new TRPCError({
          code: 'INTERNAL_SERVER_ERROR',
          message: 'Failed to get usage statistics',
        });
      }
    }),
});
