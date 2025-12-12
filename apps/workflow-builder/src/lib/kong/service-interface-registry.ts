/**
 * Service Interface Registry
 * 
 * Manages Kong routes for public service interfaces.
 * Integrates service_interfaces and public_interfaces tables with Kong.
 */

import { createHash } from 'crypto';
import { KongClient } from './client';
import { generateEndpointHash } from './hash-generator';
import { getKongConfig, isKongEnabled } from './config';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';

export interface PublicInterfaceConfig {
  httpMethod: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  httpPath: string; // e.g., "/orders"
  authType?: 'api_key' | 'oauth2' | 'jwt' | 'none';
  authConfig?: Record<string, any>;
  rateLimitPerMinute?: number;
  rateLimitPerHour?: number;
}

export interface RegisteredPublicInterface {
  id: string;
  serviceInterfaceId: string;
  kongRouteId: string;
  httpMethod: string;
  httpPath: string;
  fullPath: string;
  url: string;
}

/**
 * Register a public interface with Kong
 * Creates Kong route and stores in public_interfaces table
 */
export async function registerPublicInterface(
  serviceInterfaceId: string,
  config: PublicInterfaceConfig,
  supabase: SupabaseClient<Database>
): Promise<RegisteredPublicInterface> {
  if (!isKongEnabled()) {
    throw new Error('Kong is not enabled. Cannot register public interface.');
  }

  // Get service interface with workflow and project info
  const { data: serviceInterface, error: siError } = await supabase
    .from('service_interfaces')
    .select(`
      *,
      workflow:workflows!inner(
        id,
        created_by,
        project:projects!inner(
          id,
          created_by
        )
      )
    `)
    .eq('id', serviceInterfaceId)
    .single();

  if (siError || !serviceInterface) {
    throw new Error('Service interface not found');
  }

  const workflow = serviceInterface.workflow as any;
  const project = workflow.project as any;
  const userId = workflow.created_by;

  // Verify service interface is public
  if (!serviceInterface.is_public) {
    throw new Error('Service interface must be marked as public to create Kong route');
  }

  const kong = new KongClient();
  const kongConfig = getKongConfig();

  // Verify Kong is accessible
  const isHealthy = await kong.healthCheck();
  if (!isHealthy) {
    throw new Error('Kong Admin API is not accessible');
  }

  // Generate hash for this endpoint
  // For interface components, use component ID for consistency
  // Check if service interface is linked to a component
  const componentId = serviceInterface.activity_connection_id;
  const hashInput = componentId 
    ? `${componentId}:${config.httpPath}` // Use component ID for deterministic hashing
    : {
    userId,
    projectId: project.id,
    workflowId: workflow.id,
    endpointPath: config.httpPath,
      };
  
  const routeHash = typeof hashInput === 'string'
    ? (() => {
        const hash = createHash('sha256').update(hashInput).digest('hex');
        return `${hash.substring(0, 8)}-${hash.substring(8, 16)}`;
      })()
    : generateEndpointHash(hashInput);

  // Full path with hash: /api/v1/{hash}/{endpoint-path}
  const normalizedPath = config.httpPath.startsWith('/') 
    ? config.httpPath 
    : `/${config.httpPath}`;
  const fullPath = `/api/v1/${routeHash}${normalizedPath}`;

  // Create or get Kong service for workflow API handler
  const serviceName = 'workflow-api-handler';
  const serviceId = await kong.createService(serviceName, kongConfig.appUrl);

  // Create Kong route
  const routeName = `si-${routeHash}-${config.httpMethod.toLowerCase()}`;
  
  // Check if route already exists
  const existingRoute = await kong.getRoute(routeName);
  let routeId: string;

  if (existingRoute) {
    routeId = existingRoute.id;
    console.log(`ℹ️  Kong route already exists: ${routeName}`);
  } else {
    routeId = await kong.createRoute(
      serviceId,
      routeName,
      [fullPath],
      [config.httpMethod]
    );
    console.log(`✅ Created Kong route: ${routeName} -> ${fullPath}`);
  }

  // Enable rate limiting plugin if configured
  if (config.rateLimitPerMinute) {
    try {
      await kong.enablePlugin(routeId, 'rate-limiting', {
        minute: config.rateLimitPerMinute,
        hour: config.rateLimitPerHour || config.rateLimitPerMinute * 60,
      });
      console.log(`✅ Enabled rate limiting: ${config.rateLimitPerMinute}/min`);
    } catch (error) {
      console.warn(`⚠️  Failed to enable rate limiting: ${error}`);
    }
  }

  // Enable API key authentication if configured
  if (config.authType === 'api_key') {
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

  // Store in public_interfaces table
  const { data: publicInterface, error: dbError } = await supabase
    .from('public_interfaces')
    .upsert({
      service_interface_id: serviceInterfaceId,
      kong_route_id: routeId,
      http_method: config.httpMethod,
      http_path: config.httpPath,
      auth_type: config.authType || 'api_key',
      auth_config: config.authConfig || {},
      rate_limit_per_minute: config.rateLimitPerMinute || null,
      rate_limit_per_hour: config.rateLimitPerHour || null,
      updated_at: new Date().toISOString(),
    }, {
      onConflict: 'service_interface_id,http_method,http_path',
    })
    .select()
    .single();

  if (dbError) {
    // If database insert fails, clean up Kong route
    try {
      await kong.deleteRoute(routeId);
    } catch (cleanupError) {
      console.error('Failed to cleanup Kong route after DB error:', cleanupError);
    }
    throw new Error(`Failed to store public interface: ${dbError.message}`);
  }

  return {
    id: publicInterface.id,
    serviceInterfaceId: serviceInterface.id,
    kongRouteId: routeId,
    httpMethod: config.httpMethod,
    httpPath: config.httpPath,
    fullPath,
    url: `${kongConfig.gatewayUrl}${fullPath}`,
  };
}

/**
 * Update a public interface's Kong route
 */
export async function updatePublicInterface(
  publicInterfaceId: string,
  config: Partial<PublicInterfaceConfig>,
  supabase: SupabaseClient<Database>
): Promise<RegisteredPublicInterface> {
  if (!isKongEnabled()) {
    throw new Error('Kong is not enabled');
  }

  // Get existing public interface
  const { data: publicInterface, error: fetchError } = await supabase
    .from('public_interfaces')
    .select('*')
    .eq('id', publicInterfaceId)
    .single();

  if (fetchError || !publicInterface) {
    throw new Error('Public interface not found');
  }

  const kong = new KongClient();

  // Update Kong route plugins if route exists
  if (publicInterface.kong_route_id) {
    // Update rate limiting if changed
    if (config.rateLimitPerMinute !== undefined) {
      try {
        await kong.enablePlugin(publicInterface.kong_route_id, 'rate-limiting', {
          minute: config.rateLimitPerMinute,
          hour: config.rateLimitPerHour || config.rateLimitPerMinute * 60,
        });
      } catch (error) {
        console.warn(`⚠️  Failed to update rate limiting: ${error}`);
      }
    }

    // Update auth if changed
    if (config.authType !== undefined) {
      // Note: Kong plugins can't be easily updated, may need to delete and recreate
      // For now, we'll just update the database record
    }
  }

  // Update database record
  const updateData: Record<string, any> = {};
  if (config.rateLimitPerMinute !== undefined) updateData.rate_limit_per_minute = config.rateLimitPerMinute;
  if (config.rateLimitPerHour !== undefined) updateData.rate_limit_per_hour = config.rateLimitPerHour;
  if (config.authType !== undefined) updateData.auth_type = config.authType;
  if (config.authConfig !== undefined) updateData.auth_config = config.authConfig;

  const { data: updated, error: updateError } = await supabase
    .from('public_interfaces')
    .update(updateData)
    .eq('id', publicInterfaceId)
    .select()
    .single();

  if (updateError) {
    throw new Error(`Failed to update public interface: ${updateError.message}`);
  }

  // Get service interface for full path generation
  const { data: serviceInterface } = await supabase
    .from('service_interfaces')
    .select(`
      workflow:workflows!inner(
        id,
        created_by,
        project:projects!inner(id)
      )
    `)
    .eq('id', updated.service_interface_id)
    .single();

  const workflow = (serviceInterface as any)?.workflow;
  const project = workflow?.project;
  const kongConfig = getKongConfig();

  // Generate full path for response
  const routeHash = generateEndpointHash({
    userId: workflow.created_by,
    projectId: project.id,
    workflowId: workflow.id,
    endpointPath: updated.http_path,
  });
  const fullPath = `/api/v1/${routeHash}${updated.http_path.startsWith('/') ? updated.http_path : `/${updated.http_path}`}`;

  return {
    id: updated.id,
    serviceInterfaceId: updated.service_interface_id,
    kongRouteId: updated.kong_route_id || '',
    httpMethod: updated.http_method,
    httpPath: updated.http_path,
    fullPath,
    url: `${kongConfig.gatewayUrl}${fullPath}`,
  };
}

/**
 * Unregister a public interface (delete from Kong and database)
 */
export async function unregisterPublicInterface(
  publicInterfaceId: string,
  supabase: SupabaseClient<Database>
): Promise<void> {
  if (!isKongEnabled()) {
    return;
  }

  // Get public interface
  const { data: publicInterface, error } = await supabase
    .from('public_interfaces')
    .select('kong_route_id')
    .eq('id', publicInterfaceId)
    .single();

  if (error || !publicInterface) {
    throw new Error('Public interface not found');
  }

  // Delete from Kong if route exists
  if (publicInterface.kong_route_id) {
    const kong = new KongClient();
    try {
      await kong.deleteRoute(publicInterface.kong_route_id);
      console.log(`✅ Deleted Kong route: ${publicInterface.kong_route_id}`);
    } catch (error) {
      console.warn(`⚠️  Failed to delete Kong route: ${error}`);
    }
  }

  // Delete from database
  await supabase
    .from('public_interfaces')
    .delete()
    .eq('id', publicInterfaceId);
}

/**
 * Auto-register public interfaces when service interface is marked as public
 * This can be called from a trigger or manually
 */
export async function syncPublicInterfacesForServiceInterface(
  serviceInterfaceId: string,
  supabase: SupabaseClient<Database>
): Promise<void> {
  const { data: serviceInterface, error } = await supabase
    .from('service_interfaces')
    .select('*')
    .eq('id', serviceInterfaceId)
    .single();

  if (error || !serviceInterface) {
    throw new Error('Service interface not found');
  }

  // If marked as public, ensure there's a public interface
  if (serviceInterface.is_public) {
    // Check if public interface already exists
    const { data: existing } = await supabase
      .from('public_interfaces')
      .select('id')
      .eq('service_interface_id', serviceInterfaceId)
      .single();

    if (!existing) {
      // Auto-create public interface based on interface type
      const httpMethod = serviceInterface.interface_type === 'query' ? 'GET' : 'POST';
      const httpPath = `/${serviceInterface.name}`;

      await registerPublicInterface(serviceInterfaceId, {
        httpMethod,
        httpPath,
        authType: 'api_key',
        rateLimitPerMinute: 60,
      }, supabase);
    }
  } else {
    // If marked as not public, remove public interfaces
    const { data: publicInterfaces } = await supabase
      .from('public_interfaces')
      .select('id')
      .eq('service_interface_id', serviceInterfaceId);

    for (const pi of publicInterfaces || []) {
      await unregisterPublicInterface(pi.id, supabase);
    }
  }
}

