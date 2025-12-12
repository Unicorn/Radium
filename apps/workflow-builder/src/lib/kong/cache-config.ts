/**
 * Kong Cache Configuration Manager
 * Handles configuration and application of Kong proxy cache plugin
 */

import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import { KongClient } from './client';

export interface CacheConfig {
  cacheKey: string;
  ttlSeconds?: number;
  cacheKeyStrategy?: 'default' | 'query-string' | 'header' | 'cookie';
  contentTypes?: string[];
  responseCodes?: number[];
  connectorId: string;
}

/**
 * Get cache configuration for a component
 */
export async function getCacheConfig(
  componentId: string,
  supabase: SupabaseClient<Database>
): Promise<CacheConfig | null> {
  const { data, error } = await supabase
    .from('kong_cache_keys')
    .select('*, connector:connectors(*)')
    .eq('component_id', componentId)
    .eq('marked_for_deletion', false)
    .single();

  if (error || !data) {
    return null;
  }

  return {
    cacheKey: data.cache_key,
    ttlSeconds: data.ttl_seconds || 3600,
    cacheKeyStrategy: (data.cache_key_strategy as any) || 'default',
    contentTypes: (data.content_types as string[]) || ['application/json'],
    responseCodes: (data.response_codes as number[]) || [200, 201],
    connectorId: data.connector_id,
  };
}

/**
 * Get Redis connector configuration
 */
export async function getRedisConnectorConfig(
  connectorId: string,
  supabase: SupabaseClient<Database>
): Promise<{ host?: string; port?: number; database?: number; password?: string; url?: string } | null> {
  const { data: connector, error } = await supabase
    .from('connectors')
    .select('config_data, credentials_encrypted, name')
    .eq('id', connectorId)
    .single();

  if (error || !connector) {
    return null;
  }

  const config: any = {};

  // Parse credentials if available
  if (connector.credentials_encrypted) {
    try {
      const credentials = JSON.parse(
        Buffer.from(connector.credentials_encrypted).toString()
      );
      
      // Upstash Redis typically has REST API URL and token
      if (connector.name?.toLowerCase().includes('upstash')) {
        config.url = credentials.url || credentials.rest_url;
        config.token = credentials.token || credentials.rest_token;
      } else {
        // Standard Redis connection
        config.host = credentials.host || 'localhost';
        config.port = credentials.port || 6379;
        config.password = credentials.password;
        config.database = credentials.database || 0;
      }
    } catch (error) {
      console.warn('Failed to parse Redis connector credentials:', error);
    }
  }

  // Merge with config_data
  if (connector.config_data) {
    Object.assign(config, connector.config_data);
  }

  return config;
}

/**
 * Enable proxy cache plugin on a Kong route
 */
export async function enableCachePlugin(
  routeId: string,
  cacheConfig: CacheConfig,
  redisConfig: any,
  kong: KongClient
): Promise<void> {
  const pluginConfig: Record<string, any> = {
    response_code: cacheConfig.responseCodes || [200, 201],
    content_type: cacheConfig.contentTypes || ['application/json'],
    ttl: cacheConfig.ttlSeconds || 3600,
    strategy: 'memory', // Kong's proxy-cache supports memory or redis
  };

  // If Redis connector is configured, use Redis storage
  if (redisConfig) {
    pluginConfig.strategy = 'redis';
    
    if (redisConfig.url && redisConfig.token) {
      // Upstash Redis REST API
      pluginConfig.redis = {
        host: redisConfig.url.replace(/^https?:\/\//, '').split(':')[0],
        port: redisConfig.port || 6379,
        database: redisConfig.database || 0,
        // Note: Kong proxy-cache plugin may need additional Redis config
        // This is a simplified version - actual implementation may vary
      };
    } else {
      // Standard Redis connection
      pluginConfig.redis = {
        host: redisConfig.host || 'localhost',
        port: redisConfig.port || 6379,
        database: redisConfig.database || 0,
        password: redisConfig.password,
      };
    }
  }

  // Cache key strategy
  if (cacheConfig.cacheKeyStrategy === 'query-string') {
    pluginConfig.cache_control = true;
  }

  await kong.enablePlugin(routeId, 'proxy-cache', pluginConfig);
}

/**
 * Apply cache plugin to routes based on component configuration
 */
export async function applyCacheToRoute(
  componentId: string,
  routeId: string,
  supabase: SupabaseClient<Database>,
  kong: KongClient
): Promise<void> {
  const cacheConfig = await getCacheConfig(componentId, supabase);
  
  if (!cacheConfig) {
    return; // No cache configured for this component
  }

  const redisConfig = await getRedisConnectorConfig(cacheConfig.connectorId, supabase);

  if (!redisConfig) {
    console.warn(`⚠️  Redis connector ${cacheConfig.connectorId} not found or misconfigured`);
    return;
  }

  try {
    await enableCachePlugin(routeId, cacheConfig, redisConfig, kong);
    console.log(`✅ Enabled proxy cache on route ${routeId} with key ${cacheConfig.cacheKey}`);
  } catch (error) {
    console.error(`❌ Failed to enable cache on route ${routeId}:`, error);
  }
}

/**
 * Clean up cache keys marked for deletion
 */
export async function cleanupDeletedCacheKeys(
  projectId: string,
  supabase: SupabaseClient<Database>
): Promise<void> {
  const { data: deletedKeys, error } = await supabase
    .from('kong_cache_keys')
    .select('cache_key, connector_id')
    .eq('project_id', projectId)
    .eq('marked_for_deletion', true);

  if (error || !deletedKeys || deletedKeys.length === 0) {
    return;
  }

  // Get Redis connector to delete keys
  for (const key of deletedKeys) {
    const redisConfig = await getRedisConnectorConfig(key.connector_id, supabase);
    
    if (redisConfig) {
      // TODO: Implement actual Redis key deletion
      // This would require a Redis client to delete the keys
      console.log(`🗑️  Cache key marked for deletion: ${key.cache_key}`);
    }
  }

  // Delete from database
  await supabase
    .from('kong_cache_keys')
    .delete()
    .eq('project_id', projectId)
    .eq('marked_for_deletion', true);
}

