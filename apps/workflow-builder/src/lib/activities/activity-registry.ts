/**
 * Activity Registry Service
 *
 * Manages the registry of available Temporal activities that workflows can invoke.
 * Provides discovery, registration, and usage tracking capabilities.
 */

import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';

type Activity = Database['public']['Tables']['activities']['Row'];
type ActivityInsert = Database['public']['Tables']['activities']['Insert'];
type ActivityUpdate = Database['public']['Tables']['activities']['Update'];

export interface RegisterActivityInput {
  name: string;
  description?: string;
  inputSchema: any;
  outputSchema?: any;
  packageName: string;
  modulePath: string;
  functionName: string;
  category?: string;
  tags?: string[];
  examples?: any;
  createdBy: string;
}

export interface ListActivitiesFilters {
  category?: string;
  tags?: string[];
  search?: string;
  includeDeprecated?: boolean;
  isActive?: boolean;
}

export class ActivityRegistry {
  constructor(private supabase: SupabaseClient<Database>) {}

  /**
   * Register a new activity or update existing one
   */
  async registerActivity(input: RegisterActivityInput): Promise<Activity> {
    // Check if activity already exists
    const { data: existing } = await this.supabase
      .from('activities')
      .select('id')
      .eq('name', input.name)
      .single();

    const activityData: ActivityInsert = {
      name: input.name,
      description: input.description || null,
      input_schema: input.inputSchema,
      output_schema: input.outputSchema || null,
      package_name: input.packageName,
      module_path: input.modulePath,
      function_name: input.functionName,
      category: input.category || null,
      tags: input.tags || null,
      examples: input.examples || null,
      created_by: input.createdBy,
      is_active: true,
      deprecated: false,
      usage_count: 0,
    };

    if (existing) {
      // Update existing activity
      const { data, error } = await this.supabase
        .from('activities')
        .update({
          description: activityData.description,
          input_schema: activityData.input_schema,
          output_schema: activityData.output_schema,
          module_path: activityData.module_path,
          function_name: activityData.function_name,
          category: activityData.category,
          tags: activityData.tags,
          examples: activityData.examples,
        } as ActivityUpdate)
        .eq('id', existing.id)
        .select()
        .single();

      if (error) throw error;
      return data;
    } else {
      // Create new activity
      const { data, error } = await this.supabase
        .from('activities')
        .insert(activityData)
        .select()
        .single();

      if (error) throw error;
      return data;
    }
  }

