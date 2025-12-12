/**
 * Kong CORS Configuration Manager
 * Handles configuration and application of Kong CORS plugin
 */

import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import { KongClient } from './client';

export interface CorsConfig {
  allowedOrigins: string[];
  allowedMethods: string[];
  allowedHeaders: string[];
  exposedHeaders?: string[];
  allowCredentials?: boolean;
  maxAge?: number;
  preflightContinue?: boolean;
}

/**
 * Get CORS configuration from public interface
 */
export async function getCorsConfig(
  publicInterfaceId: string,
  supabase: SupabaseClient<Database>
): Promise<CorsConfig | null> {
  const { data, error } = await supabase
    .from('public_interfaces')
    .select('cors_config')
    .eq('id', publicInterfaceId)
    .single();

  if (error || !data || !data.cors_config) {
    return null;
  }

  const config = data.cors_config as any;

  return {
    allowedOrigins: config.allowedOrigins || config.allowed_origins || ['*'],
    allowedMethods: config.allowedMethods || config.allowed_methods || ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'OPTIONS'],
    allowedHeaders: config.allowedHeaders || config.allowed_headers || ['*'],
    exposedHeaders: config.exposedHeaders || config.exposed_headers || [],
    allowCredentials: config.allowCredentials ?? config.allow_credentials ?? false,
    maxAge: config.maxAge || config.max_age || 3600,
    preflightContinue: config.preflightContinue ?? config.preflight_continue ?? false,
  };
}

/**
 * Enable CORS plugin on a Kong route
 */
export async function enableCorsPlugin(
  routeId: string,
  corsConfig: CorsConfig,
  kong: KongClient
): Promise<void> {
  const pluginConfig: Record<string, any> = {
    origins: corsConfig.allowedOrigins,
    methods: corsConfig.allowedMethods,
    headers: corsConfig.allowedHeaders,
    exposed_headers: corsConfig.exposedHeaders || [],
    credentials: corsConfig.allowCredentials || false,
    max_age: corsConfig.maxAge || 3600,
    preflight_continue: corsConfig.preflightContinue || false,
  };

  await kong.enablePlugin(routeId, 'cors', pluginConfig);
}

/**
 * Apply CORS plugin to a route based on public interface configuration
 */
export async function applyCorsToRoute(
  publicInterfaceId: string,
  routeId: string,
  supabase: SupabaseClient<Database>,
  kong: KongClient
): Promise<void> {
  const corsConfig = await getCorsConfig(publicInterfaceId, supabase);
  
  if (!corsConfig) {
    return; // No CORS configured for this interface
  }

  try {
    await enableCorsPlugin(routeId, corsConfig, kong);
    console.log(`✅ Enabled CORS on route ${routeId}`);
  } catch (error) {
    console.error(`❌ Failed to enable CORS on route ${routeId}:`, error);
  }
}

