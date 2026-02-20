/**
 * API Key Generator
 * 
 * Generates cryptographically secure API keys for public interface authentication.
 */

import { randomBytes } from 'crypto';

/**
 * Generate a new API key
 * Format: sk_live_{random32chars} or sk_test_{random32chars}
 * 
 * @param prefix - Optional prefix (default: 'sk_live_')
 * @returns The generated API key
 */
export function generateApiKey(prefix: string = 'sk_live_'): string {
  // Generate 32 random bytes (256 bits) and convert to base64url
  const randomBytesBuffer = randomBytes(32);
  const randomString = randomBytesBuffer
    .toString('base64')
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '')
    .substring(0, 32); // Base64 produces 44 chars, we take 32
  
  return `${prefix}${randomString}`;
}

/**
 * Extract the prefix from an API key
 * 
 * @param apiKey - The API key
 * @returns The prefix (first 4-8 characters before the random part)
 */
export function extractApiKeyPrefix(apiKey: string): string {
  // Extract prefix up to the last underscore or first 8 chars
  const underscoreIndex = apiKey.lastIndexOf('_');
  if (underscoreIndex > 0) {
    return apiKey.substring(0, underscoreIndex + 1);
  }
  return apiKey.substring(0, Math.min(8, apiKey.length));
}

/**
 * Validate API key format
 * 
 * @param apiKey - The API key to validate
 * @returns True if the format is valid
 */
export function validateApiKeyFormat(apiKey: string): boolean {
  // Should start with sk_live_ or sk_test_ and have at least 32 more characters
  const pattern = /^sk_(live|test)_[A-Za-z0-9_-]{32,}$/;
  return pattern.test(apiKey);
}

