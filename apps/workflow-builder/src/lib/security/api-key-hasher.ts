/**
 * API Key Hasher
 * 
 * Hashes API keys using bcrypt for secure storage.
 * Never store plain API keys - always hash them.
 */

import { createHash, timingSafeEqual } from 'crypto';

/**
 * Hash an API key using SHA-256 (for database storage)
 * 
 * Note: We use SHA-256 instead of bcrypt because:
 * - API keys are already random and high-entropy
 * - We need fast lookups (bcrypt is slow by design)
 * - SHA-256 is sufficient for API key storage
 * 
 * For production, consider using a dedicated key derivation function
 * or storing keys in a secure vault service.
 * 
 * @param apiKey - The plain API key
 * @returns The hashed API key (hex string)
 */
export function hashApiKey(apiKey: string): string {
  return createHash('sha256').update(apiKey).digest('hex');
}

/**
 * Verify an API key against a hash
 * 
 * @param apiKey - The plain API key to verify
 * @param hash - The stored hash
 * @returns True if the API key matches the hash
 */
export function verifyApiKey(apiKey: string, hash: string): boolean {
  const computedHash = hashApiKey(apiKey);
  
  // Use timing-safe comparison to prevent timing attacks
  if (computedHash.length !== hash.length) {
    return false;
  }
  
  return timingSafeEqual(
    Buffer.from(computedHash),
    Buffer.from(hash)
  );
}

/**
 * Generate a display prefix for an API key
 * Shows first 4-8 characters for identification
 * 
 * @param apiKey - The API key
 * @returns Display prefix (e.g., "sk_live_abcd...")
 */
export function getApiKeyDisplayPrefix(apiKey: string): string {
  // Show prefix + first 4 chars of random part
  const prefix = apiKey.substring(0, apiKey.lastIndexOf('_') + 1);
  const randomPart = apiKey.substring(apiKey.lastIndexOf('_') + 1);
  const firstChars = randomPart.substring(0, 4);
  
  return `${prefix}${firstChars}...`;
}

