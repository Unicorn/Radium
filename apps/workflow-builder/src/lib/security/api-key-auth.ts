/**
 * API Key Authentication Middleware
 * 
 * Validates API keys from X-API-Key header for public interface access.
 */

import { hashApiKey, verifyApiKey } from './api-key-hasher';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';

export interface ApiKeyValidationResult {
  valid: boolean;
  apiKeyId?: string;
  userId?: string;
  projectId?: string | null;
  publicInterfaceId?: string | null;
  error?: string;
}

/**
 * Validate API key from request header
 * 
 * @param apiKey - The API key from X-API-Key header
 * @param supabase - Supabase client for database queries
 * @returns Validation result with user/project/interface info
 */
export async function validateApiKey(
  apiKey: string | null,
  supabase: SupabaseClient<Database>
): Promise<ApiKeyValidationResult> {
  if (!apiKey) {
    return {
      valid: false,
      error: 'API key required. Provide X-API-Key header.',
    };
  }

  // Hash the provided API key
  const keyHash = hashApiKey(apiKey);

  // Look up API key in database
  const { data: apiKeyRecord, error } = await supabase
    .from('api_keys')
    .select('*')
    .eq('key_hash', keyHash)
    .eq('is_active', true)
    .single();

  if (error || !apiKeyRecord) {
    return {
      valid: false,
      error: 'Invalid API key',
    };
  }

  // Check if API key is expired
  if (apiKeyRecord.expires_at) {
    const expiresAt = new Date(apiKeyRecord.expires_at);
    if (expiresAt < new Date()) {
      return {
        valid: false,
        error: 'API key has expired',
      };
    }
  }

  // Update last_used_at timestamp (fire and forget)
  supabase
    .from('api_keys')
    .update({ last_used_at: new Date().toISOString() })
    .eq('id', apiKeyRecord.id)
    .then(() => {
      // Silently handle errors - don't block request
    })
    .catch(() => {
      // Silently handle errors
    });

  return {
    valid: true,
    apiKeyId: apiKeyRecord.id,
    userId: apiKeyRecord.user_id,
    projectId: apiKeyRecord.project_id,
    publicInterfaceId: apiKeyRecord.public_interface_id,
  };
}

/**
 * Extract API key from request headers
 * 
 * @param headers - Request headers
 * @returns API key or null
 */
export function extractApiKeyFromHeaders(headers: Headers): string | null {
  // Check X-API-Key header (standard)
  const apiKey = headers.get('X-API-Key');
  if (apiKey) {
    return apiKey;
  }

  // Check Authorization header with Bearer token (alternative)
  const authHeader = headers.get('Authorization');
  if (authHeader?.startsWith('Bearer ')) {
    return authHeader.substring(7);
  }

  return null;
}

