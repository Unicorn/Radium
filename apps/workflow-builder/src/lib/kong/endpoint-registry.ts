/**
 * Endpoint Registration Service
 * 
 * Registers workflow endpoints with Kong API Gateway and stores
 * configuration in the database.
 */

import { KongClient } from './client';
import { generateEndpointHash } from './hash-generator';
import { getKongConfig, isKongEnabled } from './config';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';

export interface WorkflowEndpoint {
  endpointPath: string; // e.g., "/orders" (relative path)
  method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  description?: string;
  targetType: 'signal' | 'query' | 'start';
  targetName: string; // Signal/query name or 'start'
  authType?: 'api-key' | 'jwt' | 'none';
  authConfig?: Record<string, any>;
  rateLimitPerMinute?: number;
  rateLimitPerHour?: number;
  requestSchema?: Record<string, any>;
  responseSchema?: Record<string, any>;
}

export interface RegisteredEndpoint {
  id: string;
  hash: string;
  path: string;
  fullPath: string;
  method: string;
  url: string;
  kongRouteId: string;
}

/**
 * Register workflow endpoints with Kong
 */
export async function registerWorkflowEndpoints(
  workflowId: string,
  userId: string,
  projectId: string,
  endpoints: WorkflowEndpoint[],
  supabase: SupabaseClient<Database>
): Promise<RegisteredEndpoint[]> {
  if (!isKongEnabled()) {
    console.warn('⚠️  Kong is not enabled. Skipping endpoint registration.');
    return [];
  }

  const config = getKongConfig();
  const kong = new KongClient();

  // Verify Kong is accessible
  const isHealthy = await kong.healthCheck();
  if (!isHealthy) {
    throw new Error('Kong Admin API is not accessible. Check KONG_ADMIN_URL and ensure Kong is running.');
  }

  // Get workflow and project info
  const { data: workflow, error: workflowError } = await supabase
    .from('workflows')
    .select('*, project:projects(*)')
    .eq('id', workflowId)
    .single();

  if (workflowError || !workflow) {
    throw new Error('Workflow not found');
  }

  // Create or get Kong service for workflow API handler
  // Kong routes to the app URL, and our Next.js route handler processes the path
  const serviceName = 'workflow-api-handler';
  const serviceUrl = config.appUrl; // Base app URL, not apiBaseUrl
  
  let serviceId: string;
  try {
    serviceId = await kong.createService(serviceName, serviceUrl);
    console.log(`✅ Created/retrieved Kong service: ${serviceName}`);
  } catch (error) {
    console.error('❌ Failed to create Kong service:', error);
    throw error;
  }

  const registered: RegisteredEndpoint[] = [];

  for (const endpoint of endpoints) {
    try {
      // Generate hash for this endpoint
      const routeHash = generateEndpointHash({
        userId,
        projectId,
        workflowId,
        endpointPath: endpoint.endpointPath,
      });

      // Full path with hash: /api/v1/{hash}/{endpoint-path}
      const fullPath = `/api/v1/${routeHash}${endpoint.endpointPath.startsWith('/') ? endpoint.endpointPath : `/${endpoint.endpointPath}`}`;

      // Create Kong route with hash-based path
      const routeName = `wf-${routeHash}-${endpoint.method.toLowerCase()}`;
      
      // Check if route already exists
      const existingRoute = await kong.getRoute(routeName);
      let routeId: string;

      if (existingRoute) {
        // Route exists, use existing ID
        routeId = existingRoute.id;
        console.log(`ℹ️  Route already exists: ${routeName}`);
      } else {
        // Create new route
        routeId = await kong.createRoute(
          serviceId,
          routeName,
          [fullPath],
          [endpoint.method]
        );
        console.log(`✅ Created Kong route: ${routeName} -> ${fullPath}`);
      }

      // Enable rate limiting plugin
      if (endpoint.rateLimitPerMinute) {
        try {
          await kong.enablePlugin(routeId, 'rate-limiting', {
            minute: endpoint.rateLimitPerMinute,
            hour: endpoint.rateLimitPerHour || endpoint.rateLimitPerMinute * 60,
          });
          console.log(`✅ Enabled rate limiting: ${endpoint.rateLimitPerMinute}/min`);
        } catch (error) {
          console.warn(`⚠️  Failed to enable rate limiting: ${error}`);
        }
      }

      // Enable API key authentication
      if (endpoint.authType === 'api-key') {
        try {
          await kong.enablePlugin(routeId, 'key-auth', {
            key_names: ['X-API-Key'],
            hide_credentials: true,
          });
          console.log(`✅ Enabled API key authentication`);
        } catch (error) {
          console.warn(`⚠️  Failed to enable API key auth: ${error}`);
        }
      }

      // Store endpoint in database
      const { data: endpointRecord, error: dbError } = await supabase
        .from('workflow_endpoints')
        .upsert({
          workflow_id: workflowId,
          project_id: projectId,
          user_id: userId,
          endpoint_path: endpoint.endpointPath,
          method: endpoint.method,
          description: endpoint.description,
          route_hash: routeHash,
          full_path: fullPath,
          target_type: endpoint.targetType,
          target_name: endpoint.targetName,
          kong_route_id: routeId,
          kong_service_id: serviceId,
          kong_route_name: routeName,
          auth_type: endpoint.authType || 'api-key',
          auth_config: endpoint.authConfig || {},
          rate_limit_per_minute: endpoint.rateLimitPerMinute || 60,
          rate_limit_per_hour: endpoint.rateLimitPerHour || 1000,
          request_schema: endpoint.requestSchema || null,
          response_schema: endpoint.responseSchema || null,
          is_active: true,
          updated_at: new Date().toISOString(),
        }, {
          onConflict: 'workflow_id,endpoint_path,method',
        })
        .select()
        .single();

      if (dbError) {
        console.error('❌ Failed to store endpoint in database:', dbError);
        throw dbError;
      }

      registered.push({
        id: endpointRecord.id,
        hash: routeHash,
        path: endpoint.endpointPath,
        fullPath,
        method: endpoint.method,
        url: `${config.gatewayUrl}${fullPath}`,
        kongRouteId: routeId,
      });

      console.log(`✅ Registered endpoint: ${endpoint.method} ${fullPath}`);
    } catch (error) {
      console.error(`❌ Failed to register endpoint ${endpoint.endpointPath}:`, error);
      // Continue with other endpoints
    }
  }

  return registered;
}

/**
 * Unregister workflow endpoints (delete from Kong and database)
 */
export async function unregisterWorkflowEndpoints(
  workflowId: string,
  supabase: SupabaseClient<Database>
): Promise<void> {
  if (!isKongEnabled()) {
    return;
  }

  const kong = new KongClient();

  // Get all endpoints for this workflow
  const { data: endpoints, error } = await supabase
    .from('workflow_endpoints')
    .select('kong_route_id')
    .eq('workflow_id', workflowId)
    .eq('is_active', true);

  if (error) {
    console.error('❌ Failed to fetch endpoints:', error);
    return;
  }

  // Delete routes from Kong
  for (const endpoint of endpoints || []) {
    if (endpoint.kong_route_id) {
      try {
        await kong.deleteRoute(endpoint.kong_route_id);
        console.log(`✅ Deleted Kong route: ${endpoint.kong_route_id}`);
      } catch (error) {
        console.warn(`⚠️  Failed to delete route ${endpoint.kong_route_id}:`, error);
      }
    }
  }

  // Mark endpoints as inactive in database
  await supabase
    .from('workflow_endpoints')
    .update({ is_active: false })
    .eq('workflow_id', workflowId);
}