  /**
   * Auto-discover activities from worker package
   *
   * For now, returns known sample activities. In a full implementation,
   * this would use TypeScript AST to scan the package for exported functions.
   */
  async discoverActivities(packagePath: string, userId: string): Promise<Activity[]> {
    const knownActivities: RegisterActivityInput[] = [
      {
        name: 'sampleActivity',
        description: 'A sample activity that processes a message',
        inputSchema: {
          type: 'object',
          properties: {
            message: { type: 'string', description: 'The message to process' }
          },
          required: ['message']
        },
        outputSchema: {
          type: 'string',
          description: 'Processed message result'
        },
        packageName: 'workflow-worker-service',
        modulePath: './activities/sample.activities',
        functionName: 'sampleActivity',
        category: 'Sample',
        tags: ['demo', 'testing'],
        examples: {
          basic: {
            input: { message: 'Hello World' },
            output: 'Processed: Hello World'
          }
        },
        createdBy: userId,
      },
      {
        name: 'buildPackage',
        description: 'Build a package with configurable options',
        inputSchema: {
          type: 'object',
          properties: {
            packageName: { type: 'string', description: 'Name of the package to build' },
            version: { type: 'string', description: 'Version to build', default: 'latest' },
            buildOptions: {
              type: 'object',
              description: 'Build configuration options',
              properties: {
                minify: { type: 'boolean', default: true },
                sourceMaps: { type: 'boolean', default: true },
              }
            }
          },
          required: ['packageName']
        },
        outputSchema: {
          type: 'object',
          properties: {
            success: { type: 'boolean' },
            buildPath: { type: 'string' },
            artifacts: { type: 'array', items: { type: 'string' } }
          }
        },
        packageName: 'workflow-worker-service',
        modulePath: './activities/sample.activities',
        functionName: 'buildPackage',
        category: 'Build',
        tags: ['package', 'build', 'deployment'],
        examples: {
          basic: {
            input: { packageName: 'my-package', version: '1.0.0' },
            output: { success: true, buildPath: '/dist', artifacts: ['bundle.js'] }
          }
        },
        createdBy: userId,
      },
      {
        name: 'httpRequest',
        description: 'Make HTTP requests with timeout and retry support',
        inputSchema: {
          type: 'object',
          properties: {
            url: { type: 'string', format: 'uri', description: 'Target URL' },
            method: {
              type: 'string',
              enum: ['GET', 'POST', 'PUT', 'DELETE', 'PATCH'],
              default: 'GET',
              description: 'HTTP method'
            },
            body: { type: 'object', description: 'Request body for POST/PUT/PATCH' },
            headers: {
              type: 'object',
              description: 'HTTP headers',
              additionalProperties: { type: 'string' }
            },
            timeout: {
              type: 'number',
              default: 30000,
              description: 'Request timeout in milliseconds'
            }
          },
          required: ['url']
        },
        outputSchema: {
          type: 'object',
          properties: {
            status: { type: 'number', description: 'HTTP status code' },
            data: { description: 'Response data' },
            headers: { type: 'object', description: 'Response headers' }
          }
        },
        packageName: 'workflow-worker-service',
        modulePath: './activities/sample.activities',
        functionName: 'httpRequest',
        category: 'HTTP',
        tags: ['http', 'api', 'request', 'network'],
        examples: {
          get: {
            input: { url: 'https://api.example.com/data', method: 'GET' },
            output: { status: 200, data: { result: 'success' }, headers: {} }
          },
          post: {
            input: {
              url: 'https://api.example.com/create',
              method: 'POST',
              body: { name: 'test' }
            },
            output: { status: 201, data: { id: '123', name: 'test' }, headers: {} }
          }
        },
        createdBy: userId,
      },
      {
        name: 'executeQuery',
        description: 'Execute a database query with connection pooling',
        inputSchema: {
          type: 'object',
          properties: {
            query: { type: 'string', description: 'SQL query to execute' },
            params: {
              type: 'array',
              description: 'Query parameters',
              items: { description: 'Parameter value' }
            },
            timeout: { type: 'number', default: 10000 }
          },
          required: ['query']
        },
        outputSchema: {
          type: 'object',
          properties: {
            rows: { type: 'array', description: 'Query results' },
            rowCount: { type: 'number' },
            fields: { type: 'array' }
          }
        },
        packageName: 'workflow-worker-service',
        modulePath: './activities/database.activities',
        functionName: 'executeQuery',
        category: 'Database',
        tags: ['database', 'sql', 'query'],
        createdBy: userId,
      },
      {
        name: 'transformData',
        description: 'Transform data using JSONata expressions',
        inputSchema: {
          type: 'object',
          properties: {
            data: { description: 'Input data to transform' },
            expression: { type: 'string', description: 'JSONata expression' }
          },
          required: ['data', 'expression']
        },
        outputSchema: {
          description: 'Transformed data result'
        },
        packageName: 'workflow-worker-service',
        modulePath: './activities/transform.activities',
        functionName: 'transformData',
        category: 'Transform',
        tags: ['transform', 'data', 'jsonata'],
        createdBy: userId,
      },
    ];

    const registered: Activity[] = [];
    for (const activity of knownActivities) {
      const result = await this.registerActivity(activity);
      registered.push(result);
    }

    return registered;
  }

