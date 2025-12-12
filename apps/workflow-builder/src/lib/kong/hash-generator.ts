/**
 * Hash Generator for Endpoint Routing
 * 
 * Generates deterministic hashes for workflow endpoints to ensure
 * unique URLs across users, projects, and workflows.
 */

import { createHash } from 'crypto';

export interface HashInput {
  userId: string;
  projectId: string;
  workflowId: string;
  endpointPath: string;
}

/**
 * Generate deterministic hash for endpoint routing
 * Format: {first8chars}-{next8chars} = 16 char hash
 * Example: a3f2b1c4-d5e6f7g8
 * 
 * The hash is deterministic - same inputs always produce same hash
 */
export function generateEndpointHash(input: HashInput): string {
  // Normalize endpoint path (remove leading/trailing slashes, lowercase)
  const normalizedPath = input.endpointPath
    .toLowerCase()
    .replace(/^\/+|\/+$/g, '');
  
  // Combine all components with delimiter
  const combined = `${input.userId}:${input.projectId}:${input.workflowId}:${normalizedPath}`;
  
  // Generate SHA256 hash
  const hash = createHash('sha256').update(combined).digest('hex');
  
  // Return 16-char hash (8-8 format for readability)
  return `${hash.substring(0, 8)}-${hash.substring(8, 16)}`;
}

/**
 * Validate hash matches expected input
 * Useful for debugging and verification
 */
export function validateHash(
  hash: string,
  userId: string,
  projectId: string,
  workflowId: string,
  endpointPath: string
): boolean {
  const expected = generateEndpointHash({ userId, projectId, workflowId, endpointPath });
  return hash === expected;
}

/**
 * Extract hash from full path
 * Format: /api/v1/{hash}/{endpoint-path}
 */
export function extractHashFromPath(path: string): string | null {
  const match = path.match(/^\/api\/v1\/([a-f0-9]{8}-[a-f0-9]{8})\//);
  return match ? match[1] : null;
}

