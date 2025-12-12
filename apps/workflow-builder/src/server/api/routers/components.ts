/**
 * Components router - CRUD operations for workflow components
 */

import { z } from 'zod';
import { createTRPCRouter, protectedProcedure, publicProcedure } from '../trpc';
import { TRPCError } from '@trpc/server';
import { validateTypeScriptCode, validateActivityStructure, extractExportedFunctions } from '@/lib/typescript-validator';

export const componentsRouter = createTRPCRouter({
  // List all accessible components
  list: protectedProcedure
    .input(z.object({
      type: z.string().optional(),
      capability: z.string().optional(),
      tags: z.array(z.string()).optional(),
      includeDeprecated: z.boolean().default(false),
      page: z.number().default(1),
      pageSize: z.number().default(50),
    }))
    .query(async ({ ctx, input }) => {
      const { supabase } = ctx;
      
      let query = supabase
        .from('components')
        .select(`
          *,
          component_type:component_types(id, name, icon),
          created_by_user:users!components_created_by_fkey(id, display_name),
          visibility:component_visibility(id, name),
          category_mappings:component_category_mapping(
            is_primary,
            category:component_categories(
              id,
              name,
              display_name,
              description,
              icon,
              icon_provider,
              color,
              parent_category_id,
              sort_order
            )
          ),
          keywords:component_keywords(keyword, relevance_score),
          use_cases:component_use_cases(use_case, description)
        `, { count: 'exact' });
      
      // Apply filters
      if (input.type) {
        const { data: typeData } = await supabase
          .from('component_types')
          .select('id')
          .eq('name', input.type)
          .single();
        
        if (typeData) {
          query = query.eq('component_type_id', typeData.id);
        }
      }
      
      if (input.capability) {
        query = query.contains('capabilities', [input.capability]);
      }
      
      if (input.tags && input.tags.length > 0) {
        query = query.overlaps('tags', input.tags);
      }
      
      if (!input.includeDeprecated) {
        query = query.eq('deprecated', false);
      }
      
      // Pagination
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

      // Also fetch activities that aren't components yet and merge them
      const { data: activities } = await supabase
        .from('activities')
        .select('*')
        .eq('is_active', true)
        .eq('deprecated', false);

      // Get component type and visibility for activities
      const { data: activityType } = await supabase
        .from('component_types')
        .select('id, name, icon')
        .eq('name', 'activity')
        .single();

      const { data: publicVisibility } = await supabase
        .from('component_visibility')
        .select('id, name')
        .eq('name', 'public')
        .single();

      // Convert activities to component-like objects
      const activityComponents = (activities || [])
        .filter(activity => {
          // Only include activities that don't have a corresponding component
          return !data?.some(comp => comp.name === activity.name);
        })
        .map(activity => ({
          id: `activity-${activity.id}`, // Temporary ID
          name: activity.name,
          display_name: activity.name.replace(/([A-Z])/g, ' $1').trim() || activity.name,
          description: activity.description || '',
          component_type_id: activityType?.id || '',
          component_type: activityType || { id: '', name: 'activity', icon: null },
          version: '1.0.0',
          created_by: activity.created_by,
          created_by_user: null,
          visibility_id: publicVisibility?.id || '',
          visibility: publicVisibility || { id: '', name: 'public' },
          tags: activity.tags || [],
          capabilities: activity.category ? [activity.category.toLowerCase()] : [],
          config_schema: activity.input_schema || {},
          input_schema: activity.input_schema || {},
          output_schema: activity.output_schema || {},
          implementation_path: activity.module_path || null,
          npm_package: activity.package_name || null,
          deprecated: false,
          created_at: activity.created_at,
          updated_at: activity.updated_at,
          // Mark as activity source
          _source: 'activity' as const,
          _activity_id: activity.id,
        }));

      // Merge components and activity components
      const allComponents = [
        ...(data || []),
        ...activityComponents,
      ];

      return {
        components: allComponents,
        total: (count || 0) + activityComponents.length,
        page: input.page,
        pageSize: input.pageSize,
        totalPages: Math.ceil(((count || 0) + activityComponents.length) / input.pageSize),
      };
    }),

  // Get single component by ID
  get: publicProcedure
    .input(z.object({ id: z.string().uuid() }))
    .query(async ({ ctx, input }) => {
      const { data, error } = await ctx.supabase
        .from('components')
        .select(`
          *,
          component_type:component_types(id, name, icon, description),
          created_by_user:users!components_created_by_fkey(id, display_name, email),
          visibility:component_visibility(id, name, description),
          agent_prompt:agent_prompts(id, name, display_name, version),
          category_mappings:component_category_mapping(
            is_primary,
            category:component_categories(
              id,
              name,
              display_name,
              description,
              icon,
              icon_provider,
              color,
              parent_category_id,
              sort_order
            )
          ),
          keywords:component_keywords(keyword, relevance_score),
          use_cases:component_use_cases(use_case, description)
        `)
        .eq('id', input.id)
        .single();
      
      if (error) {
        throw new TRPCError({ 
          code: 'NOT_FOUND', 
          message: 'Component not found' 
        });
      }
      
      return data;
    }),

  // Create new component
  create: protectedProcedure
    .input(z.object({
      name: z.string().min(1).max(255),
      displayName: z.string().min(1).max(255),
      description: z.string().optional(),
      componentType: z.enum(['activity', 'agent', 'signal', 'trigger', 'data-in', 'data-out']),
      version: z.string().regex(/^\d+\.\d+\.\d+$/, 'Must be valid semver'),
      visibility: z.enum(['public', 'private', 'organization']),
      configSchema: z.any().optional(),
      inputSchema: z.any().optional(),
      outputSchema: z.any().optional(),
      tags: z.array(z.string()).optional(),
      capabilities: z.array(z.string()).optional(),
      // Agent-specific
      agentPromptId: z.string().uuid().optional(),
      modelProvider: z.string().optional(),
      modelName: z.string().optional(),
      // Implementation
      implementationPath: z.string().optional(),
      npmPackage: z.string().optional(),
      // Custom activity code
      implementationLanguage: z.string().optional(),
      implementationCode: z.string().optional(),
      // Interface component specific
      endpointPath: z.string().optional(),
      httpMethod: z.enum(['GET', 'POST', 'PATCH', 'PUT', 'DELETE']).optional(),
    }))
    .mutation(async ({ ctx, input }) => {
      const { supabase, user } = ctx;
      
      // Get type ID
      const { data: componentType } = await supabase
        .from('component_types')
        .select('id')
        .eq('name', input.componentType)
        .single();
      
      if (!componentType) {
        throw new TRPCError({ 
          code: 'BAD_REQUEST', 
          message: 'Invalid component type' 
        });
      }
      
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
      
      // Check for duplicate name/version
      const { data: existing } = await supabase
        .from('components')
        .select('id')
        .eq('name', input.name)
        .eq('version', input.version)
        .eq('created_by', user.id)
        .single();
      
      if (existing) {
        throw new TRPCError({ 
          code: 'BAD_REQUEST', 
          message: 'Component with this name and version already exists' 
        });
      }

      // Validate custom activity code if provided
      if (input.implementationCode) {
        const validation = validateTypeScriptCode(input.implementationCode);
        
        if (!validation.valid) {
          throw new TRPCError({
            code: 'BAD_REQUEST',
            message: `TypeScript validation failed: ${validation.errors.map(e => e.message).join(', ')}`,
          });
        }

        // Validate activity structure
        const structureValidation = validateActivityStructure(input.implementationCode);
        if (!structureValidation.valid) {
          throw new TRPCError({
            code: 'BAD_REQUEST',
            message: structureValidation.error || 'Invalid activity structure',
          });
        }

        // Extract exported functions for metadata
        const exportedFunctions = extractExportedFunctions(input.implementationCode);
        if (exportedFunctions.length === 0) {
          throw new TRPCError({
            code: 'BAD_REQUEST',
            message: 'Activity code must export at least one function',
          });
        }
      }
      
      // Build config schema for interface components
      let configSchema = input.configSchema || null;
      if ((input.componentType === 'data-in' || input.componentType === 'data-out') && input.endpointPath) {
        configSchema = {
          ...(configSchema || {}),
          endpointPath: input.endpointPath,
          httpMethod: input.httpMethod || (input.componentType === 'data-in' ? 'POST' : 'GET'),
        };
      }
      
      // Create component
      const { data, error } = await supabase
        .from('components')
        .insert({
          name: input.name,
          display_name: input.displayName,
          description: input.description,
          component_type_id: componentType.id,
          version: input.version,
          created_by: user.id,
          visibility_id: visibility.id,
          config_schema: configSchema,
          input_schema: input.inputSchema || null,
          output_schema: input.outputSchema || null,
          tags: input.tags || null,
          capabilities: input.capabilities || null,
          agent_prompt_id: input.agentPromptId || null,
          model_provider: input.modelProvider || null,
          model_name: input.modelName || null,
          implementation_path: input.implementationPath || null,
          npm_package: input.npmPackage || null,
          implementation_language: input.implementationLanguage || null,
          implementation_code: input.implementationCode || null,
          is_active: true,
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

  // Update component
  update: protectedProcedure
    .input(z.object({
      id: z.string().uuid(),
      displayName: z.string().min(1).max(255).optional(),
      description: z.string().optional(),
      tags: z.array(z.string()).optional(),
      capabilities: z.array(z.string()).optional(),
      deprecated: z.boolean().optional(),
      deprecatedMessage: z.string().optional(),
      migrateToComponentId: z.string().uuid().optional(),
    }))
    .mutation(async ({ ctx, input }) => {
      const { id, ...updates } = input;
      
      // Check ownership
      const { data: component } = await ctx.supabase
        .from('components')
        .select('created_by')
        .eq('id', id)
        .single();
      
      if (!component || component.created_by !== ctx.user.id) {
        throw new TRPCError({ 
          code: 'FORBIDDEN', 
          message: 'Not authorized to update this component' 
        });
      }
      
      // Build update object
      const updateData: any = {
        updated_at: new Date().toISOString(),
      };
      
      if (updates.displayName !== undefined) updateData.display_name = updates.displayName;
      if (updates.description !== undefined) updateData.description = updates.description;
      if (updates.tags !== undefined) updateData.tags = updates.tags;
      if (updates.capabilities !== undefined) updateData.capabilities = updates.capabilities;
      if (updates.deprecated !== undefined) updateData.deprecated = updates.deprecated;
      if (updates.deprecatedMessage !== undefined) updateData.deprecated_message = updates.deprecatedMessage;
      if (updates.migrateToComponentId !== undefined) updateData.migrate_to_component_id = updates.migrateToComponentId;
      
      if (updates.deprecated) {
        updateData.deprecated_since = new Date().toISOString();
      }
      
      // Update
      const { data, error } = await ctx.supabase
        .from('components')
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

  // Delete component
  delete: protectedProcedure
    .input(z.object({ id: z.string().uuid() }))
    .mutation(async ({ ctx, input }) => {
      // Check ownership
      const { data: component } = await ctx.supabase
        .from('components')
        .select('created_by')
        .eq('id', input.id)
        .single();
      
      if (!component || component.created_by !== ctx.user.id) {
        throw new TRPCError({ 
          code: 'FORBIDDEN', 
          message: 'Not authorized to delete this component' 
        });
      }
      
      // Check if component is used in any workflows
      const { count } = await ctx.supabase
        .from('workflow_nodes')
        .select('*', { count: 'exact', head: true })
        .eq('component_id', input.id);
      
      if (count && count > 0) {
        throw new TRPCError({ 
          code: 'BAD_REQUEST', 
          message: `Component is used in ${count} workflow(s). Deprecate it instead of deleting.` 
        });
      }
      
      // Delete
      const { error } = await ctx.supabase
        .from('components')
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

  // Get component types (for dropdowns)
  getTypes: publicProcedure
    .query(async ({ ctx }) => {
      const { data, error } = await ctx.supabase
        .from('component_types')
        .select('*')
        .order('name');
      
      if (error) {
        throw new TRPCError({ 
          code: 'INTERNAL_SERVER_ERROR', 
          message: error.message 
        });
      }
      
      return data;
    }),

  // Validate TypeScript code
  validateTypeScript: publicProcedure
    .input(z.object({
      code: z.string(),
    }))
    .mutation(({ input }) => {
      const validation = validateTypeScriptCode(input.code, { strict: true });
      const structureValidation = validateActivityStructure(input.code);
      const exportedFunctions = extractExportedFunctions(input.code);

      return {
        valid: validation.valid && structureValidation.valid,
        errors: [
          ...validation.errors,
          ...(structureValidation.valid ? [] : [{ 
            line: 0, 
            column: 0, 
            message: structureValidation.error || 'Invalid structure',
            code: 0 
          }]),
        ],
        warnings: validation.warnings,
        exportedFunctions,
      };
    }),

  // Get all component categories with hierarchy
  getCategories: publicProcedure
    .query(async ({ ctx }) => {
      const { supabase } = ctx;
      
      const { data, error } = await supabase
        .from('component_categories')
        .select('*')
        .order('sort_order', { ascending: true })
        .order('display_name', { ascending: true });
      
      if (error) {
        throw new TRPCError({ 
          code: 'INTERNAL_SERVER_ERROR', 
          message: error.message 
        });
      }

      // Build hierarchical structure
      const categoryMap = new Map<string, any>();
      const rootCategories: any[] = [];

      // First pass: create map
      (data || []).forEach(cat => {
        categoryMap.set(cat.id, { ...cat, children: [] });
      });

      // Second pass: build tree
      (data || []).forEach(cat => {
        const category = categoryMap.get(cat.id)!;
        if (cat.parent_category_id) {
          const parent = categoryMap.get(cat.parent_category_id);
          if (parent) {
            parent.children.push(category);
          } else {
            // Orphaned category, add to root
            rootCategories.push(category);
          }
        } else {
          rootCategories.push(category);
        }
      });

      return {
        categories: rootCategories,
        flat: data || [],
      };
    }),

  // Get category tree (hierarchical structure)
  getCategoryTree: publicProcedure
    .query(async ({ ctx }) => {
      const { supabase } = ctx;
      
      const { data, error } = await supabase
        .from('component_categories')
        .select('*')
        .order('sort_order', { ascending: true })
        .order('display_name', { ascending: true });
      
      if (error) {
        throw new TRPCError({ 
          code: 'INTERNAL_SERVER_ERROR', 
          message: error.message 
        });
      }

      // Build tree structure
      const buildTree = (parentId: string | null = null): any[] => {
        return (data || [])
          .filter(cat => cat.parent_category_id === parentId)
          .map(cat => ({
            ...cat,
            children: buildTree(cat.id),
          }))
          .sort((a, b) => (a.sort_order || 0) - (b.sort_order || 0));
      };

      return buildTree();
    }),

  // Get components grouped by category
  getComponentsByCategory: publicProcedure
    .input(z.object({
      categoryId: z.string().uuid().optional(),
      includeChildren: z.boolean().default(true),
    }))
    .query(async ({ ctx, input }) => {
      const { supabase } = ctx;
      
      let categoryIds: string[] = [];
      
      if (input.categoryId) {
        if (input.includeChildren) {
          // Get category and all children
          const { data: allCategories } = await supabase
            .from('component_categories')
            .select('id');
          
          const getChildrenIds = (parentId: string): string[] => {
            const children = (allCategories || []).filter(
              cat => cat.parent_category_id === parentId
            );
            return [
              parentId,
              ...children.flatMap(child => getChildrenIds(child.id)),
            ];
          };
          
          categoryIds = getChildrenIds(input.categoryId);
        } else {
          categoryIds = [input.categoryId];
        }
      }

      let query = supabase
        .from('components')
        .select(`
          *,
          component_type:component_types(id, name, icon),
          category_mappings:component_category_mapping(
            is_primary,
            category:component_categories(
              id,
              name,
              display_name,
              icon,
              color,
              parent_category_id
            )
          )
        `)
        .eq('is_active', true)
        .eq('deprecated', false);

      if (categoryIds.length > 0) {
        const { data: mappings } = await supabase
          .from('component_category_mapping')
          .select('component_id')
          .in('category_id', categoryIds);
        
        const componentIds = mappings?.map(m => m.component_id) || [];
        if (componentIds.length > 0) {
          query = query.in('id', componentIds);
        } else {
          // No components in this category
          return { components: [], grouped: {} };
        }
      }

      const { data, error } = await query.order('display_name', { ascending: true });

      if (error) {
        throw new TRPCError({ 
          code: 'INTERNAL_SERVER_ERROR', 
          message: error.message 
        });
      }

      // Group by primary category
      const grouped: Record<string, any[]> = {};
      
      (data || []).forEach(comp => {
        const primaryMapping = comp.category_mappings?.find((m: any) => m.is_primary);
        const categoryName = primaryMapping?.category?.name || 'uncategorized';
        
        if (!grouped[categoryName]) {
          grouped[categoryName] = [];
        }
        grouped[categoryName].push(comp);
      });

      return {
        components: data || [],
        grouped,
      };
    }),
});