  /**
   * List activities with optional filtering
   */
  async listActivities(filters?: ListActivitiesFilters): Promise<Activity[]> {
    let query = this.supabase
      .from('activities')
      .select('*');

    // Apply filters
    if (filters?.category) {
      query = query.eq('category', filters.category);
    }

    if (filters?.tags && filters.tags.length > 0) {
      query = query.overlaps('tags', filters.tags);
    }

    if (filters?.isActive !== undefined) {
      query = query.eq('is_active', filters.isActive);
    } else {
      // Default to active only
      query = query.eq('is_active', true);
    }

    if (!filters?.includeDeprecated) {
      query = query.eq('deprecated', false);
    }

    if (filters?.search) {
      // Use full-text search or simple ILIKE
      query = query.or(`name.ilike.%${filters.search}%,description.ilike.%${filters.search}%`);
    }

    // Order by usage count (most popular first), then by name
    query = query.order('usage_count', { ascending: false });
    query = query.order('name', { ascending: true });

    const { data, error } = await query;

    if (error) throw error;
    return data || [];
  }

  /**
   * Get activity by name
   */
  async getActivity(name: string): Promise<Activity | null> {
    const { data, error } = await this.supabase
      .from('activities')
      .select('*')
      .eq('name', name)
      .single();

    if (error) {
      if (error.code === 'PGRST116') return null; // Not found
      throw error;
    }

    return data;
  }

  /**
   * Get activity by ID
   */
  async getActivityById(id: string): Promise<Activity | null> {
    const { data, error } = await this.supabase
      .from('activities')
      .select('*')
      .eq('id', id)
      .single();

    if (error) {
      if (error.code === 'PGRST116') return null; // Not found
      throw error;
    }

    return data;
  }

  /**
   * Track activity usage
   */
  async trackUsage(name: string): Promise<void> {
    const { error } = await this.supabase.rpc('increment_activity_usage', {
      activity_name: name
    });

    // If RPC doesn't exist, fall back to manual update
    if (error?.code === '42883') {
      const activity = await this.getActivity(name);
      if (activity) {
        await this.supabase
          .from('activities')
          .update({
            usage_count: activity.usage_count + 1,
            last_used_at: new Date().toISOString(),
          } as ActivityUpdate)
          .eq('id', activity.id);
      }
    } else if (error) {
      throw error;
    }
  }

  /**
   * Get all activity categories
   */
  async getCategories(): Promise<string[]> {
    const { data, error } = await this.supabase
      .from('activity_categories')
      .select('name')
      .order('name');

    if (error) throw error;
    return data?.map(c => c.name) || [];
  }

  /**
   * Get category details
   */
  async getCategoryDetails() {
    const { data, error } = await this.supabase
      .from('activity_categories')
      .select('*')
      .order('name');

    if (error) throw error;
    return data || [];
  }

  /**
   * Deprecate an activity
   */
  async deprecateActivity(
    id: string,
    message?: string,
    migrateToActivityId?: string
  ): Promise<Activity> {
    const { data, error } = await this.supabase
      .from('activities')
      .update({
        deprecated: true,
        deprecated_message: message || null,
        migrate_to_activity_id: migrateToActivityId || null,
      } as ActivityUpdate)
      .eq('id', id)
      .select()
      .single();

    if (error) throw error;
    return data;
  }

  /**
   * Deactivate an activity (soft delete)
   */
  async deactivateActivity(id: string): Promise<Activity> {
    const { data, error } = await this.supabase
      .from('activities')
      .update({ is_active: false } as ActivityUpdate)
      .eq('id', id)
      .select()
      .single();

    if (error) throw error;
    return data;
  }

  /**
   * Get activity usage statistics
   */
  async getUsageStats() {
    const { data, error } = await this.supabase
      .from('activities')
      .select('name, usage_count, last_used_at, category')
      .eq('is_active', true)
      .order('usage_count', { ascending: false })
      .limit(10);

    if (error) throw error;
    return data || [];
  }
}

/**
 * Helper function to create ActivityRegistry instance
 */
export function createActivityRegistry(supabase: SupabaseClient<Database>) {
  return new ActivityRegistry(supabase);
}
